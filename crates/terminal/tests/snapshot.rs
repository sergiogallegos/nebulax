use nebulax_terminal::{
    Limits, Size, Terminal, WidthPolicy,
    snapshot::{MAX_SNAPSHOT_BYTES, Snapshot, SnapshotCell, SnapshotError},
};

fn terminal(columns: usize, lines: usize) -> Terminal {
    Terminal::new(
        Size { columns, lines },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}
#[test]
fn owned_unicode_text_and_roles_survive_mutation_resize_and_destruction() {
    let mut t = terminal(8, 3);
    t.feed("a\u{301}👩‍💻 🇺🇸".as_bytes());
    let frame = Snapshot::capture(&t, None).unwrap();
    assert_eq!(std::mem::size_of::<SnapshotCell>(), 8);
    assert_eq!(frame.cell_text(&frame.cells()[0]), Some("a\u{301}"));
    assert_eq!(frame.cell_text(&frame.cells()[1]), Some("👩‍💻"));
    assert_eq!((frame.cells()[1].width, frame.cells()[2].kind), (2, 2));
    let text = frame.text().to_vec();
    t.feed(b"\x1b[?1049hDELETED");
    t.resize(Size {
        columns: 2,
        lines: 1,
    })
    .unwrap();
    drop(t);
    std::thread::spawn(move || {
        assert_eq!(frame.text(), text);
        for cell in frame.cells() {
            assert!(frame.cell_text(cell).is_some());
        }
        assert!(frame.payload_bytes() <= MAX_SNAPSHOT_BYTES);
    })
    .join()
    .unwrap();
}
#[test]
fn row_versions_survive_skipped_frames_and_cursor_only_changes() {
    let mut t = terminal(6, 3);
    t.feed(b"a\r\nZ");
    let a = Snapshot::capture(&t, None).unwrap();
    t.feed(b"\r\x1b[2D"); // Cursor-only: row payload stays unchanged.
    let b = Snapshot::capture(&t, Some(&a)).unwrap();
    assert_eq!(a.row_versions(), b.row_versions());
    t.feed(b"Y");
    let c = Snapshot::capture(&t, Some(&b)).unwrap();
    let d = Snapshot::capture(&t, Some(&c)).unwrap();
    assert_eq!(d.row_versions(), &[1, 3, 1]);
    assert_ne!(a.row_versions()[1], d.row_versions()[1]); // Consumer skipped b/c.
    assert_eq!(a.row_versions()[0], d.row_versions()[0]);
    t.feed(b"\x1b[?1049h");
    let alternate = Snapshot::capture(&t, Some(&d)).unwrap();
    assert_eq!(alternate.row_versions(), &[5, 5, 5]);
    t.resize(Size {
        columns: 5,
        lines: 3,
    })
    .unwrap();
    let resized = Snapshot::capture(&t, Some(&alternate)).unwrap();
    assert_eq!(resized.row_versions(), &[6, 6, 6]);
}
#[test]
fn snapshot_overflow_is_explicit_and_does_not_mutate_engine_or_previous_frame() {
    let mut t = terminal(256, 256);
    let previous = Snapshot::capture(&t, None).unwrap();
    // Every cell has a bounded 64-scalar cluster, exceeding the snapshot's byte
    // budget while remaining valid engine content. No giant snapshot is built.
    let cluster = format!("a{}", "\u{301}".repeat(63));
    t.feed(cluster.repeat(16_000).as_bytes());
    let cursor = t.cursor();
    assert_eq!(
        Snapshot::capture(&t, Some(&previous)).unwrap_err(),
        SnapshotError::ByteLimit
    );
    assert_eq!(t.cursor(), cursor);
    assert!(previous.text().is_empty());
    assert!(t.invariants_hold());
}
#[test]
fn wrap_padding_and_row_wraps_are_explicit() {
    let mut t = terminal(3, 2);
    t.feed("ab界".as_bytes());
    let frame = Snapshot::capture(&t, None).unwrap();
    assert_eq!(frame.cells()[2].kind, 3);
    assert_eq!(frame.row_wraps(), &[1, 0]);
    assert_eq!(frame.cells()[3].width, 2);
    assert_eq!(frame.cells()[4].kind, 2);
}

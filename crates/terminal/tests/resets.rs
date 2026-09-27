use nebulax_terminal::{
    CellView, Cursor, FeedOutcome, Limits, OutputEvent, Size, Terminal, TitleTarget, WidthPolicy,
    input::Key, output::MAX_QUEUED_EVENTS, snapshot::Snapshot, style::Style,
};

fn terminal(columns: usize, lines: usize) -> Terminal {
    Terminal::new(
        Size { columns, lines },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}
fn feed(t: &mut Terminal, bytes: &[u8]) -> FeedOutcome {
    let out = t.feed(bytes);
    assert_eq!(out.consumed, bytes.len());
    assert!(!out.unsupported && !out.parser_limit && !out.style_limit);
    assert!(t.invariants_hold());
    out
}
fn drain(t: &mut Terminal) -> Vec<OutputEvent> {
    std::iter::from_fn(|| t.pop_output()).collect()
}
fn text(t: &Terminal) -> Vec<u8> {
    Snapshot::capture(t, None).unwrap().text().to_vec()
}

#[test]
fn soft_reset_preserves_cells_history_tabs_and_position_but_resets_control_state() {
    let mut t = terminal(16, 4);
    feed(
        &mut t,
        b"\x1b[31;44mold\r\na\r\nb\r\nc\r\nd\x1b[3g\x1b[4G\x1bH",
    );
    feed(&mut t, b"\x1b[2;3r\x1b[?1;6h\x1b[?7;25l\x1b[2;6H\x1b7");
    let rows = t.screen().to_vec();
    let history = t.history().clone();
    let cursor = t.cursor();
    let old = Snapshot::capture(&t, None).unwrap();
    let out = feed(&mut t, b"\x1b[!p");
    assert!(out.changed && !out.history_evicted);
    assert_eq!(t.screen(), rows);
    assert_eq!(t.history(), &history);
    assert_eq!(t.cursor(), cursor);
    assert_eq!(t.current_style(), Style::default());
    assert!(t.autowrap() && t.cursor_visible());
    assert_eq!(t.encode_key(Key::Up).unwrap(), b"\x1b[A");
    let new = Snapshot::capture(&t, Some(&old)).unwrap();
    assert_eq!(new.row_versions(), old.row_versions());
    assert!(!old.cursor_visible() && new.cursor_visible());
    feed(&mut t, b"\x1b8");
    assert_eq!(t.cursor(), Cursor::default());
    assert_eq!(t.current_style(), Style::default());
    feed(&mut t, b"\t");
    assert_eq!(t.cursor().column, 3); // Custom stop survived.
    feed(&mut t, b"\x1b[4;1H\n");
    assert_eq!(t.history().len(), history.len() + 1); // Full-height margins.
}

#[test]
fn soft_reset_in_alternate_preserves_hidden_primary_and_stays_alternate() {
    let mut t = terminal(8, 4);
    feed(&mut t, b"\x1b[2;4r\x1b[?6h\x1b[?7l\x1b[31m\x1b[2;3HP\x1b7");
    let cursor = t.cursor();
    let rows = t.screen().to_vec();
    let style = t.current_style();
    feed(&mut t, b"\x1b[?1049h\x1b[?1;6h\x1b[?25l\x1b[32mALT\x1b[!p");
    assert!(t.is_alternate() && t.cursor_visible());
    assert_eq!(text(&t), b"ALT");
    assert_eq!(t.current_style(), Style::default());
    feed(&mut t, b"\x1b[?1049l");
    assert_eq!(t.screen(), rows);
    assert_eq!(t.cursor(), cursor);
    assert_eq!(t.current_style(), style);
    assert!(!t.autowrap());
    feed(&mut t, b"\x1b[?6$p\x1b[H\x1b8");
    assert_eq!(drain(&mut t), [OutputEvent::Reply(b"\x1b[?6;1$y".to_vec())]);
    assert_eq!(t.cursor(), cursor); // Hidden saved cursor was not reset.
}

#[test]
fn hard_reset_returns_to_fresh_primary_and_preserves_host_limits_and_width_policy() {
    let limits = Limits {
        max_cells: 100,
        cluster_scalars: 2,
        history_rows: 2,
        history_cells: 16,
    };
    let policy = WidthPolicy {
        ambiguous_wide: true,
    };
    let size = Size {
        columns: 8,
        lines: 4,
    };
    let fresh = Terminal::new(size, limits, policy).unwrap();
    let mut t = fresh.clone();
    feed(
        &mut t,
        "\x1b[31mα\r\na\r\nb\r\nc\r\nd\x1b[3g\x1b[?1049h\x1b[?1;6h\x1b[?7;25l\x1b7ALT".as_bytes(),
    );
    assert!(feed(&mut t, b"\x1bc").history_evicted);
    assert_eq!(t, fresh);
    feed(&mut t, b"\x1b[?1049l\x1b8");
    assert_eq!(t.cursor(), Cursor::default());
    assert!(
        t.resize(Size {
            columns: 101,
            lines: 1
        })
        .is_err()
    );
    feed(&mut t, "α".as_bytes());
    assert_eq!(t.cursor().column, 2);
    let out = t.feed("\u{301}\u{302}".as_bytes());
    assert!(out.cluster_limit && t.invariants_hold());
}

#[test]
fn hard_reset_releases_old_resources_and_owned_snapshots_survive_style_id_reuse() {
    let mut t = terminal(8, 3);
    feed(&mut t, "\x1b[31mA\u{301}\r\nx\r\ny\r\nz\x1b7".as_bytes());
    let primary = Snapshot::capture(&t, None).unwrap();
    feed(&mut t, "\x1b[?1049h\x1b[32mB\u{301}".as_bytes());
    let old = Snapshot::capture(&t, Some(&primary)).unwrap();
    assert!(t.storage_usage().cluster_allocations > 0);
    feed(&mut t, b"\x1bc");
    assert_eq!(t.storage_usage().cluster_allocations, 0);
    assert_eq!(t.retained_style_count(), 1);
    assert!(t.history().is_empty() && !t.is_alternate());
    assert!(
        t.screen()
            .iter()
            .flat_map(|r| r.cells())
            .all(|c| matches!(c.view(), CellView::Empty) && c.style_id() == 0)
    );
    let blank = Snapshot::capture(&t, Some(&old)).unwrap();
    assert!(
        blank
            .row_versions()
            .iter()
            .all(|v| *v == blank.generation())
    );
    feed(&mut t, b"\x1b[34mB");
    let blue = Snapshot::capture(&t, Some(&blank)).unwrap();
    assert_ne!(blue.row_versions()[0], blank.row_versions()[0]);
    drop(t);
    assert_eq!(old.text(), "B\u{301}".as_bytes());
    assert_eq!(
        old.cell_style(&old.cells()[0]).unwrap().foreground,
        Style::indexed(2)
    );
    assert_eq!(
        blue.cell_style(&blue.cells()[0]).unwrap().foreground,
        Style::indexed(4)
    );
    assert_eq!(primary.text(), b"xyz");
}

#[test]
fn hard_reset_restores_tab_defaults_including_later_growth_and_reuses_visible_rows() {
    let mut t = terminal(136, 2);
    feed(&mut t, b"\x1b[3g\x1b[120G\x1bH");
    t.resize(Size {
        columns: 16,
        lines: 2,
    })
    .unwrap();
    let pointers: Vec<_> = t.screen().iter().map(|r| r.cells().as_ptr()).collect();
    feed(&mut t, b"\x1bc");
    assert_eq!(
        t.screen()
            .iter()
            .map(|r| r.cells().as_ptr())
            .collect::<Vec<_>>(),
        pointers
    );
    feed(&mut t, b"\t");
    assert_eq!(t.cursor().column, 8);
    t.resize(Size {
        columns: 136,
        lines: 2,
    })
    .unwrap();
    feed(&mut t, b"\x1b[64G\t");
    assert_eq!(t.cursor().column, 64);
    feed(&mut t, b"\x1b[113G\t");
    assert_eq!(t.cursor().column, 120); // Old custom column 119 was discarded.
}

#[test]
fn malformed_resets_and_reset_bytes_inside_strings_do_not_reset_state() {
    for bytes in [
        b"\x1b[0!p".as_slice(),
        b"\x1b[1!p",
        b"\x1b[;!p",
        b"\x1b[? !p",
        b"\x1b[0:0!p",
        b"\x1b[ !p",
        b"\x1b(c",
        b"\x1b]2;bad\x1bc\x1b[!p\x07",
        b"\x1bP\x1bc\x1b[!p\x1b\\",
    ] {
        let mut t = terminal(8, 3);
        feed(&mut t, b"\x1b[?7;25l\x1b[31mKEEP\0");
        let before = t.clone();
        assert!(t.feed(bytes).unsupported, "{bytes:?}");
        assert_eq!(t, before, "{bytes:?}");
    }
    let mut t = terminal(8, 3);
    feed(&mut t, b"KEEP\x1b[!\x18\x1b\x1a");
    assert_eq!(text(&t), b"KEEP");
}

#[test]
fn reset_streams_are_partition_invariant_and_close_grapheme_and_decoder_state() {
    let bytes = "ab\x1b[31mcd\x1b[6n\x1b[!p\u{301}Z\x1b[?1049hALT\x1bc\u{301}Q\x1b[6n".as_bytes();
    let mut expected = terminal(4, 2);
    let out = feed(&mut expected, bytes);
    assert!(out.orphan_mark);
    assert_eq!(text(&expected), b"Q");
    for split in 0..=bytes.len() {
        let mut t = terminal(4, 2);
        let mut result = t.feed(&bytes[..split]);
        result.merge(t.feed(&bytes[split..]));
        assert_eq!(result, out);
        assert_eq!(t, expected, "split {split}");
    }
    let mut t = terminal(4, 2);
    for byte in bytes {
        t.feed(&[*byte]);
    }
    assert_eq!(t, expected);
    feed(&mut t, b"\xe2\x1bc\x82\xac");
    assert_eq!(text(&t), "��".as_bytes()); // UTF-8 prefix before RIS cannot attach after it.
    let mut soft = terminal(4, 2);
    feed(&mut soft, b"abcd\x1b[!pZ");
    assert_eq!(text(&soft), b"abcZ"); // Soft reset cancels pending wrap without moving.
    assert!(soft.history().is_empty());
}

#[test]
fn output_pressure_defers_reset_until_its_exact_stream_position_without_losing_replies() {
    let mut t = terminal(8, 3);
    feed(&mut t, b"old");
    let prefix = b"\x1b[6n".repeat(MAX_QUEUED_EVENTS + 1);
    let mut bytes = prefix.clone();
    bytes.extend_from_slice(b"\x1bc\x1b[6nX");
    let out = t.feed(&bytes);
    assert_eq!(out.consumed, prefix.len());
    assert!(out.output_blocked);
    assert_eq!(t.feed(&bytes[out.consumed..]).consumed, 0);
    assert_eq!(text(&t), b"old");
    let mut replies = vec![t.pop_output().unwrap()];
    let resumed = t.feed(&bytes[out.consumed..]);
    assert_eq!(resumed.consumed, 6); // RIS + CPR consumed; X waits.
    assert!(resumed.output_blocked);
    assert_eq!(text(&t), b"");
    replies.extend(drain(&mut t));
    let mut expected = vec![OutputEvent::Reply(b"\x1b[1;4R".to_vec()); MAX_QUEUED_EVENTS + 1];
    expected.push(OutputEvent::Reply(b"\x1b[1;1R".to_vec()));
    assert_eq!(replies, expected);
    feed(&mut t, &bytes[out.consumed + resumed.consumed..]);
    assert_eq!(text(&t), b"X");
}

#[test]
fn both_resets_preserve_completed_effects_and_query_time_replies_in_fifo_order() {
    let mut t = terminal(8, 3);
    feed(
        &mut t,
        b"ab\x07\x1b]2;title\x07\x1b[6n\x1b[?7l\x1b[?7$p\x1b[!p\x1b[?7$p\x1bc\x1b[6n",
    );
    assert_eq!(
        drain(&mut t),
        [
            OutputEvent::Bell,
            OutputEvent::Title {
                target: TitleTarget::Window,
                text: "title".into()
            },
            OutputEvent::Reply(b"\x1b[1;3R".to_vec()),
            OutputEvent::Reply(b"\x1b[?7;2$y".to_vec()),
            OutputEvent::Reply(b"\x1b[?7;1$y".to_vec()),
            OutputEvent::Reply(b"\x1b[1;1R".to_vec())
        ]
    );
    assert_eq!(t, terminal(8, 3));
}

#[test]
fn repeated_resets_across_small_geometries_are_idempotent_and_preserve_invariants() {
    for columns in 2..=10 {
        for lines in 1..=4 {
            let fresh = terminal(columns, lines);
            let mut t = fresh.clone();
            for _ in 0..8 {
                feed(
                    &mut t,
                    "\x1b[31;44m界a\u{301}\r\n\x1b[?1049h👩‍💻\x1b7\x1b[?1;6h\x1b[?7;25l".as_bytes(),
                );
                feed(&mut t, b"\x1b[!p");
                let once = t.clone();
                feed(&mut t, b"\x1b[!p");
                assert_eq!(t, once);
                feed(&mut t, b"\x1bc\x1bc");
                assert_eq!(t, fresh);
            }
        }
    }
}

use nebulax_terminal::{
    CellView as Cell, Cursor, FeedOutcome, Limits, OutputEvent, Row, Size, Terminal, WidthPolicy,
    input::Key, snapshot::Snapshot,
};

fn terminal(columns: usize, lines: usize) -> Terminal {
    Terminal::new(
        Size { columns, lines },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}
fn rows(rows: impl IntoIterator<Item = impl std::borrow::Borrow<Row>>) -> Vec<String> {
    rows.into_iter()
        .map(|row| {
            row.borrow()
                .cells()
                .iter()
                .map(|cell| match cell.view() {
                    Cell::Lead { cluster, .. } => cluster.chars().collect::<String>(),
                    Cell::Empty => ".".into(),
                    Cell::Continuation => "~".into(),
                    Cell::WrapPadding => "_".into(),
                })
                .collect()
        })
        .collect()
}
fn feed(t: &mut Terminal, bytes: &[u8]) -> FeedOutcome {
    let out = t.feed(bytes);
    assert_eq!(out.consumed, bytes.len());
    assert!(
        !out.unsupported && !out.parser_limit && !out.output_blocked,
        "{out:?}"
    );
    assert!(t.invariants_hold());
    out
}
// Hand-written expectations below are the oracle. Whole input, every two-way
// split and one-byte delivery must all produce the same complete engine state.
fn partitions(columns: usize, lines: usize, bytes: &[u8]) -> Terminal {
    let mut expected = terminal(columns, lines);
    let out = feed(&mut expected, bytes);
    for split in 0..=bytes.len() {
        let mut t = terminal(columns, lines);
        let mut actual = feed(&mut t, &bytes[..split]);
        actual.merge(feed(&mut t, &bytes[split..]));
        assert_eq!(actual, out, "diagnostics at split {split}");
        assert_eq!(t, expected, "state at split {split}");
    }
    let mut t = terminal(columns, lines);
    for byte in bytes {
        feed(&mut t, &[*byte]);
    }
    assert_eq!(t, expected);
    expected
}

#[test]
fn positioning_defaults_clamps_and_relative_motion_do_not_scroll() {
    let t = partitions(
        6,
        5,
        b"\x1b[2;3HX\x1b[0AY\x1b[999CZ\x1b[0D!\x1b[2;2fQ\x1b[2EJ\x1b[FK\x1b[0Gx\x1b[5dy\x1b[0;0H",
    );
    assert_eq!(
        rows(t.screen()),
        ["...Y!Z", ".QX...", "x.....", "J.....", ".y...."]
    );
    assert_eq!(t.cursor(), Cursor::default());
    assert!(t.history().is_empty());
    let t = partitions(4, 3, b"\x1b[65535;65535H\x1b[65535B\x1b[65535A\x1b[65535D");
    assert_eq!(t.cursor(), Cursor::default());
    assert!(t.history().is_empty());
}

#[test]
fn region_index_scrolls_only_inside_margins_and_preserves_outside_rows() {
    let t = partitions(
        4,
        5,
        b"A\x1b[2;1HB\x1b[3;1HC\x1b[4;1HD\x1b[5;1HE\x1b[2;4r\x1b[4;1H\nX",
    );
    assert_eq!(rows(t.screen()), ["A...", "C...", "D...", "X...", "E..."]);
    assert!(t.history().is_empty());
    assert_eq!(t.cursor().row, 3);
    let t = partitions(
        4,
        5,
        b"A\x1b[2;1HB\x1b[3;1HC\x1b[4;1HD\x1b[5;1HE\x1b[2;4r\x1b[5;1H\x1bDX\x1b[1;1H\x1bMZ",
    );
    assert_eq!(rows(t.screen()), ["Z...", "B...", "C...", "D...", "X..."]);
    assert!(t.history().is_empty());
}

#[test]
fn origin_controls_home_addressing_motion_and_query_coordinates() {
    let mut t = partitions(6, 5, b"\x1b[2;4r\x1b[?6h\x1b[6n\x1b[999;999H\x1b[6n\x1b[999A\x1b[6n\x1b[?6l\x1b[6n\x1b[5;1H\x1b[999B\x1b[6n");
    for expected in [
        b"\x1b[1;1R".as_slice(),
        b"\x1b[3;6R",
        b"\x1b[1;6R",
        b"\x1b[1;1R",
        b"\x1b[5;1R",
    ] {
        assert_eq!(t.pop_output(), Some(OutputEvent::Reply(expected.to_vec())));
    }
    assert_eq!(t.pop_output(), None);
    assert!(t.history().is_empty());
}

#[test]
fn reverse_index_and_next_line_have_distinct_region_behavior() {
    let t = partitions(
        4,
        5,
        b"A\x1b[2;1HB\x1b[3;1HC\x1b[4;1HD\x1b[5;1HE\x1b[2;4r\x1b[2;2H\x1bMX\x1bEY",
    );
    assert_eq!(rows(t.screen()), ["A...", ".X..", "Y...", "C...", "E..."]);
    assert!(t.history().is_empty());
}

#[test]
fn only_full_primary_forward_scroll_enters_history() {
    let t = partitions(4, 3, b"A\r\nB\r\nC\x1bD");
    assert_eq!(rows(t.history()), ["A..."]);
    assert_eq!(rows(t.screen()), ["B...", "C...", "...."]);
    let t = partitions(4, 3, b"\x1b[?1049hA\r\nB\r\nC\x1bD");
    assert!(t.history().is_empty());
    let t = partitions(4, 3, b"A\r\nB\r\nC\x1b[H\x1bM");
    assert!(t.history().is_empty());
    assert_eq!(rows(t.screen()), ["....", "A...", "B..."]);
}

#[test]
fn saved_cursor_restores_origin_pending_wrap_and_has_per_buffer_ownership() {
    let t = partitions(
        4,
        4,
        b"\x1b[2;3r\x1b[?6hABCD\x1b7\x1b[?6l\x1b[?1049h\x1b[3;2H\x1b7\x1b[H\x1b8Z\x1b[?1049l\x1b8X",
    );
    assert_eq!(rows(t.screen()), ["....", "ABCD", "X...", "...."]);
    assert!(t.screen()[1].soft_wrapped());
    assert_eq!(
        t.cursor(),
        Cursor {
            row: 2,
            column: 1,
            wrap_pending: false
        }
    );
    let t = partitions(4, 3, b"\x1b[3;3H\x1b8X");
    assert_eq!(rows(t.screen()), ["X...", "....", "...."]);
}

#[test]
fn margins_defaults_invalid_values_and_pending_wrap_are_explicit() {
    let t = partitions(
        4,
        4,
        b"\x1b[2;3r\x1b[?6h\x1b[2;2H\x1b[3;2rX\x1b[2;99rY\x1b[0;0rZ",
    );
    assert_eq!(rows(t.screen()), ["Z...", "....", ".XY.", "...."]);
    let t = partitions(4, 2, b"ABCD\x1b[DX");
    assert_eq!(rows(t.screen()), ["ABXD", "...."]);
    let t = partitions(4, 1, b"AB\x1b[rC\nD");
    assert_eq!(rows(t.history()), ["ABC."]);
    assert_eq!(rows(t.screen()), ["...D"]);
}

#[test]
fn wide_clusters_survive_region_wrap_and_scrolling_boundary_links_are_cut() {
    let t = partitions(
        4,
        4,
        "HEAD\u{1b}[4;1HFOOT\u{1b}[2;3r\u{1b}[3;1Habc界".as_bytes(),
    );
    assert_eq!(rows(t.screen()), ["HEAD", "abc_", "界~..", "FOOT"]);
    assert!(t.screen()[1].soft_wrapped());
    assert!(t.history().is_empty());
    let mut t = partitions(4, 4, b"abcdefghijkl\x1b[2;3r\x1b[2;1H\x1bM");
    assert_eq!(rows(t.screen()), ["abcd", "....", "efgh", "...."]);
    assert!(!t.screen()[0].soft_wrapped());
    assert!(!t.screen()[2].soft_wrapped());
    t.resize(Size {
        columns: 8,
        lines: 4,
    })
    .unwrap();
    assert_eq!(
        rows(t.screen()),
        ["abcd....", "........", "efgh....", "........"]
    );
}

#[test]
fn application_cursor_mode_lists_are_atomic_and_encoding_tracks_current_mode() {
    let mut t = partitions(4, 4, b"\x1b[?1049;1h");
    assert!(t.is_alternate());
    for (key, final_byte) in [
        (Key::Up, b'A'),
        (Key::Down, b'B'),
        (Key::Left, b'D'),
        (Key::Right, b'C'),
        (Key::Home, b'H'),
        (Key::End, b'F'),
    ] {
        assert_eq!(t.encode_key(key), Some(vec![27, b'O', final_byte]));
    }
    feed(&mut t, b"\x1b[?1049l"); // Input mode is terminal-wide, not restored with the primary grid.
    assert_eq!(t.encode_key(Key::Up).unwrap(), b"\x1bOA");
    for bytes in [
        b"\x1b[?1;999l".as_slice(),
        b"\x1b[?1:6l",
        b"\x1b[?1;l",
        b"\x1b[?6;999h",
    ] {
        assert!(t.feed(bytes).unsupported);
        assert_eq!(t.encode_key(Key::Up).unwrap(), b"\x1bOA");
        assert_eq!(t.cursor(), Cursor::default());
    }
    feed(&mut t, b"\x1b[?1;6l");
    assert_eq!(t.encode_key(Key::Up).unwrap(), b"\x1b[A");
}

#[test]
fn resize_resets_both_margins_and_clamps_saved_physical_coordinates() {
    for alternate in [false, true] {
        let mut t = terminal(6, 5);
        feed(&mut t, b"\x1b[2;4r\x1b[?6h\x1b[3;6H\x1b7\x1b[?1h");
        if alternate {
            feed(&mut t, b"\x1b[?1049h\x1b[2;4r\x1b[?6h\x1b[3;6H\x1b7");
        }
        t.resize(Size {
            columns: 3,
            lines: 2,
        })
        .unwrap();
        feed(&mut t, b"\x1b8");
        assert_eq!(
            t.cursor(),
            Cursor {
                row: 1,
                column: 2,
                wrap_pending: false
            }
        );
        feed(&mut t, b"\x1b[H");
        assert_eq!(t.cursor(), Cursor::default());
        if alternate {
            feed(&mut t, b"\x1b[?1049l\x1b8");
            assert_eq!(t.cursor().row, 1);
            feed(&mut t, b"\x1b[H");
            assert_eq!(t.cursor(), Cursor::default());
        }
        assert_eq!(t.encode_key(Key::Up).unwrap(), b"\x1bOA");
        assert!(t.invariants_hold());
    }
}

#[test]
fn snapshots_only_damage_rows_that_changed_during_region_scroll() {
    let mut t = terminal(4, 5);
    feed(&mut t, b"A\x1b[2;1HB\x1b[3;1HC\x1b[4;1HD\x1b[5;1HE");
    let before = Snapshot::capture(&t, None).unwrap();
    feed(&mut t, b"\x1b[2;4r\x1b[?6h");
    let cursor = Snapshot::capture(&t, Some(&before)).unwrap();
    assert_eq!(cursor.row_versions(), before.row_versions());
    feed(&mut t, b"\x1b[3;1H\n");
    let after = Snapshot::capture(&t, Some(&cursor)).unwrap();
    assert_eq!(after.row_versions(), &[1, 3, 3, 3, 1]);
    assert_eq!(before.text(), b"ABCDE");
}

#[test]
fn scrolling_at_top_of_viewport_severs_stale_history_wrap_links() {
    for suffix in [b"\x1b[1;2r\x1b[2;1H\n".as_slice(), b"\x1b[H\x1bM"] {
        let mut input = b"abcdefghijklmnop".to_vec();
        input.extend_from_slice(suffix);
        let mut t = partitions(4, 3, &input);
        assert_eq!(rows(t.history()), ["abcd"]);
        assert!(!t.history()[0].soft_wrapped());
        t.resize(Size {
            columns: 8,
            lines: 4,
        })
        .unwrap();
        assert_eq!(rows(t.screen())[0], "abcd....");
    }
}

#[test]
fn region_cursor_modes_survive_mixed_geometry_and_byte_delivery() {
    let script = "\u{1b}[2;3r\u{1b}[?1;6h\u{1b}[99;3H界\u{1b}7\n\u{1b}M\u{1b}8\u{1b}[?1049h\u{1b}[2;3r\u{1b}[?6h👩‍💻\u{1b}7\u{1b}[?1049l".as_bytes();
    for split in 0..=script.len() {
        for size in [
            Size {
                columns: 2,
                lines: 1,
            },
            Size {
                columns: 7,
                lines: 6,
            },
        ] {
            let mut whole = terminal(5, 4);
            feed(&mut whole, &script[..split]);
            whole.resize(size).unwrap();
            feed(&mut whole, &script[split..]);
            let mut bytewise = terminal(5, 4);
            for byte in &script[..split] {
                feed(&mut bytewise, &[*byte]);
            }
            bytewise.resize(size).unwrap();
            for byte in &script[split..] {
                feed(&mut bytewise, &[*byte]);
            }
            assert_eq!(whole, bytewise, "resize at byte {split}");
        }
    }
}

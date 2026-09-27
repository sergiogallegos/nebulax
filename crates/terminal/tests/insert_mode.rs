use nebulax_terminal::{
    CellView, Limits, OutputEvent, Size, Terminal, WidthPolicy, snapshot::Snapshot, style::Style,
};

fn terminal(columns: usize, lines: usize) -> Terminal {
    Terminal::new(
        Size { columns, lines },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}
fn feed(t: &mut Terminal, bytes: &[u8]) {
    let out = t.feed(bytes);
    assert_eq!(out.consumed, bytes.len());
    assert!(!out.unsupported && !out.parser_limit && !out.style_limit);
    assert!(t.invariants_hold());
}
fn rows(t: &Terminal) -> Vec<String> {
    t.screen()
        .iter()
        .map(|r| {
            r.cells()
                .iter()
                .map(|c| match c.view() {
                    CellView::Lead { cluster, .. } => cluster.chars().collect::<String>(),
                    CellView::Continuation => "~".into(),
                    CellView::Empty => "_".into(),
                    CellView::WrapPadding => "|".into(),
                })
                .collect()
        })
        .collect()
}
fn partitions(columns: usize, lines: usize, input: &str) -> Terminal {
    let mut expected = terminal(columns, lines);
    let outcome = expected.feed(input.as_bytes());
    assert!(!outcome.unsupported && !outcome.output_blocked);
    for split in 0..=input.len() {
        let mut t = terminal(columns, lines);
        let mut out = t.feed(&input.as_bytes()[..split]);
        out.merge(t.feed(&input.as_bytes()[split..]));
        assert_eq!(out, outcome, "split {split}");
        assert_eq!(t, expected, "split {split}");
        assert!(t.invariants_hold());
    }
    let mut t = terminal(columns, lines);
    for byte in input.bytes() {
        feed(&mut t, &[byte]);
    }
    assert_eq!(t, expected);
    expected
}
fn reply(t: &mut Terminal) -> Vec<u8> {
    let Some(OutputEvent::Reply(bytes)) = t.pop_output() else {
        panic!("missing reply")
    };
    bytes
}

#[test]
fn ansi_mode_queries_and_printing_are_streaming_and_distinct_from_private_modes() {
    let mut t = partitions(
        8,
        2,
        "ABCDEF\x1b[3G\x1b[4$p\x1b[4;4h\x1b[4$pX\x1b[4lY\x1b[4$p\x1b[?4$p",
    );
    assert_eq!(rows(&t), ["ABXYDEF_", "________"]);
    for expected in [
        b"\x1b[4;2$y".as_slice(),
        b"\x1b[4;1$y",
        b"\x1b[4;2$y",
        b"\x1b[?4;0$y",
    ] {
        assert_eq!(reply(&mut t), expected);
    }
    assert!(!t.insert_mode());
    assert_eq!(t.cursor().column, 4);
}

#[test]
fn independent_owner_interval_oracle_covers_every_insert_position_and_wide_cut() {
    for columns in 2..=12 {
        for phase in 0..3 {
            let mut owners = Vec::new();
            let mut seed = String::new();
            let mut x = 0;
            while x < columns {
                let span = if (x + phase) % 3 == 0 && x + 1 < columns {
                    2
                } else {
                    1
                };
                let c = if span == 2 {
                    '語'
                } else {
                    char::from(b'a' + x as u8)
                };
                owners.push((x, span, c));
                seed.push(c);
                x += span;
            }
            for requested in 0..columns {
                for (width, c) in [(1, 'X'), (2, '界')] {
                    let at = requested.min(columns - width); // Existing no-wrap clamp policy.
                    let mut expected = vec!['_'; columns];
                    for &(x, span, old) in &owners {
                        let target = if x + span <= at {
                            Some(x)
                        } else if x >= at && x + span + width <= columns {
                            Some(x + width)
                        } else {
                            None
                        };
                        if let Some(to) = target {
                            expected[to] = old;
                            if span == 2 {
                                expected[to + 1] = '~';
                            }
                        }
                    }
                    expected[at] = c;
                    if width == 2 {
                        expected[at + 1] = '~';
                    }
                    let mut t = terminal(columns, 1);
                    feed(
                        &mut t,
                        format!("{seed}\x1b[?7l\x1b[{}G\x1b[4h{c}", requested + 1).as_bytes(),
                    );
                    assert_eq!(
                        rows(&t)[0],
                        expected.iter().collect::<String>(),
                        "columns={columns} at={requested} width={width}"
                    );
                    assert!(!t.cursor().wrap_pending && t.history().is_empty());
                }
            }
        }
    }
}

#[test]
fn grapheme_extensions_shift_only_width_delta_and_never_restore_evicted_cells() {
    for (cluster, expected, column) in [
        ("e\u{301}", "ABe\u{301}CDEFG", 3),
        ("❤\u{fe0f}", "AB❤\u{fe0f}~CDEF", 4),
        ("⌚\u{fe0e}", "AB⌚\u{fe0e}CDEF_", 3),
        ("👩‍💻", "AB👩‍💻~CDEF", 4),
        ("⌚\u{fe0e}‍🔥", "AB⌚\u{fe0e}‍🔥~CDEF", 4),
        ("e\x1b[31m\u{301}", "ABe\u{301}CDEFG", 3),
    ] {
        let t = partitions(8, 2, &format!("ABCDEFGH\x1b[3G\x1b[4h{cluster}"));
        assert_eq!(rows(&t)[0], expected);
        assert_eq!(t.cursor().column, column);
    }
}

#[test]
fn delayed_wrap_wide_relocation_and_disabled_wrap_keep_complete_owners() {
    for (input, expected) in [
        ("ABCD\x1b[4hX", ["ABCD", "X___"]),
        ("1234\x1b[2;1Habcd\x1b[1;4H\x1b[4h界", ["123|", "界~ab"]),
        (
            "1234\x1b[2;1Habcd\x1b[1;4H\x1b[4h❤\u{fe0f}",
            ["123|", "❤\u{fe0f}~ab"],
        ),
        (
            "1234\x1b[4G\x1b[?7l\x1b[4h❤\u{fe0f}",
            ["12❤\u{fe0f}~", "____"],
        ),
        (
            "1234\x1b[4G\x1b[?7l\x1b[4h⌚\u{fe0e}",
            ["12⌚\u{fe0e}_", "____"],
        ),
    ] {
        let t = partitions(4, 2, input);
        assert_eq!(rows(&t), expected);
    }
}

#[test]
fn global_mode_survives_save_screen_resize_but_both_resets_clear_it() {
    let mut t = terminal(8, 3);
    assert!(!t.insert_mode());
    feed(&mut t, b"\x1b[4h\x1b7\x1b[?1049h");
    assert!(t.insert_mode());
    feed(&mut t, b"\x1b[4l\x1b[?1049l\x1b8");
    assert!(!t.insert_mode());
    feed(&mut t, b"\x1b[4h");
    t.resize(Size {
        columns: 6,
        lines: 2,
    })
    .unwrap();
    assert!(t.insert_mode());
    for reset in [b"\x1b[!p".as_slice(), b"\x1bc"] {
        feed(&mut t, b"\x1b[4h\x1b[?1049h\x1b[4$p");
        feed(&mut t, reset);
        assert!(!t.insert_mode());
        assert_eq!(reply(&mut t), b"\x1b[4;1$y");
        feed(&mut t, b"\x1b[4$p");
        assert_eq!(reply(&mut t), b"\x1b[4;2$y");
    }
}

#[test]
fn invalid_lists_are_atomic_and_mode_changes_leave_rows_cursor_and_pending_wrap() {
    let mut t = terminal(4, 2);
    feed(&mut t, b"abcd");
    let old = Snapshot::capture(&t, None).unwrap();
    for bad in [
        b"\x1b[4;20h".as_slice(),
        b"\x1b[4;h",
        b"\x1b[h",
        b"\x1b[0h",
        b"\x1b[?4h",
        b"\x1b[4:1h",
        b"\x1b[4 h",
        b"\x1b[>4h",
    ] {
        assert!(t.feed(bad).unsupported);
        assert!(!t.insert_mode());
    }
    assert!(!t.feed(b"\x1b[4h").changed);
    assert!(t.insert_mode());
    assert!(t.feed(b"\x1b[4;20l").unsupported);
    assert!(t.insert_mode());
    let current = Snapshot::capture(&t, Some(&old)).unwrap();
    assert_eq!(current.row_versions(), old.row_versions());
    assert_eq!(current.cursor(), old.cursor());
    assert!(t.cursor().wrap_pending);
}

#[test]
fn insert_printing_preserves_incoming_wrap_and_region_history_rules() {
    let mut t = partitions(4, 3, "\x1b[4habcdefghij");
    assert!(t.screen()[0].soft_wrapped() && t.screen()[1].soft_wrapped());
    t.resize(Size {
        columns: 8,
        lines: 3,
    })
    .unwrap();
    assert_eq!(rows(&t), ["abcdefgh", "ij______", "________"]);
    let t = partitions(4, 4, "TOP\x1b[4;1HEND\x1b[2;3r\x1b[?6h\x1b[4habcdefghi");
    assert_eq!(rows(&t), ["TOP_", "efgh", "i___", "END_"]);
    assert!(t.history().is_empty());
    let t = partitions(4, 2, "\x1b[4habcdefghi");
    assert_eq!(t.history().len(), 1);
    assert!(t.history()[0].soft_wrapped());
    assert_eq!(rows(&t), ["efgh", "i___"]);
}

#[test]
fn shifts_preserve_styles_snapshots_buffers_and_release_evicted_clusters() {
    let mut t = terminal(8, 2);
    feed(
        &mut t,
        "\x1b[31mAB界CDe\u{301}f\x1b[2;1HKEEP\x1b[1;4H".as_bytes(),
    );
    let old = Snapshot::capture(&t, None).unwrap();
    let pointer = t.screen()[0].cells().as_ptr();
    feed(&mut t, b"\x1b[1;44m\x1b[4hXY");
    // Inserting in the wide tail clears its owner, then shifts that empty
    // tail twice. C and D move to columns 7/8; the two rightmost owners drop.
    assert_eq!(rows(&t), ["AB_XY_CD", "KEEP____"]);
    assert_eq!(pointer, t.screen()[0].cells().as_ptr());
    assert_eq!(t.storage_usage().cluster_bytes, 0);
    let current = Snapshot::capture(&t, Some(&old)).unwrap();
    assert_eq!(current.row_versions(), [2, 1]);
    let cells = t.screen()[0].cells();
    assert_eq!(
        t.style(cells[2].style_id()).unwrap(),
        Style {
            background: Style::indexed(4),
            ..Style::default()
        }
    );
    assert_eq!(t.style(cells[3].style_id()).unwrap().attributes, 1);
    assert_eq!(
        t.style(cells[6].style_id()).unwrap().foreground,
        Style::indexed(1)
    );
    drop(t);
    assert_eq!(
        std::str::from_utf8(old.text()).unwrap(),
        "AB界CDe\u{301}fKEEP"
    );
}

#[test]
fn controls_and_orphan_marks_do_not_insert_and_cluster_limit_stops_further_shifts() {
    let mut t = terminal(8, 2);
    feed(&mut t, b"ABCDEFGH\x1b[4h\r\t\x08\r");
    let before = rows(&t);
    assert!(t.feed("\u{301}".as_bytes()).orphan_mark);
    assert_eq!(rows(&t), before);
    let mut t = Terminal::new(
        Size {
            columns: 8,
            lines: 2,
        },
        Limits {
            cluster_scalars: 1,
            ..Limits::default()
        },
        WidthPolicy::default(),
    )
    .unwrap();
    feed(&mut t, "ABCDEFGH\x1b[3G\x1b[4h❤".as_bytes());
    assert!(t.feed("\u{fe0f}".as_bytes()).cluster_limit);
    assert_eq!(rows(&t)[0], "AB❤CDEFG");
    assert!(t.invariants_hold());
}

#[test]
fn sustained_mode_queries_capture_state_in_order_under_reply_pressure() {
    let mut t = terminal(8, 2);
    let bytes = b"\x1b[4h\x1b[4$p\x1b[4l\x1b[4$p".repeat(100);
    let mut offset = 0;
    let mut result = Vec::new();
    let mut blocked = false;
    while offset < bytes.len() {
        let out = t.feed(&bytes[offset..]);
        offset += out.consumed;
        blocked |= out.output_blocked;
        assert!(!out.unsupported);
        result.push(reply(&mut t));
    }
    while let Some(OutputEvent::Reply(bytes)) = t.pop_output() {
        result.push(bytes);
    }
    assert!(blocked);
    assert_eq!(result.len(), 200);
    for (i, bytes) in result.iter().enumerate() {
        assert_eq!(
            *bytes,
            format!("\x1b[4;{}$y", if i % 2 == 0 { 1 } else { 2 }).as_bytes()
        );
    }
    assert!(!t.insert_mode() && t.invariants_hold());
}

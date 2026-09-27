use nebulax_terminal::{
    CellView, Limits, Size, Terminal, WidthPolicy, snapshot::Snapshot, style::Style,
};
fn terminal(columns: usize, lines: usize) -> Terminal {
    Terminal::new(
        Size { columns, lines },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}
fn feed(t: &mut Terminal, input: &str) {
    let out = t.feed(input.as_bytes());
    assert_eq!(out.consumed, input.len());
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
    let result = expected.feed(input.as_bytes());
    assert!(!result.unsupported);
    for split in 0..=input.len() {
        let mut t = terminal(columns, lines);
        let mut out = t.feed(&input.as_bytes()[..split]);
        out.merge(t.feed(&input.as_bytes()[split..]));
        assert_eq!(out, result);
        assert_eq!(t, expected, "split {split}");
    }
    let mut t = terminal(columns, lines);
    for byte in input.bytes() {
        t.feed(&[byte]);
    }
    assert_eq!(t, expected);
    assert!(t.invariants_hold());
    expected
}
#[test]
fn character_counts_default_to_one_clamp_and_leave_cursor_in_place() {
    for (op, expected) in [
        ("@", "AB_CDEFG"),
        ("0@", "AB_CDEFG"),
        ("2@", "AB__CDEF"),
        ("P", "ABDEFGH_"),
        ("0P", "ABDEFGH_"),
        ("2P", "ABEFGH__"),
        ("65535@", "AB______"),
        ("65535P", "AB______"),
    ] {
        let t = partitions(8, 2, &format!("ABCDEFGH\x1b[3G\x1b[{op}"));
        assert_eq!(rows(&t), [expected, "________"], "{op}");
        assert_eq!(
            (t.cursor().row, t.cursor().column, t.cursor().wrap_pending),
            (0, 2, false)
        );
    }
}
#[test]
fn wide_shift_oracle_checks_every_cut_position_and_count_on_small_rows() {
    // Independent oracle maps complete original owner intervals to destinations.
    for width in 2..=12 {
        for phase in 0..3 {
            let mut owners = Vec::new();
            let mut input = String::new();
            let mut x = 0;
            while x < width {
                let wide = (x + phase) % 3 == 0 && x + 1 < width;
                let c = if wide {
                    '界'
                } else {
                    char::from(b'a' + x as u8)
                };
                let span = if wide { 2 } else { 1 };
                owners.push((x, span, c));
                input.push(c);
                x += span;
            }
            for start in 0..width {
                for requested in (1..=width + 1).chain([65535]) {
                    let count = requested.min(width - start);
                    for insert in [false, true] {
                        let mut expected = vec!['_'; width];
                        for &(x, span, c) in &owners {
                            let destination = if x + span <= start {
                                Some(x)
                            } else if insert && x >= start && x + span + count <= width {
                                Some(x + count)
                            } else if !insert && x >= start + count {
                                Some(x - count)
                            } else {
                                None
                            };
                            if let Some(to) = destination {
                                expected[to] = c;
                                if span == 2 {
                                    expected[to + 1] = '~';
                                }
                            }
                        }
                        let mut t = terminal(width, 1);
                        feed(
                            &mut t,
                            &format!(
                                "{input}\x1b[{}G\x1b[{requested}{}",
                                start + 1,
                                if insert { '@' } else { 'P' }
                            ),
                        );
                        assert_eq!(
                            rows(&t)[0],
                            expected.iter().collect::<String>(),
                            "width={width} start={start} count={requested} insert={insert}"
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn shifted_clusters_keep_styles_and_new_blanks_keep_only_background() {
    let mut t = partitions(8, 2, "\x1b[31mAe\u{301}界B\x1b[1;7;44m\x1b[2G\x1b[2@");
    assert_eq!(rows(&t)[0], "A__e\u{301}界~B_");
    for x in [0, 3, 4, 5, 6] {
        assert_eq!(
            t.style(t.screen()[0].cells()[x].style_id())
                .unwrap()
                .foreground,
            Style::indexed(1)
        );
    }
    for x in [1, 2] {
        assert_eq!(
            t.style(t.screen()[0].cells()[x].style_id()).unwrap(),
            Style {
                background: Style::indexed(4),
                ..Style::default()
            }
        );
    }
    feed(&mut t, "\x1b[5G\x1b[P");
    assert_eq!(rows(&t)[0], "A__e\u{301}_B__");
    assert_eq!(
        t.style(t.screen()[0].cells()[7].style_id())
            .unwrap()
            .background,
        Style::indexed(4)
    );
}
#[test]
fn line_counts_move_only_cursor_to_margin_and_preserve_column() {
    let seed = "A\x1b[2;1HB\x1b[3;1HC\x1b[4;1HD\x1b[5;1HE\x1b[2;4r\x1b[3;3H";
    for (op, expected) in [
        ("L", ["A___", "B___", "____", "C___", "E___"]),
        ("0L", ["A___", "B___", "____", "C___", "E___"]),
        ("M", ["A___", "B___", "D___", "____", "E___"]),
        ("0M", ["A___", "B___", "D___", "____", "E___"]),
        ("65535L", ["A___", "B___", "____", "____", "E___"]),
        ("65535M", ["A___", "B___", "____", "____", "E___"]),
    ] {
        let t = partitions(4, 5, &format!("{seed}\x1b[{op}"));
        assert_eq!(rows(&t), expected, "{op}");
        assert_eq!((t.cursor().row, t.cursor().column), (2, 2));
        assert!(t.history().is_empty());
    }
}
#[test]
fn outside_region_line_commands_are_noops_and_origin_resolves_physical_row() {
    let mut t = terminal(4, 5);
    feed(&mut t, "A\x1b[5;1HBCDE\x1b[2;4r\x1b[5;1HBCDE");
    let cursor = t.cursor();
    let before = t.screen().to_vec();
    for op in ["L", "M"] {
        let result = t.feed(format!("\x1b[{op}").as_bytes());
        assert!(!result.changed && !result.unsupported && !result.scrolled_without_history);
        assert_eq!(t.screen(), before);
        assert_eq!(t.cursor(), cursor);
    }
    feed(&mut t, "\x1b[?6hX\x1b[2;2HY\x1b[L");
    assert_eq!(rows(&t), ["A___", "X___", "____", "_Y__", "BCDE"]);
    assert_eq!((t.cursor().row, t.cursor().column), (2, 2));
}
#[test]
fn line_edits_reuse_buffers_release_dropped_tails_and_never_push_history() {
    let mut t = terminal(6, 3);
    feed(&mut t, "old\r\nA\r\nB\r\ne\u{301}");
    let history = t.history().clone();
    let mut pointers: Vec<_> = t.screen().iter().map(|r| r.cells().as_ptr()).collect();
    pointers.sort();
    assert!(t.storage_usage().cluster_bytes > 0);
    feed(&mut t, "\x1b[1;44m\x1b[H\x1b[L");
    assert_eq!(t.history(), &history);
    assert_eq!(t.storage_usage().cluster_bytes, 0);
    let mut after: Vec<_> = t.screen().iter().map(|r| r.cells().as_ptr()).collect();
    after.sort();
    assert_eq!(pointers, after);
    for cell in t.screen()[0].cells() {
        assert_eq!(
            t.style(cell.style_id()).unwrap(),
            Style {
                background: Style::indexed(4),
                ..Style::default()
            }
        );
    }
    feed(&mut t, "\x1b[?1049h\x1b[31m界\x1b[H\x1b[M\x1b[?1049l");
    assert_eq!(t.history(), &history);
    assert_eq!(rows(&t), ["______", "A_____", "B_____"]);
}
#[test]
fn structural_padding_is_removed_before_column_shifts_and_wraps_detach() {
    let mut t = partitions(3, 3, "ab界\x1b[1;1H\x1b[P");
    assert_eq!(rows(&t), ["b__", "界~_", "___"]);
    assert!(!t.screen()[0].soft_wrapped());
    t.resize(Size {
        columns: 6,
        lines: 3,
    })
    .unwrap();
    assert_eq!(rows(&t), ["b_____", "界~____", "______"]);
    let mut t = terminal(4, 4);
    feed(&mut t, "abcdefghijkl\x1b[1;1H\x1b[L");
    assert!(!t.screen()[0].soft_wrapped());
    assert!(t.screen()[1].soft_wrapped() && t.screen()[2].soft_wrapped());
    assert!(!t.screen()[3].soft_wrapped());
    feed(&mut t, "\x1b[2;1H\x1b[M");
    assert_eq!(rows(&t), ["____", "efgh", "ijkl", "____"]);
    assert!(t.screen()[1].soft_wrapped());
    assert!(!t.screen()[2].soft_wrapped());
}
#[test]
fn snapshots_keep_old_text_and_damage_only_changed_rows() {
    let mut t = terminal(6, 4);
    feed(
        &mut t,
        "A\x1b[2;1He\u{301}界\x1b[3;1HC\x1b[4;1HD\x1b[2;3r\x1b[2;1H",
    );
    let old = Snapshot::capture(&t, None).unwrap();
    feed(&mut t, "\x1b[L");
    let current = Snapshot::capture(&t, Some(&old)).unwrap();
    assert_eq!(current.row_versions(), [1, 2, 2, 1]);
    feed(&mut t, "\x1b[65535M");
    drop(t);
    assert_eq!(std::str::from_utf8(old.text()).unwrap(), "Ae\u{301}界CD");
    assert_eq!(std::str::from_utf8(current.text()).unwrap(), "Ae\u{301}界D");
}
#[test]
fn edits_cancel_pending_wrap_and_close_grapheme_attachment() {
    for op in ["@", "P", "L", "M"] {
        let mut t = terminal(4, 2);
        feed(&mut t, "abcd");
        feed(&mut t, &format!("\x1b[{op}"));
        assert_eq!(
            (t.cursor().row, t.cursor().column, t.cursor().wrap_pending),
            (0, 3, false)
        );
        assert!(t.feed("\u{301}".as_bytes()).orphan_mark);
        feed(&mut t, "X");
        assert_eq!(t.cursor().row, 0);
    }
}
#[test]
fn malformed_edit_commands_do_not_partially_change_state() {
    for op in [
        "\x1b[1;2@",
        "\x1b[?2P",
        "\x1b[2:1L",
        "\x1b[2 M",
        "\x1b[65536P",
    ] {
        let mut t = terminal(8, 3);
        feed(&mut t, "original");
        let before = t.screen().to_vec();
        let cursor = t.cursor();
        let result = t.feed(op.as_bytes());
        assert!(result.unsupported || result.parser_limit);
        assert_eq!(t.screen(), before);
        assert_eq!(t.cursor(), cursor);
    }
}

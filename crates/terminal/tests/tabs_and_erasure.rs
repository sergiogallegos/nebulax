use nebulax_terminal::{
    CellView, FeedOutcome, Limits, Size, Terminal, WidthPolicy, snapshot::Snapshot, style::Style,
};
fn terminal(columns: usize, lines: usize) -> Terminal {
    Terminal::new(
        Size { columns, lines },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}
fn feed(t: &mut Terminal, input: &str) -> FeedOutcome {
    let out = t.feed(input.as_bytes());
    assert_eq!(out.consumed, input.len());
    assert!(!out.unsupported && !out.parser_limit && !out.style_limit);
    assert!(t.invariants_hold());
    out
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
    let outcome = feed(&mut expected, input);
    for split in 0..=input.len() {
        let mut t = terminal(columns, lines);
        let mut result = t.feed(&input.as_bytes()[..split]);
        result.merge(t.feed(&input.as_bytes()[split..]));
        assert_eq!(result, outcome, "diagnostics split {split}");
        assert_eq!(t, expected, "state split {split}");
    }
    let mut t = terminal(columns, lines);
    for byte in input.bytes() {
        t.feed(&[byte]);
    }
    assert_eq!(t, expected);
    expected
}

#[test]
fn default_tabs_move_without_printing_or_scrolling_and_counts_clamp() {
    let mut t = partitions(20, 2, "A\tB\x1b[0IC\tD");
    assert_eq!(rows(&t), ["A_______B_______C__D", "____________________"]);
    assert!(t.cursor().wrap_pending);
    feed(&mut t, "\t");
    assert_eq!(
        (t.cursor().row, t.cursor().column, t.cursor().wrap_pending),
        (0, 19, false)
    );
    feed(&mut t, "\x1b[65535Z");
    assert_eq!(t.cursor().column, 0);
    feed(&mut t, "\x1b[65535I");
    assert_eq!(t.cursor().column, 19);
    assert!(t.history().is_empty());
    feed(&mut t, "\x1b[Z");
    assert_eq!(t.cursor().column, 16);
    feed(&mut t, "\x1b[0Z");
    assert_eq!(t.cursor().column, 8);
}
#[test]
fn custom_stops_clear_current_and_all_and_are_shared_between_screens() {
    let mut t = partitions(20, 3, "\x1b[3g\x1b[5G\x1bH\r\tA\x1b[?1049h\tB");
    assert_eq!(rows(&t)[0], "____B_______________");
    feed(&mut t, "\x1b[5G\x1b[g\x1b[?1049l\r\t");
    assert_eq!(t.cursor().column, 19);
    assert_eq!(rows(&t)[0], "____A_______________");
    feed(&mut t, "\x1b[1G\x1bH\x1b[19G\x1b[Z");
    assert_eq!(t.cursor().column, 0);
}
#[test]
fn stop_edits_are_not_saved_with_cursor_and_do_not_damage_rows() {
    let mut t = terminal(20, 2);
    feed(&mut t, "ABC\x1b7");
    let before = Snapshot::capture(&t, None).unwrap();
    feed(&mut t, "\x1b[3g\x1b[6G\x1bH\x1b8\t");
    assert_eq!(t.cursor().column, 5);
    let after = Snapshot::capture(&t, Some(&before)).unwrap();
    assert_eq!(before.row_versions(), after.row_versions());
    assert_eq!(before.text(), after.text());
}
#[test]
fn resize_retains_hidden_stops_and_clear_all_disables_future_defaults() {
    let mut t = terminal(100, 2);
    feed(&mut t, "\x1b[73G\x1b[g\x1b[76G\x1bH\x1b[H");
    t.resize(Size {
        columns: 10,
        lines: 2,
    })
    .unwrap();
    t.resize(Size {
        columns: 150,
        lines: 2,
    })
    .unwrap();
    feed(&mut t, "\x1b[65G\t");
    assert_eq!(t.cursor().column, 75);
    feed(&mut t, "\x1b[129G\t");
    assert_eq!(t.cursor().column, 136);
    feed(&mut t, "\x1b[3g\x1b[H");
    t.resize(Size {
        columns: 256,
        lines: 2,
    })
    .unwrap();
    feed(&mut t, "\t");
    assert_eq!(t.cursor().column, 255);
    assert!(t.storage_usage().tab_bytes <= 8192);
    let saved = t.clone();
    assert!(
        t.resize(Size {
            columns: 65_537,
            lines: 1
        })
        .is_err()
    );
    assert_eq!(t, saved);
}
#[test]
fn tab_storage_stays_bounded_at_maximum_geometry_and_after_clone_growth() {
    let mut t = terminal(2, 1);
    for columns in [65, 129, 257, 1025, 65_536] {
        t = t.clone();
        t.resize(Size { columns, lines: 1 }).unwrap();
        assert_eq!(t.storage_usage().tab_bytes, columns.div_ceil(64) * 8);
        feed(&mut t, "\x1b[H\x1b[65535I");
        assert_eq!(t.cursor().column, columns - 1);
    }
    t.resize(Size {
        columns: 2,
        lines: 1,
    })
    .unwrap();
    assert_eq!(t.storage_usage().tab_bytes, 8192);
}
#[test]
fn line_and_display_modes_have_independent_inclusive_expectations() {
    let seed = "\x1b[1;1HABCDEF\x1b[2;1HGHIJKL\x1b[3;1HMNOPQR\x1b[2;3H";
    for (operation, expected) in [
        ("K", ["ABCDEF", "GH____", "MNOPQR"]),
        ("0K", ["ABCDEF", "GH____", "MNOPQR"]),
        ("1K", ["ABCDEF", "___JKL", "MNOPQR"]),
        ("2K", ["ABCDEF", "______", "MNOPQR"]),
        ("J", ["ABCDEF", "GH____", "______"]),
        ("0J", ["ABCDEF", "GH____", "______"]),
        ("1J", ["______", "___JKL", "MNOPQR"]),
        ("2J", ["______", "______", "______"]),
    ] {
        let t = partitions(6, 3, &format!("{seed}\x1b[{operation}"));
        assert_eq!(rows(&t), expected, "{operation}");
        assert_eq!((t.cursor().row, t.cursor().column), (1, 2));
    }
}
#[test]
fn erasing_wide_halves_uses_current_background_and_preserves_other_cells() {
    for (mode, expected) in [(0, "a_____"), (1, "___b__"), (2, "______")] {
        let t = partitions(6, 2, &format!("a界b\x1b[3G\x1b[1;7;31;44m\x1b[{mode}K"));
        assert_eq!(rows(&t)[0], expected);
        for (i, c) in t.screen()[0].cells().iter().enumerate() {
            let erased = match mode {
                0 => i >= 1,
                1 => i <= 2,
                _ => true,
            };
            if erased {
                assert_eq!(
                    t.style(c.style_id()).unwrap(),
                    Style {
                        background: Style::indexed(4),
                        ..Style::default()
                    }
                );
            }
        }
    }
}
#[test]
fn display_erasure_ignores_scroll_margins_and_origin_but_keeps_modes() {
    let mut t = terminal(6, 4);
    feed(
        &mut t,
        "A\x1b[2;1HB\x1b[3;1HC\x1b[4;1HD\x1b[2;3r\x1b[?6h\x1b[1J",
    );
    assert_eq!(rows(&t), ["______", "______", "C_____", "D_____"]);
    assert_eq!(t.cursor().row, 1);
    feed(&mut t, "\x1b[2J\x1b[H");
    assert_eq!(t.cursor().row, 1);
    assert_eq!(rows(&t), ["______", "______", "______", "______"]);
}
#[test]
fn visible_clear_keeps_history_and_saved_line_clear_only_affects_active_primary() {
    let mut t = terminal(4, 2);
    feed(&mut t, "old\r\nmid\r\nnew");
    assert_eq!(t.history().len(), 1);
    let history = t.history().clone();
    feed(&mut t, "\x1b[2J");
    assert_eq!(t.history(), &history);
    feed(&mut t, "\x1b[?1049hALT\x1b[3J\x1b[?1049l");
    assert_eq!(t.history(), &history);
    feed(&mut t, "\x1b[Hkeep");
    let before = Snapshot::capture(&t, None).unwrap();
    let cursor = t.cursor();
    feed(&mut t, "\x1b[3J");
    assert!(t.history().is_empty());
    assert_eq!(t.cursor(), cursor);
    let after = Snapshot::capture(&t, Some(&before)).unwrap();
    assert_eq!(after.row_versions(), before.row_versions());
    assert_eq!(after.text(), before.text());
}
#[test]
fn erasures_cancel_pending_wrap_and_detach_erased_logical_boundaries() {
    let mut t = partitions(4, 3, "abcd\x1b[2KX");
    assert_eq!(rows(&t), ["___X", "____", "____"]);
    assert_eq!(t.cursor().row, 0);
    feed(&mut t, "Y\x1b[1K");
    assert!(!t.screen()[0].soft_wrapped());
    feed(&mut t, "\x1b[Habcde\x1b[1;3H\x1b[K");
    assert!(!t.screen()[0].soft_wrapped());
    t.resize(Size {
        columns: 8,
        lines: 3,
    })
    .unwrap();
    assert_eq!(rows(&t)[0], "ab______");
    assert_eq!(rows(&t)[1], "e_______");
}
#[test]
fn unsupported_erase_and_tab_parameters_never_apply_partial_operations() {
    for sequence in [
        "\x1b[3K",
        "\x1b[4J",
        "\x1b[?2J",
        "\x1b[?2K",
        "\x1b[2g",
        "\x1b[3;0g",
        "\x1b[1;2I",
        "\x1b[1:2Z",
        "\x1b[2 J",
    ] {
        let mut t = terminal(20, 3);
        feed(&mut t, "sentinel");
        let before = Snapshot::capture(&t, None).unwrap();
        let out = t.feed(sequence.as_bytes());
        assert!(out.unsupported, "{sequence:?}");
        let after = Snapshot::capture(&t, Some(&before)).unwrap();
        assert_eq!(before.text(), after.text());
        assert_eq!(before.cursor(), after.cursor());
        feed(&mut t, "\r\t");
        assert_eq!(t.cursor().column, 8);
    }
}
#[test]
fn tabs_close_grapheme_attachment_and_can_land_on_wide_continuations() {
    let mut t = partitions(12, 2, "\x1b[8G界\x1b[H\tX");
    assert_eq!(rows(&t)[0], "________X___");
    assert!(t.feed("\t\u{301}".as_bytes()).orphan_mark);
    assert!(t.invariants_hold());
}

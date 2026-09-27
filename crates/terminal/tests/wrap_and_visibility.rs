use nebulax_terminal::{CellView, Limits, Size, Terminal, WidthPolicy, snapshot::Snapshot};
fn terminal(columns: usize, lines: usize) -> Terminal {
    Terminal::new(
        Size { columns, lines },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}
fn feed(t: &mut Terminal, s: &str) {
    let out = t.feed(s.as_bytes());
    assert_eq!(out.consumed, s.len());
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
fn partitions(columns: usize, lines: usize, s: &str) -> Terminal {
    let mut expected = terminal(columns, lines);
    let out = expected.feed(s.as_bytes());
    assert!(!out.unsupported);
    for split in 0..=s.len() {
        let mut t = terminal(columns, lines);
        let mut result = t.feed(&s.as_bytes()[..split]);
        result.merge(t.feed(&s.as_bytes()[split..]));
        assert_eq!(result, out);
        assert_eq!(t, expected, "split {split}");
    }
    let mut t = terminal(columns, lines);
    for b in s.bytes() {
        t.feed(&[b]);
    }
    assert_eq!(t, expected);
    assert!(t.invariants_hold());
    expected
}
#[test]
fn defaults_and_disabled_wrap_overwrite_last_column_without_scrolling() {
    let t = terminal(4, 2);
    assert!(t.autowrap() && t.cursor_visible());
    let t = partitions(4, 2, "\x1b[?7labcdefghi");
    assert_eq!(rows(&t), ["abci", "____"]);
    assert_eq!(
        (t.cursor().row, t.cursor().column, t.cursor().wrap_pending),
        (0, 3, false)
    );
    assert!(t.history().is_empty() && !t.screen()[0].soft_wrapped());
}
#[test]
fn explicit_wrap_changes_cancel_pending_wrap_and_resume_after_next_edge_print() {
    let mut t = partitions(4, 2, "abcd\x1b[?7lX\x1b[?7hY");
    assert_eq!(rows(&t), ["abcY", "____"]);
    assert!(t.cursor().wrap_pending);
    feed(&mut t, "Z");
    assert_eq!(rows(&t), ["abcY", "Z___"]);
    let mut t = partitions(4, 2, "abcd\x1b[?7hX");
    assert_eq!(rows(&t), ["abcX", "____"]);
    feed(&mut t, "Y");
    assert_eq!(rows(&t)[1], "Y___");
}
#[test]
fn wide_characters_clamp_as_whole_owners_and_selectors_relocate_at_edge() {
    let t = partitions(4, 2, "\x1b[?7lABC界");
    assert_eq!(rows(&t), ["AB界~", "____"]);
    let t = partitions(4, 2, "\x1b[?7lABC❤\u{fe0f}");
    assert_eq!(rows(&t), ["AB❤\u{fe0f}~", "____"]);
    let t = partitions(4, 2, "\x1b[?7lABC⌚\u{fe0e}");
    assert_eq!(rows(&t), ["AB⌚\u{fe0e}_", "____"]);
    let t = partitions(4, 2, "\x1b[?7lABC界X\u{301}");
    assert_eq!(rows(&t), ["AB_X\u{301}", "____"]);
}
#[test]
fn wrap_off_policy_holds_across_geometries_styles_and_all_start_columns() {
    for width in 2..=12 {
        for start in 0..width {
            for text in ["界", "❤\u{fe0f}", "👩‍💻", "a\u{301}"] {
                let mut t = terminal(width, 1);
                feed(
                    &mut t,
                    &format!("\x1b[?7l\x1b[31;44m\x1b[{}G{text}", start + 1),
                );
                assert!(!t.cursor().wrap_pending && t.history().is_empty());
                assert_eq!(t.cursor().row, 0);
                let frame = Snapshot::capture(&t, None).unwrap();
                assert_eq!(std::str::from_utf8(frame.text()).unwrap(), text);
                assert!(!t.screen()[0].soft_wrapped());
            }
        }
    }
}
#[test]
fn saved_cursor_restores_wrap_and_pending_state_but_not_visibility() {
    let mut t = partitions(4, 2, "abcd\x1b7\x1b[?7;25l\x1b8");
    assert!(t.autowrap() && t.cursor().wrap_pending && !t.cursor_visible());
    feed(&mut t, "X");
    assert_eq!(rows(&t), ["abcd", "X___"]);
    feed(&mut t, "\x1b[?7l\x1b7\x1b[?7h\x1b8");
    assert!(!t.autowrap());
    let mut t = terminal(4, 2);
    feed(&mut t, "\x1b[?7l\x1b8");
    assert!(t.autowrap());
}
#[test]
fn alternate_screens_keep_wrap_independent_and_visibility_is_shared() {
    let mut t = partitions(4, 2, "\x1b[?7;25l\x1b[?1049h");
    assert!(t.autowrap() && !t.cursor_visible());
    feed(&mut t, "abcdX\x1b[?25h\x1b[?1049l");
    assert!(!t.autowrap() && t.cursor_visible());
    assert_eq!(rows(&t), ["____", "____"]);
    feed(&mut t, "abcdX");
    assert_eq!(rows(&t)[0], "abcX");
}
#[test]
fn resize_keeps_wrap_modes_and_saved_state_without_creating_pending_wrap() {
    let mut t = terminal(8, 3);
    feed(&mut t, "\x1b[?7lABCDEFGH\x1b7\x1b[?1049h\x1b[?7l12345678");
    for columns in [4, 12, 2, 8] {
        t.resize(Size { columns, lines: 3 }).unwrap();
        assert!(!t.autowrap() && !t.cursor().wrap_pending);
        assert!(t.invariants_hold());
    }
    feed(&mut t, "\x1b[?1049l\x1b8");
    assert!(!t.autowrap() && !t.cursor().wrap_pending);
}
#[test]
fn visibility_only_updates_leave_rows_unchanged_and_owned_frames_independent() {
    let mut t = terminal(8, 2);
    feed(&mut t, "hello");
    let old = Snapshot::capture(&t, None).unwrap();
    let out = t.feed(b"\x1b[?25l");
    assert!(out.changed);
    let hidden = Snapshot::capture(&t, Some(&old)).unwrap();
    assert!(!hidden.cursor_visible() && old.cursor_visible());
    assert_eq!(old.row_versions(), hidden.row_versions());
    assert_eq!(old.cursor(), hidden.cursor());
    assert!(!t.feed(b"\x1b[?25l").changed);
    assert!(t.feed(b"\x1b[?25h").changed);
    drop(t);
    assert!(!hidden.cursor_visible());
    assert_eq!(old.text(), hidden.text());
}
#[test]
fn malformed_mode_lists_are_atomic_for_wrap_visibility_and_screen_switches() {
    for s in [
        "\x1b[?7;25;999l",
        "\x1b[?25;1049;999h",
        "\x1b[?7:25l",
        "\x1b[?25 l",
    ] {
        let mut t = terminal(4, 2);
        feed(&mut t, "abcd");
        let before = t.cursor();
        let grid = t.screen().to_vec();
        assert!(t.feed(s.as_bytes()).unsupported);
        assert!(t.autowrap() && t.cursor_visible() && !t.is_alternate());
        assert_eq!(t.cursor(), before);
        assert_eq!(t.screen(), grid);
    }
}
#[test]
fn wrap_off_does_not_disable_explicit_region_scrolling() {
    let mut t = terminal(4, 4);
    feed(&mut t, "T\x1b[4;1HB\x1b[2;3r\x1b[?7l\x1b[3;1Habcdefghi");
    assert_eq!(rows(&t), ["T___", "____", "abci", "B___"]);
    feed(&mut t, "\r\nX");
    assert_eq!(rows(&t), ["T___", "abci", "X___", "B___"]);
    assert!(t.history().is_empty());
}

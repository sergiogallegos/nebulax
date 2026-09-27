use nebulax_terminal::{
    CellView, FeedOutcome, Limits, Size, Terminal, WidthPolicy, snapshot::Snapshot, style::*,
};
fn terminal(columns: usize, lines: usize) -> Terminal {
    Terminal::new(
        Size { columns, lines },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}
fn feed(t: &mut Terminal, s: &str) -> FeedOutcome {
    let result = t.feed(s.as_bytes());
    assert_eq!(result.consumed, s.len());
    assert!(t.invariants_hold());
    result
}
fn at(t: &Terminal, row: usize, column: usize) -> Style {
    t.style(t.screen()[row].cells()[column].style_id()).unwrap()
}
fn partitions(input: &str) -> Terminal {
    let mut expected = terminal(20, 4);
    let out = feed(&mut expected, input);
    assert!(!out.unsupported && !out.style_limit);
    for split in 0..=input.len() {
        let mut t = terminal(20, 4);
        let mut result = t.feed(&input.as_bytes()[..split]);
        result.merge(t.feed(&input.as_bytes()[split..]));
        assert_eq!(result, out);
        assert_eq!(t, expected, "split {split}");
    }
    let mut t = terminal(20, 4);
    for byte in input.bytes() {
        t.feed(&[byte]);
    }
    assert_eq!(t, expected);
    expected
}
fn rgb(n: usize) -> String {
    format!(
        "\x1b[38;2;{};{};{}m",
        (n >> 16) & 255,
        (n >> 8) & 255,
        n & 255
    )
}

#[test]
fn colors_and_attribute_resets_have_independent_handwritten_expectations() {
    let t = partitions(
        "\x1b[1;2;3;4;7;8;9;31;44mA\x1b[22;23;24;27;28;29mB\x1b[39;49mC\x1b[91;104mD\x1b[mE",
    );
    assert_eq!(
        at(&t, 0, 0),
        Style {
            foreground: Style::indexed(1),
            background: Style::indexed(4),
            attributes: BOLD | FAINT | ITALIC | UNDERLINE | INVERSE | HIDDEN | STRIKE
        }
    );
    assert_eq!(
        at(&t, 0, 1),
        Style {
            foreground: Style::indexed(1),
            background: Style::indexed(4),
            attributes: 0
        }
    );
    assert_eq!(at(&t, 0, 2), Style::default());
    assert_eq!(
        at(&t, 0, 3),
        Style {
            foreground: Style::indexed(9),
            background: Style::indexed(12),
            attributes: 0
        }
    );
    assert_eq!(at(&t, 0, 4), Style::default());
}
#[test]
fn indexed_rgb_colon_and_semicolon_forms_are_chunk_independent() {
    let t = partitions(
        "\x1b[38;5;255;48;2;1;2;3mA\x1b[38:2::4:5:6;48:5:17mB\x1b[38:2:0:7:8:9mC\x1b[38:2:10:11:12mD\x1b[4:2mE\x1b[4:0mF\x1b[21mG\x1b[24mH",
    );
    assert_eq!(
        at(&t, 0, 0),
        Style {
            foreground: Style::indexed(255),
            background: Style::rgb(1, 2, 3),
            attributes: 0
        }
    );
    assert_eq!(at(&t, 0, 1).foreground, Style::rgb(4, 5, 6));
    assert_eq!(at(&t, 0, 1).background, Style::indexed(17));
    assert_eq!(at(&t, 0, 2).foreground, Style::rgb(7, 8, 9));
    assert_eq!(at(&t, 0, 3).foreground, Style::rgb(10, 11, 12));
    assert_eq!(at(&t, 0, 4).attributes, DOUBLE_UNDERLINE);
    assert_eq!(at(&t, 0, 5).attributes, 0);
    assert_eq!(at(&t, 0, 6).attributes, DOUBLE_UNDERLINE);
    assert_eq!(at(&t, 0, 7).attributes, 0);
}
#[test]
fn malformed_or_unsupported_sgr_never_partially_changes_rendition() {
    for bad in [
        "1;999",
        "1;38;5;256",
        "1;38;2;1;2",
        "1;38;2;;2;3",
        "1;38:2:5:1:2:3",
        "1;38;2:1:2:3",
        "1;4:3",
        "1;38:5",
        "1;38:5:2:3",
        "1;5",
        "1;38;2;1;2;3:4",
    ] {
        let mut t = terminal(8, 2);
        feed(&mut t, "\x1b[32m");
        assert!(feed(&mut t, &format!("\x1b[{bad}m")).unsupported, "{bad}");
        assert_eq!(t.current_style().foreground, Style::indexed(2));
        assert_eq!(t.current_style().attributes, 0);
    }
    let t = partitions("\x1b[1;31mA\x1b[;32mB");
    assert_eq!(
        at(&t, 0, 1),
        Style {
            foreground: Style::indexed(2),
            ..Style::default()
        }
    );
}
#[test]
fn sgr_keeps_graphemes_and_pending_wrap_with_the_original_lead_style() {
    let t = partitions("e\x1b[31m\u{301}!👩\x1b[32m\u{200d}💻X");
    let CellView::Lead { cluster, .. } = t.screen()[0].cells()[0].view() else {
        panic!()
    };
    assert_eq!(cluster.chars().collect::<String>(), "e\u{301}");
    assert_eq!(at(&t, 0, 0), Style::default());
    assert_eq!(at(&t, 0, 1).foreground, Style::indexed(1));
    let CellView::Lead { cluster, width } = t.screen()[0].cells()[2].view() else {
        panic!()
    };
    assert_eq!(cluster.chars().collect::<String>(), "👩‍💻");
    assert_eq!(width, 2);
    assert_eq!(at(&t, 0, 2), at(&t, 0, 3));
    assert_eq!(at(&t, 0, 2).foreground, Style::indexed(1));
    assert_eq!(at(&t, 0, 4).foreground, Style::indexed(2));
    let mut t = terminal(2, 2);
    feed(&mut t, "ab\x1b[31m");
    assert!(t.cursor().wrap_pending);
    feed(&mut t, "c");
    assert_eq!(at(&t, 1, 0).foreground, Style::indexed(1));
}
#[test]
fn erasure_repairs_wide_owners_and_keeps_only_current_background() {
    let t = partitions("\x1b[31m界\x1b[1;3;7;44m\x1b[2G\x1b[X");
    for col in 0..2 {
        assert_eq!(t.screen()[0].cells()[col].view(), CellView::Empty);
        assert_eq!(
            at(&t, 0, col),
            Style {
                background: Style::indexed(4),
                ..Style::default()
            }
        );
    }
    let mut t = terminal(4, 2);
    feed(&mut t, "\x1b[44m\x1b[K\r\n\x1b[K\n");
    assert_eq!(
        t.style(t.history()[0].cells()[0].style_id())
            .unwrap()
            .background,
        Style::indexed(4)
    );
    assert_eq!(at(&t, 1, 3).background, Style::indexed(4));
    feed(&mut t, "\x1b[H\x1bM");
    assert_eq!(at(&t, 0, 3).background, Style::indexed(4));
}
#[test]
fn style_only_blank_rows_survive_reflow_and_width_changes_keep_styles() {
    let mut t = terminal(4, 3);
    feed(&mut t, "\x1b[41m\x1b[K\x1b[0m\x1b[H");
    t.resize(Size {
        columns: 2,
        lines: 3,
    })
    .unwrap();
    assert_eq!(at(&t, 0, 0).background, Style::indexed(1));
    assert_eq!(at(&t, 1, 1).background, Style::indexed(1));
    feed(&mut t, "\x1b[?1049h\x1b[41m\x1b[K");
    let cropped = t
        .resize(Size {
            columns: 2,
            lines: 1,
        })
        .unwrap();
    // Hidden primary loses a row containing two styled empty cells.
    assert_eq!(cropped.cropped_cells, 2);
    let mut t = terminal(3, 2);
    feed(&mut t, "ab\x1b[31m❤\x1b[32m\u{fe0f}");
    assert_eq!(at(&t, 1, 0).foreground, Style::indexed(1));
    assert_eq!(at(&t, 1, 1), at(&t, 1, 0));
    t.resize(Size {
        columns: 8,
        lines: 2,
    })
    .unwrap();
    assert!(t.invariants_hold());
    let lead = t.screen()[0]
        .cells()
        .iter()
        .find(|c| matches!(c.view(), CellView::Lead { width: 2, .. }))
        .unwrap();
    assert_eq!(
        t.style(lead.style_id()).unwrap().foreground,
        Style::indexed(1)
    );
}
#[test]
fn saved_rendition_hidden_primary_and_clones_are_gc_roots() {
    let mut t = terminal(8, 3);
    feed(&mut t, "\x1b[31;44m\x1b7\x1b[0m\x1b[?1049h");
    for n in 0..MAX_STYLES * 3 {
        assert!(!feed(&mut t, &rgb(n)).style_limit);
    }
    let mut copy = t.clone();
    feed(&mut copy, "\x1b[?1049l\x1b8X");
    assert_eq!(
        at(&copy, 0, 0),
        Style {
            foreground: Style::indexed(1),
            background: Style::indexed(4),
            attributes: 0
        }
    );
    feed(&mut t, "\x1b[?1049lX");
    assert_eq!(at(&t, 0, 0), Style::default());
}
#[test]
fn style_churn_reuses_bounded_slots_and_never_mutates_held_frames() {
    let mut t = terminal(8, 3);
    feed(&mut t, "\x1b[31mX");
    let old = Snapshot::capture(&t, None).unwrap();
    let held = *old.cell_style(&old.cells()[0]).unwrap();
    for n in 0..MAX_STYLES * 4 {
        let out = feed(&mut t, &format!("{}\rY", rgb(n)));
        assert!(!out.style_limit);
        assert!(t.retained_style_count() <= MAX_STYLES);
        assert!(t.storage_usage().style_bytes <= MAX_STYLES * std::mem::size_of::<Option<Style>>());
    }
    drop(t);
    assert_eq!(old.cell_style(&old.cells()[0]), Some(&held));
    assert_eq!(held.foreground, Style::indexed(1));
}
#[test]
fn exhaustion_is_atomic_and_recovers_after_cells_release_styles() {
    let mut t = terminal(64, 32);
    for n in 0..MAX_STYLES - 1 {
        assert!(!feed(&mut t, &format!("{}X", rgb(n))).style_limit);
    }
    let current = t.current_style();
    assert!(feed(&mut t, &rgb(0xabcdef)).style_limit);
    assert_eq!(t.current_style(), current);
    let before = Snapshot::capture(&t, None).unwrap();
    feed(&mut t, "\x1b[0m\x1b[H\x1b[X");
    assert!(!feed(&mut t, &rgb(0xabcdef)).style_limit);
    feed(&mut t, "X");
    let after = Snapshot::capture(&t, Some(&before)).unwrap();
    assert_eq!(before.cells()[0].style_id, after.cells()[0].style_id); // ID was reused.
    assert_ne!(
        before.cell_style(&before.cells()[0]),
        after.cell_style(&after.cells()[0])
    );
    assert_ne!(before.row_versions()[0], after.row_versions()[0]);
    assert_eq!(before.row_versions()[1], after.row_versions()[1]);
}
#[test]
fn color_only_edits_damage_rows_but_rendition_changes_alone_do_not() {
    let mut t = terminal(8, 3);
    feed(&mut t, "X");
    let a = Snapshot::capture(&t, None).unwrap();
    feed(&mut t, "\x1b[31m");
    let b = Snapshot::capture(&t, Some(&a)).unwrap();
    assert_eq!(a.row_versions(), b.row_versions());
    feed(&mut t, "\rX");
    let c = Snapshot::capture(&t, Some(&b)).unwrap();
    assert_eq!(c.row_versions(), [3, 1, 1]);
    assert_eq!(a.text(), c.text());
    assert!(c.payload_bytes() <= 2 * 1024 * 1024);
}

#[test]
fn two_slot_rendition_allocation_rolls_back_when_only_one_slot_is_free() {
    let mut t = terminal(64, 32);
    for n in 0..MAX_STYLES - 2 {
        assert!(!feed(&mut t, &format!("{}X", rgb(n))).style_limit);
    }
    let previous = t.current_style();
    // This needs both a full rendition and a background-only erase rendition.
    assert!(feed(&mut t, "\x1b[1;48;2;17;34;51m").style_limit);
    assert_eq!(t.current_style(), previous);
    feed(&mut t, "\x1b[H\x1b[X");
    assert!(!feed(&mut t, "\x1b[1;48;2;17;34;51m").style_limit);
    feed(&mut t, "\x1b[K");
    assert_eq!(
        at(&t, 0, 0),
        Style {
            background: Style::rgb(17, 34, 51),
            ..Style::default()
        }
    );
}

#[test]
fn history_and_hidden_cells_keep_their_styles_during_collection() {
    let mut t = terminal(4, 2);
    feed(&mut t, "\x1b[31mA\r\n\x1b[32mB\r\n\x1b[34mC\x1b[?1049h");
    for n in 0..MAX_STYLES * 2 {
        assert!(!feed(&mut t, &rgb(n)).style_limit);
    }
    feed(&mut t, "\x1b[?1049l");
    let historical = t.history()[0].cells()[0].style_id();
    assert_eq!(t.style(historical).unwrap().foreground, Style::indexed(1));
    assert_eq!(at(&t, 0, 0).foreground, Style::indexed(2));
    assert_eq!(at(&t, 1, 0).foreground, Style::indexed(4));
}

use nebulax_terminal::{Cell, FeedOutcome, Limits, Size, Terminal, WidthPolicy};

fn terminal(columns: usize, lines: usize) -> Terminal {
    Terminal::new(
        Size { columns, lines },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}

fn text(t: &Terminal, row: usize) -> String {
    t.screen()[row]
        .cells()
        .iter()
        .map(|c| match c {
            Cell::Lead { cluster, .. } => cluster.chars().collect(),
            Cell::Empty => " ".to_owned(),
            _ => String::new(),
        })
        .collect::<String>()
        .trim_end_matches(' ')
        .to_owned()
}

fn assert_partitions(base: Terminal, input: &[u8]) -> (Terminal, FeedOutcome) {
    let mut expected = base.clone();
    let expected_outcome = expected.feed(input);
    assert!(expected.invariants_hold());
    for split in 0..=input.len() {
        let mut actual = base.clone();
        let mut outcome = actual.feed(&input[..split]);
        assert!(actual.invariants_hold());
        outcome.merge(actual.feed(&input[split..]));
        assert_eq!(actual, expected, "split {split} for {input:x?}");
        assert_eq!(outcome, expected_outcome);
    }
    for n in [1, 2, 3, 7] {
        let mut actual = base.clone();
        let mut outcome = FeedOutcome::default();
        for part in input.chunks(n) {
            outcome.merge(actual.feed(part));
            assert!(actual.invariants_hold());
        }
        assert_eq!(actual, expected);
        assert_eq!(outcome, expected_outcome);
    }
    (expected, expected_outcome)
}

#[test]
fn width_changes_at_edge_repair_cursor_cells_and_wrap() {
    let (wide, _) = assert_partitions(terminal(5, 3), "abcd❤\u{fe0f}!".as_bytes());
    assert_eq!(text(&wide, 0), "abcd");
    assert_eq!(text(&wide, 1), "❤️!");
    assert_eq!(wide.cursor().column, 3);
    assert!(wide.screen()[0].soft_wrapped());
    assert_eq!(wide.screen()[0].cells()[4], Cell::WrapPadding);
    let (narrow, _) = assert_partitions(terminal(5, 3), "abc⌚\u{fe0e}!".as_bytes());
    assert_eq!(text(&narrow, 0), "abc⌚︎!");
    assert_eq!(narrow.cursor().row, 0);
    assert!(narrow.cursor().wrap_pending);
    assert!(matches!(
        narrow.screen()[0].cells()[3],
        Cell::Lead { width: 1, .. }
    ));
}

#[test]
fn erasing_or_overwriting_half_a_wide_cell_clears_the_owner() {
    for input in ["界\u{8}\u{1b}[X", "界\r\u{1b}[X", "界\rZ", "界\u{8}Z"] {
        let (t, _) = assert_partitions(terminal(6, 2), input.as_bytes());
        assert!(
            !t.screen()[0]
                .cells()
                .iter()
                .any(|c| matches!(c, Cell::Continuation))
        );
        assert!(!text(&t, 0).contains('界'));
    }
    let (t, _) = assert_partitions(terminal(6, 2), "界\u{8}Z".as_bytes());
    assert_eq!(text(&t, 0), " Z");
}

#[test]
fn controls_close_cluster_and_unsupported_sequences_are_visible() {
    let (t, out) = assert_partitions(terminal(8, 2), "e\r\u{301}!".as_bytes());
    assert_eq!(text(&t, 0), "!");
    assert!(out.orphan_mark);
    let (t, out) = assert_partitions(terminal(8, 2), "e\u{1b}[31m\u{301}!".as_bytes());
    assert!(out.unsupported && out.orphan_mark);
    assert_eq!(text(&t, 0), "e!"); // SGR is explicitly unsupported in this slice.
    let (t, out) = assert_partitions(terminal(8, 2), b"a\x1b[?1048hZ");
    assert!(out.unsupported);
    assert_eq!(text(&t, 0), "aZ");
}

#[test]
fn string_payloads_never_become_screen_or_native_effects() {
    for bytes in [
        b"\x1b]52;c;SECRET\x07OK".as_slice(),
        b"\x1bPSECRET\x1b\\OK",
        b"\x1b_SECRET\x1bXSTILL_SECRET\x1b\\OK",
        b"\x1b]SECRET\x18OK",
    ] {
        let (t, out) = assert_partitions(terminal(8, 2), bytes);
        assert_eq!(text(&t, 0), "OK");
        assert!(out.unsupported);
    }
}

#[test]
fn malformed_utf8_and_incomplete_input_do_not_depend_on_feeds() {
    for bytes in [
        b"A\xe2\x82!".as_slice(),
        b"\xed\xa0\x80Z",
        b"\xf4\x90\x80\x80",
        b"\xc0\xaf",
        b"\xe2\x1b[2DZ",
    ] {
        assert_partitions(terminal(16, 2), bytes);
    }
    let mut t = terminal(8, 2);
    assert!(!t.feed(&[0xf0, 0x9f]).changed);
    assert_eq!(text(&t, 0), "");
    t.feed(&[0x91, 0xa9]);
    assert_eq!(text(&t, 0), "👩");
    t.feed(&[0xe2, 0x82]);
    t.finish();
    assert_eq!(text(&t, 0), "👩�");
    assert!(!t.finish().changed);
}

#[test]
fn bounded_clusters_continue_segmentation_and_recover_at_next_boundary() {
    let limits = Limits {
        cluster_scalars: 3,
        ..Limits::default()
    };
    let base = Terminal::new(
        Size {
            columns: 8,
            lines: 2,
        },
        limits,
        WidthPolicy::default(),
    )
    .unwrap();
    let (t, out) = assert_partitions(base.clone(), "e\u{301}\u{302}\u{303}\u{304}!".as_bytes());
    assert!(out.cluster_limit);
    assert_eq!(text(&t, 0), "e\u{301}\u{302}!");
    let input = format!("e{}!", "\u{301}".repeat(20_000));
    let mut t = base;
    let out = t.feed(input.as_bytes());
    assert!(out.cluster_limit);
    assert_eq!(text(&t, 0), "e\u{301}\u{301}!");
    assert!(t.invariants_hold());
}

#[test]
fn parser_limits_discard_payload_and_recover_on_final_or_cancel() {
    let (t, out) = assert_partitions(terminal(8, 2), b"abc\x1b[999999999999999999D!");
    assert!(out.parser_limit);
    assert_eq!(text(&t, 0), "abc!");
    let input = format!("abc\x1b[{}D!", ";".repeat(100));
    let (t, out) = assert_partitions(terminal(8, 2), input.as_bytes());
    assert!(out.parser_limit);
    assert_eq!(text(&t, 0), "abc!");
    let mut t = terminal(8, 2);
    let input = format!("\x1b]{}\x1b\\OK", "x".repeat(100_000));
    assert!(t.feed(input.as_bytes()).unsupported);
    assert_eq!(text(&t, 0), "OK");
}

#[test]
fn geometry_caps_reject_overflow_and_bottom_scroll_reports_missing_history() {
    for size in [
        Size {
            columns: usize::MAX,
            lines: 2,
        },
        Size {
            columns: 2,
            lines: 0,
        },
        Size {
            columns: 1,
            lines: 2,
        },
        Size {
            columns: 1000,
            lines: 1000,
        },
    ] {
        assert!(Terminal::new(size, Limits::default(), WidthPolicy::default()).is_err());
    }
    let (t, out) = assert_partitions(
        Terminal::new(
            Size {
                columns: 4,
                lines: 1,
            },
            Limits {
                history_rows: 0,
                ..Limits::default()
            },
            WidthPolicy::default(),
        )
        .unwrap(),
        "abc❤\u{fe0f}!".as_bytes(),
    );
    assert!(out.scrolled_without_history);
    assert_eq!(text(&t, 0), "❤️!");
    assert!(t.invariants_hold());
}

#[test]
fn ambiguous_width_is_explicit_and_terminal_segmentation_handles_indic_and_hangul() {
    let mut narrow = terminal(16, 2);
    let mut wide = Terminal::new(
        Size {
            columns: 16,
            lines: 2,
        },
        Limits::default(),
        WidthPolicy {
            ambiguous_wide: true,
        },
    )
    .unwrap();
    narrow.feed("·".as_bytes());
    wide.feed("·".as_bytes());
    assert_eq!(narrow.cursor().column, 1);
    assert_eq!(wide.cursor().column, 2);
    let (t, _) = assert_partitions(terminal(16, 2), "각क्ष!".as_bytes());
    assert_eq!(text(&t, 0), "각क्ष!");
    assert_eq!(t.cursor().column, 4);
}

#[test]
fn deterministic_mixed_streams_preserve_state_invariants_for_each_byte() {
    let tokens: &[&[u8]] = &[
        b"a",
        b"\r",
        b"\n",
        b"\x08",
        b"\x1b[2D",
        b"\x1b[K",
        b"\x1b[1X",
        "界".as_bytes(),
        "👩‍💻".as_bytes(),
        "\u{fe0f}".as_bytes(),
        "\u{301}".as_bytes(),
        b"\xe2",
        b"\x1b[999999X",
        b"\x1b]ignore\x07",
        b"\x18",
        b"\x1b[?1049h",
    ];
    let mut seed = 0x1234_5678_u32;
    for columns in 2..=8 {
        let mut input = Vec::new();
        for _ in 0..1000 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            input.extend_from_slice(tokens[seed as usize % tokens.len()]);
        }
        let mut whole = terminal(columns, 3);
        let expected = whole.feed(&input);
        let mut fragmented = terminal(columns, 3);
        let mut actual = FeedOutcome::default();
        for byte in &input {
            actual.merge(fragmented.feed(&[*byte]));
            assert!(fragmented.invariants_hold());
        }
        assert_eq!(whole, fragmented);
        assert_eq!(expected, actual);
    }
}

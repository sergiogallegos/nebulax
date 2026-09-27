use nebulax_terminal::{Cell, Cursor, Limits, ResizeOutcome, Row, Size, Terminal, WidthPolicy};

fn new(columns: usize, lines: usize, history_rows: usize, history_cells: usize) -> Terminal {
    Terminal::new(
        Size { columns, lines },
        Limits {
            history_rows,
            history_cells,
            ..Limits::default()
        },
        WidthPolicy::default(),
    )
    .unwrap()
}

fn row_text(row: &Row) -> String {
    row.cells()
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

fn visible(t: &Terminal) -> Vec<String> {
    t.screen().iter().map(row_text).collect()
}
fn history(t: &Terminal) -> Vec<String> {
    t.history().iter().map(row_text).collect()
}

// Logical text oracle ignores structural padding/tails but preserves printed
// spaces and hard boundaries. Trim only unused trailing empty screen rows.
fn logical(t: &Terminal) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    for row in t.history().iter().chain(t.screen()) {
        let end = if row.soft_wrapped() {
            row.cells().len()
        } else {
            row.cells()
                .iter()
                .rposition(|c| !matches!(c, Cell::Empty | Cell::WrapPadding))
                .map_or(0, |n| n + 1)
        };
        for c in &row.cells()[..end] {
            match c {
                Cell::Lead { cluster, .. } => current.extend(cluster.chars()),
                Cell::Empty => current.push(' '),
                _ => {}
            }
        }
        if !row.soft_wrapped() {
            result.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    while result.last().is_some_and(String::is_empty) {
        result.pop();
    }
    result
}

#[test]
fn row_and_cell_history_limits_evict_oldest_physical_rows() {
    let mut t = new(4, 2, 10, 8); // Cell budget is the tighter bound: two history rows.
    let out = t.feed(b"one\r\ntwo\r\ntri\r\nfour\r\nfive");
    assert!(out.history_evicted);
    assert!(!out.scrolled_without_history);
    assert_eq!(history(&t), ["two", "tri"]);
    assert_eq!(visible(&t), ["four", "five"]);
    let mut row_limited = new(4, 2, 1, 100);
    row_limited.feed(b"one\r\ntwo\r\ntri\r\nfour");
    assert_eq!(history(&row_limited), ["two"]);
    assert!(row_limited.invariants_hold());
}

#[test]
fn disabled_history_is_explicit_and_limits_are_checked() {
    let mut t = new(4, 1, 0, 0);
    assert!(t.feed(b"a\r\nb").scrolled_without_history);
    assert!(t.history().is_empty());
    assert!(
        Terminal::new(
            Size {
                columns: 4,
                lines: 2
            },
            Limits {
                history_cells: usize::MAX,
                ..Limits::default()
            },
            WidthPolicy::default()
        )
        .is_err()
    );
}

#[test]
fn alternate_screen_has_no_history_and_restores_primary_pending_cursor() {
    let mut t = new(4, 2, 8, 64);
    t.feed(b"old\r\nrow\r\nmain");
    let before = (history(&t), visible(&t), t.cursor());
    assert!(t.cursor().wrap_pending);
    for _ in 0..4 {
        assert!(!t.feed(b"\x1b[?1049h").unsupported);
        assert!(t.is_alternate());
        assert_eq!(visible(&t), ["", ""]);
        assert!(t.history().is_empty());
        t.feed(b"alt");
        t.feed(b"\x1b[?1049h"); // Idempotent repeated enable doesn't overwrite primary.
        assert_eq!(visible(&t)[0], "alt");
        assert!(t.feed(b"\r\nx\r\ny").scrolled_without_history);
        t.feed(b"\x1b[?1049l\x1b[?1049l");
        assert!(!t.is_alternate());
        assert_eq!((history(&t), visible(&t), t.cursor()), before);
    }
}

#[test]
fn primary_reflows_while_inactive_alternate_crops_wide_edges() {
    let mut t = new(8, 3, 16, 128);
    t.feed(b"abcdef\x1b[?1049h");
    t.feed("ab界".as_bytes());
    let out = t
        .resize(Size {
            columns: 3,
            lines: 3,
        })
        .unwrap();
    assert_eq!(out.cropped_cells, 2);
    assert!(t.is_alternate());
    assert_eq!(visible(&t), ["ab", "", ""]);
    assert!(t.invariants_hold());
    t.feed(b"\x1b[?1049l");
    assert_eq!(visible(&t), ["abc", "def", ""]);
    assert_eq!(
        t.cursor(),
        Cursor {
            row: 1,
            column: 2,
            wrap_pending: true
        }
    );
    t.resize(Size {
        columns: 8,
        lines: 3,
    })
    .unwrap();
    assert_eq!(visible(&t), ["abcdef", "", ""]);
    assert_eq!(t.cursor().column, 6);
}

#[test]
fn narrow_wide_round_trips_preserve_clusters_spaces_and_hard_breaks() {
    let mut t = new(9, 4, 100, 4096);
    t.feed("abcd界X  👩‍💻🇺🇸!\r\n\r\ne\u{301}각⌚\u{fe0e}  end".as_bytes());
    let expected = logical(&t);
    assert_eq!(
        expected,
        ["abcd界X  👩‍💻🇺🇸!", "", "e\u{301}각⌚\u{fe0e}  end"]
    );
    for columns in [2, 3, 4, 16, 5, 9, 2, 9] {
        let out = t.resize(Size { columns, lines: 4 }).unwrap();
        assert_eq!(out, ResizeOutcome::default());
        assert_eq!(logical(&t), expected, "columns {columns}");
        assert!(t.invariants_hold());
    }
}

#[test]
fn height_shrink_moves_rows_to_history_and_growth_pulls_them_back() {
    let mut t = new(8, 3, 16, 128);
    t.feed(b"a\r\nb\r\nc");
    let cursor = t.cursor();
    t.resize(Size {
        columns: 8,
        lines: 1,
    })
    .unwrap();
    assert_eq!(history(&t), ["a", "b"]);
    assert_eq!(visible(&t), ["c"]);
    t.resize(Size {
        columns: 8,
        lines: 3,
    })
    .unwrap();
    assert!(t.history().is_empty());
    assert_eq!(visible(&t), ["a", "b", "c"]);
    assert_eq!(t.cursor(), cursor);
}

#[test]
fn exact_edge_cursor_survives_growth_narrowing_and_next_print() {
    let mut t = new(4, 3, 16, 128);
    t.feed(b"abcd");
    t.resize(Size {
        columns: 8,
        lines: 3,
    })
    .unwrap();
    assert_eq!(
        t.cursor(),
        Cursor {
            row: 0,
            column: 4,
            wrap_pending: false
        }
    );
    t.resize(Size {
        columns: 2,
        lines: 3,
    })
    .unwrap();
    assert_eq!(
        t.cursor(),
        Cursor {
            row: 1,
            column: 1,
            wrap_pending: true
        }
    );
    t.feed(b"!");
    assert_eq!(visible(&t), ["ab", "cd", "!"]);
    assert_eq!(logical(&t), ["abcd!"]);
}

#[test]
fn cursor_inside_a_wide_tail_remains_at_same_cluster_column() {
    let mut t = new(8, 3, 16, 128);
    t.feed("ab界\u{8}".as_bytes());
    assert_eq!(t.cursor().column, 3);
    t.resize(Size {
        columns: 3,
        lines: 3,
    })
    .unwrap();
    assert_eq!(visible(&t), ["ab", "界", ""]);
    assert_eq!(
        t.cursor(),
        Cursor {
            row: 1,
            column: 1,
            wrap_pending: false
        }
    );
    t.feed(b"\x1b[X");
    assert_eq!(visible(&t), ["ab", "", ""]);
    assert!(t.invariants_hold());
}

#[test]
fn resize_reevaluates_history_cell_budget_without_losing_cursor() {
    let mut t = new(2, 1, 10, 12);
    t.feed(b"0\r\n1\r\n2\r\n3\r\n4\r\n5\r\n");
    assert_eq!(t.history().len(), 6);
    let out = t
        .resize(Size {
            columns: 6,
            lines: 1,
        })
        .unwrap();
    assert_eq!(out.history_evicted, 4);
    assert_eq!(history(&t), ["4", "5"]);
    assert_eq!(t.cursor(), Cursor::default());
    assert!(t.invariants_hold());
}

#[test]
fn invalid_and_noop_resize_are_atomic_even_with_partial_input() {
    let mut t = new(8, 2, 16, 128);
    t.feed(b"main\x1b[?1049h\xf0\x9f");
    let before = t.clone();
    assert!(
        t.resize(Size {
            columns: usize::MAX,
            lines: 2
        })
        .is_err()
    );
    assert_eq!(t, before);
    assert_eq!(t.resize(t.size()).unwrap(), ResizeOutcome::default());
    assert_eq!(t, before);
    t.resize(Size {
        columns: 2,
        lines: 2,
    })
    .unwrap();
    t.feed(&[0x91, 0xa9]);
    assert_eq!(visible(&t), ["👩", ""]);
    t.feed(b"\x1b[?104");
    t.resize(Size {
        columns: 4,
        lines: 2,
    })
    .unwrap();
    t.feed(b"9l");
    assert!(!t.is_alternate());
    assert_eq!(visible(&t), ["main", ""]);
}

#[test]
fn malformed_private_modes_do_not_trigger_screen_switching() {
    for sequence in [
        b"\x1b[??1049h".as_slice(),
        b"\x1b[1049h",
        b"\x1b[?1049;999h", // A supported mode list now has separate positive coverage.
        b"\x1b[?1048h",
    ] {
        let mut t = new(8, 2, 16, 128);
        assert!(t.feed(sequence).unsupported);
        assert!(!t.is_alternate());
    }
}

#[test]
fn primary_crop_reports_loss_and_growth_does_not_restore_discarded_text() {
    let mut t = new(8, 2, 16, 128);
    t.feed(b"abcdefgh\x1b[7D");
    assert_eq!(
        t.resize(Size {
            columns: 2,
            lines: 1
        })
        .unwrap(),
        ResizeOutcome {
            history_evicted: 0,
            cropped_rows: 3,
            cropped_cells: 6
        }
    );
    assert_eq!(visible(&t), ["ab"]);
    assert_eq!(t.cursor(), Cursor::default());
    assert!(!t.screen()[0].soft_wrapped());
    assert_eq!(t.resize(t.size()).unwrap(), ResizeOutcome::default());
    assert_eq!(
        t.resize(Size {
            columns: 8,
            lines: 2
        })
        .unwrap(),
        ResizeOutcome::default()
    );
    assert_eq!(logical(&t), ["ab"]);
    t.feed(b"\r\nZ");
    assert_eq!(logical(&t), ["ab", "Z"]);
    t.resize(Size {
        columns: 2,
        lines: 1,
    })
    .unwrap();
    assert_eq!(logical(&t), ["ab", "Z"]);
}

#[test]
fn hidden_primary_crop_cannot_block_alternate_resize_or_partial_mode_input() {
    let mut t = new(8, 2, 16, 128);
    t.feed(b"abcdefgh\x1b[7D\x1b[?1049h");
    t.feed("ab界\r\nxyz\u{1b}[?104".as_bytes());
    assert_eq!(
        t.resize(Size {
            columns: 3,
            lines: 1
        })
        .unwrap(),
        ResizeOutcome {
            history_evicted: 0,
            cropped_rows: 3,
            cropped_cells: 10
        }
    );
    assert_eq!(
        t.size(),
        Size {
            columns: 3,
            lines: 1
        }
    );
    assert!(t.is_alternate());
    assert_eq!(visible(&t), ["ab"]);
    assert!(t.history().is_empty());
    assert!(t.invariants_hold());
    t.feed(b"9l");
    assert!(!t.is_alternate());
    assert_eq!(visible(&t), ["abc"]);
    assert_eq!(t.cursor(), Cursor::default());
    assert!(!t.screen()[0].soft_wrapped());
    assert_eq!(
        t.resize(Size {
            columns: 8,
            lines: 2
        })
        .unwrap(),
        ResizeOutcome::default()
    );
    assert_eq!(logical(&t), ["abc"]);
}

#[test]
fn primary_crop_removes_dangling_wide_padding_and_keeps_partial_utf8() {
    let mut t = new(8, 2, 16, 128);
    t.feed("ab界cd\r".as_bytes());
    t.feed(&[0xf0, 0x9f]);
    assert_eq!(
        t.resize(Size {
            columns: 3,
            lines: 1
        })
        .unwrap(),
        ResizeOutcome {
            history_evicted: 0,
            cropped_rows: 2,
            cropped_cells: 4
        }
    );
    assert_eq!(visible(&t), ["ab"]);
    assert_eq!(t.screen()[0].cells()[2], Cell::Empty);
    assert!(!t.screen()[0].soft_wrapped());
    t.feed(&[0x91, 0xa9]);
    assert_eq!(visible(&t), ["👩"]);
    assert!(t.invariants_hold());
    t.resize(Size {
        columns: 8,
        lines: 2,
    })
    .unwrap();
    assert_eq!(logical(&t), ["👩"]);
}

#[test]
fn primary_crop_and_history_eviction_are_counted_separately() {
    for cap in [0, 1, 4] {
        let mut t = new(8, 2, cap, 128);
        t.feed(b"H\r\nabcdefgh\x1b[3D");
        assert_eq!(
            t.resize(Size {
                columns: 2,
                lines: 1
            })
            .unwrap(),
            ResizeOutcome {
                history_evicted: 3_usize.saturating_sub(cap),
                cropped_rows: 1,
                cropped_cells: 2
            }
        );
        let expected_history = ["H", "ab", "cd"];
        assert_eq!(history(&t), expected_history[3_usize.saturating_sub(cap)..]);
        assert_eq!(visible(&t), ["ef"]);
        assert_eq!(t.cursor(), Cursor::default());
        assert!(!t.screen()[0].soft_wrapped());
        assert!(t.invariants_hold());
    }
}

#[test]
fn height_only_crop_closes_primary_and_alternate_wraps() {
    // A width change creates rows after the primary cursor without needing
    // unsupported cursor-up controls. Then exercise height-only shrinking.
    let mut t = new(8, 4, 16, 128);
    t.feed(b"abcdefgh\r");
    assert_eq!(
        t.resize(Size {
            columns: 2,
            lines: 4
        })
        .unwrap(),
        ResizeOutcome::default()
    );
    assert_eq!(
        t.resize(Size {
            columns: 2,
            lines: 2
        })
        .unwrap(),
        ResizeOutcome {
            history_evicted: 0,
            cropped_rows: 2,
            cropped_cells: 4
        }
    );
    assert_eq!(visible(&t), ["ab", "cd"]);
    assert!(t.screen()[0].soft_wrapped());
    assert!(!t.screen()[1].soft_wrapped());
    t.feed(b"\x1b[?1049habcd!");
    assert!(t.screen()[0].soft_wrapped());
    assert_eq!(
        t.resize(Size {
            columns: 2,
            lines: 1
        })
        .unwrap(),
        ResizeOutcome {
            history_evicted: 0,
            cropped_rows: 2,
            cropped_cells: 3
        }
    );
    assert_eq!(visible(&t), ["cd"]);
    assert!(!t.screen()[0].soft_wrapped());
    t.feed(b"\x1b[?1049l");
    assert_eq!(visible(&t), ["ab"]);
    assert_eq!(logical(&t), ["ab"]);
}

#[test]
fn cropped_spaces_and_multiscalar_wide_owners_have_exact_cell_counts() {
    let mut t = new(8, 2, 16, 128);
    t.feed("Ae\u{301} 👩‍💻 \r".as_bytes());
    assert_eq!(
        t.resize(Size {
            columns: 2,
            lines: 1
        })
        .unwrap(),
        ResizeOutcome {
            history_evicted: 0,
            cropped_rows: 3,
            cropped_cells: 4
        }
    );
    assert_eq!(visible(&t), ["Ae\u{301}"]);
    assert!(!t.screen()[0].soft_wrapped());
    assert!(t.feed("\u{301}".as_bytes()).orphan_mark);
    assert!(t.invariants_hold());
}

#[test]
fn repeated_resize_of_hidden_primary_matches_visible_primary() {
    let mut primary = new(8, 3, 1, 16);
    primary.feed("old\r\nab界cdEF\r".as_bytes());
    let mut hidden = primary.clone();
    hidden.feed(b"\x1b[?1049hTUI");
    for (columns, lines) in [(3, 1), (8, 4), (2, 1), (4, 2), (8, 3), (2, 1)] {
        primary.resize(Size { columns, lines }).unwrap();
        hidden.resize(Size { columns, lines }).unwrap();
        assert!(hidden.is_alternate());
        assert!(hidden.history().is_empty());
        // Observe a copy so the actual sequence keeps the primary hidden.
        let mut restored = hidden.clone();
        restored.feed(b"\x1b[?1049l");
        assert_eq!(restored.screen(), primary.screen());
        assert_eq!(restored.history(), primary.history());
        assert_eq!(restored.cursor(), primary.cursor());
        assert!(hidden.invariants_hold());
    }
}

#[test]
fn all_small_ascii_geometries_follow_cursor_anchored_crop_and_history_policy() {
    // Independent text/index oracle: no Screen or ReflowSink implementation.
    for old_columns in 2..=12 {
        let text: String = (0..old_columns).map(|i| (b'a' + i as u8) as char).collect();
        for cursor in 0..old_columns - 1 {
            for cap in [0, 1, 16] {
                let mut original = new(old_columns, 2, cap, 128);
                original.feed(text.as_bytes());
                original.feed(format!("\x1b[{}D", old_columns - 1 - cursor).as_bytes());
                for columns in 2..=12 {
                    for lines in 1..=5 {
                        let mut t = original.clone();
                        let packed: Vec<_> = text
                            .as_bytes()
                            .chunks(columns)
                            .map(|b| String::from_utf8(b.to_vec()).unwrap())
                            .collect();
                        let cursor_row = cursor / columns;
                        let end = packed.len().min(cursor_row + lines);
                        let start = end.saturating_sub(lines);
                        let evicted = start.saturating_sub(cap);
                        let out = t.resize(Size { columns, lines }).unwrap();
                        assert_eq!(
                            out,
                            ResizeOutcome {
                                history_evicted: evicted,
                                cropped_rows: packed.len() - end,
                                cropped_cells: old_columns.saturating_sub(end * columns),
                            }
                        );
                        assert_eq!(history(&t), packed[evicted..start]);
                        let mut expected = packed[start..end].to_vec();
                        expected.resize(lines, String::new());
                        assert_eq!(visible(&t), expected);
                        assert_eq!(
                            t.cursor(),
                            Cursor {
                                row: cursor_row - start,
                                column: cursor % columns,
                                wrap_pending: false
                            }
                        );
                        assert!(t.invariants_hold());
                        if out.cropped_rows > 0 {
                            assert!(!t.screen()[lines - 1].soft_wrapped());
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn extreme_aspect_ratio_reflow_materializes_only_bounded_retained_rows() {
    let mut t = new(2, 4096, 65_536, 65_536);
    t.feed("x\r\n".repeat(4095).as_bytes());
    let out = t
        .resize(Size {
            columns: 65_536,
            lines: 1,
        })
        .unwrap();
    assert_eq!(out.history_evicted, 4094);
    assert_eq!(history(&t), ["x"]);
    assert_eq!(visible(&t), [""]);
    assert!(t.invariants_hold());
}

#[test]
fn mixed_feed_resize_and_screen_switches_are_byte_partition_equivalent() {
    let mut whole = new(8, 3, 4, 32);
    let mut chunks = whole.clone();
    let inputs = [
        "abcd👩‍💻!\r\ne\u{301}",
        "\u{1b}[?1049hTUI界",
        "\u{1b}[?1049l\r\nnext",
        "⌚\u{fe0e}\u{8}\u{1b}[X",
    ];
    let mut seed = 19_u32;
    for i in 0..120 {
        let input = inputs[i % inputs.len()].as_bytes();
        let expected = whole.feed(input);
        let mut actual = nebulax_terminal::FeedOutcome::default();
        for b in input {
            actual.merge(chunks.feed(&[*b]));
            assert!(chunks.invariants_hold());
        }
        assert_eq!(expected, actual);
        assert_eq!(whole, chunks);
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let size = Size {
            columns: 2 + (seed as usize % 11),
            lines: 1 + ((seed >> 8) as usize % 5),
        };
        assert_eq!(whole.resize(size).unwrap(), chunks.resize(size).unwrap());
        assert_eq!(whole, chunks);
        assert!(whole.invariants_hold());
    }
}

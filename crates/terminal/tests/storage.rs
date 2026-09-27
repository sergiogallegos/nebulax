use nebulax_terminal::{Cell, CellView, Limits, Size, Terminal, WidthPolicy, snapshot::Snapshot};
use std::collections::BTreeSet;

fn terminal(columns: usize, lines: usize, history: usize) -> Terminal {
    Terminal::new(
        Size { columns, lines },
        Limits {
            history_rows: history,
            ..Limits::default()
        },
        WidthPolicy::default(),
    )
    .unwrap()
}
fn feed(t: &mut Terminal, text: &str) {
    let out = t.feed(text.as_bytes());
    assert_eq!(out.consumed, text.len());
    assert!(!out.unsupported && !out.parser_limit && !out.output_blocked);
    assert!(t.invariants_hold());
}
fn text(t: &Terminal) -> String {
    t.screen()
        .iter()
        .flat_map(|r| r.cells())
        .filter_map(|c| match c.view() {
            CellView::Lead { cluster, .. } => Some(cluster.chars().collect::<String>()),
            _ => None,
        })
        .collect()
}
fn buffers(t: &Terminal) -> BTreeSet<usize> {
    t.screen()
        .iter()
        .chain(t.history())
        .map(|r| r.cells().as_ptr() as usize)
        .collect()
}

#[test]
fn compact_cells_reserve_style_without_allocating_for_single_scalars() {
    assert_eq!(std::mem::size_of::<Cell>(), 16);
    let mut t = terminal(80, 40, 0);
    feed(&mut t, "ASCII 界");
    let usage = t.storage_usage();
    assert_eq!(usage.cell_bytes, 80 * 40 * 16);
    assert_eq!(usage.cluster_allocations, 0);
    assert_eq!(usage.cluster_bytes, 0);
    assert!(
        t.screen()
            .iter()
            .flat_map(|r| r.cells())
            .all(|c| c.style_id() == 0)
    );
}

#[test]
fn overwrite_and_wide_tail_erase_reclaim_all_side_capacity_immediately() {
    let mut t = terminal(8, 2, 0);
    feed(&mut t, "e\u{301}👩‍💻");
    assert_eq!(t.storage_usage().cluster_allocations, 4); // Header and scalar buffer for each lead.
    feed(&mut t, "\rX");
    assert_eq!(t.storage_usage().cluster_allocations, 2);
    feed(&mut t, "\x1b[3G\x1b[X"); // Erasing the wide tail drops the whole cluster.
    assert_eq!(t.storage_usage().cluster_allocations, 0);
    assert_eq!(t.storage_usage().cluster_bytes, 0);
    assert_eq!(text(&t), "X");
}

#[test]
fn clones_and_extensions_have_independent_bounded_tails_at_every_length() {
    for n in 1..64 {
        let original = format!("a{}", "\u{301}".repeat(n));
        let mut t = terminal(2, 1, 0);
        feed(&mut t, &original);
        let before = Snapshot::capture(&t, None).unwrap();
        let mut copy = t.clone();
        let extra = "\u{302}".repeat(100);
        let result = copy.feed(extra.as_bytes());
        assert!(result.cluster_limit);
        assert!(copy.invariants_hold());
        let CellView::Lead { cluster, .. } = copy.screen()[0].cells()[0].view() else {
            panic!()
        };
        assert_eq!(cluster.scalar_count(), 64);
        assert!(copy.storage_usage().cluster_bytes <= std::mem::size_of::<Vec<char>>() + 63 * 4);
        assert_eq!(text(&t), original);
        feed(&mut t, "\r\x1b[K");
        assert_eq!(t.storage_usage().cluster_bytes, 0);
        drop(t);
        assert_eq!(std::str::from_utf8(before.text()).unwrap(), original);
    }
}

#[test]
fn history_eviction_alternate_drop_and_resize_crop_release_owned_tails() {
    let mut t = terminal(4, 2, 1);
    feed(&mut t, "e\u{301}\r\na\u{301}\r\nb\u{301}");
    assert_eq!(t.storage_usage().cluster_allocations, 6);
    feed(&mut t, "\r\nx\r\ny\r\nz");
    assert_eq!(t.storage_usage().cluster_bytes, 0);
    feed(&mut t, "\x1b[?1049h👩‍💻\x1b[2;1He\u{301}");
    assert_eq!(t.storage_usage().cluster_allocations, 4);
    t.resize(Size {
        columns: 4,
        lines: 1,
    })
    .unwrap();
    assert_eq!(t.storage_usage().cluster_allocations, 2);
    feed(&mut t, "\x1b[?1049l");
    assert_eq!(t.storage_usage().cluster_bytes, 0);
    feed(&mut t, "\x1b[?1049h\x1b[4G👩‍💻\r\x1b[K");
    assert_eq!(t.storage_usage().cluster_bytes, 0);
}

#[test]
fn snapshots_own_exact_text_after_row_reuse_reflow_and_terminal_destruction() {
    let mut t = terminal(8, 3, 3);
    feed(&mut t, "e\u{301}👩‍💻\r\n界");
    let a = Snapshot::capture(&t, None).unwrap();
    let expected = a.text().to_vec();
    for _ in 0..20 {
        feed(&mut t, "abcdefgh\r\n");
    }
    t.resize(Size {
        columns: 3,
        lines: 2,
    })
    .unwrap();
    let b = Snapshot::capture(&t, Some(&a)).unwrap();
    assert_eq!(t.storage_usage().cluster_bytes, 0);
    drop(t);
    std::thread::spawn(move || {
        assert_eq!(a.text(), expected);
        assert_ne!(a.text(), b.text());
        assert_eq!(a.payload_bytes(), 8 * 3 * 12 + 3 * 9 + expected.len() + 12);
    })
    .join()
    .unwrap();
}

#[test]
fn steady_scroll_and_reverse_index_reuse_rows_without_retaining_cluster_tails() {
    for history in [0, 3] {
        let mut t = terminal(8, 4, history);
        for _ in 0..history + 4 {
            feed(&mut t, "e\u{301}👩‍💻\r\n");
        }
        let initial = buffers(&t);
        for _ in 0..100 {
            feed(&mut t, "abcdefgh\r\n");
        }
        assert_eq!(buffers(&t), initial);
        assert_eq!(t.storage_usage().cluster_bytes, 0);
        feed(&mut t, "\x1b[2;3r\x1b[3;1H");
        for _ in 0..100 {
            feed(&mut t, "\n\x1b[2;1H\x1bM\x1b[3;1H");
        }
        assert_eq!(buffers(&t), initial);
    }
}

#[test]
fn side_capacity_is_bounded_by_live_cells_after_mixed_edit_and_resize_churn() {
    let mut t = terminal(8, 4, 4);
    let cluster = format!("a{}", "\u{301}".repeat(63));
    for n in 0..100 {
        feed(&mut t, &cluster.repeat(6));
        feed(&mut t, "\x1b[2;1H\x1b[X\x1b[?1049h👩‍💻\x1b[?1049l\r\n");
        t.resize(Size {
            columns: 2 + n % 9,
            lines: 1 + n % 5,
        })
        .unwrap();
        let usage = t.storage_usage();
        let cells = t
            .screen()
            .iter()
            .chain(t.history())
            .map(|r| r.cells().len())
            .sum::<usize>();
        assert!(usage.cluster_allocations <= cells * 2);
        assert!(usage.cluster_bytes <= cells * (std::mem::size_of::<Vec<char>>() + 63 * 4));
        assert!(t.invariants_hold());
    }
}

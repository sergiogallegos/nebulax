//! Identical full-engine workload for before/after storage measurements.
use nebulax_terminal::{Cell, Limits, Row, Size, Terminal, WidthPolicy, snapshot::Snapshot};
use std::{hint::black_box, time::Instant};

fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}
fn line(kind: &str, columns: usize) -> Vec<u8> {
    let mut text = String::new();
    let mut x = 0;
    while x < columns {
        let (s, width) = match kind {
            "long" => (format!("a{}", "\u{301}".repeat(63)), 1),
            "dense" if x + 2 <= columns => ("👩‍💻".into(), 2),
            "mixed" if x % 16 == 0 => ("e\u{301}".into(), 1),
            "mixed" if x % 16 == 4 && x + 2 <= columns => ("👩‍💻".into(), 2),
            _ => ("x".into(), 1),
        };
        text.push_str(&s);
        x += width;
    }
    text.push_str("\r\n");
    text.into_bytes()
}
fn feed(t: &mut Terminal, bytes: &[u8]) {
    let out = t.feed(black_box(bytes));
    assert_eq!(out.consumed, bytes.len());
    assert!(!out.unsupported && !out.cluster_limit && !out.output_blocked);
}
fn main() {
    let scenario = std::env::args().nth(1).expect("scenario");
    let (kind, columns, history) = match scenario.as_str() {
        "visible" => ("ascii", 80, 0),
        "ascii" => ("ascii", 160, 384),
        "mixed" => ("mixed", 160, 256),
        "dense" => ("dense", 160, 128),
        "long" => ("long", 80, 128),
        _ => panic!("unknown scenario"),
    };
    let source = line(kind, columns);
    let ascii = line("ascii", columns);
    let mut t = Terminal::new(
        Size { columns, lines: 40 },
        Limits {
            history_rows: history,
            ..Limits::default()
        },
        WidthPolicy::default(),
    )
    .unwrap();
    let now = Instant::now();
    for _ in 0..history + 40 {
        feed(&mut t, &source);
    }
    let fill_ns = now.elapsed().as_nanos();
    let filled = t.storage_usage();
    let before = Snapshot::capture(&t, None).unwrap();
    let checksum = hash(before.text());
    let now = Instant::now();
    for _ in 0..200 {
        feed(&mut t, &source);
    }
    let scroll_ns = now.elapsed().as_nanos();
    let now = Instant::now();
    let frame = Snapshot::capture(&t, Some(&before)).unwrap();
    let snapshot_ns = now.elapsed().as_nanos();
    assert_eq!(hash(frame.text()), checksum);
    assert_eq!(frame.row_versions(), before.row_versions());
    let now = Instant::now();
    t.resize(Size {
        columns: columns / 2 + 1,
        lines: 40,
    })
    .unwrap();
    t.resize(Size { columns, lines: 40 }).unwrap();
    let resize_ns = now.elapsed().as_nanos();
    let resized = Snapshot::capture(&t, None).unwrap();
    let resized_checksum = hash(resized.text());
    for _ in 0..history + 41 {
        feed(&mut t, &ascii);
    }
    let reclaimed = t.storage_usage();
    assert_eq!(reclaimed.cluster_bytes, 0);
    assert_eq!(reclaimed.cluster_allocations, 0);
    assert!(t.invariants_hold());
    drop(t);
    assert_eq!(hash(frame.text()), checksum);
    println!(
        "{{\"scenario\":\"{scenario}\",\"cell_size\":{},\"row_size\":{},\"filled_bytes\":{},\"cell_bytes\":{},\"row_bytes\":{},\"cluster_bytes\":{},\"cluster_allocations\":{},\"reclaimed_bytes\":{},\"reclaimed_cluster_bytes\":{},\"snapshot_bytes\":{},\"checksum\":\"{checksum:016x}\",\"resized_checksum\":\"{resized_checksum:016x}\",\"fill_ns\":{fill_ns},\"scroll_ns\":{scroll_ns},\"resize_ns\":{resize_ns},\"snapshot_ns\":{snapshot_ns}}}",
        std::mem::size_of::<Cell>(),
        std::mem::size_of::<Row>(),
        filled.heap_bytes(),
        filled.cell_bytes,
        filled.row_bytes,
        filled.cluster_bytes,
        filled.cluster_allocations,
        reclaimed.heap_bytes(),
        reclaimed.cluster_bytes,
        frame.payload_bytes()
    );
}

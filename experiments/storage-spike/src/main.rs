use nebulax_storage_spike::{
    Allocations, Cell8, Cell16, DeepCell, Grid, Snapshot, StorageCell, input, layout,
};
use serde_json::{Value, json};
use std::hint::black_box;
use std::time::Instant;

fn allocations(a: Allocations) -> Value {
    json!({"container_growths": a.growths, "requested_capacity_bytes": a.requested_bytes})
}
fn run<C: StorageCell>(variant: &str, scenario: &str, chunked: bool) -> Value {
    let (kind, columns, rows, scrolls) = match scenario {
        "visible" => ("ascii", 80, 40, 200),
        "history_ascii" => ("ascii", 200, 10040, 1000),
        "history_mixed" => ("mixed", 160, 2040, 500),
        "history_dense" => ("dense", 160, 1040, 500),
        "cluster_limit" => ("long", 80, 168, 200),
        _ => panic!("unknown scenario"),
    };
    let source = input(kind, columns);
    let ascii = input("ascii", columns);
    let mut grid = Grid::<C>::new(columns, rows, chunked);
    let start = Instant::now();
    for _ in 0..rows {
        grid.push(black_box(&source));
    }
    let fill_ns = start.elapsed().as_nanos();
    let filled_bytes = grid.heap_bytes();
    let fill_allocations = grid.stats;
    let initial_checksum = grid.checksum();

    let start = Instant::now();
    for _ in 0..scrolls {
        grid.push(black_box(&source));
    }
    let scroll_ns = start.elapsed().as_nanos();
    let scroll_allocations = grid.stats.since(fill_allocations);
    assert_eq!(initial_checksum, grid.checksum());
    let slots_after_scroll = grid.arena_slots();

    let start = Instant::now();
    let (frame, snapshot_allocations) = grid.snapshot(40, None);
    let full_snapshot_ns = start.elapsed().as_nanos();
    let snapshot_checksum = frame.checksum();
    let full_snapshot_bytes = Snapshot::retained_bytes(&[&frame]);
    let start = Instant::now();
    let (unchanged, unchanged_allocations) = grid.snapshot(40, Some(&frame));
    let unchanged_snapshot_ns = start.elapsed().as_nanos();
    assert_eq!(unchanged.checksum(), snapshot_checksum);
    let unchanged_two_frames_bytes = Snapshot::retained_bytes(&[&frame, &unchanged]);
    drop(unchanged);

    let mut changed = ascii.clone();
    changed.atoms[0].text = vec!['Z'];
    grid.replace(rows - 1, &changed, false);
    let start = Instant::now();
    let (updated, changed_allocations) = grid.snapshot(40, Some(&frame));
    let changed_snapshot_ns = start.elapsed().as_nanos();
    assert_ne!(updated.checksum(), snapshot_checksum);
    assert_eq!(frame.checksum(), snapshot_checksum);
    let changed_two_frames_bytes = Snapshot::retained_bytes(&[&frame, &updated]);
    grid.replace(rows - 1, &source, false);

    let baseline_clone = if !chunked {
        let start = Instant::now();
        let (bytes, stats) = grid.baseline_clone_drop(40);
        Some(
            json!({"clone_and_drop_ns": start.elapsed().as_nanos(), "capacity_bytes": bytes, "allocations": allocations(stats)}),
        )
    } else {
        None
    };

    let before_repack = grid.stats;
    let start = Instant::now();
    let first_peak = grid.repack(columns / 2 + 1);
    let second_peak = grid.repack(columns);
    let repack_roundtrip_ns = start.elapsed().as_nanos();
    let repack_allocations = grid.stats.since(before_repack);
    assert_eq!(initial_checksum, grid.checksum());
    assert_eq!(snapshot_checksum, frame.checksum());
    let repack_peak_bytes = first_peak.max(second_peak);

    // Evict all cluster content via replacement, then explicitly trim free text.
    for row in 0..grid.len() {
        grid.replace(row, &ascii, false);
    }
    assert_eq!(grid.live_clusters(), 0);
    let after_ascii_bytes = grid.heap_bytes();
    let start = Instant::now();
    grid.trim_free_text();
    let trim_ns = start.elapsed().as_nanos();
    let after_trim_bytes = grid.heap_bytes();
    // Snapshots retain text despite source reclamation and can outlive the grid.
    drop(grid);
    assert_eq!(frame.checksum(), snapshot_checksum);
    assert_ne!(updated.checksum(), snapshot_checksum);

    json!({
        "schema_version": 1, "variant": variant, "scenario": scenario,
        "columns": columns, "retained_rows": rows, "visible_rows": 40, "scrolls": scrolls,
        "layout": layout(), "checksum": format!("{initial_checksum:016x}"),
        "snapshot_checksum": format!("{snapshot_checksum:016x}"),
        "capacity_bytes": {"filled_grid": filled_bytes, "snapshot_full": full_snapshot_bytes,
            "two_unchanged_frames": unchanged_two_frames_bytes, "two_frames_one_changed_row": changed_two_frames_bytes,
            "repack_peak_grid": repack_peak_bytes, "after_ascii_replacement": after_ascii_bytes, "after_trim": after_trim_bytes},
        "arena_slots_after_scroll": slots_after_scroll,
        "timings_ns": {"fill": fill_ns, "scroll_batch": scroll_ns, "snapshot_full": full_snapshot_ns,
            "snapshot_unchanged": unchanged_snapshot_ns, "snapshot_one_changed_row": changed_snapshot_ns,
            "repack_roundtrip": repack_roundtrip_ns, "trim_free_text": trim_ns},
        "allocations": {"fill": allocations(fill_allocations), "scroll_batch": allocations(scroll_allocations),
            "snapshot_full": allocations(snapshot_allocations), "snapshot_unchanged": allocations(unchanged_allocations),
            "snapshot_one_changed_row": allocations(changed_allocations), "repack_roundtrip": allocations(repack_allocations)},
        "baseline_deep_clone": baseline_clone,
        "checks": {"scroll_content_preserved": true, "repack_roundtrip_preserved": true,
            "snapshot_survives_mutation_reclamation_and_grid_drop": true}
    })
}
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!(
            "usage: nebulax-storage-spike baseline|compact8|compact16 visible|history_ascii|history_mixed|history_dense|cluster_limit"
        );
        std::process::exit(2);
    }
    let report = match args[0].as_str() {
        "baseline" => run::<DeepCell>(&args[0], &args[1], false),
        "compact8" => run::<Cell8>(&args[0], &args[1], true),
        "compact16" => run::<Cell16>(&args[0], &args[1], true),
        _ => {
            eprintln!("unknown variant");
            std::process::exit(2);
        }
    };
    println!("{}", serde_json::to_string(&report).unwrap());
}

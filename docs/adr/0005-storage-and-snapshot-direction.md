# 0005 — Storage and snapshot experiment direction

Status: measured Phase 0 direction on 2026-09-26; implementation candidate, not a final cell layout, public ABI or completed product migration. The [experiment findings](../../research/2026-09-26-storage-snapshot-spike.md) contain method, raw evidence, counterexamples and limits.

## Direction

Use a **16-byte candidate cell** for subsequent integration design, keeping the measured 8-byte format as an alternative. Both compact formats carry a scalar/cluster reference and style ID; the 16-byte version also leaves explicit hyperlink/attribute fields. This is a complexity/feature-space preference, not a claim of universal memory or speed superiority. Style dictionaries and their churn still need measurement before finalizing the representation.

Prefer lazy row chunks with reuse on scrolling, explicit row identity/generation, and immutable visible snapshots whose text remains valid after engine mutation or destruction. Snapshots should reuse unchanged rows and be bounded by bytes and in-flight frame count. Their ownership contract must allow shaping/presentation without holding an engine borrow/lock. The prototype uses Arc internally; no Rust layout or Arc pointer is a promised Swift ABI.

**Do not adopt the prototype's global per-cluster slot arena unchanged.** It can lose to the baseline on dense/long clusters and retain metadata after text eviction. Production side storage needs reclaimable ownership units, total byte accounting including free capacity, and defined overflow behavior. Private slots must not escape as unvalidated snapshot handles. Snapshot text allocation also needs deliberate sizing: geometric capacity growth lost to native deep cloning in the maximum-cluster case.

The experiment's initial four-million-cell bound is only a research workload limit. Product geometry/history caps, OOM behavior, generation wraparound and resource recovery remain separate design work. No caps are removed by this decision.

## Consequences

The current dependency-free terminal core remains unchanged; preserve its behavior fixtures while migrating later. The spike establishes storage tradeoffs and snapshot lifetime feasibility, but does not prove style correctness, FFI teardown, RSS, renderer cost or terminal throughput. Its safe Rust and reused research-only dependencies do not change the product dependency policy.

Next define generic parser events and bounded replies/effects, with terminal semantics separate from syntax. Carry row/snapshot ownership constraints into the early PTY/native integration. Final storage migration must address the measured retention/over-allocation cases before broad SGR coverage; it is not necessary to finish a standalone optimized storage framework before testing integration.

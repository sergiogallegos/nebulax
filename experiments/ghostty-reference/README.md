# Isolated Ghostty reference

Research only, approved in [ADR 0002](../../docs/adr/0002-owned-rust-engine-and-dependency-policy.md). Ghostty is not a Nebulax product dependency. This directory contains original adapter/orchestration code; downloaded upstream source, dependencies and binaries stay under ignored `target/`.

## Reproduce on this Mac

Use stable Zig 0.16.0, the recorded Xcode toolchain and Python 3.14.7. The build downloads the approved commit and hash-pinned build dependencies; network permission may be needed. No global installation is performed.

```sh
python3 experiments/ghostty-reference/build.py
GHOSTTY_REFERENCE_ADAPTER=target/ghostty-reference/adapter scripts/verify
python3 experiments/ghostty-reference/replay.py \
  --adapter target/ghostty-reference/adapter \
  --build-manifest target/ghostty-reference/build.json \
  --output target/ghostty-reference/replay-unicode
python3 experiments/ghostty-reference/replay.py \
  --adapter target/ghostty-reference/adapter \
  --build-manifest target/ghostty-reference/build.json \
  --legacy --record-differences \
  --output target/ghostty-reference/replay-legacy
```

Choose new output directories for every run. `build.py --archive /path/to/archive.tar.gz` can reuse the exact source archive; its SHA-256 is checked. The build verifies all 5,901 regular upstream files against the archive before and after compilation, and checks upstream symlinks. It records commands, toolchains, static-library and executable hashes, package archive hashes and discovered license-file hashes. These are provenance checks, not a release license audit or an assurance about build reproducibility on other hosts.

## Measurement boundary

The adapter uses the pinned full-terminal C API. A terminal is created and freed for each replay; no grid reference survives a mutation. It sets the fixture history line limit and explicitly sets/reads DEC mode 2027. Ghostty's history limit is approximate at page granularity; these small fixtures do not exercise pruning. Other library defaults are retained. No PTY, shell, UI, clipboard or external-effects callbacks are installed.

For each fixture, run whole delivery, a repeated whole delivery, chunks of 1/2/3/7 bytes and every interior two-part split index of the longest feed (clamped for shorter feeds). Snapshot after each **fixture operation**, not every delivery chunk. This is the same 244-replay schedule used by the Alacritty baseline.

Raw snapshots include active-screen cells as codepoints plus width roles, history rows, soft-wrap rows, dimensions, cursor, pending wrap, active-screen ID, mode 2027 and VT processing-error status. Compare those raw checkpoints for delivery equivalence. This does not cover styles, hyperlinks, other modes, inactive-screen internals, parser internals or external effects. It is narrower than the Alacritty internal snapshot and does not claim full-state equivalence.

Normalize only equivalent semantics for the existing independent fixture expectations: width roles map to fixture bits 32/64/1024; a soft-wrap row maps to bit 16 on its last cell. Empty codepoint content becomes a blank; spacer cells are excluded from row text; trailing ASCII spaces are trimmed. Native packed flags are never compared between engines. Raw codepoints and roles remain in the report so this mapping can be inspected.

Strict replay exits 1 for any unmet expectation or delivery difference. `--record-differences` allows research capture of expectation differences but still fails delivery differences. It is not the Alacritty fingerprint-based known-gap gate and must not be used as a regression pass criterion. Processing errors, unexpected mode changes, crashes, diagnostics or incomplete snapshots fail the run.

Default workspace verification runs two Python detection/normalization tests and skips one live test unless `GHOSTTY_REFERENCE_ADAPTER` is set. The live test checks grapheme-mode control, repeated create/feed/read/free and malformed adapter input. The reference is not downloaded or compiled by default CI.

See the [executed comparison](../../research/2026-09-26-ghostty-reference-comparison.md) and [owned Rust design](../../docs/architecture/OWNED_ENGINE_SLICE.md).

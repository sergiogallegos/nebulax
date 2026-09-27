# PTY lifecycle experiment

One macOS PTY connects a deterministic child to the owned Rust engine. This is research infrastructure, not an interactive terminal app. `Session` owns the nonblocking master, direct child and `Pump`; `Pump` owns the single authoritative terminal, a fixed 4,096-byte unread buffer, and one output event. The low-level session owns no thread. The later `worker` module owns this session on one thread and publishes bounded owned frames; no async framework or second authoritative grid is introduced.

```sh
cargo test -p nebulax-pty-session --locked
scripts/pty-session --output target/pty-session/manual
```

The recorder requires macOS. Nine safe pump/queue tests also run on other hosts; sixteen OS/lifecycle/worker/input/shell tests are macOS-only. Python 3 is only a synthetic test child, using standard-library PTY/signal APIs. It checks controlling-terminal/session/foreground identity and inherited descriptors, and has a ten-second alarm. Tests have five-second progress deadlines. The separate clean Bash test disables startup files and uses `/dev/null` history. No user shell configuration is read.

## Ownership and flow

`tick` performs at most 64 pump operations using nonblocking reads/writes and separately observes the child's exit. `Step` distinguishes read/write/effect pressure, cooperative yield and fully drained input/output. Each step consumes saved input before another read, preserves reply offsets across short writes, and retries interrupted operations within its turn budget. A write error is returned with the event still owned. A callback sees only inert bell/title requests; returning false must mean it did not accept the event. It is retried with the same event. There is no default native effect execution.

Runtime retention includes one 4 KiB PTY read buffer, one current write (up to 4 KiB for native input), one staged native input (up to 4 KiB), and a 64-event/16-KiB shared native-input mailbox, plus the engine's bounded output queue/parser/grid/history. Kernel queues are OS-owned. This is a logical retention bound, not a process RSS measurement. Effect pressure or a full engine output queue stops feeding/reading and can block the child. When a write is blocked, bounded readable echo is drained while preserving the current write offset, preventing duplex deadlock. The caller chooses readiness scheduling; the tests use short sleeps without establishing a product scheduler.

Replies are written in event order before later effects are accepted. The native input mailbox now merges input with replies through the same current-write slot; already generated replies precede staged input, and a partial event always finishes before another is selected. A `Terminal` supplied to `Session::spawn` establishes initial geometry. `pump().terminal()` is read-only; all subsequent engine mutation belongs to the session owner.

Resize prepares a cloned terminal, validates both engine limits and 16-bit PTY geometry, calls `TIOCSWINSZ`, then publishes the prepared terminal. Validation/ioctl failure leaves engine state intact. The owner performs no pumping between OS resize and publication. This temporarily doubles bounded engine storage and is intentionally an integration policy, not a final efficient resize implementation. Bytes already emitted by a concurrent child have no universal ordering against resize; processed queries retain their original replies.

EOF finishes partial UTF-8/parser state only after saved input and output are drained. Child exit and EOF are distinct: a clean completion requires both. Explicit `shutdown` closes the master and kills/reaps the direct child; it abandons outstanding data rather than claiming clean drain. Drop is a best-effort fallback. Direct-session reaping can block; the `worker` module now owns it off the caller/UI thread. Shell job groups, descendant supervision, graceful timeout escalation and externally modified SIGCHLD behavior are outside this experiment.

## OS boundary and dependency

Only private `os.rs` allows unsafe code. It wraps macOS libc calls with owned `File` descriptors and documents each unsafe block. The package denies unsafe elsewhere; the engine still forbids it. Rust opens master/slave close-on-exec; the slave remains blocking. The child hook sets a new session, claims the controlling terminal and resets the signal mask/common dispositions using only OS calls and errno errors. Parent slave handles are dropped after spawn so they cannot hide EOF. No copied terminal implementation or handwritten platform struct layout is used.

`libc = 0.2.189` supplies ABI bindings, already present in the workspace lockfile; it adds no new third-party package/version and no normal transitive dependency. It is newly direct for this macOS experiment. See [ADR 0007](../../docs/adr/0007-pty-lifecycle-experiment.md) for rationale and references.

## Evidence and remaining work

The original eleven tests cover real PTY identity/replies, initial and signalled geometry, output/effect pressure, final output and decoder flush, EOF before child exit, nonzero exit, invalid resize, failed launch, repeated sessions, explicit cancellation and drop reaping. Scripted transports force short writes and `WouldBlock` after every reply byte, interrupted reads/writes, zero/failed writes, partial UTF-8/CSI input and a 1,000-record ordered flood. Forced write pressure is deterministic transport evidence; no claim is made that a particular kernel write-buffer limit was reached.

The [original retained run](../../benchmarks/results/p0-08-pty-session/summary.json) records those eleven tests and source/executable hashes. The [native-boundary evidence](../../benchmarks/results/p0-09-native-boundary/summary.json) records the later fifteen-test worker stage. The current twenty-five-test package also covers native input, duplex echo, dispatch-time application-key encoding, a live cursor/region peer and three clean Bash cases; [window findings](../../research/2026-09-26-native-window.md) record that stage. No throughput, latency, full terminal compatibility or production resource claim follows. Owned snapshots and the session-worker/private C boundary are now implemented; see [ADR 0008](../../docs/adr/0008-snapshot-worker-and-private-ffi.md). The [native-window preview](../native-window/README.md) now provides basic input and an AppKit grid; cursor/mode/region ownership remains in the engine.

The [tab/erase slice](../../research/2026-09-27-tabs-and-erasure.md) adds a clean Bash interaction that emits literal HT, ED 2 and background-colored EL 1 across a wide owner. Output processing is disabled for that probe; octal escapes generate UTF-8 independently of the shell's clean locale.

The insertion/deletion shell case verifies ICH/DCH output outside a region and IL/DL with background-only blanks inside it, preserving outside rows. [Findings](../../research/2026-09-27-insertion-deletion.md) distinguish this controlled interaction from broad application compatibility.

A visibility-only PTY handshake now verifies that a new snapshot is published without changing text, cursor position or row versions; held hidden/visible frames remain independent after worker teardown. It also checks no-wrap overwrite at the right edge.

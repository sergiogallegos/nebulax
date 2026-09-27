# 0020 — Owned macOS PTY bindings without external Cargo crates

Status: implemented on 2026-09-27. Replaces ADR 0007's libc-crate dependency under the std/core-only rule in ADR 0018. PTY behavior, public Rust API and private C ABI v3 remain unchanged.

## Decision and ABI boundary

Remove the macOS libc dependency from `nebulax-pty-session`. The terminal/session/native-bridge closure now contains only these three owned workspace packages. Existing reference harness dependencies remain outside that closure; libc stays in the workspace lockfile because the reference graph still uses it. This is not a zero-dependency claim about the entire research workspace or the system frameworks linked by Rust/native UI.

`experiments/pty-session/src/os/bindings.rs` declares only the user-space functions, layouts and constants consumed by the PTY boundary. It calls system libSystem C entry points for ioctl, PTY grant/unlock, session creation, signal-mask/disposition reset and test-only waitpid. Standard-library `File`, `OpenOptions`, `Command`, errno conversion and RAII still own descriptors and child execution. No C build script, generated runtime binding, third-party macro or vendored crate replaces libc.

The bindings were authored against the installed macOS SDK headers: `sys/ttycom.h`, `sys/ioctl.h`, `sys/fcntl.h`, `sys/signal.h`, `signal.h`, `sys/_types.h`, `sys/wait.h`, `sys/errno.h`, `unistd.h` and `_stdlib.h`. The retained SDK input manifest hashes the compiler's actual transitive header inputs; no SDK source is redistributed. These are ABI facts and narrow original declarations, not a translated libc implementation.

`WindowSize` uses four C unsigned-short fields. `SignalSet` is the SDK's 32-bit unsigned set, and `Pid` its signed 32-bit process ID. `SignalAction` represents the public user-space `struct sigaction`, not the different kernel structure with a trampoline field. It is used only to install SIG_DFL with zero flags and no old-action output. Its handler field is a nullable C function pointer, occupying the SDK union's pointer slot; it is never used to read or call arbitrary handler addresses. Rust `repr(C)` and std C integer types specify layout/calling conventions, and the C oracle validates the resulting sizes, alignments, offsets and constants. Calling the public sigaction wrapper preserves the system's trampoline handling.

Declarations are gated to 64-bit macOS arm64/x86_64; other native targets require an explicit ABI extension. This session executes the probes and lifecycle tests on arm64 only. The check runs against the active host SDK in macOS verification; x86_64 runtime behavior and older deployment targets are not newly certified here. The core and portable transport still compile independently of these declarations on other operating systems.

## Verification gates

`scripts/pty-abi` builds an independent C program against SDK headers and a Rust program importing the actual owned declarations. It compares **39 numeric facts**: type sizes/alignments, struct offsets, open/ioctl/wait/error constants, reset signal numbers, empty signal-mask value and default handler representation. Compile-time C and Rust assignments/assertions check **eight function signatures**, including variadic ioctl. The Rust executable also calls the bound sigemptyset function. Optional recording retains commands, results, source hashes, SDK header hashes, toolchain details and probe binary hashes.

The new Rust lifecycle test runs in an isolated subprocess. A Python launcher installs ignored dispositions and a nonempty signal mask, then execs an exact single-test Rust runner. An SDK C control probe first confirms those inherited preconditions. The same C program then runs through our PTY path and verifies an empty mask, all ten default dispositions, terminal descriptors and session/foreground identity. Parent test-process signal state is never modified. Existing tests cover resize/SIGWINCH, descriptor inheritance, nonblocking reads, replies, partial writes, EOF/exit, failed spawn and reaping. The post-fork hook still performs only OS calls and errno-only error construction, with no allocation, logging or locks; see [Rust's pre_exec contract](https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html#tymethod.pre_exec).

`scripts/check-product-deps` checks Cargo manifest metadata rather than only active-feature resolution. It traverses an explicit allowlist of owned package names/paths and rejects foreign registry/Git/path packages across normal, build, dev/test, optional and target-specific declarations. It also rejects pulling the reference graph into the owned graph. New owned packages require deliberately extending the allowlist. This guards manifest dependencies; it cannot infer the provenance of handwritten source. Two Python tests exercise accepted isolation and rejected dependency variants.

Both checks run in `scripts/verify`. Unsafe remains confined to the private OS subtree and audited bridge; the terminal core retains `forbid`. No new toolchain or third-party version was selected. [Findings and evidence](../../research/2026-09-27-owned-pty-bindings.md) record local validation and remaining limits.

## Next scope

Resume the bounded input-protocol work with bracketed paste: engine-owned mode 2004, bounded explicit paste events, reset/query behavior and ordered PTY writes. Clipboard access stays in native UI policy; the core must not read the host clipboard. Public embedding/Metal typography and production lifecycle acceptance remain separate gates.

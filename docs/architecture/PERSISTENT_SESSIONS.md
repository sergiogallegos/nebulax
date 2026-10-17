# Persistent sessions, remote clients and collaboration

Status: owner-raised future proposal, recorded 2026-10-17. This is feasibility/design planning, not implementation approval for a service, cloud deployment or a change to daemon-free v1. The existing macOS-first native MVP and [roadmap](../../plans/ROADMAP.md) remain the delivery sequence. No dates or hosting model are selected.

The owner proposed a multiplexer that organizes terminal blocks in long-lived sessions, survives closing the application, reconnects from other devices, supports native macOS/iOS and web clients, and shares live sessions. Interactive, automatic and production work could eventually use a common session interface. The owned reusable Rust engine and separable session/presentation components are a useful foundation; the current private C ABI and local PTY worker do not already provide persistence or remote access.

## Suggested progression

| Candidate stage | Behavior | Evidence needed |
|---|---|---|
| Native terminal foundation | Correct terminal behavior, native scrolling/selection, input/IME/accessibility and bounded rendering | Existing MVP gates; scrollback is authoritative engine data, presentation is client-owned |
| Local durable multiplexer | Several independent PTYs in named sessions; application close detaches while a separate service owns the processes | Attachment/detachment, lifecycle, bounded history, crash/cancellation handling and explicit resize ownership |
| Remote attachment | Reconnect to an existing service from another desktop; authenticated transport with resynchronization | Stable identities/versioned protocol, ordered output, bounded queues, disconnect/replay gaps, credentials and revocation |
| Additional clients | Web and native iOS clients consume the same session protocol | Client-specific rendering/input/accessibility, reconnect, viewport and resource acceptance; distinct from desktop ports |
| Collaboration | Explicit viewers/controllers can share a running terminal | Read/control grants, arbitration, revocation, output confidentiality and slow-client isolation |
| Other work streams | Jobs, agent runs and production logs appear beside interactive terminals where appropriate | Distinct process/job semantics, cancellation/retry/retention and typed capabilities |

This is an ordering recommendation within the proposed feature family, not a rescheduling of Linux/Windows or the existing CLI/MCP commitments. A first local multiplexer need not include hosted accounts, billing or an internet service. Hosting and a company/product expansion would be separate decisions.

## Boundaries to preserve

Each PTY has one authoritative engine and one process owner in the optional session service. Clients consume immutable state/output and send typed user-authorized input. They can retain bounded presentation data without becoming independently authoritative grids. The standalone engine remains embeddable without the service, network, accounts or Nebulax application configuration.

A reconnect protocol needs a versioned snapshot plus ordered sequence identity. Reconnect from a bounded retained window when possible; explicitly resynchronize when data has been evicted. Surviving application close is different from surviving service exit or host reboot: a process cannot be restored merely by replaying its terminal output. Disk restoration and process restart policies remain open.

Sharing one PTY also shares one terminal geometry. Choose an explicit controller/resize policy; spectators must not race to resize the underlying process. Scrolling, selection and viewport position are per-client presentation concerns over authoritative retained history. Multiple writers need deliberate arbitration or an exclusive control token. A viewer grant must not imply permission to send input, resize, change configuration or invoke host actions.

Remote input is a new control-authority boundary. Authentication, encrypted transport, grant expiration/revocation, session discovery and output privacy require an explicit design. Prefer reviewed system security facilities through owned adapters; transport/security implementation choices remain open under the zero-third-party-Rust-crate rule. PTY bytes, replay fixtures and terminal escape sequences remain data, never authorization for configuration or native side effects.

The current prohibition on arbitrary agent screen reads/keystroke injection remains in force. Agent/job capabilities would need their own explicit policy and acceptance; a human collaboration channel must not implicitly grant those capabilities. Interactive terminals, automated jobs and production logs can share identity/transport/presentation infrastructure while retaining their distinct authority, restart and retention semantics.

When this phase is selected, first promote one bounded local attach/detach experiment into a GitHub issue. Do not scaffold a daemon, network stack, iOS app or web frontend before selecting and validating that scope.

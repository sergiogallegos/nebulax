# 0017 — Reusable terminal library, native shells and direct GPU backends

Status: direction clarified by the owner on 2026-09-27. Reusable library is an explicit product goal; native frontend ownership remains accepted. Direct GPU backends supersede ADR 0001's requirement to compare wgpu before choosing the macOS direction. Module/API details and platform acceptance still require implementation evidence.

## Owner intent and boundaries

Nebulax should provide a reusable Rust terminal library so other developers can build their own applications without rebuilding terminal behavior. Nebulax's own platform applications consume that same library. Cross-platform refers to shared terminal behavior and portable interfaces; each frontend owns native presentation and integration.

Keep these responsibilities separable within the library family; this is a logical design, not permission to scaffold empty crates:

| Component | Owns | Must not require |
|---|---|---|
| Headless terminal state | VT parsing/semantics, Unicode segmentation/width policy, grid/history/reflow, styles, modes, input encoding, bounded replies/effects and immutable snapshots | Window toolkit, fonts, GPU, PTY, runtime or Nebulax application configuration |
| Optional text/render support | Cell-to-glyph mapping, text-run preparation, shaping integration, font fallback/rasterization, bounded glyph caches, render data and direct platform backends | Nebulax tabs, menus, configuration/control services or application identity |
| Optional session/runtime support | PTY/ConPTY adaptation, input/reply transport, scheduling and lifecycle | A particular window toolkit or a mandatory singleton/global event loop |
| Native applications | Windows, tabs, split presentation, menus, preferences, native input/IME, accessibility, clipboard permissions and surface lifecycle | A duplicate authoritative terminal grid |

The existing headless Rust crate has zero Cargo dependencies, uses Rust's standard library, performs no I/O and forbids unsafe code. The subsequent [ADR 0018](0018-standard-library-only-rust.md) extends the zero-third-party-crate goal across owned Rust rendering/session components as well. Call platform APIs through owned bindings/adapters; no third-party Rust text or wrapper crate is assumed. Native frameworks remain system dependencies, so this is not a claim that rendering works without platform services. Shared Rust library ownership does not imply every platform adapter must be handwritten in Rust; owned C/Swift adapters remain compatible with the language boundaries.

Third parties should be able to use the headless engine with their own renderer and event loop, or opt into our rendering/session components. Rust callers use Rust APIs. Swift, C#, C++ and other callers need a documented C-compatible boundary with explicit ownership, errors, thread rules, resource bounds and versioning. Today's checked private ABI v3 is a research boundary, not a released/stable SDK. The crate remains unpublished (`publish = false`). Public API/ABI stability, embedder examples, capability/version identity policy and release compatibility tests must be designed before publishing. Application config, CLI/MCP authority and approval policy must not become prerequisites for engine reuse.

## Native presentation and rendering

| Platform | Native shell direction | Terminal GPU direction |
|---|---|---|
| macOS | Swift/AppKit; SwiftUI may compose appropriate native UI | Direct Metal, with Core Text shaping/font integration |
| Linux | GTK4 is the owner's candidate; C++ frontend remains consistent with earlier language intent | Direct OpenGL candidate; context, Wayland/X11 and driver coverage require validation |
| Windows | Native Windows UI; C# frontend remains the earlier language intent | Direct platform backend; exact graphics API/toolkit/version is not selected by this clarification |

Native tabs/splits/menus are frontend responsibilities. Session identities and reusable domain rules may still be shared; the engine does not draw application chrome. A GTK frontend targets GTK desktop conventions; this does not promise identical native appearance on every Linux desktop.

Share terminal-specific render data/contracts where useful. Metal command encoders, OpenGL contexts and future Windows device/swapchain lifetimes remain backend-specific. An owned boundary for snapshots, glyph runs, damage and surface lifecycle is compatible with direct backends; a mandatory general-purpose GPU abstraction is not the selected direction. wgpu is no longer a prerequisite comparison or planned mandatory product dependency. This is an owner architecture choice, not a measured claim that direct backends are always faster. Backend duplication, shader maintenance and separate platform tests are accepted costs.

## Ligatures and correctness

Ligatures are an explicit typography requirement. Shaping selects glyphs; Metal draws the resulting rendering data. Core Text provides character-to-glyph conversion with ligatures; Metal by itself does not implement a shaping policy. Shape suitable runs spanning adjacent cells, retain mappings to the authoritative cell/text positions, and preserve cursor, selection, accessibility, style boundaries and terminal widths. Test enabled/disabled ligatures with known fonts, fallback, combining characters, emoji and clipping. A ligature must never collapse logical grid cells or alter the bytes exposed for copying.

Current AppKit research drawing shapes individual lead-cell clusters using Core Text and draws through a graphics context. It is not our direct Metal backend and does not establish programming ligatures across adjacent cells. No Linux/Windows frontend or reusable renderer has been implemented. Those facts remain distinct from this accepted direction.

## Consequences and validation

The current protocol/reset work continues inside the headless boundary. Before the renderer implementation, define the embedder-facing surface/snapshot/text-run ownership contract and build the macOS Core Text-to-Metal vertical slice. Validate ligature cell mapping, native input/IME/accessibility, bounded caches, GPU in-flight lifetimes and teardown without holding the engine lock. A standalone embedding example must eventually demonstrate that consumers do not need Nebulax's application shell. Linux/Windows remain later platform work; no toolkit/GPU dependency is added by this decision.

References: [Core Text](https://developer.apple.com/documentation/CoreText), [GTK4 GLArea](https://docs.gtk.org/gtk4/class.GLArea.html), [existing native preview contract](0009-native-window-and-basic-input.md), [dependency policy](0002-owned-rust-engine-and-dependency-policy.md). Framework documentation supports feasibility, not completed Nebulax behavior.

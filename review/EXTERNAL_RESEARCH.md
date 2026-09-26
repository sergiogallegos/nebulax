# Phase A: external research

Date: 2026-09-25. Status: research, not accepted architecture.

## Question and method

Can a small project combine a Rust-heavy terminal engine, native macOS integration, bounded resource use, and typed agent control without taking on three platforms at once?

Method: inspect the eight required public repositories, selected manifests and implementation files, official protocol/API documentation, and the owner's existing local repository. Compare responsibilities and development practices, not just directory names. No competitor was launched, no performance experiment was run, and no complete codebase audit was attempted. Remote default-branch observations are dated, mutable observations, not stable-release guarantees. Phase 0 must fetch exact tags/commits and record them with each result.

Evidence labels: **observed** means directly inspected; **recommendation** is a project choice; **estimate** is a model awaiting measurement; **open** requires additional evidence or owner decision.

## Required repositories: conventions and fit

| Repository and inspected evidence | Observed convention / problem solved | Recommendation for this project |
|---|---|---|
| [`sergiogallegos/rust-ethernet-ip`](https://github.com/sergiogallegos/rust-ethernet-ip), local `AGENTS.md`, `CONTRIBUTING.md`, root tree | Compact agent entry point routes to human build/architecture docs; validation artifacts distinguish simulator from hardware evidence; scoped reusable skills | **Adopt** evidence labels and navigable instructions. **Adapt** language-boundary checks to Swift/Rust. **Reject** copying PLC-specific restrictions or maintaining a separate wiki/task board before needed |
| [`openai/codex`](https://github.com/openai/codex), root tree and [agent guide](https://raw.githubusercontent.com/openai/codex/main/AGENTS.md) | Rust workspace, explicit schema regeneration, snapshot review, subsystem test commands, safeguards against an ever-growing core crate | **Adopt** schema drift checks and explicit test entry points. **Adapt** crate extraction to demonstrated boundaries. **Reject** copying Bazel plus Cargo and product-specific policies into a new small project |
| [`openclaw/openclaw`](https://github.com/openclaw/openclaw), [agent guide](https://raw.githubusercontent.com/openclaw/openclaw/main/AGENTS.md) | Instructions assign one authoritative owner per responsibility and route to scoped guidance; working trees are protected during concurrent work | **Adopt** one config transaction owner and task isolation. **Adapt** scoped instructions when complexity appears. **Reject** building a plugin ecosystem or an agent orchestration system for v1 |
| [`rust-lang/rust`](https://github.com/rust-lang/rust), [contributor entry](https://raw.githubusercontent.com/rust-lang/rust/main/CONTRIBUTING.md), `compiler/`, `library/`, `tests/` | Component separation and specialist documentation support a very large codebase | **Adopt** explicit subsystem ownership and conformance assets. **Adapt** a single task runner idea. **Reject** compiler bootstrap machinery and large-team governance |
| [`ghostty-org/ghostty`](https://github.com/ghostty-org/ghostty), [architecture description](https://ghostty.org/docs/about), [terminal page implementation](https://raw.githubusercontent.com/ghostty-org/ghostty/main/src/terminal/page.zig) | Platform shells consume a shared core; terminal storage explicitly manages pages and uncommon data | **Adopt** the native-shell/shared-domain boundary. **Adapt** bounded uncommon-data storage through experiments. **Reject** copying its language choice or promising its memory behavior |
| [`alacritty/alacritty`](https://github.com/alacritty/alacritty), [terminal manifest](https://raw.githubusercontent.com/alacritty/alacritty/master/alacritty_terminal/Cargo.toml), [cell implementation](https://raw.githubusercontent.com/alacritty/alacritty/master/alacritty_terminal/src/term/cell.rs) | Reusable terminal library separated from application; cells keep uncommon data separately | **Adopt** separation and baseline methodology. **Evaluate** complete core reuse before a new emulator. **Reject** inheriting the GUI feature scope: this project requires tabs |
| [`wez/wezterm`](https://github.com/wez/wezterm), redirects to `wezterm/wezterm`; [workspace manifest](https://raw.githubusercontent.com/wezterm/wezterm/main/Cargo.toml), [multiplexing documentation](https://wezterm.org/multiplexing.html) | Separate GUI, mux server, cell, escape parser, surface and transport components | **Adapt** responsibility boundaries and workspace concepts. **Reject** starting with its broad crate graph, Lua config, or mux daemon |
| [`kovidgoyal/kitty`](https://github.com/kovidgoyal/kitty), [keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/), [remote control](https://sw.kovidgoyal.net/kitty/remote-control/) | Published terminal extensions and explicit control interfaces make advanced behavior discoverable | **Adopt** executable protocol fixtures and typed commands. **Adapt** fine-grained control policy. **Reject** broad screen reading/input injection and escape-sequence config control in v1 |

The local EtherNet/IP checkout was observed at commit `2a0b28f7982b403543896e0dc99437594bb33c2e`. It was only read; its policies govern that repository, not this one. No remote branch SHA is invented for the other projects.

## Terminal implementations

| Project / primary source | Verified observation | Design implication / confidence |
|---|---|---|
| [Alacritty README](https://github.com/alacritty/alacritty) and [releases](https://github.com/alacritty/alacritty/releases) | Rust/OpenGL application; TOML config; deliberately excludes built-in tabs/splits; release page lists 0.17.0 | High confidence about documented scope. Compare one-window workloads fairly; additional Alacritty windows are not native tabs. No verified memory/startup number |
| [Ghostty about](https://ghostty.org/docs/about) and [download](https://ghostty.org/download) | Swift/AppKit/SwiftUI macOS shell and Zig GTK shell share libghostty; download page advertises 1.3.1 | Strong architectural reference. Its about-page library-stability note refers to initial release: current standalone VT ABI maturity remains open, not disproved by an old note |
| [Kitty protocols](https://sw.kovidgoyal.net/kitty/keyboard-protocol/) | Keyboard extension defines negotiated modes and encoded key events | Implement negotiation and reset behavior, not merely a few key encodings. Treat protocol text as authoritative; tests as corroboration |
| [WezTerm multiplexing](https://wezterm.org/multiplexing.html) | Distinguishes multiplexing domains and an external mux server | Metadata restoration and live process survival are different features; omit daemon by default |
| [iTerm2 repository](https://github.com/gnachman/iTerm2) and [session restoration](https://iterm2.com/documentation-restoration.html) | Mature macOS application with documented session restoration behavior | Use as a UX reference for tab lifecycle and crash recovery; do not promise equivalent live-state restoration without its supporting architecture |
| [Rio Sugarloaf manifest](https://raw.githubusercontent.com/raphamorim/rio/main/sugarloaf/Cargo.toml) | Inspected main-branch manifest makes wgpu optional and describes native Metal/Vulkan preferences on macOS/Linux, with wgpu for Windows/WASM | Older summaries that call Rio simply a wgpu terminal are insufficient. Study selected release code before benchmarking; no inference that native wins every workload |

## Emulation reuse

| Candidate | Evidence and scope | Recommended disposition |
|---|---|---|
| [`vte`](https://github.com/alacritty/vte) | Rust parser with `Perform` callbacks; README explicitly separates parsing from semantics; MIT/Apache-2.0 | Preferred tokenizer if we own state. Does not remove grid, modes, Unicode, resize, or protocol work |
| [`alacritty_terminal`](https://docs.rs/alacritty_terminal/latest/alacritty_terminal/) | Exposes `Term`, `Grid`, PTY/event-loop modules. [Manifest](https://raw.githubusercontent.com/alacritty/alacritty/master/alacritty_terminal/Cargo.toml) identifies Apache-2.0; main is 0.26.1-dev while package docs show 0.26.0 | First complete-core Phase 0 candidate. Profile adaptation cost, protocol gaps, data access, dependency impact, and upstream strategy; do not choose a development branch to satisfy “newest” |
| Ghostty terminal | [MIT license](https://raw.githubusercontent.com/ghostty-org/ghostty/main/LICENSE); [source](https://raw.githubusercontent.com/ghostty-org/ghostty/main/src/terminal/page.zig) uses contiguous pages, styles, graphemes and hyperlink side storage | Reference oracle and reuse challenger. Requires Zig/toolchain and ABI evaluation; current standalone distribution/guarantees must be verified if promoted to a dependency |
| [`termwiz`](https://docs.rs/termwiz/latest/termwiz/) / WezTerm internals | Parsing, terminal interaction and surface utilities exist in a broad Rust ecosystem | Investigate targeted reuse; do not assume a utility crate is a drop-in modern GUI engine |
| [`vt100`](https://docs.rs/vt100/latest/vt100/) | In-memory parser/screen representation | Useful test/replay comparison; modern protocol and Unicode coverage must be demonstrated before production selection |
| [`portable-pty`](https://docs.rs/portable-pty/latest/portable_pty/) | Portable PTY abstraction from the WezTerm ecosystem | Throughput/ownership spike candidate; judge whether blocking IO/control surface fits the chosen scheduler |

A permissively licensed dependency does not become MIT just because the application's original code is MIT. Preserve third-party licenses/notices and the provenance of adapted files. Kitty/iTerm2 are behavior references here, not planned code donors; any copied code requires its own license review.

## Rendering, input and text

[wgpu](https://docs.rs/wgpu/latest/wgpu/) is a native Rust GPU API with Metal, Vulkan, D3D12 and other backends. Its relationship to WebGPU does not imply browser rendering. Compare selected backend features against direct Metal; binary size and resource cost are empirical questions.

Apple API web pages for `NSTextInputClient` and Core Text returned JavaScript-only content and their Markdown links failed in this research session. Installed Xcode 26.6 SDK headers were therefore inspected directly: `AppKit.framework/Headers/NSTextInputClient.h`, `AppKit.framework/Headers/NSWindow.h`, and `MetalKit.framework/Headers/MTKView.h`. They establish that the input protocol, native tabbing property and event-driven redraw controls exist; no performance claim follows from their existence. Recheck against the selected stable SDK during Phase 0.

[GTK documentation](https://docs.gtk.org/gtk4/) exposes input, accessibility, Wayland and X11 integration; its currently served docs describe a development version. Recommend GTK4/libadwaita provisionally for GNOME integration and sustainable IME/accessibility work. Raw Wayland/X11 means owning those integrations and compositor-specific testing; “Linux-native” is not a single desktop style.

[Windows pseudoconsole documentation](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session) describes host pipe IO and process integration. [CreatePseudoConsole](https://learn.microsoft.com/en-us/windows/console/createpseudoconsole) establishes the API floor; [Windows 10 lifecycle](https://learn.microsoft.com/en-us/lifecycle/products/windows-10-home-and-pro) means that API availability should not be equated with a sensible supported-OS policy. [windows-rs](https://github.com/microsoft/windows-rs) provides Windows API access from Rust. No demonstrated need for C++ or a .NET shell emerged.

Unicode [UAX #29](https://unicode.org/reports/tr29/) defines grapheme boundaries; [UAX #11](https://unicode.org/reports/tr11/) does not by itself settle terminal cell width. The inspected UAX #29 is stable Unicode 18.0.0. Separate protocol width policy from font advance and shaping. Test Unicode-version mismatches with applications explicitly.

[cosmic-text](https://docs.rs/cosmic-text/latest/cosmic_text/) is a shared text-stack candidate, not automatically a replacement for terminal layout. Compare `rustybuzz`/`swash`/`fontdb` component reuse with Core Text; the macOS recommendation favors native fallback and rasterization until measurements justify otherwise.

## Config, control and distribution sources

[`toml_edit`](https://docs.rs/toml_edit/latest/toml_edit/) preserves comments/spacing/order with documented limitations, so arbitrary edits cannot promise byte identity. [`schemars`](https://docs.rs/schemars/latest/schemars/) generates schema from Rust types; semantic rules and source provenance still require application logic. [`cbindgen`](https://github.com/mozilla/cbindgen) generates C declarations; [UniFFI](https://mozilla.github.io/uniffi-rs/latest/) is an alternative for higher-level bindings, not a proof of suitability for frame buffers.

The current [MCP specification](https://modelcontextprotocol.io/specification/2026-07-28) and [transports](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports) were checked: select local stdio and a separate helper. Do not copy older connection-handshake assumptions into a new SDK integration. Protocol metadata/versioning changes make a client compatibility spike necessary.

The [symbols-only Nerd Font license file](https://raw.githubusercontent.com/ryanoasis/nerd-fonts/master/patched-fonts/NerdFontsSymbolsOnly/LICENSE) inspected is MIT. The [repository license inventory](https://github.com/ryanoasis/nerd-fonts/blob/master/LICENSE) covers multiple upstream components; the exact shipped font and all accompanying notices must be inventoried at its pinned release. SF Symbols remains a platform UI facility, not an asset to redistribute cross-platform.

[Zola](https://www.getzola.org/documentation/getting-started/overview/) is a reasonable static documentation generator with a single executable. A site is deferred until public release work; plain HTML/CSS is sufficient for an earlier small landing page. Astro static output is acceptable only if its ecosystem benefits justify Node tooling.

## Findings and unresolved evidence

Recommendation: proceed to a small macOS-first research repository after review, using a shared Rust domain, Swift AppKit shell, native Metal/Core Text candidate, and strict separation between terminal output and typed control. Keep complete terminal-core reuse as the first experiment, not an afterthought.

Confidence is high about the structural separation and the need for bounded state, medium about the proposed native renderer and scheduler, and low about unmeasured relative memory/latency. These are explicitly unresolved: final VT engine, comparative footprint, key-to-photon latency, Core Text cache growth, native-tab behavior under detach/restore, Windows minimum policy, and actual MCP client interoperability.

No quantitative competitor conclusion is possible yet. [vtebench](https://github.com/alacritty/vtebench) provides throughput workloads; it does not replace end-to-end latency or a verified completion barrier. See [Phase 0](PHASE_0_PLAN.md) for exact measurement controls and exit criteria.

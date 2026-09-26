# Bootstrap Prompt: Architecture, Repository Design, Research, and Phased Implementation for a New Open-Source Terminal Emulator

## 0. Purpose of This Prompt

This document is the **project bootstrap specification** for a new open-source terminal emulator.

The purpose of this prompt is **not** to immediately generate a full terminal emulator.

Instead, the work must proceed in controlled phases:

1. Research the problem space, competing implementations, repository structures, and relevant technologies.
2. Challenge assumptions in this specification.
3. Produce an architecture and repository operating model.
4. Stop for owner review before implementation begins.
5. After approval, bootstrap the repository and project infrastructure.
6. Execute focused Phase 0 research spikes and benchmark baselines.
7. Only then begin the macOS implementation.
8. Expand later to Linux and Windows while preserving the architectural principles established earlier.

The project must be designed from the beginning for both:

- high-quality human engineering and open-source contribution, and
- effective development by modern coding agents such as Codex, Claude Code, GitHub Copilot agents, and future agent systems.

The repository itself is part of the product's engineering architecture.

---

# 1. Your Role

Act as a **principal systems engineer and open-source project architect** with deep expertise in:

- terminal emulation
- VT/xterm protocols
- PTYs
- Unicode and grapheme handling
- font shaping and rasterization
- GPU text rendering
- low-latency native desktop applications
- macOS AppKit, Metal, and Core Text
- Linux Wayland/X11, GTK/libadwaita, OpenGL/Vulkan
- Windows Win32, ConPTY, DirectWrite, Direct3D/DXGI
- Rust systems programming
- C and ABI design
- performance engineering
- memory optimization
- benchmarking methodology
- asynchronous IO and concurrency
- repository architecture
- open-source governance
- CI/CD
- release engineering
- secure IPC
- MCP and coding-agent workflows
- documentation architecture for long-lived projects

You must be **opinionated but evidence-driven**.

Do not simply agree with this prompt.

If an assumption is wrong, premature, expensive, misleading, or architecturally harmful:

1. identify it,
2. explain why,
3. provide evidence or examples where possible,
4. propose a better alternative.

Honesty is more important than agreement.

Clearly distinguish:

- verified facts,
- current implementation details observed in other projects,
- design recommendations,
- estimates,
- assumptions,
- unresolved questions.

---

# 2. Operating Rules

## 2.1 Architecture before implementation

Do **not** immediately generate the terminal implementation.

The workflow must be:

```text
Research
    ↓
Architecture Proposal
    ↓
Repository Operating Model
    ↓
Open Questions / Owner Decisions
    ↓
STOP FOR REVIEW
    ↓
Repository Bootstrap
    ↓
Phase 0 Research Spikes
    ↓
Architecture Confirmation / ADRs
    ↓
Phase 1 macOS MVP
    ↓
Later Phases
```

No major production implementation should begin before the architecture review gate.

---

## 2.2 Avoid premature abstraction

The long-term project is cross-platform, but **macOS is the first implementation target**.

Do not force every macOS implementation detail behind unnecessary abstractions merely because Linux and Windows will exist later.

Architectural boundaries should preserve future portability without sacrificing clarity, performance, or native integration today.

Create shared abstractions only where there is a demonstrated shared concept.

---

## 2.3 Human documentation is canonical

Human-facing project documentation is the source of truth.

Files such as:

```text
docs/
README.md
CONTRIBUTING.md
SECURITY.md
GOVERNANCE.md
ADRs
```

define the project.

Agent-specific files such as:

```text
AGENTS.md
CLAUDE.md
.codex/
.agents/
```

must summarize, operationalize, or route to those canonical documents rather than creating a parallel source of truth.

Avoid duplicated policies across agent-specific files.

---

## 2.4 Preserve engineering history

Important decisions, research, measurements, and implementation plans must survive individual agent sessions.

Do not rely on chat history as project memory.

Use repository artifacts such as:

- ADRs
- research notes
- benchmark records
- implementation plans
- changelogs
- issue history
- pull requests
- validation evidence

---

## 2.5 Evidence over claims

Performance, correctness, compatibility, memory usage, and architectural claims must be backed by reproducible evidence.

Never write statements such as:

> `<NAME> uses 18 MB of RAM`

without context.

Performance evidence must record:

- commit
- version
- OS
- OS version
- CPU
- GPU
- RAM
- architecture
- font
- shell
- window size
- grid size
- resolution
- terminal configuration
- competing terminal version
- measurement tool
- methodology
- number of runs
- raw measurements where practical
- summary statistics

---

# 3. Project Summary

- **Name:** TBD
- Use `<NAME>` for the GUI application.
- Use `<name>` for the CLI binary.
- You may propose names, but naming is not a blocking architecture decision.
- **License:** MIT unless research identifies a reason to reconsider.
- **Project model:** open source with lightweight governance suitable for an initially small or solo-maintained project.
- **Initial platform:** macOS.
- **Later platforms:** Linux, then Windows.

Working positioning:

> A lightweight, truly native terminal emulator designed for both humans and software agents.

Potential conceptual tagline:

> Native terminal performance, built for humans and agents.

Do **not** permanently define the product identity as "lighter than Alacritty."

Alacritty should instead be the primary benchmark target.

---

# 4. Product Pillars

Treat the project as having three equally important architectural pillars.

## 4.1 Performance

The terminal should target extremely low:

- idle memory
- input latency
- CPU usage
- startup time
- redraw overhead

It should target high:

- PTY throughput
- parser throughput
- rendering throughput
- responsiveness under heavy TUI output

Alacritty is the primary performance baseline.

Ghostty, Kitty, WezTerm, iTerm2, Rio, and other modern terminals should be studied for specific architectural ideas and trade-offs.

---

## 4.2 Native platform integration

The terminal should feel native on each operating system.

Avoid a cross-platform web or heavyweight UI abstraction that compromises:

- startup time
- memory
- platform conventions
- text quality
- input behavior
- accessibility
- window management
- native tabbing
- system integration

Platform-specific code is acceptable where it materially improves the result.

---

## 4.3 Agent-oriented terminal design

The terminal should treat configuration and structural control as **typed, validated, discoverable interfaces** suitable for software agents.

The intended experience includes cases such as:

```text
"Use JetBrains Mono 14."
"Apply Catppuccin Mocha."
"Make the cursor a bar."
"Create four tabs named api, web, db, and logs."
"Open the shop workspace."
"Show me the effective config."
"Undo the last configuration change."
```

Agents such as Codex CLI or Claude Code running inside the terminal should be able to perform these operations safely and deterministically.

This is a core differentiator, not a secondary convenience feature.

---

# 5. Non-Goals and Hard Constraints

Unless research provides a compelling reason to change them:

- no Electron
- no web view for the terminal surface
- no browser-based rendering pipeline
- no embedded Lua runtime in the terminal core
- no embedded Python runtime in the terminal core
- no managed runtime whose baseline memory undermines the lightweight objective
- built-in splits are not required for v1
- a full multiplexer daemon is not required for v1
- reading arbitrary terminal contents through the agent API is not required for v1
- agents injecting arbitrary keystrokes into arbitrary tabs is not required for v1

tmux and zellij remain valid solutions for multiplexing in early releases.

---

# 6. Owner Technology Preferences

These are preferences, not unquestionable truths.

## 6.1 Preferred language

Rust is preferred for everything that can reasonably be Rust.

Rust should be strongly considered for:

- VT parsing
- terminal state
- grid
- scrollback
- PTY layer
- config
- IPC
- CLI
- MCP
- protocol handling
- shared state
- rendering infrastructure where practical
- Linux implementation
- Windows implementation
- performance-sensitive utilities

---

## 6.2 Platform-native languages

The owner is open to platform-native languages where they materially improve integration.

### macOS

Potential stack:

- Swift
- AppKit
- Metal
- Core Text

### Linux

Potential stack:

- Rust
- gtk4-rs/libadwaita
- direct Wayland/X11
- OpenGL or Vulkan

### Windows

Potential stack:

- Rust with `windows-rs`
- Win32
- ConPTY
- DirectWrite
- Direct3D 11 or equivalent

C++ may be used only where it provides a clear benefit.

C# should be evaluated, but the starting hypothesis is that a .NET desktop shell may conflict with the lightweight-memory goal.

---

# 7. Starting Architecture Hypothesis to Evaluate

The initial hypothesis is:

```text
                    Shared Rust Domain/Core
                           │
             ┌─────────────┼─────────────┐
             │             │             │
           macOS         Linux        Windows
             │             │             │
      Swift/AppKit      Rust shell     Rust shell
             │
         FFI boundary
```

Do **not** assume that all platforms need a C ABI.

A C ABI may be appropriate primarily where a language boundary exists, such as Swift ↔ Rust.

Rust components should call Rust APIs directly when possible.

Evaluate:

- stable C ABI
- internal Rust API
- UniFFI
- cbindgen
- direct FFI wrappers
- generated headers
- opaque handles
- ownership rules
- callback models
- versioning strategy

---

# 8. Platform Rollout

## Phase order

1. macOS
2. Linux
3. Windows

---

## 8.1 macOS

Primary first-release target.

Evaluate:

- AppKit
- Swift
- Metal
- Core Text
- native `NSWindow` tabbing
- native drag/drop
- accessibility APIs
- IME
- native window restoration
- Apple Silicon optimization
- Intel compatibility

Explicitly decide whether:

- macOS arm64 is Tier 1
- macOS x86_64 is Tier 2 or deferred

---

## 8.2 Linux

Wayland first-class.

X11 supported.

Evaluate:

### Option A

`gtk4-rs` + libadwaita

### Option B

raw Wayland + X11 integration

Compare:

- memory
- native feel
- accessibility
- IME
- maintenance burden
- Wayland maturity
- packaging
- decoration handling
- tab UI
- startup cost
- dependency cost

---

## 8.3 Windows

Target:

- Windows 10 1809+
- Windows 11
- ConPTY

Evaluate:

- Rust + windows-rs
- Win32
- DirectWrite
- Direct3D 11 / DXGI
- DirectComposition if useful
- IME
- native tabs/custom tab strip
- accessibility

Determine whether C++ contributes any significant advantage.

---

# 9. Rendering Architecture

Evaluate:

## Option A: native rendering backends

```text
Metal
OpenGL/Vulkan
Direct3D
```

behind a thin internal Rust abstraction.

## Option B: wgpu

Compare:

- binary size
- idle memory
- GPU memory
- startup
- shader complexity
- driver compatibility
- platform maintenance
- render latency
- text rendering flexibility
- dependency count
- compilation cost

The design should optimize for a terminal workload rather than general 3D graphics.

Evaluate:

- instanced glyph rendering
- texture atlases
- row damage
- rectangular damage
- full-frame redraw thresholds
- vsync
- frame pacing
- immediate redraw after input
- hidden-tab resource release
- GPU resource sharing between tabs
- zero render activity while idle

---

# 10. Text and Font Architecture

Evaluate platform-native text stacks versus shared Rust implementations.

## macOS

- Core Text
- Core Graphics
- Metal texture upload

## Linux

- FreeType
- HarfBuzz
- fontconfig

## Windows

- DirectWrite

## Rust alternatives

- swash
- rustybuzz
- fontdb
- cosmic-text components if relevant

Evaluate:

- ligatures
- emoji
- fallback fonts
- CJK
- grapheme clusters
- combining marks
- width measurement
- variable fonts
- font feature control
- rasterization quality
- glyph caching
- fallback-chain memory

---

# 11. Performance Goals

"Fast" and "light" must become measurable acceptance criteria.

Compare `<NAME>` to the latest stable Alacritty release on the same hardware and software environment.

Also benchmark Ghostty where useful.

Required metrics:

| Metric | Suggested methodology |
|---|---|
| Idle memory: first window | macOS `footprint` / `vmmap`, Linux PSS, Windows private working set |
| Additional tab memory | same methodology |
| GPU memory | measure separately where possible |
| Key-to-photon input latency | Typometer and/or high-speed camera |
| PTY/output throughput | `vtebench`, large output floods |
| Parser throughput | controlled replay corpus |
| Heavy redraw | Neovim, Claude Code, Codex CLI, scrolling |
| Frame pacing | frame-time distribution |
| Cold startup | process launch to usable prompt |
| Idle CPU | approximately 0% |
| Idle energy | platform energy tools |
| Binary size | stripped/unstripped |
| Dependency count | Cargo dependency graph |
| Incremental compile impact | optional developer metric |

---

# 12. Performance Targets

Do not invent absolute targets before baseline measurements.

Phase 0 must establish real competitor baselines.

Then define targets such as:

```text
<Name> <= Alacritty idle memory
<Name> <= Alacritty median input latency
<Name> >= Alacritty throughput
<Name> ~= 0% idle CPU
```

If beating Alacritty on a specific metric is unrealistic because of native platform frameworks or feature requirements, state that clearly.

Performance goals should be:

- tracked continuously,
- treated as regression gates where feasible,
- but not used as misleading marketing claims.

---

# 13. Memory Budget

Produce a detailed memory model.

Break down:

- executable code pages
- Rust runtime/library overhead
- native platform frameworks
- terminal grid
- scrollback
- styles
- hyperlinks
- grapheme storage
- PTY buffers
- parser state
- per-tab state
- glyph atlas
- font objects
- fallback font objects
- GPU buffers
- config
- IPC
- MCP when enabled
- session system when enabled
- symbol font when enabled

Estimate:

```text
first window
each additional tab
background tab
hidden tab
optional subsystem activation
```

Clearly mark estimates.

---

# 14. Pay-for-What-You-Use Principle

Optional features should approach zero runtime cost when disabled.

This includes:

- session persistence
- scrollback snapshots
- symbol fallback fonts
- workspace layouts
- MCP server
- control socket where practical
- tab icons
- advanced tracing
- agent audit history beyond minimal metadata

Evaluate:

- runtime lazy initialization
- Cargo features
- separate helper processes
- optional binaries

Avoid excessive compile-time fragmentation if runtime toggles provide an acceptable cost.

---

# 15. Core Data Structure Research

Investigate and propose concrete layouts.

## 15.1 Cell representation

Determine:

- bytes per cell
- character representation
- style reference
- width flags
- hyperlink reference
- grapheme handling
- continuation-cell representation

Evaluate keeping uncommon data outside the hot cell structure.

---

## 15.2 Style storage

Evaluate:

- direct style per cell
- style interning
- reference-counted style tables
- page-local style dictionaries

Optimize cache locality.

---

## 15.3 Grid

Evaluate:

- rows
- pages
- slabs
- ring buffers
- chunked storage

Consider resize/reflow cost.

---

## 15.4 Scrollback

Evaluate:

- lazy allocation
- capped ring
- page-based storage
- cold-page compression
- disk-backed scrollback as a non-default future option

---

# 16. Threading and IO Architecture

Evaluate:

## Model A

one IO thread per PTY

## Model B

shared async reactor

## Model C

hybrid model

Consider:

- memory per tab
- latency
- wakeups
- scheduling
- implementation complexity
- portability
- Rust async runtime cost

Do not automatically select Tokio simply because the project uses Rust.

Measure whether Tokio, mio, platform event loops, or direct OS APIs are appropriate.

---

# 17. Parser Performance

Investigate:

- zero-allocation hot loops
- ASCII fast paths
- UTF-8 fast paths
- SIMD opportunities
- batching printable runs
- state-machine layout
- branch prediction
- parser/state separation

Correctness is more important than clever micro-optimization.

Optimization should follow profiling.

---

# 18. Terminal Emulation Strategy

This is one of the highest-risk parts of the project.

Decide whether to:

- build from scratch
- reuse
- fork
- adapt

Study at minimum:

- `vte`
- `alacritty_terminal`
- Ghostty's terminal implementation
- Kitty behavior/protocols
- relevant Rust terminal libraries

Evaluate:

- license
- architecture
- correctness
- Unicode handling
- long-term control
- performance
- test coverage
- ease of extension

---

# 19. Minimum v1 Terminal Protocol Matrix

Support at minimum:

- xterm-compatible core
- correct `TERM`
- custom terminfo entry
- fallback to `xterm-256color`
- 24-bit color
- styled underlines
- colored underlines
- undercurl
- synchronized output / DEC mode 2026
- Kitty keyboard protocol
- bracketed paste
- focus events
- SGR mouse
- alternate screen
- OSC 8 hyperlinks
- OSC 52 clipboard with security policy
- OSC 7 working directory
- OSC 133 shell integration
- Unicode width correctness
- grapheme clustering
- emoji ZWJ sequences
- combining marks
- CJK
- optional ligatures
- native IME support

After v1:

- Kitty graphics
- Sixel
- additional modern protocols

---

# 20. Nerd Font and Symbol Support

Programs such as:

- Neovim
- `eza`
- `starship`
- lazygit

emit PUA characters.

The terminal should render these correctly.

Evaluate shipping a symbols-only Nerd Font fallback.

Verify license.

Design:

```toml
font.symbols = "builtin"
```

or:

```toml
font.symbols = "system"
```

or:

```toml
font.symbols = "none"
```

Requirements:

- lazy load symbol font
- no memory cost before first needed PUA glyph
- configurable width
- glyph constraints
- 1-cell / 2-cell policies
- advanced range mapping
- memory accounting

Terminal-owned UI icons should default to native platform symbols where appropriate, such as SF Symbols on macOS.

---

# 21. Tabs

Tabs are a v1 feature.

Use native tabs where appropriate.

macOS should strongly evaluate `NSWindow` tabbing.

Support:

- create
- close
- rename
- focus
- list
- attributes
- cwd
- profile
- theme override
- optional icon
- optional accent

---

# 22. Title Precedence

Programs may set titles using OSC 0/2.

Define deterministic rules.

Example model:

1. explicit user/agent tab name
2. application title
3. shell cwd/default title

Potential modes:

```text
locked explicit title
program title
combined title: "api — nvim"
```

---

# 23. Internal Surface Model

Splits are not required for v1.

However, model the UI conceptually as:

```text
Window
  └── Surface Tree
       └── Terminal Surface
```

so splits can be introduced later without replacing the entire tab/window state model.

Do not overengineer the tree before splits exist.

---

# 24. Workspaces / Layouts

Use declarative layout files.

Example:

```text
layouts/shop.toml
```

A layout can describe:

- windows
- tabs
- tab name
- cwd
- profile
- theme
- icon
- startup command

CLI examples:

```bash
<name> layout open shop
<name> layout save shop
<name> layout validate shop
<name> layout list
```

Startup commands are protected settings.

---

# 25. Session Save and Restore

Be precise about what can and cannot be restored.

## Cheaply restorable

- window geometry
- tabs
- active tab
- title
- attributes
- cwd
- theme/profile

## Optional

- capped scrollback snapshot

Default:

```text
off
```

because of privacy and storage.

## Not normally restorable

Live processes such as:

- SSH
- unsaved Neovim state
- shell jobs

unless a daemon/multiplexer owns the PTYs.

Evaluate an optional session daemon, but do not assume it belongs in v1.

Default architecture should remain daemon-free.

---

# 26. Session Configuration

Example:

```toml
[session]
restore = "ask"
save_scrollback = false
```

Supported:

```text
never
ask
always
```

Re-running commands is protected-tier functionality.

Session files must be:

- versioned
- atomically written
- crash tolerant
- user-only permissions
- migratable

---

# 27. Agent-Friendly Configuration

This is a core product differentiator.

The configuration system must be:

- typed
- validated
- introspectable
- versioned
- migratable
- machine-readable
- human-readable
- deterministic
- reversible

---

# 28. Config Format

Starting hypothesis:

**TOML**

Evaluate against:

- JSON
- JSONC
- KDL
- YAML only if there is a strong reason

Compare:

- human readability
- editing reliability
- comments
- formatting preservation
- Rust support
- schema support
- agent editing success
- merge semantics

Starting approach:

- Rust config structs as primary type model
- `serde`
- `schemars`
- generated JSON Schema
- `toml_edit` for format/comment preservation

---

# 29. Config Structure

Example:

```text
~/.config/<name>/
├── config.toml
├── keybinds.toml
├── themes/
├── profiles/
├── layouts/
├── AGENTS.md
└── history/
```

Define:

- include rules
- override precedence
- merge semantics
- array behavior
- table behavior
- environment expansion
- file path expansion
- cycle detection

---

# 30. Config Schema

Generate JSON Schema from the canonical Rust config model if feasible.

Support editor tooling such as Taplo.

Example:

```toml
#:schema https://...
```

Schema should power:

- editors
- validation
- CLI help
- docs generation
- MCP
- website config reference

Avoid maintaining multiple independent config definitions.

---

# 31. Config CLI

Design commands such as:

```bash
<name> config get
<name> config set
<name> config unset
<name> config validate
<name> config diff
<name> config apply
<name> config apply --dry-run
<name> config history
<name> config rollback
<name> config schema
<name> config docs
```

Machine output:

```bash
--json
```

Errors must include:

- path
- current value
- expected type
- allowed values
- validation reason
- possible fix

---

# 32. Terminal Control CLI

Examples:

```bash
<name> tab new
<name> tab rename
<name> tab close
<name> tab list
<name> tab focus
<name> tab set-icon
<name> tab set-color

<name> layout open
<name> layout save
<name> layout list
<name> layout validate

<name> session save
<name> session restore
<name> session clear
```

CLI should communicate with a running instance through authenticated local IPC.

---

# 33. IPC

Starting hypothesis:

### macOS/Linux

Unix domain socket

### Windows

named pipe

Requirements:

- owner-only permissions
- secure default location
- instance discovery
- protocol version
- structured messages
- capability negotiation

Evaluate whether an additional per-session token materially improves security.

---

# 34. MCP

Provide:

```bash
<name> mcp
```

The MCP server should expose typed tools/resources for:

- schema
- effective config
- themes
- fonts
- capabilities
- tabs
- layouts
- sessions
- config validation
- config mutation
- rollback

Do not expose arbitrary terminal-screen reading in v1.

Do not expose arbitrary keystroke injection in v1.

---

# 35. Config Hot Reload

Requirements:

- watch configuration
- parse candidate config
- validate candidate config
- preserve last-known-good state
- apply atomically
- report errors non-intrusively
- never partially apply invalid state

---

# 36. Config History

Configuration mutations should be auditable.

Track:

- timestamp
- source
- changed keys
- diff
- previous version
- new version

Possible source values:

```text
user-editor
cli
mcp
migration
system
```

Do not attempt unreliable identity attribution beyond what can actually be known.

---

# 37. Protected Configuration Tier

Protected settings include:

- shell command
- startup command
- environment injection
- command-executing keybinds
- OSC 52 permissions
- session "re-run command"
- layout startup commands

Changes require explicit approval or privileged CLI flag.

The exact UX must be researched.

---

# 38. Escape Sequence Security Boundary

PTY output must **never** directly mutate configuration.

Terminal escape sequences must not be able to:

- rewrite config
- open MCP permissions
- modify startup commands
- change protected settings
- bypass confirmation

Treat PTY output as untrusted input.

---

# 39. Clipboard and Hyperlinks

Define explicit policy for:

## OSC 52

- read
- write
- prompt
- allowlist
- deny

## OSC 8

- display
- activation confirmation if appropriate
- unsafe scheme handling

---

# 40. Agent-Development Architecture

Agent-oriented product features and agent-oriented repository development are separate concerns.

The repository must support development by:

- humans
- Codex
- Claude Code
- GitHub Copilot agents
- future coding agents

without requiring hidden conversation history.

---

# 41. Repository Design Goals

The repository should remain understandable with:

- 10 files
- 1,000 files
- multiple platform implementations
- several years of history
- parallel contributors
- parallel agents
- large architectural documentation
- benchmark history

Optimize for discoverability and durable context.

---

# 42. Repositories to Study

Research current structures and relevant conventions from at least:

- `https://github.com/sergiogallegos/rust-ethernet-ip`
- `https://github.com/openai/codex`
- `https://github.com/openclaw/openclaw`
- `https://github.com/rust-lang/rust`
- `https://github.com/ghostty-org/ghostty`
- `https://github.com/alacritty/alacritty`
- `https://github.com/wez/wezterm`
- `https://github.com/kovidgoyal/kitty`

Do not copy structures blindly.

For each useful convention identify:

```text
What problem it solves
Why it fits or does not fit this project
Whether to adopt, adapt, or reject it
```

Also inspect other modern repositories if they demonstrate better agent-development patterns.

---

# 43. Proposed Repository Family Structure

Evaluate and refine a structure similar to:

```text
<name>/
│
├── README.md
├── AGENTS.md
├── CLAUDE.md
├── CONTRIBUTING.md
├── GOVERNANCE.md
├── SECURITY.md
├── CODE_OF_CONDUCT.md
├── CHANGELOG.md
├── LICENSE
│
├── Cargo.toml
├── rust-toolchain.toml
│
├── .agents/
│   ├── README.md
│   ├── skills/
│   ├── workflows/
│   └── roles/
│
├── .codex/
│   ├── README.md
│   └── environments/
│
├── .github/
│   ├── workflows/
│   ├── ISSUE_TEMPLATE/
│   ├── PULL_REQUEST_TEMPLATE.md
│   ├── CODEOWNERS
│   └── dependabot.yml
│
├── crates/
│   ├── terminal-core/
│   ├── terminal-vt/
│   ├── terminal-grid/
│   ├── terminal-pty/
│   ├── terminal-font/
│   ├── terminal-render/
│   ├── terminal-render-metal/
│   ├── terminal-render-opengl/
│   ├── terminal-render-d3d/
│   ├── terminal-config/
│   ├── terminal-control/
│   ├── terminal-cli/
│   └── terminal-mcp/
│
├── platforms/
│   ├── macos/
│   ├── linux/
│   └── windows/
│
├── docs/
│   ├── architecture/
│   ├── adr/
│   ├── design/
│   ├── protocols/
│   ├── performance/
│   ├── security/
│   ├── development/
│   ├── agents/
│   └── releases/
│
├── research/
│   ├── terminal-emulation/
│   ├── rendering/
│   ├── fonts/
│   ├── memory/
│   ├── latency/
│   ├── platforms/
│   └── competitors/
│
├── benchmarks/
│   ├── harness/
│   ├── baselines/
│   ├── results/
│   └── README.md
│
├── tests/
│   ├── conformance/
│   ├── integration/
│   ├── fixtures/
│   ├── replay/
│   └── golden/
│
├── plans/
│   ├── ROADMAP.md
│   ├── NOW.md
│   ├── BACKLOG.md
│   └── completed/
│
├── website/
├── scripts/
├── tools/
└── xtask/
```

This is a hypothesis.

Research should simplify or restructure it if necessary.

Do not create empty directory bureaucracy merely because it appears in this prompt.

---

# 44. Repository Knowledge Architecture

Files should have clear responsibilities.

## `README.md`

Public project overview.

## `docs/`

Current engineering truth.

## `research/`

Investigations, experiments, and evidence not yet accepted as architecture.

## `docs/adr/`

Accepted architectural decisions and superseded decisions.

## `plans/`

Future and active implementation work.

## `benchmarks/`

Performance methodology and evidence.

## `tests/`

Validation assets.

## `AGENTS.md`

Agent operating rules.

## `.agents/`

Reusable agent workflows and skills.

---

# 45. Documentation Freshness Rule

Use this distinction:

```text
Current truth → docs/
Future work → plans/
Unresolved investigation → research/
Accepted decision history → docs/adr/
Measured evidence → benchmarks/
```

Do not allow aspirational future design to masquerade as current implemented architecture.

---

# 46. Architecture Decision Records

ADRs are mandatory for significant architectural choices.

Suggested files:

```text
docs/adr/0001-core-platform-boundary.md
docs/adr/0002-macos-ui-stack.md
docs/adr/0003-config-format.md
docs/adr/0004-render-backend-strategy.md
```

ADR template:

```text
Title
Status
Date
Context
Decision
Options Considered
Consequences
Performance Implications
Security Implications
Compatibility Implications
References
```

Statuses:

```text
proposed
accepted
rejected
superseded
deprecated
```

An accepted ADR must not be silently overturned.

If new evidence invalidates it, create a superseding ADR.

---

# 47. Research Documents

Research should be preserved.

Example:

```text
research/2026-09-rendering-backends.md
research/2026-09-terminal-core-reuse.md
research/2026-09-font-stack.md
research/2026-09-alacritty-memory-baseline.md
```

Template:

```text
Question
Context
Hypothesis
Systems / Repositories Examined
Experiment
Methodology
Measurements
Findings
Recommendation
Confidence
Open Questions
Sources
```

Accepted research should feed into ADRs.

Lifecycle:

```text
Research
    ↓
Recommendation
    ↓
ADR
    ↓
Implementation Plan
    ↓
Code
    ↓
Validation
    ↓
Historical Evidence
```

---

# 48. Plans

Use persistent plans for non-trivial work.

## `plans/ROADMAP.md`

Long-range milestones.

## `plans/NOW.md`

Current milestone and current priorities.

Keep this short.

## `plans/BACKLOG.md`

Ideas and potential work.

This is not a commitment list.

## Feature plans

Example:

```text
plans/config-hot-reload.md
plans/metal-renderer.md
plans/osc52-security.md
```

Completed plans move to:

```text
plans/completed/
```

Do not mirror every GitHub issue into Markdown.

---

# 49. GitHub Task Model

Preferred model:

```text
GitHub Issues → actionable work
GitHub Projects → optional planning view
plans/ → durable implementation context
ROADMAP → long-term direction
ADRs → architecture history
```

Avoid duplicated task state.

---

# 50. Agent Entry Point

Root:

```text
AGENTS.md
```

should be concise.

It should explain:

- project purpose
- repository map
- canonical docs
- build/test commands
- architecture rules
- required validation
- commit expectations
- ADR rules
- performance-sensitive areas
- security-sensitive areas

Do not turn root `AGENTS.md` into a 50-page architecture manual.

Route agents to the right documents.

---

# 51. Hierarchical AGENTS.md

Evaluate using scoped `AGENTS.md` files for complex areas.

Potential examples:

```text
/AGENTS.md

/crates/terminal-vt/AGENTS.md
/crates/terminal-render/AGENTS.md
/crates/terminal-config/AGENTS.md

/platforms/macos/AGENTS.md
/platforms/linux/AGENTS.md
/platforms/windows/AGENTS.md

/benchmarks/AGENTS.md
/website/AGENTS.md
```

Root rules apply globally.

Nested rules contain subsystem-specific context.

Avoid unnecessary nested instruction files.

---

# 52. CLAUDE.md and Tool-Specific Files

Do not duplicate the complete project rules in:

```text
CLAUDE.md
CODEX.md
COPILOT.md
```

Prefer a compatibility shim such as:

```text
Read and follow AGENTS.md.
Canonical project rules are defined there and in docs/.
```

Tool-specific additions should exist only where truly needed.

---

# 53. Agent Skills

Evaluate reusable workflows under:

```text
.agents/skills/
```

Possible future skills:

```text
add-terminal-protocol/
benchmark-memory/
benchmark-latency/
add-config-option/
add-escape-sequence/
update-json-schema/
add-platform-feature/
run-conformance-suite/
prepare-release/
security-review/
```

Each skill should describe:

- purpose
- relevant files
- prerequisites
- implementation rules
- required tests
- required documentation
- benchmark requirements
- completion criteria

---

# 54. Repository Map

Create:

```text
docs/architecture/REPOSITORY_MAP.md
```

It should help both humans and agents navigate quickly.

Example:

```text
Need to modify VT parsing?
→ crates/terminal-vt
→ read ADR-00xx
→ run terminal conformance suite

Need to modify font fallback?
→ crates/terminal-font
→ docs/design/fonts.md
→ golden font tests

Need to modify macOS windows?
→ platforms/macos
→ platforms/macos/AGENTS.md
→ docs/architecture/macos.md
```

Keep it practical.

---

# 55. Parallel Agent Work

Research whether the repository should explicitly support:

- Git worktrees
- per-task branches
- parallel agents
- independent build directories
- conflict-minimizing ownership boundaries

If adopted, document:

- branch naming
- worktree location
- generated-file conflicts
- lockfiles
- benchmark serialization
- merge policy

Do not design excessive orchestration until needed.

---

# 56. Git Conventions

Evaluate Conventional Commits.

Potential examples:

```text
feat(config):
feat(vt):
fix(render):
perf(grid):
docs(architecture):
test(vt):
chore(release):
```

If adopted, keep the rule lightweight.

---

# 57. PR Requirements

Architecture-affecting PRs should reference an ADR or proposed ADR.

Performance-sensitive PRs should include benchmark evidence when appropriate.

Security-sensitive PRs should include explicit security reasoning.

Terminal protocol changes should include conformance tests.

Config-schema changes should include:

- schema regeneration
- migration impact
- documentation update
- validation tests

---

# 58. Definition of Done

A non-trivial feature should not be considered complete until applicable items are done:

- implementation
- unit tests
- integration tests
- conformance tests
- benchmark
- documentation
- config schema
- migration
- ADR update
- security review
- cross-platform considerations
- release notes

Not every feature requires every item.

Document applicability rules.

---

# 59. Website

The website lives in:

```text
website/
```

Requirements:

- static
- lightweight
- fast
- accessible
- simple build pipeline

Evaluate:

- plain HTML/CSS
- Zola
- Astro static output
- another minimal generator

Avoid a large JavaScript application.

Pages:

- Home
- Install
- Configuration
- Configure with AI Agents
- CLI Reference
- MCP Guide
- Benchmarks
- Architecture
- Changelog
- Contributing

Config documentation should be generated from the canonical schema where practical.

---

# 60. Distribution

## macOS

Evaluate:

- signed `.dmg`
- notarization
- Homebrew cask
- Sparkle auto-update

Requirements include Apple Developer credentials.

---

## Linux

Evaluate minimal launch set from:

- AppImage
- Flatpak
- `.deb`
- `.rpm`
- AUR
- Nix

Do not attempt every package ecosystem at first release.

---

## Windows

Evaluate:

- MSI
- MSIX
- winget
- Scoop

Document code-signing requirements.

---

# 61. CI/CD

GitHub Actions should eventually cover:

- formatting
- lint
- unit tests
- integration tests
- conformance
- platform builds
- fuzzing
- documentation
- schema generation
- website
- benchmark smoke tests
- release packaging
- checksums
- signing
- notarization

Performance CI must distinguish noisy microbenchmarks from stable regression tests.

---

# 62. Verification and Validation

Use:

- `vttest`
- `esctest`
- real terminal-session replays
- Neovim
- Claude Code
- Codex CLI
- tmux
- zellij
- lazygit
- htop/btop

Add:

- parser fuzzing
- config fuzzing
- property tests
- resize/reflow tests
- golden image rendering tests
- Unicode corpus
- OSC security tests
- IPC security tests
- migration tests

---

# 63. Fuzzing

Use `cargo-fuzz` or equivalent for:

- VT parser
- escape sequence handling
- Unicode edge cases
- config parser
- config migration
- IPC protocol decoding

Fuzz targets should remain easy to run locally.

---

# 64. Golden Rendering Tests

Each renderer should eventually have controlled golden tests for:

- ASCII
- colors
- underline styles
- ligatures
- combining marks
- emoji
- CJK
- Nerd Font symbols
- cursor
- selections
- hyperlinks
- damage behavior

Account for platform rasterization differences.

---

# 65. Benchmark Repository Structure

Suggested:

```text
benchmarks/
├── README.md
├── harness/
├── workloads/
├── baselines/
│   ├── alacritty/
│   ├── ghostty/
│   └── <name>/
└── results/
```

Result metadata must include the environment.

Prefer machine-readable raw results plus summarized Markdown.

---

# 66. Competitor Research

Competitor analysis is qualitative and quantitative.

Research:

## Alacritty

Primary performance benchmark.

Verify current:

- tabs
- renderer
- config
- architecture
- memory
- startup
- feature scope

## Ghostty

Primary reference for:

- native platform integration
- architecture separation
- terminal correctness
- font handling
- tabs
- open-source organization

## Kitty

Primary reference for:

- modern terminal protocols
- graphics
- keyboard protocol
- symbol mapping

## WezTerm

Reference for:

- integrated multiplexer
- workspaces
- configuration flexibility
- trade-offs of Lua
- platform breadth

## iTerm2

Reference for:

- macOS-native feature completeness
- session behavior
- mature UX

## Rio and others

Study if technically relevant.

---

# 67. Security Model

The architecture proposal must contain a threat model.

Assets include:

- terminal config
- shell commands
- clipboard
- session history
- environment variables
- local IPC
- workspace startup commands
- MCP controls

Threat sources include:

- malicious terminal output
- prompt injection
- compromised CLI tools
- local untrusted processes
- malicious layout files
- malicious config edits
- unsafe OSC sequences

Define trust boundaries.

---

# 68. Open-Source Governance

Initial governance should remain lightweight.

Avoid designing a foundation-scale governance model for a solo project.

Research useful ideas from mature open-source projects.

Potential documents:

```text
GOVERNANCE.md
CONTRIBUTING.md
CODE_OF_CONDUCT.md
SECURITY.md
```

Clarify:

- maintainer authority
- contribution expectations
- review requirements
- security reporting
- release authority

---

# 69. Burnout and Scope Risk

Treat small-team sustainability as an architectural constraint.

Major risk areas:

- VT correctness long tail
- three platform shells
- three rendering stacks
- Unicode correctness
- IME
- accessibility
- packaging
- signing
- session management
- MCP/security
- benchmark infrastructure

Prefer staged delivery over simultaneous completeness.

---

# 70. Phased Execution Model

This project must proceed in phases.

---

# Phase A — External Research

## Goal

Understand the problem before fixing architecture.

Research:

- terminal architecture
- VT reuse options
- rendering backends
- font stacks
- competing terminals
- agent-oriented repositories
- repo governance
- performance baselines
- platform-specific APIs

Create research documents.

Do not generate production terminal code.

---

# Phase B — Architecture Proposal

Produce:

- executive summary
- challenged assumptions
- architecture diagram
- component boundaries
- platform strategy
- data flow
- threading model
- memory model
- rendering model
- font model
- VT strategy
- config strategy
- IPC strategy
- MCP strategy
- tabs/workspaces/session strategy
- security model
- repository architecture
- agent-development architecture
- performance strategy
- V&V strategy
- release strategy
- risks
- open questions

Still no full production implementation.

---

# Phase C — Owner Decision Gate

At the end of Phase B:

**STOP.**

Produce a concise list of decisions requiring owner approval.

Categorize them:

```text
Must decide before bootstrap
Can defer until Phase 0
Can defer until implementation
```

Do not silently choose high-impact unresolved items merely to continue.

---

# Phase D — Repository Bootstrap

Only after architecture approval.

Create the agreed project structure.

Expected bootstrap may include:

- `README.md`
- `LICENSE`
- `AGENTS.md`
- minimal `CLAUDE.md`
- `CONTRIBUTING.md`
- `GOVERNANCE.md`
- `SECURITY.md`
- `CODE_OF_CONDUCT.md`
- `Cargo.toml`
- Rust toolchain config
- docs hierarchy
- ADR hierarchy
- research hierarchy
- plans hierarchy
- benchmark hierarchy
- tests hierarchy
- GitHub Actions skeleton
- issue templates
- PR template
- architecture map
- ROADMAP
- NOW
- BACKLOG

Do not create dozens of meaningless placeholder files.

Every created file should have a defined purpose.

---

# Phase 0 — Benchmark Harness and Research Spikes

Before serious implementation, execute focused spikes.

Required candidates:

1. Alacritty memory baseline
2. Ghostty comparison baseline
3. Metal text renderer spike
4. Core Text shaping/rasterization spike
5. PTY throughput spike
6. VT parser/reuse comparison
7. candidate cell-layout benchmark
8. threading/reactor benchmark
9. Swift ↔ Rust FFI spike
10. config/schema round-trip spike

Capture results under `research/` and `benchmarks/`.

Use results to accept or supersede initial ADRs.

---

# Phase 1 — macOS MVP

Goal:

A correct, fast, native macOS terminal.

Minimum scope:

- one native application
- terminal core
- PTY
- VT
- GPU rendering
- Core Text
- native input
- IME
- multiple tabs
- tab rename
- working directory tracking
- core modern protocols
- Nerd Font fallback
- config loading
- basic CLI
- initial benchmarks

Do not include every planned agent feature yet.

---

# Phase 2 — Agent Configuration and Control

Implement:

- full typed config
- schema
- validation
- migrations
- hot reload
- CLI mutation
- config diff
- config history
- rollback
- protected tier
- IPC
- MCP
- tab control
- layouts
- workspace control
- session metadata restore
- agent config guide

Security review required.

---

# Phase 3 — Public macOS Release

Requirements:

- documentation
- website
- stable config
- install flow
- `.dmg`
- signing
- notarization
- Homebrew
- updater decision
- benchmark publication
- contribution docs
- release process

---

# Phase 4 — Linux

Implement Linux using conclusions from prior architecture research.

Wayland first-class.

Do not compromise macOS quality merely to force identical platform code.

Reuse Rust core where appropriate.

---

# Phase 5 — Windows

Implement:

- ConPTY
- native Windows shell
- DirectWrite
- Windows GPU backend
- packaging
- signing
- winget/Scoop as selected

---

# Phase 6+ — Advanced Features

Potential:

- splits
- graphics protocols
- richer shell integration
- optional process-preserving daemon
- remote control
- advanced agent automation
- screen introspection with a dedicated security model
- plugin ecosystem if justified

---

# 71. Required Phase-B Architecture Deliverable

Produce a comprehensive Markdown architecture report containing:

## 1. Executive Summary

One-page architecture recommendation.

Include feasibility assessment for:

- memory vs Alacritty
- input latency vs Alacritty
- throughput vs Alacritty
- macOS
- Linux
- Windows

Do not overclaim.

---

## 2. Challenged Assumptions

Explicitly identify assumptions in this specification that should change.

---

## 3. Architecture

Include Mermaid diagrams for:

- component architecture
- runtime data flow
- platform boundary
- agent-control architecture

---

## 4. ADR Summaries

For major decisions provide:

- options
- decision
- rationale
- trade-offs
- risks

Topics:

- platform languages
- macOS UI
- Linux shell
- Windows shell
- renderer
- text stack
- VT implementation
- config format
- IPC
- MCP
- core/platform boundary

---

## 5. Core Data Structures

Provide:

- proposed cell layout
- bytes/cell estimate
- grid
- scrollback
- styles
- grapheme storage
- glyph atlas

---

## 6. Memory Budget

Provide estimates and assumptions.

---

## 7. Performance Plan

Provide exact measurement methodology.

---

## 8. Config Specification

Include:

- file layout
- schema model
- merge rules
- migrations
- CLI table
- example config

---

## 9. Agent Control Specification

Include:

- IPC
- MCP tool list
- permission model
- audit model

---

## 10. Tabs / Workspaces / Sessions

Include:

- title precedence
- layout example
- session format
- restore semantics
- daemon recommendation

---

## 11. Icons / Symbols

Include:

- fallback strategy
- lazy loading
- width policy
- memory cost

---

## 12. Repository Architecture

Include final recommended tree.

Explain every major top-level directory.

---

## 13. Agent Development Model

Include:

- AGENTS hierarchy
- CLAUDE strategy
- skills
- ADR workflow
- research workflow
- plans
- GitHub issues
- worktrees
- agent parallelism

---

## 14. Documentation Model

Clarify:

```text
docs
research
plans
ADRs
benchmarks
```

---

## 15. Website

Include technology recommendation.

---

## 16. Release Engineering

Per platform.

---

## 17. Verification & Validation

Detailed test matrix.

---

## 18. Security

Threat model and mitigations.

---

## 19. Roadmap

Use the project phases in this specification.

---

## 20. Risks

At minimum:

- terminal correctness
- Unicode
- native platform maintenance
- renderer maintenance
- code signing
- agent-control attack surface
- scope growth
- small-team burnout

---

## 21. Owner Decision List

Split into:

```text
Required now
Required after Phase 0
Can defer
```

Then STOP.

---

# 72. Phase-D Bootstrap Deliverable

After explicit owner approval of Phase B, generate the actual repository bootstrap.

Before creating files:

1. restate accepted decisions,
2. identify any changed decisions,
3. ensure ADRs reflect them.

Then create repository infrastructure.

---

# 73. Bootstrap Repository Quality Rules

Do not:

- create placeholder modules with meaningless APIs
- create empty folders solely to match a diagram
- generate thousands of lines of speculative code
- define stable public APIs before research
- freeze cross-platform abstractions prematurely
- duplicate documentation
- duplicate task state
- hide design decisions in comments only

Do:

- keep bootstrap small
- make structure discoverable
- create real build/test entry points
- document rationale
- create initial ADRs
- preserve research history
- establish validation commands

---

# 74. Initial Git History Recommendation

Suggested early commits:

```text
chore: bootstrap project governance and repository structure

docs: add architecture proposal and initial ADRs

bench: add competitor baseline harness

experiment: add macos rendering spike

feat: begin terminal core
```

Do not require this exact history if a better sequence is justified.

---

# 75. Questions the Architecture Phase Must Resolve

At minimum investigate:

1. Is beating Alacritty a release requirement or optimization target?
2. Is native UI mandatory on every platform?
3. Is macOS arm64 the initial Tier-1 target?
4. Is macOS x86_64 supported initially?
5. Should Linux use GTK/libadwaita or raw Wayland?
6. Should Windows stay entirely Rust?
7. Should the Rust core expose a C ABI only to Swift?
8. Which renderer strategy is best?
9. Which font stack is best?
10. Should VT be reused or custom?
11. Tokio, mio, threads, or hybrid IO?
12. Exact config format?
13. Exact config mutation policy?
14. MCP process model?
15. IPC authentication?
16. Session daemon: reject, defer, or design?
17. GitHub Issues vs repository-local task system?
18. Conventional Commits?
19. Worktree support?
20. How should parallel agents coordinate?
21. Which changes require ADRs?
22. Which changes require benchmark evidence?
23. Which features belong in v1 versus later?
24. What objective benchmark thresholds are realistic?

---

# 76. Preferred Owner Defaults

Unless research strongly argues otherwise, assume:

- macOS first
- Rust-heavy architecture
- Swift only where it improves macOS integration
- no universal C ABI requirement
- native platform rendering
- TOML config
- JSON Schema
- CLI + MCP
- daemon-free default
- GitHub Issues for actionable tasks
- plans for durable implementation context
- ADRs for architecture
- research files for unresolved investigations
- nested AGENTS only where useful
- human docs are canonical
- performance evidence is reproducible
- security-sensitive control APIs remain narrow
- agents do not get arbitrary screen read / key injection in v1

---

# 77. Style Requirements for All Deliverables

Write with the precision expected of a serious systems project.

Prefer:

- exact crate names
- APIs
- measured numbers
- tables
- diagrams
- data structures
- explicit trade-offs
- concrete repository paths
- reproducible commands

Avoid:

- vague marketing language
- unsupported performance claims
- architecture astronautics
- generic AI buzzwords
- unnecessary abstraction
- unexplained complexity

Mark estimates clearly as estimates.

Mark unresolved questions clearly.

---

# 78. Final Instruction

Begin with **Phase A: External Research**.

Inspect the specified repositories and relevant terminal implementations.

Then produce **Phase B: Architecture Proposal**.

Do not bootstrap or implement the repository yet.

At the end of Phase B:

1. provide the architecture proposal,
2. provide challenged assumptions,
3. provide the recommended repository design,
4. provide the recommended agent-development model,
5. provide ADR candidates,
6. provide open questions and owner decisions,
7. classify decisions by urgency,
8. stop and wait for owner review.

Only after explicit approval should you continue to **Phase D: Repository Bootstrap** and later **Phase 0 implementation research**.

The goal is not to produce code quickly.

The goal is to build the architectural, repository, performance, security, and development foundations for a terminal emulator that could remain maintainable for many years and be effectively developed by both humans and software agents.

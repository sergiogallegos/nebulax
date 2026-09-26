# Nebulax: naming decision and research

Checked: 2026-09-25. Status: **owner selected Nebulax for the terminal and `nebulaxterm` for the CLI**.

## Accepted name

The owner confirmed this pairing on 2026-09-25. **Nebulax** is the product/display name; **`nebulaxterm`** is the executable name. The inspiration is the owner's family connection to space and the Astro Bot character. The short display name and more specific command work well together.

Exact web searches for `"nebulaxterm"`, including GitHub and crates.io/Homebrew/npm filters, returned no indexed matches in this session. This is a limited search result, not proof of registry, domain or trademark availability. Existing uses of Nebulax are documented below. The owner subsequently authorized the initial README push to [sergiogallegos/nebulax](https://github.com/sergiogallegos/nebulax); no package, domain or trademark registration has been performed.

The architecture examples now use these names. At the owner's request, the project directory was renamed to `/Users/sergiogallegos/projects/nebulax`, and the GitHub repository is `sergiogallegos/nebulax`. The proposed config slug is also `nebulax`. Architecture approval remains a separate pending decision. The following sections preserve the earlier naming research.

## Earlier Space / spaceterm proposal

The owner suggested **Space** for the GUI and **spaceterm** for the CLI during architecture research.

### Initial recommendation

Avoid `spaceterm`: there are direct collisions in the terminal category. Space is short, memorable, and compatible with the project's visual direction, but as a common word it is difficult to search and distinguish. A more distinctive space-themed name shared by the app and CLI would be easier to document and distribute. This is a practical naming recommendation, not trademark clearance.

### Evidence

| Check | Finding | Meaning |
|---|---|---|
| [sadiksaifi/SpaceTerm](https://github.com/sadiksaifi/SpaceTerm) | Existing native macOS terminal project using Rust, GPUI and libghostty-vt | Direct product-category collision |
| [taquangtrung/spaceterm](https://github.com/taquangtrung/spaceterm) | A separate repository uses the same name | GitHub/search ambiguity already exists |
| [`spaceterm` 0.0.1](https://docs.rs/crate/spaceterm/0.0.1), [package listing](https://docs.rs/crate/spaceterm/latest) | Published Rust package described as a SpaceTerm native app; package record points to taquangtrung/spaceterm | The unqualified Rust package name is occupied |
| [`spaceterm-core`](https://docs.rs/crate/spaceterm-core/latest) | Related published PTY/terminal core package | The prospective crate prefix is also in use |
| `spaceterm.com` | Search found a domain marketplace listing; direct page could not be retrieved | Domain availability was not established; do not assume it is freely registrable |
| Homebrew/npm exact-name web searches | No conclusive registry result obtained | Unknown, not evidence of availability |
| Broad search for Space terminal apps | Common-word results mix terminal projects, workspaces, and macOS Spaces | Weak discoverability; not proof that Space is legally unavailable |

No domain, package, repository, account, or trademark was registered. No legal database clearance was conducted. An empty search or available GitHub path would not establish exclusivity.

## Family and curiosity direction

The owner subsequently requested names inspired by Luca and Lia, space/planets, human body systems and colors, with room for personal interests. A short name that works as both display name and lowercase CLI avoids a second naming problem.

Preliminary creative candidates, **not cleared names**:

| Name / CLI | Intended inspiration | Assessment |
|---|---|---|
| Lialu / `lialu` | Lia + Lu(ca) | Shortest and clearest compact blend; preferred early candidate |
| Lucalia / `lucalia` | Luca + Lia | Most direct tribute; longer but pronounceable |
| Lulion / `lulion` | Lu(ca) + Li(a) + an Orion-inspired ending | Invented cosmic-sounding blend; meaning is our proposed story, not an existing etymology |

Exact-name and software/terminal web searches did not surface a direct terminal product for these three in this session. They are **not proven available**: direct GitHub search API and crates.io API requests were inaccessible through the web tool, so registry status remains unknown. General-word/personal/nonsoftware uses can also exist. Domains and trademarks were not checked for these candidates. The owner has not selected one.

Several broader ideas were less attractive after initial screening: [Lunali already names a Minecraft library](https://modrinth.com/mod/lunali), [Lunelia is a mobile app](https://play.google.com/store/apps/details?id=com.lunelia.app), [Lumilia has an App Store listing](https://apps.apple.com/my/app/lumilia/id6760238716), and [Lunivo is used for software products](https://lunivolabs.com/). This does not establish legal unavailability; it reduces distinctiveness.

## Nebula

The owner asked whether Nebula exists. It has material naming conflicts:

* [Slack's Nebula](https://github.com/slackhq/nebula) is an established networking project. The [Homebrew formula](https://formulae.brew.sh/formula/nebula) installs an actual `nebula` command.
* [Pebrel](https://github.com/Kuddev/pebrel), a terminal emulator, explicitly identifies itself as formerly Nebula; its internal Cargo package still uses that name in the inspected README.

Recommendation: avoid Nebula / `nebula` as the new app/CLI pair because of command and terminal-category confusion.

## Nebulax

The owner suggested Nebulax, inspired by Space Bully Nebulax from Astro Bot. As a sound/typing choice it is memorable, seven ASCII letters, and more distinctive than Space or Nebula. The game connection provides the owner's intended personal story, while also tying the name to an existing character rather than an original product identity.

Verified existing software uses:

* [`nebulaX` on PyPI](https://pypi.org/project/nebulaX/) is a published developer library for machine-learning experiment tracking.
* [NebulaX](https://nebulax.com.br/) is an existing software/API/integration business.
* A package mirror lists an [npm `nebulax` game package](https://www.skypack.dev/view/nebulax); the direct npm page could not be retrieved, so this is a secondary registry indication rather than a freshly verified npm API response.

No exact terminal-emulator collision was found in the performed Nebulax searches. That narrow result does not mean the word is unused, the CLI cannot conflict, domains are available, or the branding is legally cleared. No conclusion about Sony's trademark rights or permission requirements has been established. Recommend retaining it as a candidate if the personal connection matters, while preferring an original variation for a distinctive long-lived brand. Do not use game character art or imply an official affiliation in a future identity.

The recommendation above preceded the owner's final choice. **Nebulax / `nebulaxterm` is now selected**; the earlier optional naming questions no longer block progress. This decision does not waive the architecture approval gate.

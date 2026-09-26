# Unicode data provenance

These are upstream **data and conformance tests**, not a third-party terminal or segmentation implementation. Original Nebulax code remains MIT; Unicode files and generated property data retain [Unicode License V3](LICENSE.txt). The terminal crate declares `MIT AND Unicode-3.0` accordingly.

Selected **Unicode 18.0.0** on 2026-09-26. The official [UCD ReadMe](https://www.unicode.org/Public/UCD/latest/ucd/ReadMe.txt) identifies the current files as final 18.0.0 data, and [UAX #29 revision 49](https://www.unicode.org/reports/tr29/tr29-49.html) identifies itself as the stable published Unicode 18 annex. The version landing page still contains draft boilerplate; the explicit final-data and stable-annex statements were used to resolve that inconsistency. All downloaded data paths are versioned, not `latest`.

[manifest.json](manifest.json) records URLs and SHA-256 for six unmodified data/test files and the license. `scripts/generate-unicode.py` verifies these hashes, extracts only the properties we use, merges adjacent identical ranges, and writes `crates/terminal/src/tables.rs`. The generator uses Python's standard library, runs offline and is not part of the Rust build. `scripts/verify` regenerates in memory and compares the result; changing inputs or tables requires deliberate review.

The table packs Grapheme_Cluster_Break (bits 0–3), Indic_Conjunct_Break (4–5), Extended_Pictographic (6), Emoji_Presentation (7), East_Asian_Width W/F (8), ambiguous width (9), Default_Ignorable_Code_Point (10) and valid emoji variation bases (11). Missing values use the data files' defaults. The standard's full extended-grapheme test file is retained and run by Rust tests, including Unicode 18's revised GB9c rule.

No crate was added for segmentation or width. A focused Unicode crate was considered as an alternative to maintaining tables/rules; direct registry metadata requests returned HTTP 403 during this check, so no assertion is made about their latest versions. The chosen implementation is small, generated reproducibly from official data and checked against the official segmentation suite. This takes on Unicode update maintenance; it does not imply generated tables make terminal width universal or terminal conformance complete.

To update: verify the next stable data/annex, inspect rule changes, download exact versioned files and license, review checksums, update generator/rules/version constants, regenerate, and run conformance plus terminal policy/regression tests. Never rewrite expected tests to make a new version pass.

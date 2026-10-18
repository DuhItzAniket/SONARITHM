---
status: accepted
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0010 · Licensing: MIT OR Apache-2.0 for code, CC BY 4.0 for docs and data, DCO for contributions

## Context and problem statement

The repository had no licence. Without one, the public code was "all rights reserved": others could not legally use, modify or contribute to it, and JOSS requires an OSI-approved licence. The choice affects adoption, contributor terms, and compatibility with other projects (GPL-3.0 MAIDR, proprietary Highcharts, MIT Erie, Apache STRAUSS).

This decision belongs to the project owner. The owner chose the recommended option on 2026-10-18 (OD-01, OD-09).

## Decision drivers

* Maximum adoption in academia and industry, and embedding in other tools.
* Patent protection for users and contributors.
* Compatibility with the Rust ecosystem norm and with GPLv2 projects.
* Low-friction contributions.

## Considered options

* **MIT OR Apache-2.0** (dual licence, the Rust norm), contributions under the Developer Certificate of Origin (DCO)
* Apache-2.0 only
* MPL-2.0 (file-level copyleft)
* GPL-3.0 or AGPL-3.0 (strong copyleft)
* A CLA instead of the DCO

## Decision outcome

Chosen option: **MIT OR Apache-2.0 for code, with DCO sign-off for contributions.**

| Material | Licence | SPDX expression |
|---|---|---|
| Code: Rust crates, CLI, Python bindings, desktop app, frontend packages, build and CI configuration | MIT or Apache-2.0, at the user's option | `MIT OR Apache-2.0` |
| Documentation (`docs/`, Markdown files, `CITATION.cff`) | Creative Commons Attribution 4.0 | `CC-BY-4.0` |
| Example datasets and study stimuli (`examples/`, research materials) | Creative Commons Attribution 4.0, unless a dataset's own source licence says otherwise (recorded per dataset) | `CC-BY-4.0` |
| Bundled fonts (from B02) | SIL Open Font License 1.1, texts included | `OFL-1.1` |
| Icons (from B02) | Lucide ISC, plus the Feather MIT notice for derived glyphs | `ISC`, `MIT` |

**Copyright notice wording:** `SONARITHM contributors`. Source files carry `SPDX-FileCopyrightText: 2026 SONARITHM contributors`. Contributors keep the copyright in their contributions and license them inbound = outbound under the licence of the file they change.

**Files:** `LICENSE-MIT` and `LICENSE-APACHE` at the root (for GitHub and crates.io licence detection), the full texts in `LICENSES/` per the REUSE specification, and `REUSE.toml` annotating files that cannot carry headers ([supply chain §REUSE](../engineering/supply-chain-and-licensing.md#reuse-and-spdx-headers)).

### Consequences

* Good: Apache-2.0 contributes a patent grant. MIT keeps GPLv2 compatibility. This matches the Rust API Guidelines and most dependencies ([notes/07 §5](../research/notes/07-engineering-publication.md)).
* Good: a permissive licence positions SONARITHM as infrastructure others can embed, complementary to the GPL-3.0 MAIDR and proprietary Highcharts.
* Good: the DCO is lighter than a CLA and is accepted by JOSS and foundations.
* Good: CC BY 4.0 for documentation and data lets others reuse the research synthesis, design rules and datasets with attribution.
* Bad: permissive licences allow closed-source forks. The owner accepted this trade-off for adoption.

### Confirmation

A01-T13 adds `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSES/`, `REUSE.toml`, SPDX headers in source files, `license = "MIT OR Apache-2.0"` in every `Cargo.toml` and `package.json`, the CITATION.cff licence field, and a `reuse lint` check in CI. `cargo deny check` also checks the workspace crates' own licences.

## More information

Revisit only if the project joins a foundation whose policy requires a different licence. Relicensing would then need the agreement of all copyright holders.

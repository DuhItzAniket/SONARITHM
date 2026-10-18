---
status: proposed (owner decision required)
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0010 · Licensing: MIT OR Apache-2.0 with DCO

## Context and problem statement

The repository has no licence yet. Without one, the public code is "all rights reserved": others cannot legally use, modify or contribute to it, and JOSS requires an OSI-approved licence. The choice affects adoption, contributor terms, and compatibility with other projects (GPL-3.0 MAIDR, proprietary Highcharts, MIT Erie, Apache STRAUSS).

**This decision belongs to the project owner.** This ADR records the recommendation and its reasoning.

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

## Recommended outcome

**MIT OR Apache-2.0 for code, DCO sign-off for contributions**, plus:

* example datasets and study stimuli under **CC BY 4.0** (licensed separately);
* documentation under **CC BY 4.0** (or the same as code: owner's choice);
* bundled fonts under **OFL-1.1**, with the texts included;
* icons under **Lucide ISC + Feather MIT** notices.

### Consequences

* Good: Apache-2.0 contributes a patent grant. MIT keeps GPLv2 compatibility. This matches the Rust API Guidelines and most dependencies ([notes/07 §5](../research/notes/07-engineering-publication.md)).
* Good: a permissive licence positions SONARITHM as infrastructure others can embed, complementary to the GPL-3.0 MAIDR and proprietary Highcharts.
* Good: the DCO is lighter than a CLA and is accepted by JOSS and foundations.
* Bad: permissive licences allow closed-source forks. If the owner wants derivatives to stay open, MPL-2.0 is the middle ground.

### Confirmation

A01 exit criterion: the owner accepts this ADR (or chooses an alternative), and A01 then adds `LICENSE-MIT`, `LICENSE-APACHE`, the `license` field in every `Cargo.toml` and `package.json`, the CITATION.cff licence field, the REUSE configuration and the cargo-deny allowlist.

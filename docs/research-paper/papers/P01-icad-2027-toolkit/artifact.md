# P01 artifact package

> **Status:** Draft (2026-10-19). **Owner:** Paper lead.
> **Related:** [publication plan, artifact policy](../../publication-plan.md#artifact-and-reproducibility-package-policy) · [draft](src/paper.md) · [datasets](../../datasets.md) · [benchmarks log](../../benchmarks-log.md)

The package lets a reader regenerate every number, figure and sound in the paper. It is archived on Zenodo with the cited release.

| Item | Where | State |
|---|---|---|
| Release and DOI | `v0.2.0-alpha` and its Zenodo DOI | TODO: the owner tags the release (A08-T06); the DOI comes from the Zenodo integration (A07-T06) |
| Stimulus generators | `sonarithm_echometric::harness::generators` (GEN-001 to GEN-004), generator version 1 | Merged (#163). Pinned manifests: `crates/engines/sonarithm-echometric/tests/snapshots/` |
| Replication gallery | Specifications, data, manifests and deviation notes per item | TODO: A08-T02 |
| Benchmark scripts and results | Criterion benches, the latency tool, the golden-audio harness, benchmarks-log rows with method notes | Partly merged (A04, A06); A05 in review; A08-T04 report |
| Study runner and pilot protocol | `sonarithm study run`, a pilot protocol with scripted responses (no participants) | TODO: A08-T03 |
| Source verification | F-110 to F-113 in the findings log. Citation check of the draft (2026-10-19): all 28 records checked against Crossref or the publisher; none wrong; 6 incomplete records completed in `references.bib` (errata in notes 02 and 07); `tuckerbrown2022evaluating` keeps initials as MNRAS publishes them | Logged |
| Reproduction steps | A README in the archive: install, `cargo xtask gallery` (or the CLI), expected hashes | TODO with A08-T02 and T04 |
| Licences | Code MIT OR Apache-2.0; data, stimuli and figures CC BY 4.0 | Set (ADR-0010) |

No participant data exists, so none is archived.

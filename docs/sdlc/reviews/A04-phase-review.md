# A04 phase review: Data layer

> **Status:** Final (2026-10-19). **Phase:** A04. **Author:** A04 agent session (Claude Code) for the project lead. **Reviewers:** coordinator session (owner's delegate), @DuhItzAniket.
> **Related:** [reviews README](README.md) · [roadmap A04](../../planning/roadmap.md#a04--data-layer) · [A04 retrospective](../retrospectives/A04-retro.md) · [worklog: A04 data layer](../worklog/2026-10-19-a04-data-layer.md) · [03 data model](../../architecture/03-data-model.md)

## Summary

| Item | Value |
|---|---|
| Phase | A04 · Data layer ([roadmap](../../planning/roadmap.md#a04--data-layer)) |
| Planned window | 2026-11-22 – 2026-12-13 |
| Actual window | 2026-10-19 – 2026-10-19 |
| Outcome | **Done** |

A04 delivered `sonarithm-data`:

- Arrow-backed tables and bundles with `snr.*` field metadata;
- sources for CSV/TSV, JSON/NDJSON, Parquet, Arrow IPC and XLSX/ODS, all running text through one type inference;
- type, role and concept inference, provenance columns, eight transforms with a sandboxed rhai subset;
- M4 and LTTB downsampling with a tile cache, and fingerprints over canonical Arrow IPC.

All four exit criteria are met. A background agent built the code overnight as one branch per task; this session rebased it and landed it one task PR at a time under the fast-track rule. Fuzzing found crash and huge-allocation bugs in the arrow-ipc and calamine decoders, and the first Linux run found an arrow-array assertion reached from the provenance code; all are mitigated on our side (F-071). One fuzz finding is open: a 1.8 KB Parquet file stalls parquet's page-header decoder for about 12 s before failing ([#146](https://github.com/DuhItzAniket/SONARITHM/issues/146)), so the nightly `fuzz (columnar_read)` job stays red until it is fixed. Parquet, ODS, XLS and XLSB are not yet proven safe by fuzzing; the nightly job covers them from now on.

## Exit criteria

| # | Criterion (verbatim from the roadmap) | Result | Evidence |
|---|---|---|---|
| 1 | Loads a 1M-row CSV in ≤ 1.5 s on reference hardware. | Met | PB-05 criterion group `pb05_csv_ingest_1m_rows_5_columns` (`benches/csv_ingest.rs`): 0.506 s median on HW-01 and 0.912 s on HW-01e (four E-cores, the stand-in for the reference laptop); [benchmarks log](../../research-paper/benchmarks-log.md) "A04", [F-070](../../research-paper/findings-log.md) |
| 2 | Property tests on transforms pass. | Met | Eight property tests in `transform::rows`, `transform::aggregate` and `transform::series` (for example `sort_orders_a_permutation_and_is_idempotent`, `every_value_lies_in_its_bin`, `resampled_times_are_a_regular_grid`), green on three OSes in #139 |
| 3 | Downsampling is pixel-exact for M4 against a reference implementation. | Met | Property test `m4_is_pixel_exact_against_the_full_polyline`, which also checks the selection against `reference_m4`, the paper's M4 query executed literally (#141) |
| 4 | Fingerprints are stable across OSes. | Met | `corpus_fingerprints_are_pinned` (`tests/fingerprints.rs`) passes in `clippy and test` on ubuntu, windows and macos (#142) |

Waivers: none. Criterion 1 is measured on the development laptop; HW-01e (`start /affinity F0000`, four E-cores) stands in for the reference laptop, as in the A02 gate report.

## Definition of Done (phase)

- [x] All issues in the phase are closed: #45 to #54.
- [x] No open critical bug from the phase. [#146](https://github.com/DuhItzAniket/SONARITHM/issues/146) (a slow Parquet input, no crash or memory growth) stays open as a tracked bug.
- [x] Docs match what was built: 03 (implementation notes in §3.1, the propagation table in §4, transforms and downsampling), tech stack, testing strategy §7, CI/CD inventory. Deviation: `sonarithm-data` builds for wasm32 without its default features (`fs`, `zstd`), recorded in 02 §2 ("Partial (no fs)") and `xtask/src/ci.rs`.
- [x] [status.md](../status.md) updated.
- [x] Findings logged: F-070 (PB-05) and F-071 (decoder hardening). Measurements in the benchmarks log.
- [x] Traceability matrix: FR-001, FR-002 and FR-003 set to Done here; FR-008 stays In progress (its announcement half is C01).
- [x] Risk register updated: R-23 opened (malformed files against third-party decoders).
- [x] Coverage targets: not measured in A04 (no coverage tooling in CI yet); every module has unit tests and the invariants have property tests.
- [x] Retrospective held: [A04-retro.md](../retrospectives/A04-retro.md).

## Metrics against budget

| Metric | Budget | Measured | Conditions | Evidence | Verdict |
|---|---|---|---|---|---|
| PB-05 CSV ingest, 1M rows × 5 columns | ≤ 1.5 s | 0.506 s median (34%); 0.433 s without semantic inference | HW-01, release, Rust 1.99.0, criterion 0.8.2, 10 samples | F-070, benchmarks log | Within |
| PB-05 on the reference stand-in | ≤ 1.5 s | 0.912 s median (61%) | HW-01e, four E-cores | as above | Within |
| Type, role and concept inference precision | Not budgeted | 100% type, 98.5% role (one known miss), 100% concept | 27-file corpus, 74 fields | `inference_reaches_95_percent_precision_on_the_corpus` | — |

## Scope

**Delivered**

| Deliverable (from the roadmap) | Where | PRs |
|---|---|---|
| `DataBundle`, tables, `FieldMeta` in `snr.*` metadata (T01) | `table`, `bundle`, `meta`, `codes` | #113, #114 |
| CSV, TSV, JSON/NDJSON sources (T02) | `source::csv`, `source::json`, `infer::text` | #133 |
| Parquet and Arrow IPC (T03) | `source::columnar` | #116 |
| XLSX/ODS (T04) | `source::spreadsheet` (calamine) | #135 |
| Type, role and concept inference (T05) | `infer::semantic` | #136 |
| Provenance columns and propagation (T06) | `provenance` | #138 |
| Transforms with a sandboxed rhai subset (T07) | `transform` | #139 |
| M4 and LTTB downsampling, tile cache (T08) | `downsample` | #141 |
| Data fingerprinting (T09) | `fingerprint` | #142 |
| PB-05 benchmark, fuzz targets (T10) | `benches/`, `fuzz/`, `nightly.yml` | #143 |

**Deferred or dropped**

| Item | Reason | New target phase | Issue |
|---|---|---|---|
| One-hour Linux fuzz runs of `json_read` and `columnar_read`; Parquet, ODS, XLS and XLSB triage | libFuzzer on Windows reports contained panics as crashes; the nightly job runs on Linux | A05–A07 (nightly results) | — |
| Mapping spec `data.fields` and `transform` onto `ReadOptions` and `Transform` | Belongs to the compiler | A06 | — |
| A 1.8 KB Parquet file that stalls parquet's page-header decoder for about 12 s | Inside parquet 60's Thrift decoding; needs a bounded header pre-scan or an upstream fix | Next data-layer work | [#146](https://github.com/DuhItzAniket/SONARITHM/issues/146) |

## Decisions

| ADR / RFC | Title | Status |
|---|---|---|
| [ADR-0006](../../decisions/0006-arrow-data-model.md) | Arrow data model | Accepted (implemented, no change) |
| [ADR-0013](../../decisions/0013-determinism-and-provenance.md) | Determinism and provenance | Accepted (implemented, no change) |
| `deny.toml` | RUSTSEC-2026-0249 (`smartstring`, unmaintained, via rhai) ignored with a reason | Owner-approved; tracked in #115 |

## Risks

| Risk ID | Change | Note |
|---|---|---|
| R-23 | New | Malformed files crash or exhaust memory in third-party decoders (arrow-ipc, calamine, parquet). Mitigated by validation before decoding, panic containment and nightly fuzzing; Parquet, ODS, XLS and XLSB unproven |
| R-15 | Note | `smartstring` (via rhai) is unmaintained; tracked in #115 |

## Findings and research hooks

| Item | Status | Link |
|---|---|---|
| F-070 · CSV ingest of 1M rows × 5 columns takes about 0.51 s on the development laptop and 0.91 s on four E-cores, inside PB-05 | Logged | [findings log](../../research-paper/findings-log.md) |
| F-071 · arrow-ipc 60 and calamine 0.37 panic or allocate what a malformed file declares, so the data layer validates and contains them | Logged | [findings log](../../research-paper/findings-log.md) |

## Accessibility

Not applicable: no UI or announcement behaviour changed. Inference and provenance feed later announcements (estimates are qualified, rule 7 of AGENTS.md).

## Lessons

- One PR per task with auto-merge landed the phase faster than a deep stack merged by hand.
- Generated files and shared docs (Cargo.lock, status.md, findings log) conflict on every rebase when phases run in parallel; scripted resolutions kept it mechanical.
- Fuzzing third-party decoders found real crashes within minutes; budget for it in every importer task.
- The two-minute Linux smoke fuzz on the T10 pull request found a crash the Windows runs could not show (panic containment); run new targets on Linux before calling them clean.

## Follow-up actions

| Action | Owner | Due | Issue |
|---|---|---|---|
| Fix or contain the slow Parquet page header; ask the owner before reporting it upstream with the input | Next data-layer owner | Before Stage A ends | [#146](https://github.com/DuhItzAniket/SONARITHM/issues/146) |
| Triage the Linux nightly fuzz runs of the data targets | Next data-layer owner | A05 | — |
| Decide on #115 (`smartstring` advisory) when rhai releases a fix | Project lead | When rhai updates | #115 |

## Sign-off

| Role | Handle | Date | Decision |
|---|---|---|---|
| Project lead | @DuhItzAniket | | |

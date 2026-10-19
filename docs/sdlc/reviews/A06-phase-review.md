# A06 phase review: NUMISONA v1

> **Status:** Final (2026-10-19). **Phase:** A06. **Author:** A06 agent session (Claude Code) for the project lead. **Reviewers:** coordinator session (owner's delegate), @DuhItzAniket.
> **Related:** [reviews README](README.md) · [roadmap A06](../../planning/roadmap.md#a06--numisona-v1-parameter-mapping) · [worklog: A06 NUMISONA](../worklog/2026-10-19-a06-numisona.md) · [07 engines, NUMISONA](../../architecture/07-engines.md) · [design rules](../../research/design-rules.md)

## Summary

| Item | Value |
|---|---|
| Phase | A06 · NUMISONA v1: parameter mapping ([roadmap](../../planning/roadmap.md#a06--numisona-v1-parameter-mapping)) |
| Planned window | after A03 and A04, about 3 weeks |
| Actual window | 2026-10-19 – 2026-10-19 |
| Outcome | **Done** |

A06 delivered `sonarithm-numisona`, the first engine:

- scales, pitch spaces with the 400–1400 Hz default, psychophysical scaling, and the concept table with polarity resolution and the one-key flip;
- the `tone`, `glide` and experimental `auto` encoders, with row references and confidence on every note, redundant tap and duration encodings, and timbre per category;
- six chart templates and an Erie importer;
- a rule-conformance suite and the PB-04 benchmark.

To compile real data end to end, the compiler now compiles the data layer's `DataBundle` with spec transforms applied, through a `Platform` engine for inline and file sources (#152). Core's `TrackRequest` gained the substrate, field semantics, missing-value treatment and mode (ADR-0014, additive). All three exit criteria are met. The phase ran in parallel with A04 and A05, one PR per task under the fast-track rule.

## Exit criteria

| # | Criterion (verbatim from the roadmap) | Result | Evidence |
|---|---|---|---|
| 1 | Every SDR rule owned by NUMISONA has a unit test referencing its ID. | Met | `every_rule_numisona_owns_has_a_named_test` (`tests/conformance.rs`) parses the Owner column of the design-rules catalogue and requires a test named after each rule: 15 SDR and 2 A11Y rules, plus SDR-05 and SDR-15, which NUMISONA implements for other owners. The coverage report is its snapshot |
| 2 | The Erie gallery subset imports and renders. | Met | `the_erie_gallery_subset_imports_as_valid_specs` (8 gallery specs, report of 19 approximated or unmapped constructs, #150) and `sdr_05_the_erie_gallery_subset_imports_and_renders` (all 9 imported specs compile and render through the A05 render graph: finite, audible, unclipped) |
| 3 | Compile ≤ 200 ms for 100k rows × 4 tracks. | Met | PB-04 criterion group `pb04_compile`: 35.1 ms median on HW-01 and 54.3 ms on HW-01e (four E-cores), F-083 |

Waivers: none. Criterion 3 is measured on the development laptop and on HW-01e as the stand-in for the reference laptop, as in A02 and A04.

## Definition of Done (phase)

- [x] All issues in the phase are closed: #68 to #75 by their PRs, #76 by the T09 PR.
- [x] No open critical bug from the phase.
- [x] Docs match what was built: 03 §3.3 (the shipped concept table), 04 §9 (the importer), 05 §2–§4.2 (compiler data, `TrackRequest`, invariant 4 per pitch space), 07 (templates). Deviations: templates are spec builders because spec v0.1 reserves `template`; the concept table and polarity profile live in NUMISONA, not `CompileContext`.
- [x] [status.md](../status.md) updated.
- [x] Findings logged: F-080 to F-084. Measurements in the benchmarks log.
- [x] Traceability matrix: FR-024, FR-025, FR-026 and FR-030 to FR-035 set to Done; NFR-01 records PB-04. FR-032's UI flip key stays with B03.
- [x] Risk register: no change.
- [x] Coverage targets: not measured (no coverage tooling in CI yet); every module has unit tests, invariants have property tests, and the encoders, templates and importer have snapshots.
- [x] Retrospective: folded into "Lessons" below (one agent session ran the phase).

## Metrics against budget

| Metric | Budget | Measured | Conditions | Evidence | Verdict |
|---|---|---|---|---|---|
| PB-04 compile, 100k rows × 4 tracks | ≤ 200 ms | 35.1 ms median (18%) | HW-01, release, Rust 1.99.0, criterion 0.8.2, 20 samples | F-083, benchmarks log | Within |
| PB-04 on the reference stand-in | ≤ 200 ms | 54.3 ms (27%) | HW-01e, four E-cores | as above | Within |
| PB-04 compile, 1M rows | ≤ 1 s | 135 ms (1 track), 362 ms (4 tracks) on HW-01; 192 ms and 545 ms on HW-01e | as above | as above | Within |

## Scope

**Delivered**

| Deliverable (from the roadmap) | Where | PRs |
|---|---|---|
| Scales: linear, log, symlog, sqrt, pow, quantile, band, ordinal (T01) | `scale` | #134 |
| Pitch units and spaces; range defaults (T02) | `pitch` | #137 |
| Concept table with polarity defaults and the one-key flip (T03) | `data/concepts.toml`, `polarity` | #144 |
| Channels pitch, loudness, duration, timbre, pan, brightness; redundant duration and taps; `tone` and `glide` (T04) | `encode`, `examples` | #151 |
| Psychophysical scaling (T05) | `scale::psychophysical` | #140 |
| Mark switching above density thresholds, experimental (T06) | `density`, `encode` | #153 |
| Templates for line, bar, scatter, histogram, box and heat-map charts (T07) | `templates`; compiler data #152 | #152, #154 |
| Erie spec importer, subset (T08) | `import::erie` | #150 |
| Rule-conformance suite, compile benchmarks (T09) | `tests/conformance.rs`, `benches/compile.rs` | this PR |

**Deferred or dropped**

| Item | Reason | New target phase | Issue |
|---|---|---|---|
| The `rate` channel (glide modulation) | Score IR v1 has no place for it; the encoder reports it | When the IR gains a modulation field | — |
| Gap earcons for `missing: "earcon"`, off-scale cues | Earcons are AUDAXIS's | B03 | — |
| Per-category timbre on glides | A glide has one timbre in IR v1 | With a multi-timbre glide in the IR | — |
| Moving the SDR-19/21/23/27 and A11Y-14 warnings into the perceptual lint, with hysteresis for reversals | ECHOMETRIC owns the lint | C05 | — |
| Quartile aggregates in the spec (box plots would not need precomputation) | A spec minor version | A later spec version | — |

## Decisions

| ADR / RFC | Title | Status |
|---|---|---|
| [ADR-0014](../../decisions/0014-capability-trait-signatures.md) | Capability traits generic over the data type, spec fragments | Accepted; `TrackRequest` extended additively, `DataBundle` instantiated as `D` |
| [ADR-0013](../../decisions/0013-determinism-and-provenance.md) | Determinism and provenance | Accepted (implemented: `libm` maths, sorted outputs, a pinned Score hash checked on three OSes) |
| [RFC-0001](../../rfcs/0001-spec-v0.1.md) | Spec v0.1 | Accepted; no change |

## Risks

| Risk ID | Change | Note |
|---|---|---|
| — | None | The proposal ranges (F-082) and the volatility threshold (F-084) are research gaps, not risks; both are reported to the user when applied |

## Findings and research hooks

| Item | Status | Link |
|---|---|---|
| F-080 · A polarity flip mirrors pitches in the pitch space's own coordinate | Logged | [findings log](../../research-paper/findings-log.md) |
| F-081 · Spec v0.1 expresses 8 of Erie's 15 documented channels | Logged | as above |
| F-082 · Proposal ranges for duration, taps and loudness mappings | Logged | as above |
| F-083 · PB-04: 35 ms for 100k rows × 4 tracks | Logged | as above |
| F-084 · The SDR-21 volatility threshold rests on the studied reversal counts (TODO(verify)) | Logged | as above |
| H-02 (auto mark threshold), H-03 (400–1400 Hz) | Instrumented: `auto` and the default range are ready for EXP-002 | [hypotheses](../../research-paper/hypotheses.md) |

## Accessibility

No UI changed. The engine supplies what the accessible UI and announcements need: every note keeps its row (`RowRef`) and confidence for value announcements on pause (A11Y-09) and estimate qualifiers (03 §4); every assumption is an `Info` diagnostic with its rule, readable as text (FR-026); redundant tap and duration encodings serve precise reading (A11Y-13); pan alone and timbre-only categories warn (A11Y-14). Screen-reader checks of these messages belong to the CLI (A07) and the desktop app (B03).

## Lessons

- One PR per task with auto-merge kept the phase moving; at most two open PRs meant local branches stacked behind them, and every squash merge needed a rebase. A small script for the shared tables (status, traceability) saved most of the conflict time; it needs one-cell keys for task rows.
- The first engine needed integration work no task named: `TrackRequest` lacked the substrate, and the compiler could not yet compile `DataBundle`. Plan engine phases with an explicit integration task.
- Measuring early pays: the first PB-04 run showed per-note string clones and unneeded provenance reads; fixing them cut compile time by 40%.
- Writing the Erie importer from documentation only worked, but Erie's documentation leaves defaults unstated (pitch range, band), so some mappings rely on SONARITHM's defaults; the report says so.

## Follow-up actions

| Action | Owner | Due | Issue |
|---|---|---|---|
| Verify the trend-reversal levels of Nees & Walker 2008 behind F-084 | Research lead | Before A08 | — |
| Make `ReadOptions::without_inference()` keep declared fields, or document that it drops them | Data-layer owner | A07 | — |
| Register `Platform` and `Numisona` in the facade and CLI | A07 | A07 | — |
| Move the encoder's perceptual warnings into ECHOMETRIC's lint | ECHOMETRIC | C05 | — |

## Sign-off

| Role | Handle | Date | Decision |
|---|---|---|---|
| Project lead | @DuhItzAniket | | |

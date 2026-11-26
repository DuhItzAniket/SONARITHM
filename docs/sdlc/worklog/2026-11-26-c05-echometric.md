# 2026-11-26: C05 ECHOMETRIC, Lane R

> **Date(s):** 2026-11-26. **Phase:** C05. **Author:** project lead, written by an AI agent (Claude Code), reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [07 ECHOMETRIC](../../architecture/07-engines.md) · [implementation plan C05](../../planning/implementation-plan.md#c05--echometric-v1-analytics-descriptions-and-lint) · [ADR-0031](../../decisions/0031-echometric-numerics.md) · [ADR-0032](../../decisions/0032-the-lint-pass.md) · [RFC-0009](../../rfcs/0009-echometric-insights-in-the-score.md)

## Context and phase

- Phase: C05, Lane R: tasks C05-T01 to T10, batched into about eight PRs, each finished locally before it opens (the owner's working rule of 2026-11-26).
- Issue(s) / PR(s):
  - [#499](https://github.com/DuhItzAniket/SONARITHM/pull/499): the contracts;
  - [#500](https://github.com/DuhItzAniket/SONARITHM/issues/500): AUDAXIS's overlapping lint warnings, a follow-up outside C05;
  - [#333](https://github.com/DuhItzAniket/SONARITHM/issues/333) and [#334](https://github.com/DuhItzAniket/SONARITHM/issues/334): C05-T01 and T02, in this PR.
- Starting point: ECHOMETRIC held only A08's harness. The compiler already mapped five detector names to the `echometric` owner and reported them as unavailable.

## What was done

- **Contracts (#499), approved by the coordinator under the owner's delegation:**
  - ADR-0031: the detectors run on `libm`, with statrs and augurs only as test references in a crate at Rust 1.89.
  - ADR-0032: lint is a core `Linter` pass that writes nothing to the Score.
  - RFC-0009: detector parameters and the analyzed field in annotations; insight markers only for ECHOMETRIC's detectors.
- **C05-T01:**
  - Core: `Annotation.field` and `params`, and `AnalysisRequest` with the fields the spec's tracks play (ADR-0014 amended).
  - The compiler fills that request and returns the annotations; the facade exposes them as `Compiled::annotations()`.
  - AUDAXIS links an insight to its field's series.
  - The `Echometric` engine and `SNR-ENG-640`.
  - The facade adds ECHOMETRIC only to specs that name its detectors.
  - `reference/` compares the robust z-scores with statrs.
- **C05-T02:**
  - `extrema`: global extremes, and local extremes at reversals beyond a hysteresis band.
  - `outliers`: a robust z over moving-median residuals (F-205).
  - GEN-011, labelled outliers (DS-010), with precision and recall tests.

- **C05-T03:**
  - `changepoints`: PELT with an L2 cost, its penalty scaled by a robust noise estimate from the first differences.
  - `periodicity`: the first strong peak of the normalised autocorrelation, so a period beats its multiples.
  - `trend-segments`: the runs between `extrema`'s turns.
  - GEN-012, labelled level shifts and periods (DS-010).
  - Each finding's score is recorded under its own name (`robustZ`, `shift`, `correlation`).

## Decisions

| Decision | Why | Record |
|---|---|---|
| Detectors on `libm`; statrs and augurs as test references in their own crate | Detector output reaches the Score, so it must be identical on every platform (F-204); statrs needs Rust 1.89, while engines build at 1.88 and the CI `msrv` job checks every crate's tests at its own `rust-version` | ADR-0031 |
| Annotations name their field, and analyzers get the played fields | Without it, an outlier in a second series was linked to the first and announced as the wrong series | RFC-0009 §1 amendment |
| GEN-011, not GEN-005 | GEN-005 to GEN-010 are planned for other studies (datasets.md) | `harness/generators/outliers.rs` |
| Outliers from moving-median residuals | A robust z over the whole series missed spikes on a periodic base; one inside small windows called noise an outlier | F-205 |
| The first and last values are never judged as outliers, and the limitation is stated, not silent | The coordinator's condition for the ADR-0031 amendment; a fair judgement of an end needs the local trend | F-205, #507 |
| PELT's penalty scales with σ² from the MAD of first differences | A level shift moves one difference and does not inflate the estimate, so the same shift is found in any unit | `detect/changepoints.rs` |
| A period is the first strong autocorrelation peak within 0.1 of the highest | A period's multiples correlate almost as well; the highest peak alone can pick 2P | `detect/periodicity.rs` |
| A GEN-001 turn may sit one row from the generator's on a tie | At one point per hour, a sine peak falls between two equal rows; the count is exact | `tests/detectors.rs` |

## Findings to log

- F-205: a robust z over a whole series misses spikes on a periodic base, and one inside small windows calls noise an outlier — [findings log](../../research-paper/findings-log.md)

## Next

- C05-T03's augurs references (change points and seasonality), with the builder slot.
- C05-T06 starts after Lane U's batched explorer PR, extending `nav_query` (c05-t06 proposal, approved).

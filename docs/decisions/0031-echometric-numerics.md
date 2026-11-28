---
status: accepted
date: 2026-11-26
decision-makers: ["coordinator session, under @DuhItzAniket's delegation"]
consulted: ["ECHOMETRIC session (C05 phase plan, D1)"]
---

# 0031 · ECHOMETRIC numerics: detectors on libm, statrs and augurs as test references

## Context and problem statement

C05-T01 to T03 add ECHOMETRIC's detectors: extrema, outliers, change points, periodicity and trend segments (07 ECHOMETRIC). The implementation plan says they run "through augurs and statrs", as notes/04 §7 recommended. Two facts rule that out for the shipped code:

- **Determinism.** Detector output becomes annotations. Annotations become NavTree insight nodes, insight markers and cues in the Score ([RFC-0009](../rfcs/0009-echometric-insights-in-the-score.md)), and the facts behind descriptions and query answers. [ADR-0013](0013-determinism-and-provenance.md) §1 requires the same Score for the same spec and data on every platform. The std float functions (`ln`, `exp`, `powf`, and the rest) can differ in the last bits between platforms. A function sampled with them gave three different Scores on Linux, macOS and Windows ([F-204](../research-paper/findings-log.md)). statrs and augurs compute with the std functions (to be confirmed by reading their sources in C05-T01). One last-bit difference near a threshold adds or drops an insight.
- **MSRV and wasm.** statrs 0.19.1 declares Rust 1.89. Engine crates build at the core MSRV, 1.88, and for `wasm32-unknown-unknown` (02 §2, AGENTS rule 4). The CI `msrv` job checks every crate's targets, tests included, at its own `rust-version`. augurs declares no MSRV, and its wasm status is not verified (notes/04 §7).

## Decision drivers

- ADR-0013 §1 and F-204: the same insights on every platform.
- The core MSRV (1.88) and the wasm32 build of engine crates.
- Detector results checked against established implementations, not only against our own reading of the algorithms.
- Licences: MIT and Apache-2.0 only, dev-dependencies included (`cargo deny` checks them).

## Considered options

1. **augurs and statrs in the detectors**, as the plan wrote: raise ECHOMETRIC's `rust-version` to 1.89 and accept platform-dependent last bits.
2. **Port the few needed routines from statrs and augurs to `libm`** inside ECHOMETRIC.
3. **ECHOMETRIC's own detectors on `libm`**, with statrs and augurs only as test references, in a separate test crate.

## Decision outcome

Chosen: **option 3** (coordinator under the owner's delegation, 2026-11-26).

ECHOMETRIC's detectors use IEEE arithmetic (`+ − × ÷`, `sqrt`, `abs`), `libm` and `sonarithm_core::math` only. Sums and sorts run in row order, and there is no parallel reduction, so the order of operations is fixed. The crate's `clippy.toml` bans the std float functions, as CURVASONA's does (C03 canary, [#488](https://github.com/DuhItzAniket/SONARITHM/pull/488)). The algorithms:

| Detector | Method | Parameters (recorded in each annotation, RFC-0009 §2) |
|---|---|---|
| `extrema` | Global minimum and maximum, plus local extrema between reversals | the hysteresis band (shared with `trend-segments`) |
| `outliers` | Robust z-score from the median and the median absolute deviation (Iglewicz & Hoaglin 1993), of the residuals from a moving median (F-205, C05-T02) | the threshold (3.5); the window (21) |
| `changepoints` | PELT (Killick et al. 2012) with an L2 (mean-shift) cost | the penalty; the minimum segment length |
| `periodicity` | The peak of the normalized autocorrelation over candidate lags | the minimum correlation; the lag range |
| `trend-segments` | Monotonic runs between reversals that exceed a hysteresis band of the value range, which gives SDR-21's reversal count | the band, as a fraction of the range |

No design rule sets these thresholds, so every one is **INFER**:

- flagged as experimental in 07 and in every annotation's parameters;
- listed in the design-rules change log, like SDR-43 and SDR-44;
- recorded as a finding in C05-T01.

They change only through design-rules change control.

**Test references.** statrs (0.19.1, MIT) and the augurs crates for outliers, change points and seasonality (MIT OR Apache-2.0) are dev-dependencies of a new, unpublished test crate, `crates/engines/sonarithm-echometric/reference`. It declares `rust-version = "1.89"` and `publish = false`, so the MSRV job checks it at 1.89 and ECHOMETRIC itself stays at 1.88. It runs ECHOMETRIC's detectors and the reference implementations on the A08 generators' series, and checks agreement within a stated tolerance:

- extrema and outliers: the same rows;
- change points: within the minimum segment length;
- periodicity: the same lag.

The references never run in the compile path. C05-T01 confirms their licences, and those of every crate they pull in, with `cargo deny check`.

### Consequences

- Good: the same insights, Scores and descriptions on every platform. The `disallowed-methods` canary catches a regression.
- Good: ECHOMETRIC stays at the core MSRV and builds for wasm32. No new runtime dependency.
- Good: the reference tests compare against established implementations.
- Bad: about five short algorithms to maintain, and bugs of our own. The reference tests and the precision and recall tests on labelled sets (C05-T02, T03) are the guard.
- Neutral: augurs' forecasting and MSTL features are not available to the detectors. If a later detector needs them, it needs a deterministic port or a new ADR.

### Confirmation

- C05-T01:
  - the canary test under `tests/disallowed_methods.rs`;
  - `cargo deny check` over the workspace, the reference crate included;
  - a determinism property test (byte-identical annotations across runs).
- C05-T02 and T03: precision and recall per detector on the labelled generators (the C05 exit criterion), and the reference crate's agreement tests.

## Pros and cons of the options

### Option 1

- Good: least code.
- Bad: platform-dependent insights (F-204's failure again), and the 1.89 MSRV in an engine crate, against the core MSRV (02 §2).

### Option 2

- Good: deterministic.
- Bad: a port carries statrs' and augurs' structure without their tests. The licence obligation (keep the copyright notice) applies to every ported file.

### Option 3

- Good: deterministic, at the core MSRV, and checked against the originals.
- Bad: own code to maintain (above).

## More information

- [ADR-0013](0013-determinism-and-provenance.md) §1
- [F-204](../research-paper/findings-log.md)
- notes/04 §7 (statrs Adopt, augurs Trial, anomaly_detection Avoid)
- [supply chain](../engineering/supply-chain-and-licensing.md): anomaly_detection stays banned (`deny.toml`)
- implementation plan C05-T01 to T03, whose "through augurs and statrs" this ADR replaces

Revisit if statrs or augurs gains a `libm` mode or a deterministic build option.

**Amendment (2026-11-26, C05-T02).** The outlier method's table row now reads the robust z-score of residuals from a 21-value moving median, not of the raw values. A robust z over the whole series missed spikes on GEN-011's periodic base (recall 0.66), and one inside each window called noise an outlier (precision 0.87); the residual form gave precision 0.96 and recall 1.0 ([F-205](../research-paper/findings-log.md)). The method is still a robust z-score on `libm` arithmetic, with statrs as its reference.

**Amendment (2026-11-28, #506 review).** On the outlier detector's fallback path (a zero MAD), a value must also be at least `FALLBACK_SEPARATION` = 3 times the largest other residual, so sparse counts and integer readings that rarely change no longer flood with outliers. The factor is INFER, and annotations record it as `fallbackSeparation` ([F-208](../research-paper/findings-log.md)). The normal path is unchanged. Also from that review: an annotation's confidence is now the lowest among the rows it covers (AGENTS rule 7), and `extrema` no longer lists a global extreme a second time as a local one.

**Amendment (2026-11-28, C05 batch: #507 and the #506 review's findings 4 to 6).**
- **`extrema`'s band:** it is 10 % of the robust range, the 5th to 95th percentile (INFER, [F-209](../research-paper/findings-log.md)), so one spike no longer hides every reversal.
- **The outliers' ends:** they stay unjudged by default. Theil–Sen end lines (`endFit` `theil-sen`, experimental) found every end spike, but raised false alarms at ordinary ends to about seven times the interior's rate ([F-207](../research-paper/findings-log.md)).
- **Confidence:** an estimate without a stated confidence counts as 0 in an annotation's confidence (SDR-43's rule).

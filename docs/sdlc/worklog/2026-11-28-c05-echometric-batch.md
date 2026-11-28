# 2026-11-28: C05 ECHOMETRIC batch (T03, #507, #508 and the reviews' follow-ups)

> **Date(s):** 2026-11-26 to 2026-11-28. **Phase:** C05. **Author:** project lead, written by an AI agent (Claude Code), reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [07 ECHOMETRIC](../../architecture/07-engines.md) · [ADR-0031](../../decisions/0031-echometric-numerics.md) · [RFC-0009](../../rfcs/0009-echometric-insights-in-the-score.md) · [ADR-0017](../../decisions/0017-ipc-protocol-v1.md) · [earlier worklog](2026-11-26-c05-echometric.md)

## Context and phase

- **Phase:** C05, Lane R. It is one batch PR under the owner's rule of 2026-11-28: build large pieces locally and push rarely.
- **Base:** #506 (C05-T01 and T02), with main merged through #498.

## What was done

- **C05-T03:**
  - `changepoints` (PELT), `periodicity` and `trend-segments`;
  - GEN-012;
  - AUDAXIS's track-aware semantic segments;
  - augurs references (a pinned augurs-changepoint 0.10.3 fixture, and augurs-seasons live).
- **#507:** Theil–Sen end lines for outliers, as an experimental option. The default is unchanged by measurement ([F-207](../../research-paper/findings-log.md)).
- **#508:** `sonarithm_data::dictionary_keys` replaces arrow's `normalized_keys` in NUMISONA, AUDAXIS and the data view, and a clippy entry bans it. `fingerprint.rs` stays as #509 left it.
- **The #506 review:**
  - an insight's confidence follows its rows (SDR-43's rule for an unstated estimate);
  - insights on estimates say so, a trend segment over all its points;
  - F-208's cost stated exactly;
  - the extrema band from the robust range ([F-209](../../research-paper/findings-log.md));
  - the detectors read every type NUMISONA plays;
  - `audify` honours its track `filter` through `play_rows` and is analysed on the field it plays (`SNR-ENG-466`).
- **The #498 review:** `function_info` gives the function behind a dataset by its ID, so the Inspector shows a second function, or one reopened from a project (ADR-0017).

## Measurements

- **F-207, 1000 seeds per base:**
  - Theil–Sen ends find every end spike.
  - Its false alarms at ordinary ends are 0.80 to 0.90 %, against the interior's 0.12 % per value, so the ends stay unjudged.
  - A 100-seed first run could not tell the two apart.
- **F-209:** one spike no longer hides the reversals. GEN-001's counts and T03's tests are unchanged.
- **Replication:** all 129 generated files match. The `audify` `src` change moved no golden file or record.

## Surprises

- `audify` ignored a track's `filter`.
- `audify` tracks were never analysed, because they have no field channel.
- The first F-207 run's end precision was 1.0 by construction (both ends always spiked). It needed the false-alarm half.

## Next

- C05-T04 onwards. T06's second half (navigation queries, composer and CLI) waits for Lane U's explorer batch.

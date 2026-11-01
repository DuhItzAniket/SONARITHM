# 2026-11-01: NUMISONA glide hold under `secondsPerUnit` (#440)

> **Date(s):** 2026-11-01. **Phase:** C03 (found in C03-T07; NUMISONA code from A06). **Author:** project lead, written by an AI agent (Claude Code), reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [#440](https://github.com/DuhItzAniket/SONARITHM/issues/440) · [#441](https://github.com/DuhItzAniket/SONARITHM/issues/441)

## Context and phase

- Issue: #440, found while CURVASONA investigated F-200; it blocks #441 (function specs on x-linear time).
- Question: a glide paced by `secondsPerUnit` ended one unit after its last point. Deliberate rule or bug?

## What was done

- Traced the hold to `Timeline.step` in `sonarithm_numisona::encode`. It is documented as "the nominal step between points" (A06-T04, #151: glides "hold their last value one step"). Under `secondsPerUnit` it was set to one unit. No doc, RFC or ADR defines a one-unit hold, so this is a bug.
- Fix: under `secondsPerUnit` the step is the mean spacing of the points in time-field units times the seconds per unit, or one unit for a single point. The other paces are unchanged.
- Conformance test `glides_hold_one_point_spacing_under_seconds_per_unit`; doc 07 NUMISONA "Timing as built"; F-220.

## Decisions

| Decision | Why | Record |
|---|---|---|
| Mean spacing, not median or last gap | It is deterministic and needs no sort. It matches `duration`, where the hold is the even spacing, and unevenly placed points (for example inserted critical points) do not make the hold depend on one gap | `encode::mean_spacing` |
| Spacing computed in units, then scaled | Integer-spaced data then keeps a hold of exactly `seconds` (no rounding), so existing Scores and hashes do not change | `encode::timeline` |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| Score changes from the fix | None in the replication gallery (86 files), golden-audio cases or snapshots | `cargo xtask replication`, `cargo xtask golden`, workspace tests, Windows | F-220 |

## Findings to log

- F-220: the `secondsPerUnit` glide hold was a whole unit, not one point's spacing.

## Next steps

- [ ] #441 (CURVASONA): move function specs to x-linear time, now that a glide ends one spacing after its last point.

# 2026-11-01: render voice pools and steals (#443)

> **Date(s):** 2026-11-01. **Phase:** B04 (render code from A05). **Author:** project lead, written by an AI agent (Claude Code), reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [#443](https://github.com/DuhItzAniket/SONARITHM/issues/443) · F-211 · F-221

## Context and phase

- #443, from Lane U's F-211: playback voice pools could overflow and a stolen voice click. Real-time code (AGENTS rule 2), so the design went to the coordinator first. It was approved with a cap condition, at 4× speed.

## What was done

- **Reproduced first.** Two tests failed on main: 12 steals for 2 ms notes 3 ms apart at 4× speed, and a 0.293 output step from a second seek inside a seek crossfade.
- **Pools sized from sounding length.** `plan::build::sounding_end` uses at least two ramps times the fastest speed (4×). Polyphony is capped at `MAX_TRACK_POLYPHONY` = 256, the same bound as the seek's resume scan.
- **Steal choice.** A voice already fading out is taken first, then the oldest (`VoicePool::allocate`).
- **De-click.** Each voice keeps its last output (updated once per run). On a steal, `Graph`'s per-track `Residual` adds that value and fades it to zero over 5 ms.
- **Tests.** The two reproductions; a forced steal with a pool of one; a pathological track past the cap; a `no_alloc` steal case with two seeks in a row; the polyphony property tests counting sounding length.
- **Docs.** 05 §5 (pool sizing), 06 §3 (voices), F-221.

## Decisions

| Decision | Why | Record |
|---|---|---|
| Count at 4× speed | No steals at any supported speed; voice memory is small next to correctness (coordinator) | `sounding_end` |
| Cap 256 polyphony per track | No voice limit existed. A seek restarts at most 256 events per track, so the 2× guarantee is only defined up to it. A capped pool is 512 × 520 bytes ≈ 266 KB | `MAX_TRACK_POLYPHONY` |
| De-click by residual, not extra voices | No allocation, no new voice slots, a continuous output at the steal | `graph::Residual` |
| Keep 2× pools for seeks | A double seek is rare; the fading-first steal and the de-click cover it (coordinator) | 05 §5 |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| Steals before / after, 2 ms notes 3 ms apart at 4× | 12 / 0 | 48 kHz, offline graph, block 256 | F-221 |
| Output step at a steal, second seek inside a crossfade, before | 0.293 (fails ≤ 2× the sine's natural step); after: passes | as above | F-221 |
| `size_of::<Voice>()` | 520 bytes | Windows x86-64, debug | 05 §5 |
| Golden and replication output | unchanged | `cargo xtask golden`, `cargo xtask replication` | F-221 |

## Findings to log

- F-221: steals came from playback speed and double seeks, not short notes at 1×, and a steal clicked; fixed by sizing, steal choice and a residual de-click.

## Next steps

- [ ] PB-03: a benchmark of a dense short-note track at the cap, when the next PB-03 rerun happens.

# 2026-11-01: the compiler records context providers that contribute (#434)

> **Date(s):** 2026-11-01. **Phase:** C03 follow-up (compiler). **Author:** project lead, written by an AI agent (Claude Code), reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [#434](https://github.com/DuhItzAniket/SONARITHM/issues/434) · F-197 (CURVASONA)

## Context and phase

- #434, from F-197. The compiler recorded every registered context provider in `header.engines`, whether or not it added a track. It also warned `SNR-COMP-023` only when no provider was registered at all, so CURVASONA's or SONOTRACE's provider silenced a warning that is about AUDAXIS's cues.
- `header.engines` is provenance (ADR-0013), so the design went to the coordinator first: the definition, the measured impact, and the warning's trigger.

## What was done

- **Header:** a context provider is recorded only when it adds a track. Shapers are still recorded whenever they run, and the docs now say so.
- **Warning:** `ContextProvider::serves_context_cues()` is a default method in `sonarithm-core`, `false` by default and `true` for AUDAXIS. `SNR-COMP-023` fires when no registered provider serves the cues. Its new text drops the stale "AUDAXIS arrives in B03".
- **Tests:** `a_context_provider_is_recorded_only_when_it_adds_a_track` and `snr_comp_023_fires_only_when_no_provider_serves_context_cues`.
- **Dated amendments:** 05 §4 (`header.engines`), a note in ADR-0013, and the doc comments on `ScoreHeader.engines` and the manifest's `engines`.

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| Golden cases changed | 0 of 10 (30 checks identical) | `cargo xtask golden`, Windows | this worklog |
| Replication files changed | 0 of 86 | `cargo xtask replication` | this worklog |
| Parity Score and render | unchanged | `cargo test -p sonarithm --test parity` | this worklog |
| Snapshots changed | 7 CLI snapshots, the `SNR-COMP-023` message text only (2 sat behind the first snapshot of the same test) | workspace tests | PR body |

## Decisions

| Decision | Why | Record |
|---|---|---|
| "Contributed" for context providers, "ran" for shapers | It matches the documented meaning of `header.engines`; telling a no-op shaper apart would need a before-and-after comparison of the Score | 05 §4, ADR-0013 note |
| An availability trigger, not an output trigger | AUDAXIS may rightly add nothing when a spec turns every cue off; the warning is about whether the cues can be produced at all | `serves_context_cues` |

## Next steps

- [ ] CURVASONA: drop `uses_curvasona` and register CURVASONA in every facade compile once this lands (the follow-up named in #434).

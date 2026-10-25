# 2026-10-25: Stage B Lane R, resumed

> **Date(s):** 2026-10-25. **Phase:** B03, B04, B05, B06 (Lane R). **Author:** AI agent (Claude Code) in the app-core and SONOTRACE session, reviewed by the coordinator session.
> **Related:** [worklog conventions](README.md) · [the 2026-10-21 entry](2026-10-21-b-lane-r-app-core.md) · [implementation plan §3](../../planning/implementation-plan.md#3-stage-b--accessible-desktop-task-level)

## Context and phase

- Phase: Stage B, Lane R, after the owner's go to continue. `main` had B01-T01, T02 and T06 (#270, #271, #272), ADR-0020 (#273) and AUDAXIS's earcon set (#275).
- Issue(s) / PR(s): B04-T03 [#277](https://github.com/DuhItzAniket/SONARITHM/pull/277); B06-T04 (#244).
- Starting point: six tasks built and tested on a stack of branches (B04-T03, B05-T01, B05-T04, B03-T07, B06-T05, B03-T08), waiting for their PR slots.

## Goals

- Rebase the stack onto `main` and open its PRs in order, at most two code PRs at a time.
- Build B06-T04, now that B03-T06's earcons are in, as the second open PR: it is independent of the stack.

## What was done

- The stack rebased onto `main` (7516b60); conflicts were documentation, the render crate's export list (`EarconSound` beside `DUCK_LEVEL`) and the i18n catalogue list (`earcons.ftl` beside `stream.ftl`).
- B04-T03 opened as #277 after `cargo xtask ci` passed locally.
- B06-T04: `sonotrace::monitor`, the monitoring soundscape as a context provider: a bed whose brightness follows detector events (SDR-35), `E-STREAM` per event, `E-ALARM` only for configured high-salience alarms, the experimental comfort report (SDR-39); golden case `monitoring-soundscape`.

## Decisions

| Decision | Why | Record |
|---|---|---|
| The soundscape is a context provider active only in `monitoring` mode | Providers get the whole spec and the annotations; a shaper sees neither | 07 SONOTRACE as built |
| Salience moves brightness, not level | sound-design §10.2 and hearing safety | F-143 |
| `E-ALARM` only for `playback.monitor.alarms` with `salience: "high"` | 07's "alarm tier only for salience: high"; SDR-35 reserves urgency for true alarms | 07 SONOTRACE as built |
| The comfort budget is measured and reported, not enforced | SDR-39 is INFER; testing-strategy §6 asks for the behaviour present and flagged experimental | `SNR-ENG-241` |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| No detectors to react to | ECHOMETRIC's detectors arrive in C05 | The soundscape reads the compile's annotations from any analyzer; tests pass annotations directly |

## Findings to log

- F-143: on the monitoring bed, event salience shows as a spectral-centroid rise of 39 %, 51 % and 97 % at constant level — [findings log](../../research-paper/findings-log.md)

## Next steps

- [ ] B05-T01, B05-T04, B03-T07, B06-T05, B03-T08 as their predecessors merge (agent).
- [ ] B06-T07 stream soak once B06-T05 is in (agent).
- [ ] HP-11: the owner's listening check of the monitoring soundscape (owner, #129).

## Addendum 2026-10-26: the rest of the stack, B06-T07, and the Lane R summary

### What was done

- The stack landed in order, each rebased onto `main` and verified with `cargo xtask ci` first: B05-T01 (#280), B05-T04 (#283), B03-T07 (#285), B06-T05 (#286) and B03-T08 (#352). #286 first failed the `frontend` typecheck: the rebase had left a duplicate `stream` key in the pseudo-locale samples. The local checks now include `pnpm -r typecheck`.
- B06-T07 (#247): live playback through the app core (`live_start`, `live_stop`).
  - About once a second, the newest window of the stream is compiled (`Sonarithm::compile_stream` over `Snapshot::within`), placed on the stream's timeline (`Score::offset`) and swapped in while playing (`Player::swap`).
  - `TransportState` gained `live` and `xruns`.
  - The `stream_soak` example and the nightly `stream-soak` job measure PB-11.
- The 1-hour soak on HW-01 (null device) passed (F-144):
  - memory flat at about 26 MiB;
  - all 3,601,204 rows received;
  - the playhead at most 2.04 s behind the newest data;
  - a full 10 s window compiles in 8.2 ms, so no `encode_incremental` RFC is needed.
- #282 filed: the `no_alloc_device` test needs more than 200 callbacks in 3 s, and a slow Windows runner fell short.

### Decisions

| Decision | Why | Record |
|---|---|---|
| Live playback recompiles the whole window and swaps the Score, rather than appending events | The compile takes about 1 % of the period, and a swap keeps one code path (spec in, Score out) | 07 SONOTRACE and 09 §2.1 as built; F-144 |
| Each window's Score is shifted onto the stream's timeline | The graph's position carries over a swap, so no seek is needed and nothing jumps | `Score::offset` |
| The stream keeps 100,000 rows; the spec's `window` limits only what is compiled | Memory stays bounded whatever the spec says (PB-11) | 03 §8 as built |
| Underruns on the null device are reported, not gated | They count timer misses under other load; PB-11's xrun check needs a real device | testing-strategy §9.3 |

### Lane R summary (Stage B before Gate U)

Lane R built the headless app core and SONOTRACE's stream path, everything the desktop UI will drive after Gate U.

| Phase | Tasks done (PR) |
|---|---|
| B01 | T01 app core and IPC v1 (#270), T02 typed IPC (#271), T06 network-deny harness (#272) |
| B03 | T07 speech sequencing and self-voicing (#285), T08 profiles and presets (#352) |
| B04 | T03 view tiles and table pages (#277) |
| B05 | T01 spec patches with undo (#280), T04 project bundles, autosave and recovery (#283) |
| B06 | T01 `StreamTable` (#261), T02 stream adapters with consent (#265), T03 SONOTRACE live and transport (#263), T04 monitoring soundscape (#278), T05 stream notices and controls (#286), T07 stream soak (in review) |

- **Decisions:** ADR-0017 (IPC protocol v1), RFC-0002 and ADR-0018 (project format v1), ADR-0020 (profiles and settings layers).
- **Findings:** F-140 to F-144.
- **Left for Gate U:** the Lane U tasks, including B06-T06's stream UI.
- **Left for the owner (#129):**
  - HP-11 listening checks: the monitoring soundscape and the scrub grains;
  - the M-01 screen-reader runs of speech sequencing (NVDA ducking modes);
  - PB-11's underrun check on a real device: `cargo run -p sonarithm-app --example stream_soak --release -- --output system --seconds 3600 --max-xruns 0`, at −60 dB;
  - optionally, making `network-deny` a required check.

### Next steps

- [ ] B06-T07 merges (agent; auto-merge on).
- [ ] The owner steps above (owner, #129).
- [ ] #282: make `no_alloc_device` count to a minimum instead of a fixed 3 s window (agent or owner).

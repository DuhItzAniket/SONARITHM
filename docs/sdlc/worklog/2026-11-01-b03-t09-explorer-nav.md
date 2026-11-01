# 2026-11-01: B03-T09 (1), the explorer's verbs on the one cursor

> **Date(s):** 2026-11-01. **Phase:** B03. **Author:** Lane U, written by an AI agent (Claude Code), reviewed by the owner.
> **Related:** [worklog conventions](README.md) · [ADR-0017 "Navigation"](../../decisions/0017-ipc-protocol-v1.md#navigation)

## Context and phase

- Phase: B03 ([roadmap](../../planning/roadmap.md)). Task B03-T09: the chart explorer, keymap v0 and remapping.
- Starting point: AUDAXIS had the cursor, its verbs and "where am I" (B03-T02), and the app had Rust's one cursor for the table and the plot (B04-T06). No IPC reached the verbs.

## Goals

- A design note to the coordinator before code: the cursor, keymap v0 against screen-reader keys, where remapping is stored, and the IPC additions.
- PR (1): `nav_move` and `nav_where` on the one cursor.

## What was done

- **Design approved** in three PRs: (1) the app core, (2) the explorer UI and keymap v0, (3) remapping (`Settings.keymap`, a new `Settings.keyboard.singleKeyCommands` for KB-02's group switch; project and spec layers cannot set keys).
- **AUDAXIS:** `Cursor::place`, `set_excluded` and `series`, and a composer `Memory`, so an unchanged series is not named again across commands (keyboard §8.1).
- **App core** (`app/nav.rs`):
  - The cursor model keeps AUDAXIS's cursor beside the row. A verb moves the node, and the row follows: the point's row, or the series' point nearest x.
  - A compile finds the node again from the row, and the excluded series by name.
- **Filters** (keyboard §3): Enter on a filter takes its series out of playback and view.
  - The transport plays the compiled Score with that track removed and nothing else changed. A test compares every other track, the header and the markers byte for byte.
  - `TransportState.excludedTracks` names the removed tracks.
  - Exports render the whole Score. An export started while a series is excluded announces "Export includes all series; exclusions apply to playback only".
- The readout lists each excluded series in words.

## Decisions

| Decision | Why | Record |
|---|---|---|
| Exclusion covers playback in B03, by loading a filtered copy of the Score | keyboard §3 says "playback and view"; the copy is still the compiler's Score IR | ADR-0017 "Navigation" |
| `excluded` as `{ node, label }` | the readout names the series | ADR-0017 "Navigation" |
| The point cue is its own PR | the transport has no one-shot cue path yet; the render graph's `Command::Cue` exists | design note to the coordinator, approved |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| A stolen voice clicks | `Voice::start` replaces the whole voice | F-211; cues in the point-cue PR (8 slots, 4 sounding), playback voices in #443 |
| Clippy: a needless `Result`, a single-arm `match`, a by-value test helper | first build of the branch | fixed |

## Measurements

None.

## Findings to log

- F-211 (logged).

## The point cue (2026-11-01)

- The design was approved before code. The render graph already had `Command::Cue` and preallocated cue voices under `no_alloc`, so the control thread does most of the work: `Transport::cue`, dropped while playing (keyboard §9.2 item 3), the output opened on the first cue, then `Sequencer::cue` so speech waits (A11Y-20).
- The cue is the point's own note: a binary search of its track's events by the point's time, then the note whose `src` is the row. That avoids a per-row index, which would cost about 80 MB at a million rows.
- **The steal clicked** (F-211): `Voice::start` replaces a voice at once. Now there are 8 cue slots with at most 4 sounding, and the oldest sounding cue fades over the ramp. Tests: the step at the fade stays within twice a sine's own steepest step; a full pool steals a fading voice and counts it; `no_alloc` sends ten cues in one block.
- Not measured: the 50 ms key-to-onset budget, which needs the loopback harness on HW-01 (an owner step on #129).

## Next steps

- PR (2): the Explorer component, keymap v0, the query menu, modality and verbosity toggles, E2E for every verb.
- PR (3): remapping.
- The point cue, after PR (1).

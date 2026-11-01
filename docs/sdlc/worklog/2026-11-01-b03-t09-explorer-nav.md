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

## PR (2): the explorer and keymap v0 (2026-11-01)

- `Explorer` (`apps/desktop/src/explorer/`): one Tab stop in the Plot tab. Only the chart surface has `role="application"`; it is named "Chart explorer, <dataset>", described by a visible hint, and has no `aria-activedescendant`, so a move is spoken once, by the app core (OD-30). It holds no position: Rust's cursor does.
- Keymap v0: 25 commands in the `explorer` scope (keyboard §4.2, §5.4), registered for the whole app by `ExplorerCommands` through a new `useCommandTable`, so the palette runs them from any tab (KB-01) while their keys work only in the explorer (KB-02). G and Primary+F open a value dialog that returns the focus to its opener; W, R, and H or F1 (`help.explorer`).
- **KB-04 against KB-07.** The lint refused Alt+arrows on every platform as webview history, but KB-07 makes Jump Option+arrows on macOS. History is Alt+arrows in WebView2 and WebKitGTK only (WKWebView uses Cmd), so the lint and KB-04's text now limit that rule to Windows and Linux.
- Not yet: the query menu with `nav_query`, remapping and the group switch (PR 3), compare, selection and polarity (their features).
- The E2E checks which verb each key sends against the mock host; the app core's tests cover what each verb does.

## PLOTSONA's review of #451 (2026-11-01)

- **The toggles announced changes nothing made.** T, S, B, V and Shift+T wrote `settings.modalities` and `verbosity` and said "Speech off", but nothing reads those values, and the effective value comes from the profile layers (`profile::effective`), not the base settings. They are out of this PR. They come back in their own change, where the app core gates speech, cues, highlight and braille on the effective modalities, writes the session layer, and announces with the mode earcon.
- **The commands vanished with the Plot tab.** React Aria unmounts an inactive tab panel, so `view.focusExplorer` and the explorer's palette entries disappeared on the Table tab. The commands are now registered by `ExplorerCommands` in App. "Go to the chart explorer" selects the Plot tab with `flushSync`, then focuses the surface.
- The value dialog no longer forces the focus onto the explorer; React Aria returns it to the opener.
- F1 in the explorer runs `help.explorer` (§5.4). The deferral note names L (legend) and Z and Shift+Z (zoom to node).
- The palette test walked the list from the top for each command, with a check per key, which grew quadratically. It now presses straight to each command's position and checks once.

## Next steps

- PR (2): the Explorer component, keymap v0, the query menu, modality and verbosity toggles, E2E for every verb.
- PR (3): remapping.
- The point cue, after PR (1).

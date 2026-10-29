# 2026-10-29: A02 NVDA walkthrough, agent-driven

> **Date(s):** 2026-10-29. **Phase:** A02 (checkpoint under OD-15). **Author:** AI agent (Claude Code, Claude Opus 5.5) in a session started by the coordinator session, to be reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [A02 gate report, Spike 5](../reviews/A02-prototype-gate.md#nvda-agent-driven-speech-viewer-transcript) · [WALKTHROUGH.md §1](../../../spikes/tauri-webview/WALKTHROUGH.md) · [F-046](../../research-paper/findings-log.md#f-046--nvda-20263-speaks-the-managed-live-regions-as-designed-but-nothing-from-the-hosts-ui-automation-notifications) · [Gate U worklog](2026-10-29-gate-u-provisional.md)

## Context and phase

- OD-15 (amended 2026-10-29) opened Gate U provisionally. The NVDA walkthrough runs as a checkpoint, which the coordinator drives through desktop control, reading NVDA's Speech Viewer.
- Starting point: the prototype binary was already downloaded. NVDA was not installed when the session started; the owner installed NVDA 2026.3.

## Goals

- Run WALKTHROUGH.md §1, steps 1–19, and record what NVDA said. Skip step 20, a system setting.
- Give an OD-05 reading from steps 11–15, and leave the decision to the coordinator.

## What was done

- Ran steps 1–19 against `spike-tauri-webview.exe --page a11y.html` in WebView2 156.0.4314.50, with NVDA 2026.3 on default settings. Recorded the table and the OD-05 reading in the gate report.
- Added `spikes/tauri-webview/results/a11y-windows-nvda.json`: the per-step transcript and the page's announcement log.
- Logged F-046; updated the A02-T07 row and the log in status.md.

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| Clicks and keys did nothing in NVDA's menu, and the open menu then blocked the mouse | NVDA runs with UI Access; Windows blocks injected input to it | The owner opened the Speech Viewer by hand. Nothing in NVDA was changed |
| The NVDA key could not be pressed | The desktop-control tool sends no chord with Insert | Not needed: steps 1–19 use only page keys and quick navigation, which NVDA takes from injected input |
| The Speech Viewer was hidden behind the maximized prototype | Window stacking | Restored the prototype window and moved it to the right of the Speech Viewer |
| `--out` wrote no file | `a11y.html` never calls the host's `report` command | Assembled the JSON from the Speech Viewer and the page log, and said so in the file |
| A held arrow key gave one key-down | Injected key-downs do not auto-repeat | Approximated with 10 presses about 35 ms apart; a real held key is left to the owner |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| NVDA walkthrough tally | 16 pass, 2 partial (5, 16), 1 fail (14), 1 skipped (20) | HW-01, NVDA 2026.3, WebView2 156 | Gate report, F-046 |
| Live route, 10 updates in 1 ms | 1 spoken ("Rapid update 10 of 10") | Steps 12 and 17 | F-046 |
| Native route | 0 of 3 texts spoken; every call returned success | Step 14 | F-046 |

## Next steps

- Owner: step 20 (contrast theme), and listening checks on steps 4, 13 and 17 (focus-mode sound, alert interruption, held key).
- Owner: the Narrator walkthrough (§2). Steps 14 and 15 there decide whether the native route has any value.
- Coordinator: decide OD-05. Also decide whether to file the native-route failure as an `a11y` rework issue under R-33, or to try a different sender element for `UiaRaiseNotificationEvent` (TODO(verify)).
- B02: the tree reads as "table", with "not selected" on every row (F-045, F-046).

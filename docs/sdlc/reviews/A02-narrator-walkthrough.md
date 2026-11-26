# A02 Narrator walkthrough: script, tasks and pass criteria

> **Status:** Prepared, **not run** (2026-11-26). Written by an AI agent for the coordinator session. The owner chose an agent-driven run ([OD-33](../../planning/open-decisions.md#closed-decisions)). It starts on the owner's laptop (HW-01) only when the owner says go in the walkthrough session: desktop control takes over the owner's screen for the whole run, and the owner grants app access when asked.
> **Related:** [A02 close-out](A02-closeout.md) · [gate report, Spike 5](A02-prototype-gate.md#spike-5-screen-readers-a02-t07) · [WALKTHROUGH.md §1–2](../../../spikes/tauri-webview/WALKTHROUGH.md) · [NVDA run worklog](../worklog/2026-10-29-a02-nvda-walkthrough.md) · [F-046](../../research-paper/findings-log.md#implementation-findings) · [OD-05](../../planning/open-decisions.md) · [08 §4–5](../../architecture/08-interaction-and-accessibility.md) · [severity scale](../../design/accessibility-conformance-plan.md#9-accessibility-issue-severity)

This is the Narrator half of HP-01. It reuses the NVDA run's materials: the same prototype page, the same 19 steps of WALKTHROUGH.md §1, and the same results format (`spikes/tauri-webview/results/a11y-windows-nvda.json`). It adds what Narrator needs and what the NVDA run left open.

## What it decides

| Question | Steps | Feeds |
|---|---|---|
| Does Narrator speak the live-region route once, superseded and not dropped? | 11–13, 17 | OD-05 final |
| Does Narrator speak the native UI Automation route that NVDA ignored (F-046)? | 14, 15 | OD-05 final, [#382](https://github.com/DuhItzAniket/SONARITHM/issues/382) |
| Are React Aria's Tree, Toolbar, Slider and RadioGroup usable with Narrator in WebView2? | 1–10, 16–19 | ADR-0003 and ADR-0002 confirmed for Windows (HP-04); B02-T10 may start |
| Do React Aria's own announcements and the app's announcer avoid speaking over each other in the desktop app? | Part B | #388, B03-T04 |
| The NVDA leftovers: focus-mode sound, alert interruption, held key, contrast theme | Part C | P1 in the gate report |

## Two ways to run it

| | Owner-run | Agent-run (desktop control) |
|---|---|---|
| Who presses keys | The owner | The agent, through injected keys and clicks |
| Transcript | The owner writes what Narrator said | Windows Live captions, if pre-check P0 shows that it captions Narrator; otherwise the owner listens and the agent writes it down |
| Screen | The owner's own session | **The agent takes over the screen** for the whole run. The owner shouldn't use the laptop meanwhile |
| Needs | About an hour of the owner's time, TODO(verify) | The owner's explicit go-ahead; the owner starts Narrator and turns scan mode on or off when asked; the owner listens for Part C |

The NVDA run found three limits that will also apply here ([worklog](../worklog/2026-10-29-a02-nvda-walkthrough.md#problems-and-how-they-were-solved)):

- injected input may not reach the screen reader's own windows;
- the screen-reader key (Insert or Caps Lock) could not be sent in a chord;
- a held key gives one key-down, without auto-repeat.

So in an agent-run, the owner starts Narrator (`Win+Ctrl+Enter`) and toggles scan mode (`Narrator+Space`). Pre-check P0 tests whether the agent can send those itself.

## Setup

1. **Machine.** HW-01, on mains power, headphones on. Close other apps that speak or play sound.
2. **Versions.** Write these down:
   - the Windows version and build (`winver`);
   - the WebView2 Runtime version: `(Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}').pv` in PowerShell (the key is present on HW-01);
   - Narrator's settings, if anything differs from the defaults (Settings → Accessibility → Narrator). Use the defaults: don't change verbosity or the voice.
3. **The prototype (Part A).** Use the binary the NVDA run used: `.claude/a02-walkthrough/src-tauri/target/release/spike-tauri-webview.exe` in the main checkout (local, not in git; the NVDA results note that its build should be checked against the spike, TODO(verify)). Or build it: `pnpm install && pnpm tauri build --no-bundle` in `spikes/tauri-webview`. That is a Rust build, so it needs the coordinator's builder slot. Start it with `--page a11y.html`. Leave the page's announcement log visible: it shows what the page sent, with times.
4. **The desktop app (Part B).** A local debug build of the desktop shell from `main`, so nothing is downloaded: `pnpm install`, then `pnpm -C apps/desktop tauri build --debug --no-bundle`, which builds the frontend and embeds it. The binary is `target/debug/sonarithm-desktop.exe`. The Rust part is a full workspace debug build, so it needs the coordinator's builder slot shortly before the run. The fallback is the latest `desktop-nightly` Windows bundle, installed as [release-process](../../engineering/release-process.md#desktop-release) says; that is a file download, so the owner fetches it. Sample data: `examples/data/ds-001-temperature.csv`.
5. **Narrator.** Start it with `Win+Ctrl+Enter`. The Narrator key is Caps Lock or Insert. `Narrator+Space` turns scan mode on and off. In scan mode, `H` and `Shift+H` move by heading and `D` and `Shift+D` by landmark. `Ctrl` stops speech.

### Readiness on HW-01 (checked 2026-11-26, read-only)

| Item | State |
|---|---|
| Windows | 26H2, build 26300.9974 |
| WebView2 Runtime | 158.0.4370.33. The NVDA run had 156.0.4314.50, so the two runs differ by webview version; record both |
| Narrator | Never configured: the `HKCU\Software\Microsoft\Narrator` key holds only `NoRoam\RunningState`, so every setting is the default. Not running |
| Audio | One output endpoint, "Speakers (Realtek(R) Audio)". Narrator and Live captions both use the default output. If headphones show up as a second endpoint when plugged in, make them the default before starting |
| Live captions | `LiveCaptions.exe` is installed. Whether it captions Narrator is P0 |
| Prototype (Part A) | Present: the NVDA run's binary, built 2026-10-27 |
| Desktop app (Part B) | Not built yet. `pnpm install` and the frontend build (`pnpm -C apps/desktop build`) pass in the walkthrough worktree; the Rust debug build waits for the builder slot |
| Settings folder | `%APPDATA%\org.sonarithm.desktop` doesn't exist yet: the app creates it on its first start. Check it after B1, before B2 |

### Pre-checks (agent-run only)

| # | Check | If it fails |
|---|---|---|
| P0 | Open Live captions (`Win+Ctrl+L`), then let Narrator read the prototype's title. Do the captions show Narrator's words? | No transcript from captions. The owner listens, and the agent writes down what the owner reports |
| P1 | Can the agent toggle scan mode with injected `Caps Lock+Space`, then `Insert+Space`? | The owner toggles scan mode when a step says so |
| P2 | Does an injected `Tab` move focus in the prototype while Narrator runs? | Stop: the run can't be agent-driven |

Captions are a transcription, not Narrator's own text: they can mishear a word, and they show no timing. A step judged from captions gets the note "from captions".

## Part A: the prototype, steps 1–20

Run WALKTHROUGH.md §1 steps 1–20 with Narrator in place of NVDA, with these Narrator notes. "Scan mode" is Narrator's version of NVDA's browse mode. The expected results are *content*: the name, role and state that must be spoken. Narrator's wording and order may differ from NVDA's; that is not a finding.

| # | Action (WALKTHROUGH.md §1) | Must be spoken | Narrator notes |
|---|---|---|---|
| 1 | Start the app | Window title, then the page | Record whether scan mode is on at the start |
| 2 | `H` repeatedly, then `Shift+H` | The level-1 heading, then each level-2 section heading | Scan mode on |
| 3 | `D` repeatedly | Banner, main and the named regions | Scan mode on |
| 4 | `Tab` to the radio group, `Down` | Group name; "ARIA live region", radio button, checked, 1 of 3; `Down` selects option 2 | Record whether `Down` changed the selection with scan mode on, or only after turning it off |
| 5 | `Tab` to the tree | "Dataset", the row "temps.csv, 20 rows", expanded, level 1 | Record the role Narrator gives (tree, table or grid; NVDA said "table", [#383](https://github.com/DuhItzAniket/SONARITHM/issues/383)) and whether the name is doubled |
| 6 | `Down`, `Up`, `Right` on "Fields", `Left`, `Enter` | Each row; expanded and collapsed states; "Opened fields" | If the arrows move the scan cursor instead of the tree, turn scan mode off for steps 6–10 and record that it was needed |
| 7 | `Tab` to the toolbar; `Right`, `Left`; `Tab` | "Transport", toolbar, "Play", button; arrows stay in the toolbar; `Tab` leaves it | As step 6 |
| 8 | `Space` on Loop, twice | Toggle button, pressed and not pressed; "Loop on", "Loop off" | |
| 9 | `Enter` on Play | "Playing"; the button is then named Pause | |
| 10 | `Tab` to the slider, `Right` three times | "Playback speed, times real time", slider, 1; then 1.25, 1.5, 1.75 | Note any double reading from the `<output>` live region |
| 11 | Route: ARIA live region. **Announce once** | "Compiled 4 tracks in 120 milliseconds", once | |
| 12 | **Announce 10 rapid updates** | Only "Rapid update 10 of 10" is ideal (A11Y-17) | Write down exactly what was spoken and how many updates |
| 13 | **Assertive alert** while Narrator is still speaking | "Audio device lost. Playback stopped." | Owner listening: did it interrupt the earlier speech, or follow it? |
| 14 | Route: **Native notification**; repeat 11–13 | The same three texts, from UI Automation notifications | **The key step.** Count what was spoken, and note any delay, drop, queue or interruption. Compare the page log (every call's result) |
| 15 | Route: **Both**; repeat 11 | The text once | Note whether it is spoken twice |
| 16 | `Tab` to the chart explorer | "Chart explorer: temperature over time", its description; the role if Narrator speaks it | Record whether scan mode turned off by itself on the `role="application"` element |
| 17 | In the chart: `Right` five times slowly, then hold `Right` | Each point, for example "2 March, 13.4 degrees Celsius, point 2 of 20"; while held, stale points dropped | Agent-run: approximate the held key with 10 presses about 35 ms apart, as for NVDA, and leave the real held key to Part C |
| 18 | `Home`, `End`, `H` | First point, last point, the help text | |
| 19 | `Tab` out of the chart, then `Shift+Tab` back | Scan mode back after leaving; the chart's keys work again on re-entry | |
| 20 | Contrast theme (Settings → Accessibility → Contrast themes) | Focus rings, the selected radio, the toggled Loop button and the chart cursor stay visible | A system setting: the owner runs it. Switch back afterwards |

## Part B: the desktop app (B03-T04, #388)

In the desktop app (the debug build, or the nightly), with `examples/data/ds-001-temperature.csv` open:

| # | Action | Must happen |
|---|---|---|
| B1 | With the default route (`liveRegion`), move through the chart explorer and play and pause once | Each app announcement is spoken once |
| B2 | Close the app. In `settings.json` in the app's config folder (`%APPDATA%\org.sonarithm.desktop\`, TODO(verify) the folder), set `"speech": { "announcer": "native" }`, keeping the other fields. Start the app and repeat B1 | Each app announcement is spoken once, through the native route. If the native call fails, the announcer falls back to the live regions (08, B03-T04), so a spoken text alone doesn't show which route spoke. Record what was heard; whether the app logs the route it used is TODO(verify) |
| B3 | Set the route back to `liveRegion` | — |
| B4 | React Aria's own announcements (#388): select rows in the data table and items in a list; drag and drop in the announcement-part list; type in the combo box | "selected"; the drag instructions and "dropped"; the option count. Each is spoken once, and not over an app announcement |

## Part C: listening checks (the owner, about 10 minutes, TODO(verify))

These are the NVDA run's leftovers and the timing that captions can't show. They need someone listening.

| # | Check | With |
|---|---|---|
| C1 | NVDA step 4: the focus-mode sound on entering the radio group | NVDA |
| C2 | NVDA step 13 and Narrator step 13: did the assertive alert cut the earlier speech short? | Both |
| C3 | NVDA step 17 and Narrator step 17: a really held `Right` key in the chart: only recent points spoken, no backlog after release | Both |
| C4 | NVDA step 20, if not done in Part A: the contrast theme | — |

## Pass criteria

**Per step.** Use the same three marks as the NVDA run:

- **Pass:** everything in "must be spoken" was spoken, once, and the keys did what the step says.
- **Partial:** the information is reachable but something is missing or extra: a role or state missing, a doubled name, or scan mode had to be turned off by hand where NVDA switched by itself.
- **Fail:** information can't be reached, an announcement is missing, an announcement is doubled or queued against A11Y-17, or a key doesn't work.

A partial or fail gets a severity from the [conformance plan's scale](../../design/accessibility-conformance-plan.md#9-accessibility-issue-severity) and, when it affects a component, an `a11y` issue under R-33, as #382 and #383 were.

**OD-05 (steps 11–15 and B1–B2).** The coordinator reads these against OD-05's options:

| Live route (11–13) | Native route (14) | Reading |
|---|---|---|
| Pass | Silent or worse | **Live regions only.** OD-05 final as decided provisionally; the native route stays off by default behind the announcer interface |
| Pass | Pass | **Live regions stay the default.** The native route is a working option for Narrator. Whether to offer it depends on step 15: if "Both" doubles, Both is not offered |
| Drops, doubles or queues | Pass | **Re-open OD-05 with the hybrid option.** NVDA ignores the native route (F-046), so a hybrid would need a route choice per screen reader, which is a new design question |
| Fails | Fails | **Escalate R-02.** Critical severity: announcements don't work with Narrator |

**ADR-0002 and ADR-0003, Windows (HP-04).** Narrator supports confirming both for Windows when, together with the NVDA run:

- no step in 1–13 or 16–19 is a critical or high failure; or
- each such failure has a filed issue whose fix lies in the components or the announcer (rework under R-33), not in Tauri or WebView2.

A failure caused by the shell or the webview (for example WebView2 not exposing a role or a notification) goes to the coordinator as a possible ADR-0002 issue, not a component fix. Linux and macOS stay provisional either way ([close-out §3](A02-closeout.md#3-recommendations), item 10).

## Recording the results

- **Results file:** `spikes/tauri-webview/results/a11y-windows-narrator.json`, in the shape of `a11y-windows-nvda.json`: `suite`, `date`, `walkthrough`, `method` (owner-run or agent-run, and the transcript source), `host` (machine, OS build, WebView2 version, Narrator settings changed or not), `limits`, `steps` (each with `step`, `keys`, `result`, `spoken`, `notes`, and `route` and `count` for steps 11–15), plus the page's announcement log.
- **Gate report:** a Narrator table in Spike 5, in the NVDA table's format, a tally and an OD-05 reading. Then update P2 and the deliverables matrix.
- **Also:** a finding (the next free F-number) and the A02-T07 row in `status.md`.

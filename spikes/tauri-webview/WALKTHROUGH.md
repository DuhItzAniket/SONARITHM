# A02 manual walkthroughs: screen readers, Linux and macOS

> **Status:** Owner and volunteer script for the parts of A02 that need a person or another operating system: A02-T07 (screen readers, OD-05) and the WKWebView and WebKitGTK parts of A02-T05/T06. Record what happened, not what should have happened. A step that fails because the tester did not know a screen-reader command is not a finding (dev-environment.md). Results go into the gate report, `docs/sdlc/reviews/A02-prototype-gate.md`, as a comment or a follow-up commit on its PR.

## 0. Get the build

| OS | Build |
|---|---|
| Windows | `pnpm install && pnpm tauri build --no-bundle` in `spikes/tauri-webview`, then `src-tauri\target\release\spike-tauri-webview.exe --page a11y.html`. Or download the `spike-tauri-windows-latest` artifact of the latest `spike-a02-tauri` workflow run |
| Linux | The `spike-tauri-ubuntu-24.04` artifact: a `.deb` |
| macOS | The `spike-tauri-macos-latest` artifact: an unsigned `.app` |

The artifacts are under **Actions → spike-a02-tauri → the latest run → Artifacts** (sign in to GitHub). They are kept for 30 days; re-run the workflow for a fresh build.

Every run writes JSON with `--out`. Keep the files: they are the evidence.

## 1. Windows with NVDA (owner, development laptop)

**Setup.**

1. Install NVDA from nvaccess.org. Use an installed copy, not the portable one. Write down the version (NVDA menu → Help → About).
2. Open **NVDA menu → Tools → Speech Viewer**, so spoken text is visible and can be copied.
3. Start the prototype: `spike-tauri-webview.exe --page a11y.html`. The page's announcement log shows what the page sent.
4. Wear headphones; nothing in the prototype plays audio.

**Steps.** For each step, write **Pass**, **Fail** or **Partial** and what NVDA said (copy it from the Speech Viewer).

| # | Action | Expected |
|---|---|---|
| 1 | Start the app | NVDA reads the window title and the document; browse mode is active |
| 2 | Press `H` repeatedly, then `Shift+H` | Moves through the headings: "A02 accessibility prototype" (level 1), then the level-2 section headings |
| 3 | Press `D` repeatedly | Moves through landmarks: banner, main and the named regions (Announcement route, Dataset, Transport, …) |
| 4 | `Tab` to the radio group | "Announcement route, grouping, ARIA live region, radio button, checked, 1 of 3". NVDA switches to focus mode (you hear the focus-mode sound). `Down arrow` selects the next option |
| 5 | `Tab` to the tree | Focus mode; "Dataset" and the first row, "temps.csv, 20 rows", expanded, level 1. Note whether the name is doubled ("Collapse temps.csv … temps.csv …"): the UIA tree shows that the row's cell name includes the expand button's label |
| 6 | In the tree: `Down`, `Up`, `Right` on "Fields", `Left`, `Enter` | Moves between rows; `Right` expands, `Left` collapses (states spoken); `Enter` gives "Opened fields" through the live region |
| 7 | `Tab` to the toolbar | "Transport, tool bar, Play, button". `Right` and `Left` move between the buttons without leaving the toolbar; `Tab` leaves it |
| 8 | In the toolbar: `Space` on Loop, then again | "Loop, toggle button, pressed", then "not pressed", plus "Loop on" and "Loop off" from the live region |
| 9 | `Enter` on Play | "Playing" is spoken; the button is now named Pause |
| 10 | `Tab` to the slider, then `Right` three times | "Playback speed, times real time, slider, 1", then 1.25, 1.5, 1.75. Note any double reading: the visible value is an `<output>` element, which is a live region |
| 11 | Route: ARIA live region. Press **Announce once** | "Compiled 4 tracks in 120 milliseconds", spoken once |
| 12 | Press **Announce 10 rapid updates** | Ideally only "Rapid update 10 of 10" (supersede, A11Y-17). Write down exactly what was spoken |
| 13 | Press **Assertive alert** while NVDA is still speaking | "Audio device lost. Playback stopped." interrupts or follows at once |
| 14 | Select route **Native notification**, repeat steps 11–13 | The same texts, spoken from UI Automation notifications. Write down any difference (delay, dropped, queued, interrupted) |
| 15 | Select route **Both**, repeat step 11 | Note whether the text is spoken twice |
| 16 | `Tab` to the chart explorer | Focus mode; "Chart explorer: temperature over time, application", then the description |
| 17 | In the chart: `Right` five times slowly, then hold `Right` | Each point is read, for example "2 March, 13.4 degrees Celsius, point 2 of 20". While the key is held, stale points should be dropped, not queued |
| 18 | In the chart: `Home`, `End`, `H` | First point, last point, the help text |
| 19 | `Tab` out of the chart, then `Shift+Tab` back | Browse mode returns after leaving; focus mode returns on re-entry |
| 20 | Optional: Settings → Accessibility → Contrast themes → any theme | Focus rings, the selected radio, the toggled Loop button and the chart cursor stay visible (no colour-only meaning). Switch back afterwards |

## 2. Windows with Narrator (owner)

Start Narrator with `Win+Ctrl+Enter`; scan mode toggles with `Caps Lock+Space`. Repeat steps 1–19. Narrator is the reference consumer of UI Automation notifications, so steps 14 and 15 matter most here.

## 3. Linux from an Ubuntu live USB (owner, same laptop)

**Setup.**

1. Write a current Ubuntu LTS desktop image to a USB stick, boot it from the firmware boot menu and choose **Try Ubuntu**.
2. Connect to the network. In Firefox, sign in to GitHub and download the `spike-tauri-ubuntu-24.04` artifact.
3. In a terminal, from the download folder:

   ```bash
   unzip spike-tauri-ubuntu-24.04.zip
   sudo apt update && sudo apt install -y ./src-tauri/target/release/bundle/deb/sonarithm-a02-spike_0.0.0_amd64.deb mesa-utils
   dpkg -L sonarithm-a02-spike | grep /bin/
   ```

   The last command prints the program's path; `APP` below means that path.
4. Write down the WebKitGTK version (`apt-cache policy libwebkit2gtk-4.1-0`), the session type (`echo $XDG_SESSION_TYPE`) and the GPU (`glxinfo -B | grep -E "renderer|vendor"`).
5. **Copy every JSON file to another USB stick or upload it before shutting down.** The live session keeps nothing.

**Measurements (A02-T05 and T06 on WebKitGTK).**

| # | Command | What it covers |
|---|---|---|
| L1 | `APP --autorun ipc --out ~/ipc-linux.json` | PB-09 on WebKitGTK on real hardware |
| L2 | `APP --autorun webgl --out ~/webgl-linux-intel.json` | PB-08 on the Intel GPU (Mesa) |
| L3 | `DRI_PRIME=1 glxinfo -B \| grep renderer`, then `DRI_PRIME=1 APP --autorun webgl --out ~/webgl-linux-dgpu.json` | The discrete GPU through the live session's open-source driver (nouveau). The proprietary NVIDIA driver needs an installed system: it stays pending (dual boot) |
| L4 | `WEBKIT_DISABLE_DMABUF_RENDERER=1 APP --autorun webgl --out ~/webgl-linux-nodmabuf.json` | The NVIDIA workaround of Tauri's Linux graphics guide, and what it costs |
| L5 | If the login screen offers "Ubuntu on Xorg": repeat L2 in that session | X11 versus Wayland |

**Orca walkthrough.** Start Orca with `Super+Alt+S`. `Orca+A` toggles browse and focus mode, and structural navigation uses `H` and other letter keys. Start `APP --page a11y.html` and run steps 1–19 of section 1, writing down what Orca says. Orca on WebKitGTK is the least mature combination (08 §5). A failure here that cannot be fixed within a week triggers the Electron fallback (ADR-0002).

## 4. macOS (volunteer or a university lab Mac)

**Setup.** Unzip the `spike-tauri-macos-latest` artifact. Artifacts do not keep the executable bit, and the app is unsigned, so restore the bit, clear the quarantine flag and, if macOS still refuses to start it, sign it ad hoc:

```bash
cd src-tauri/target/release/bundle/macos
chmod +x sonarithm-a02-spike.app/Contents/MacOS/spike-tauri-webview
xattr -dr com.apple.quarantine sonarithm-a02-spike.app
codesign --force --deep --sign - sonarithm-a02-spike.app
```

`APP` below means `sonarithm-a02-spike.app/Contents/MacOS/spike-tauri-webview`. Write down the macOS version and the Mac model.

| # | Command or action | What it covers |
|---|---|---|
| M1 | `APP --autorun ipc --out ~/ipc-macos.json`, with the window in front | PB-09 on WKWebView on real hardware |
| M2 | M1 again, hiding the app with `Cmd+H` right after it starts | App Nap: on the CI runner the 30 Hz Channel stream arrived at 8 messages per second (`channel30Hz1KB.msgsPerS`); compare both runs |
| M3 | `APP --autorun webgl --out ~/webgl-macos.json` | PB-08 on WKWebView |
| M4 | VoiceOver (`Cmd+F5`): run steps 1–19 of section 1 with `VO+Right`, `VO+Shift+Down` (interact) and the rotor (`VO+U`) | ADR-0003 confirmation on VoiceOver. The native route (steps 14 and 15) is Windows-only and should report an error in the log |

The audio latency harness (`spikes/cpal-latency`) is Windows-only. On macOS it needs a loopback device such as BlackHole and a port of its capture side, so that measurement stays open (risk R-22).

## 5. Recording results

Paste one table per screen reader into the gate report PR:

```markdown
| Step | NVDA <version> + WebView2 <version> | Notes (what was spoken) |
|---|---|---|
| 1 | Pass / Fail / Partial | |
```

For OD-05, steps 11–15 decide:

- **Live regions only:** they are spoken once, superseded as expected, and not dropped by NVDA or Narrator.
- **Hybrid (native notifications on Windows):** live regions double-speak, queue or drop, and the native route does not.

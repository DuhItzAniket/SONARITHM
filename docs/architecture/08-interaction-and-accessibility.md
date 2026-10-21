# 08 · Interaction and Accessibility Architecture

> **Status:** Baseline v1 (2026-10-18). Implemented in B03 (AUDAXIS v1), B02 (UI kit) and B07 (conformance).
> **Bindings and wording:** [design/keyboard-and-navigation.md](../design/keyboard-and-navigation.md). **Conformance:** [design/accessibility-conformance-plan.md](../design/accessibility-conformance-plan.md).
> **Requirements:** [A11Y-01..34](../research/design-rules.md#a11y-accessibility-requirements).

## 1. Principles

1. **One cursor, many modalities.** Sound, speech, braille, visual highlight and (later) haptics all render the *same* cursor (A11Y-10). No modality has private state.
2. **Overview first, then control, then the data** (Sharif et al. 2021; MAIDR). The default flow is: summary → play overview → explore → table.
3. **Structure is announced, not inferred.** Users always know where they are, what level they are on and where the boundaries are (Zong et al. 2022; A11Y-04, A11Y-07).
4. **The user's screen reader is the voice.** Speech goes through the user's assistive technology by default, preserving their voice, rate and braille routing (A11Y-19). Self-voicing is optional.
5. **Discrete steps underneath.** Every interaction resolves to a discrete navigation step, so keyboards, switches, gamepads and braille routing keys can all drive it (A11Y-27).
6. **The authoring UI is held to the same bar** as the exploration UI (A11Y-29).

## 2. The navigation model

AUDAXIS builds a **NavTree** for every compiled Score ([05 §8](05-compiler-and-score-ir.md#8-navtree-summary)).

```mermaid
flowchart TB
  D[Dataset<br/>title, summary L1/L2] --> S1[Series: temperature]
  D --> S2[Series: CO₂]
  D --> AX[Axes<br/>x: 1880–2025 year<br/>y: −0.5–1.3 °C]
  D --> IN[Insights<br/>max, min, outliers,<br/>change points]
  D --> F[Filters / facets]
  S1 --> G1[Segment 1880–1929]
  S1 --> G2[Segment 1930–1979]
  S1 --> G3[Segment 1980–2025]
  G3 --> P1[Point 1980]
  G3 --> P2[Point …]
  G3 --> P3[Point 2025]
  IN --> I1[Max 2024: 1.29 °C]
```

| Node kind | Children | Generated from |
|---|---|---|
| `Dataset` | Series, Axes, Insights, Filters | Score header, data, spec |
| `Series` | Segments (bins) | Data tracks (`TrackRole::Data`) |
| `Axes` | One node per axis, with range, unit and polarity | Channel ranges and legends |
| `Insights` | One node per annotation | ECHOMETRIC and CURVASONA annotations |
| `Segment` | Points | Semantic segments (trend-segment annotations that partition the series), else even bins of at least 8 points and at most 12 per series. The bin sizes are provisional (OD-18) |
| `Point` | — | A data row (`RowRef`) the track reads, in the encoder's time order, with its value (or none, for a missing value), provenance and onset. Without a time field, a field with role `category` or `label` names the points (bars) |
| `Node`, `Edge` (networks, Stage D) | Neighbours, incident edges | Graph data (Data Navigator model; requires H-05 before defaults ship) |

**Movement verbs** (keys in [keyboard-and-navigation](../design/keyboard-and-navigation.md)):

| Verb | Meaning | Example |
|---|---|---|
| Local | Parent / first child | Point → its segment; Series → first segment |
| Lateral | Same position in a sibling structure | Temperature @1980 → CO₂ @1980 |
| Spatial | Step along the ordering axis; jump by 10%; first/last | Next point; jump forward a tenth |
| Targeted | Jump to a landmark or a search result | Next insight; maximum; value nearest 1.0 °C |
| Where am I | Report position, level and neighbours | "Point 47 of 146, segment 1980–2025, series temperature" |

**Invariants** (property-tested): every node is reachable; no verb leaves the cursor undefined; boundary moves produce the boundary earcon and leave the cursor in place (Chart Reader "bonk", A11Y-08).

## 3. What a cursor move produces

```mermaid
sequenceDiagram
  participant K as Input (key/switch/gamepad)
  participant UI as UI (focus owner)
  participant H as Host / AUDAXIS
  participant RT as Audio thread
  participant AN as Announcer
  K->>UI: Next point
  UI->>H: nav.move(Spatial::Next)
  H->>H: resolve node, compose announcement
  H-->>UI: {node, highlight, announcement, cue}
  H->>RT: TriggerCue(point tone, value)   %% ≤ 150 ms
  UI->>UI: move visual highlight, update review line
  H->>AN: announce(text, priority, after = cue end)
```

| Output | Content | Controlled by |
|---|---|---|
| **Cue** | A short tone (≤ 150 ms) for the point's value, using the track's instrument and mapping. Boundary and invalid moves play earcons instead | Modality toggle "sonification" |
| **Announcement** | Composed text: value + unit, x position, optional series name, provenance qualifier for estimates, rank or insight tags | Verbosity (terse/verbose), part order (A11Y-12), modality "speech" |
| **Visual highlight** | Crosshair and marker on the plot; row selection in the table | Modality "visual" |
| **Braille review line** | A persistent focused text element with the current value. Braille "sparkline" for series overviews (A11Y-21) | Modality "braille" |

## 4. Announcement pipeline

```mermaid
flowchart LR
  C[Composer<br/>templates, locale,<br/>units, verbosity] --> Q[Announcement queue<br/>priority + supersede]
  Q --> SEQ[Sequencer<br/>after cue, on pause,<br/>rate limits]
  SEQ --> R{Route}
  R -->|default| SR[Screen-reader route]
  R -->|optional| SV[Self-voicing route<br/>tts crate]
  SR --> LR[ARIA live region<br/>managed announcer]
  SR -.evaluate A02/B03.-> NAT[Native notification<br/>UIA / NSAccessibility]
  SV --> DUCK[Duck sonification −20 dB<br/>while speaking]
```

| Concern | Policy |
|---|---|
| **Composition** | AUDAXIS composes each announcement from parts, each a message of the shared catalogue, joined by the localized `announce-join` message in the listener's order. Every announcement has a spoken form (unit names) and a display form (unit symbols) from the same value and precision (DL-P12). The provenance part of a value that is not stated plainly is added even if the listener's order leaves it out (B03-T03) |
| **Supersede** | Rapid navigation replaces queued, stale announcements. Only the latest cursor announcement survives (A11Y-17) |
| **Priority** | `polite` for navigation and status; `assertive` only for errors, device loss and true alarms |
| **Sequencing** | During playback, speech is suppressed except for alarms. On pause, the current value is announced (A11Y-09). During exploration, the announcement follows the cue (A11Y-20) |
| **Rate limits** | Streams: at most one announcement per N seconds per stream (configurable), with coalescing (A11Y-16) |
| **Routing (default)** | A single managed ARIA live region pair (polite and assertive) inside the webview. It uses the clear-then-set technique for reliable re-announcement. WebView2, WKWebView and WebKitGTK expose live regions to UIA, NSAccessibility and AT-SPI respectively |
| **Routing (native, to evaluate)** | Where live regions prove unreliable (queueing, double speech), a small Tauri plugin raises native notifications: `UiaRaiseNotificationEvent` on Windows, `announcementRequested` with priority on macOS. Linux stays on live regions/AT-SPI. Decided in the A02 spike and B03 |
| **Self-voicing** | `sonarithm-speech` (tts crate: SAPI/WinRT, AVSpeechSynthesizer, speech-dispatcher). For users without a screen reader or who prefer it. Because SONARITHM controls the timing here, it ducks the sonification bus while speaking |
| **Exports** | Speech never goes into the audio file. A WebVTT cue sidecar is produced instead ([06 §7](06-audio-engine.md#7-export)) |

**As built (B03-T07).** `sonarithm_app::speech::Sequencer` runs on the transport's control thread, which owns the player, so it can pause or duck playback itself. It takes AUDAXIS's queue as is and adds the timing: nothing but errors and alarms is spoken before the current cue ends or while an arrow key repeats (the queue keeps only the latest navigation meanwhile); during playback, navigation and earcon texts are logged and not spoken, status and stream notices wait for the pause (stream notices still expire after 10 s), and a response pauses playback first, or ducks it with the `duckInsteadOfPause` setting. Anything spoken while sonification plays is ducked by the render graph's ducker (`GainTarget::Duck`, `DUCK_LEVEL` = −20 dB) until the speech is estimated to end: words at the listener's rate (`speech.rateWpm`, 180 words per minute by default, TODO(verify) in B07), extended while a self-voicing engine reports it still speaks. On `transport_pause` the app core composes "Paused. <value>" for the point of the first series at the playhead (A11Y-09); `announce_repeat` (`nav.repeat`, R) speaks the last announcement again. Each delivery reaches the host as an `announcement` event; with `speech.route` set to `selfVoicing` and a speaker offered by the host, SONARITHM speaks it itself and marks it `selfVoiced`, so the host shows it without passing it to the screen reader. The app compiles with AUDAXIS registered (`Builder::with_navigation`), which builds the navigation tree and applies AUDAXIS's shaping (F-142).

## 5. Webview accessibility realities

The desktop UI runs in a system webview, so assistive technology treats it as **web content**. That brings the mature ARIA ecosystem, and also browse-mode behaviour that must be designed for:

| Issue | Design response | Verify in |
|---|---|---|
| NVDA and JAWS default to **browse mode** for web content, and capture arrow keys | Every region is a proper composite widget (`tree`, `grid`, `toolbar`, `tablist`), so screen readers switch to focus mode automatically. The chart exploration surface uses `role="application"` **only** inside the chart, with an accessible name, a description and an always-available help command | A02 (5), B03 |
| VoiceOver interaction model (VO keys, rotor) | Landmarks and headings for regions, rotor-friendly labelling, no reliance on VO-key combinations | B03 |
| Orca on WebKitGTK and Wayland is the least mature | It is part of the release matrix (A11Y-32). Known issues are tracked in the ACR. Electron is the fallback if it blocks (ADR-0002) | A02, B07 |
| Canvas and WebGL plots are invisible to assistive technology | The chart's accessible tree comes from the NavTree (DOM nodes generated on demand for the current neighbourhood), never from canvas pixels | B03, B04 |
| Focus visibility | Focus rings of 2 px or more with ≥ 3:1 contrast; nothing ever obscures focus (WCAG 2.4.11) | B02 |
| Shortcut conflicts | Single-key shortcuts are only active while the chart has focus, and can be remapped or disabled (A11Y-06). Default bindings avoid Insert, CapsLock and Ctrl+Option combinations | B03 |

## 6. Modalities, profiles and settings

- **Modalities** (A11Y-10): `sonification`, `speech`, `braille`, `visual`, `haptics` (future). Each toggles independently and persists in the user profile.
- **Profiles** (SDR-38) bundle pitch range, polarity flips per concept, timbre preset, discrete vs continuous mark preference, pace, verbosity, announcement order, modalities and keymap overrides. Presets ship for "BLV chart reading", "Analyst", "Classroom" and "Presentation" ([sound-design](../design/sound-design.md)). Profiles are portable JSON, schema-validated and importable.
- **Precedence** ([12 §3](12-cross-cutting-concerns.md#3-configuration-and-settings), [ADR-0020](../decisions/0020-profile-format-v1.md)): built-in defaults < user settings < active profile < project profile < spec < explicit UI action. The effective value of any setting is inspectable, and announced on request.

**As built (B03-T08).** `sonarithm-app::profile` holds the profile document (format 1.0; schema `schemas/sonarithm-profile-1.schema.json`), the five presets of sound-design §13 as read-only proposals, and the layer merge. Interaction settings (verbosity, part order, modalities, speech, keymap) are resolved field by field in the app core and drive the announcer and the sequencer. Sound preferences become the effective spec: the app core fills pitch range, instrument, pace and context cues where the spec is silent, and writes every polarity a profile decides (by concept or by the one-key flip) as an explicit value resolved with NUMISONA, so the compile never sees a profile and the render manifest's spec alone reproduces the Score. `settings_effective` lists each value with its layer (`default`, `settings`, `profile`, `project`, `spec`, `session`); `profile_flip_polarity` is the one-key flip (FR-032), saved in the active profile (a copy when a preset or nothing is active) and followed by a recompile; the project profile is `profiles/project-profile.json` in the bundle, saved with `profile_save` and the ID `project`.

## 7. Non-audio equivalents (Deaf, hard-of-hearing and DeafBlind users)

Every sonified insight has a text, visual or braille equivalent (A11Y-24):
- an insights list and data table (always available);
- a visual highlight of the cursor and insights;
- WebVTT cue sidecars on export;
- earcons with text equivalents in the announcement history panel;
- a braille review line.

## 8. Input methods

| Input | Support |
|---|---|
| Keyboard | Full (A11Y-05); F6 region cycling; command palette for every action |
| Pointer | Hover and scrub on plots, with single-pointer alternatives to dragging (A11Y-26); the minimum target size is met |
| Switch access / scanning | The discrete-step model plus the command palette. A "next action" scanning order is defined for the transport and the cursor |
| Gamepad | D02 maps it for RESONUM excitation; navigation mapping is optional from B03 (DIXTRAL precedent) |
| Braille routing keys | Routing on the review line moves the cursor to the corresponding point (where the screen reader supports it) |
| Voice commands | Out of scope until Stage D; OS-level voice control works through standard accessibility |

## 9. Localization of announcements

Templates are localized with **Fluent** (`fluent-rs` in `sonarithm-i18n` on the Rust side; the same catalogue files and message IDs in the frontend, [12 §5](12-cross-cutting-concerns.md#5-internationalization-and-localization)). Numbers, dates and units are formatted with ICU4X per locale. Unit names are spoken in full at verbose and abbreviated at terse ("°C" → "degrees Celsius"). English ships in B03 and a second locale in B07 to prove the pipeline.

## 10. Open questions (tracked in [open-decisions](../planning/open-decisions.md))

| ID | Question | Settled by |
|---|---|---|
| OD-05 | ARIA live regions only, or a native notification plugin, per OS? | A02 spike (5), B03 |
| OD-06 | Default keymap for lateral and local moves | B07 co-design round 1 |
| OD-07 | Is the timbre treatment for low-confidence data helpful or confusing for BLV users? | H-06 / P08 study (D06) |

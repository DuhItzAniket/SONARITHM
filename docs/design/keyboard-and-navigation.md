# Keyboard and navigation

> **Status:** Baseline v1 (2026-10-18). **Owner:** Accessibility lead.
> Related: [interaction and accessibility architecture](../architecture/08-interaction-and-accessibility.md) (NavTree, announcer, platform bridge), [desktop app](../architecture/09-desktop-app.md), [engine catalogue: AUDAXIS](../product/engines.md), [design rules](../research/design-rules.md) (A11Y-xx, SDR-xx), [research notes 03 §2, §4, §7](../research/notes/03-accessibility-hci.md), [sound design](sound-design.md), [design language](design-language.md), [accessibility conformance plan](accessibility-conformance-plan.md), [findings log](../research-paper/findings-log.md).

**This keymap is v0.** It is a proposal built from the published navigation research (Zong et al. 2022, Chart Reader, MAIDR, VoxLens, Data Navigator; summarised in [notes 03](../research/notes/03-accessibility-hci.md)). It has not been tested with SONARITHM users. B03 implements it; B07 validates it in co-design with blind and low-vision participants (A11Y-34). Every change after validation is logged as a finding (`F-###`) and in the change log at the end of this document.

## 1. Principles

| ID | Rule | Basis |
|---|---|---|
| KB-01 | Every action is a named command reachable from a menu and the command palette. Shortcuts are accelerators, never the only path. | A11Y-05; WCAG 2.1.1 |
| KB-02 | Single-character shortcuts (letters, including Shift+letter, and punctuation) exist only inside the chart explorer, and can be disabled as a group or remapped one by one. | A11Y-06; WCAG 2.1.4 |
| KB-03 | No default binding uses a screen-reader modifier: Insert or CapsLock (NVDA, JAWS, Narrator, Orca) or Ctrl+Option (VoiceOver). | A11Y-06; [notes 03 §4](../research/notes/03-accessibility-hci.md) |
| KB-04 | No Ctrl+Alt+letter (AltGr types characters on many layouts), no Ctrl+Alt+arrows (screen-reader table navigation, Windows Magnifier panning, some graphics drivers), no Alt+arrows on Windows and Linux (webview history; on macOS, Option+arrows are Jump, KB-07), no Ctrl+Space (input-method switching). | Section 12 |
| KB-05 | F6 and Shift+F6 cycle the fixed regions in a constant order. Tab never traps focus. | A11Y-05; [design language §4](design-language.md) |
| KB-06 | One modifier at most for frequent chart commands. No command depends on timing (no double-press, no long-press). | Motor access; WCAG 2.2.1 |
| KB-07 | Platform conventions apply: **Primary** = Ctrl on Windows and Linux, Cmd on macOS; **Jump** = Ctrl on Windows and Linux, Option on macOS (the platforms' word-jump modifiers). On macOS, Ctrl+arrows belong to Mission Control and Spaces, so Jump uses Option there. | Section 12 |
| KB-08 | Letters bind by character (an X is X on any layout). Punctuation and digits bind by physical key and display the localised label. Essential commands have a non-punctuation alternative. | Section 13 |
| KB-09 | Every shortcut is discoverable: menus and tooltips show the current binding, Shift+F1 lists all bindings for the focused scope, and help (F1, or H in the explorer) explains them. | A11Y-30; WCAG 3.2.6 |

## 2. Scopes and focus

| Scope | Active when focus is in | Examples |
|---|---|---|
| Global | Anywhere in the main window (a text field keeps its own editing keys) | Palette, open, save, stop all audio, F6 |
| Region | A given region | Tab switching in the Workspace |
| Chart explorer | The explorer element in the Plot tab | Arrows, letters, Space |
| Table, Tree, Inspector, Editor | That widget | APG keys; CodeMirror keys |
| Overlay | An open menu, popover or dialog | Escape closes; arrows move |

Precedence, highest first: overlay → text input or editor → focused widget scope → region → global. An overlay or text field always keeps standard editing keys; for example Primary+Z in a text field undoes typing, not the project.

**As built (B02-T07).** `apps/desktop/src/commands` holds:
- the command registry, with the v0 keymap of §5.1–§5.2 as data;
- one keyboard dispatcher for the window;
- the command palette (React Aria Autocomplete; the result count is announced);
- the Shift+F1 reference.

While a dialog or menu is open, only overlay commands and `transport.stopAll` run, so no command moves focus out from under a modal. A text field keeps every key without Ctrl, Alt or Meta, except function keys. A lint test (`apps/desktop/test/commands.test.ts`) checks every default binding against KB-02, KB-03, KB-04 and the reserved keys of §13 rule 6, on Windows, Linux and macOS. The palette lists a command once its feature registers a handler (KB-01).

**The chart explorer** is a single Tab stop inside the Plot tab. Tab and Shift+Tab leave it. Its keys only work when the screen reader passes keys through (NVDA and JAWS focus or forms mode, Narrator scan mode off, VoiceOver Quick Nav off, Orca focus mode). The explorer therefore exposes a role that makes NVDA and JAWS switch automatically; which role, and how Narrator and Orca behave with it, is decided in [08](../architecture/08-interaction-and-accessibility.md) and verified in the A02 spike and B07 [CHECK]. Help and the tutorial explain how to switch modes manually.

## 3. Regions and F6

| F6 order | Region | Lands on |
|---|---|---|
| 1 | Top bar | Last focused control; first time, the transport Play button |
| 2 | Project (left) | Selected tree item |
| 3 | Workspace (centre) | Chart explorer (Plot tab), focused cell (Table tab) or editor (Spec tab) |
| 4 | Inspector (right) | Last focused control; first time, the first section header |
| 5 | Status (bottom) | Latest announcement in the log |

F6 moves forward, Shift+F6 backward, wrapping. The order is the same at every breakpoint; in the Narrow layout, F6 also brings the target region into view ([design language §4.3](design-language.md)). Collapsed regions stay in the cycle as an "Expand" control. Each region is also a landmark, so screen-reader landmark navigation works in browse mode.

## 4. The NavTree and movement verbs

### 4.1 Levels

AUDAXIS builds the navigation tree from the dataset ([engine catalogue](../product/engines.md); A11Y-04). One shared cursor points at one node. Moving the cursor drives four outputs: a point cue (a short tone of the value), an announcement, the visual highlight, and the braille review line (A11Y-10).

| Level | Node | Children | Notes |
|---|---|---|---|
| 0 | **Dataset** (root) | Series 1…n, Axes, Insights, Filters | Announces the summary (A11Y-01, A11Y-02) |
| 1 | **Series** k | Segments | Segments are x-bins (for example months, or equal bins) or analysis segments. B03 builds even bins of at least 8 points, at most 12 per series, until co-design settles the size (OD-18) |
| 1 | **Axes** | X axis, Y axis | Extra grouping level, as in Chart Reader |
| 1 | **Insights** | Insight types (extrema, outliers, change points, trend segments) | Extra grouping level; insights come from ECHOMETRIC analyzers |
| 1 | **Filters** | One filter per series | Enter toggles a series in or out of playback and view |
| 2 | **Segment** / axis / insight type | Points / bins / insights | — |
| 3 | **Point** (or bin, or insight) | — | Leaf |
| Stage D | **Node** (networks) | Neighbours, Edges | Provisional; see 4.4 |

### 4.2 Verbs to keys

Notation: "Jump" is Ctrl on Windows and Linux and Option on macOS (KB-07).

| Verb | Key | Behaviour |
|---|---|---|
| Spatial: step | Left / Right | Previous / next node at the current level, in x order |
| Spatial: jump | Jump+Left / Jump+Right | Back / forward by 10 % of the nodes at this level (at least one) |
| Spatial: ends | Home / End | First / last node at this level |
| Local: in | Down, or Enter | Into the child that contains the current x position, else the first child. Enter also activates: on an insight it jumps to its data; on a filter it toggles; on a point it speaks the full verbose description |
| Local: out | Up, or Backspace | To the parent, keeping the x position |
| Lateral | PageUp / PageDown | Previous / next series at the same position: the same segment, or the point with the same x (nearest x if absent, and the announcement says "nearest") |
| Targeted: extremes | Jump+Up / Jump+Down | Maximum / minimum point of the current scope (selection if any, else the series), landing at point level |
| Targeted: insights | N / Shift+N | Next / previous insight in the current series, in x order. The cursor lands on the insight's point, with the landmark earcon, so repeated presses stay in the series |
| Targeted: landmarks | X, Y, I, F, D | X axis, Y axis, Insights, Filters, Dataset root |
| Targeted: go to | G | "Go to x" field (number, date or category); moves to the nearest point |
| Targeted: search | Primary+F | "Find value" field; moves to the next point equal to or crossing the value |
| Select | Shift+Left / Shift+Right; Shift+Home / Shift+End; Jump+Shift+Left / Jump+Shift+Right | Extend the selection by one node, to the first or last node, or by 10 %. The selection becomes the scope for playback, extremes and queries |

At a boundary the cursor does not move or wrap: the boundary earcon plays and the boundary text is announced (section 11). A verb that does not apply (for example lateral movement with one series) plays the invalid-action earcon with a reason.

Rationale: x maps to Left/Right because users build a spatial model of the chart (Zong et al. 2022); levels map to Up/Down so that "up" always means "zoom out"; series map to PageUp/PageDown as in MAIDR's layer keys; letter landmarks follow Chart Reader's "escape hatch" jump keys ([notes 03 §2](../research/notes/03-accessibility-hci.md)). This departs from the APG tree pattern, where Up/Down move between visible siblings; the role and its announced semantics are settled in [08](../architecture/08-interaction-and-accessibility.md).

### 4.3 Verb behaviour by level

| Level | Left/Right | Down/Enter | Up | PageUp/PageDown | Home/End | Jump+Left/Right | Jump+Up/Down |
|---|---|---|---|---|---|---|---|
| Dataset | Boundary | First child (Series 1) | Boundary | Invalid | Boundary | Boundary | Invalid |
| Series / Axes / Insights / Filters | Previous/next sibling | Child at current x | Dataset | Previous/next series (from a series) | First/last sibling | One sibling | Max/min of this series |
| Segment, axis, insight type | Previous/next | Point at current x, else first | Branch | Same segment in the adjacent series | First/last | ±10 % | Max/min of the series |
| Point, bin, insight | Previous/next | Speak full description (Enter); boundary (Down) | Segment | Same x in the adjacent series | First/last point of the series | ±10 % of the series | Max/min of the series |

Entering a child keeps the x position, so a user who drills from a series into a segment and then into points stays where they were in time.

At point level, the siblings are all points of the series in x order; segments only group them. Right at the last point of a segment continues into the next segment, and the new segment name is announced as context ("August"). This avoids a boundary every few points. Bins on an axis and insights in a type behave the same way within their branch.

### 4.4 Networks (Stage D, provisional)

No blind-user evaluation of network sonification was found, and graph concepts have no published polarity data ([research report](../research/research-report.md), "Blind users want an overview…"). The model below extrapolates from Data Navigator's node-edge navigation and is not validated:

| Key | Behaviour |
|---|---|
| Left / Right | Previous / next neighbour of the current node (ordered by edge weight, then label) |
| Down / Enter | Traverse to the focused neighbour; it becomes the current node |
| Up / Backspace | Back along the traversal history |
| PageUp / PageDown | Switch between the Neighbours and Edges lists |
| G | Go to node by name |

## 5. Complete v0 keymap

Columns give Windows and Linux bindings, then macOS where it differs. Command IDs are stable and are what users remap.

### 5.1 Global

| Command ID | Action | Windows / Linux | macOS |
|---|---|---|---|
| `app.palette` | Command palette | Ctrl+Shift+P (alternative Ctrl+K) | Cmd+Shift+P (alternative Cmd+K) |
| `file.new` | New project | Ctrl+N | Cmd+N |
| `file.open` | Open project or data file | Ctrl+O | Cmd+O |
| `file.save` | Save project | Ctrl+S | Cmd+S |
| `file.saveAs` | Save project as | Ctrl+Shift+S | Cmd+Shift+S |
| `file.export` | Export audio | Ctrl+E | Cmd+E |
| `edit.undo` / `edit.redo` | Undo / redo | Ctrl+Z / Ctrl+Y or Ctrl+Shift+Z | Cmd+Z / Cmd+Shift+Z |
| `edit.find` | Find in focused region (tree filter, table search, value search in the explorer) | Ctrl+F | Cmd+F |
| `app.preferences` | Preferences | Ctrl+Comma | Cmd+Comma |
| `help.context` | Help for the focused element | F1 | F1 |
| `help.keyboard` | Keyboard reference for the focused scope | Shift+F1 | Shift+F1 |
| `view.zoomIn` / `zoomOut` / `zoomReset` | UI zoom | Ctrl+Plus / Ctrl+Minus / Ctrl+0 | Cmd+Plus / Cmd+Minus / Cmd+0 |
| `view.focusExplorer` | Move focus to the chart explorer | Ctrl+Shift+C | Cmd+Shift+C |
| `view.announcements` | Show and focus the announcement history | Ctrl+Shift+H | Cmd+Shift+H |
| `transport.playPause` | Play or pause from the cursor | Ctrl+Shift+Space | Cmd+Shift+Space [CHECK] |
| `transport.stopAll` | Stop all audio: sonification, earcons, self-voicing | Ctrl+Period | Cmd+Period |

The macOS binding for `transport.playPause` must be checked against system and input-method shortcuts in B03. Cmd+Period is the macOS cancel convention. Pressing Ctrl also silences NVDA and JAWS speech, which suits "stop all audio".

### 5.2 Regions and panels

| Command ID | Action | Windows / Linux | macOS |
|---|---|---|---|
| `view.nextRegion` / `prevRegion` | Next / previous region | F6 / Shift+F6 | F6 / Shift+F6 (Fn+F6 on keyboards with media-key defaults) |
| `view.menuBar` | Focus the menu bar | F10 or Alt | System: Ctrl+F2 (with keyboard navigation enabled) |
| `view.contextMenu` | Context menu for the focused item | Shift+F10 or Menu key | Shift+F10 |
| `view.nextTab` / `prevTab` | Next / previous tab in the focused region (Plot · Table · Spec; Announcements · Diagnostics) | Ctrl+Tab / Ctrl+Shift+Tab | Ctrl+Tab / Ctrl+Shift+Tab |
| `view.toggleRegion` | Collapse or expand the focused side region | Ctrl+Shift+Backslash | Cmd+Shift+Backslash |

Splitters follow the APG window-splitter pattern: arrows resize in 8 px steps, Enter collapses or restores (A11Y-26).

### 5.3 Transport

| Command ID | Action | Scope | Key |
|---|---|---|---|
| `transport.playPause` | Play or pause from the cursor (plays the selection if one exists) | Chart explorer, Table | Space |
| `transport.playScope` | Play the current series (or selection) from its start: the overview (A11Y-09) | Chart explorer | Shift+Space |
| `transport.stop` | Stop | Chart explorer | Escape (see precedence below) |
| `transport.faster` / `slower` | Rate up / down one step | Chart explorer | Shift+Up / Shift+Down |
| `transport.legend` | Play the auditory legend: axes, ranges, units, polarity (SDR-18) | Chart explorer | L |
| `transport.loop` | Loop on or off | Toolbar, menu, palette | (unbound) |
| `audio.levelUp` / `levelDown` | Sonification level ±1 step | Toolbar slider, menu, palette | (unbound; open question for B07) |

Escape precedence in the explorer: close an open overlay → stop playback → clear the selection → nothing. Because some screen-reader configurations may consume Escape to leave focus mode [CHECK in A02], stopping never depends on Escape alone: Ctrl+Period (Cmd+Period) and the toolbar Stop button always work.

Rate steps (proposal): 1, 1.5, 2, 3, 4, 6, 8, 12, 16 points/s. The default is 3 points/s, inside the 2–4 points/s overview guidance (SDR-19). Moving above 8 points/s announces the rate with the guidance ("12 points per second, above the 8 points per second guidance").

### 5.4 Chart explorer

Movement keys are in section 4.2. The remaining explorer commands:

| Command ID | Action | Key |
|---|---|---|
| `nav.whereAmI` | Where am I (section 6) | W |
| `nav.repeat` | Repeat the last announcement | R |
| `nav.compare` | Compare: read every series at the current x | C |
| `query.menu` | Open the query menu (section 7) | Q |
| `map.flipPolarity` | Flip polarity of the current series (undoable; SDR-15) | P |
| `plot.zoomToNode` / `plot.zoomOut` | Zoom the plot to the current node or selection / zoom out one step (keyboard alternative to drag zoom; A11Y-26) | Z / Shift+Z |
| `help.explorer` | Explorer help (same as F1 here) | H |
| — | Leave the explorer (reserved; not remappable) | Tab / Shift+Tab |

Letters used in the explorer: B, C, D, F, G, H, I, L, N, P, Q, R, S, T, V, W, X, Y, Z. Free: A, J, K, M, O, U. Reserved: E (edges, for networks).

### 5.5 Modalities and verbosity

Each output modality toggles independently and shares the one cursor (A11Y-10). Settings persist per user profile. The letters follow MAIDR's B/T/S toggles, which MAIDR's blind participants used ([notes 03 §1](../research/notes/03-accessibility-hci.md)).

| Command ID | Action | Explorer key | Elsewhere |
|---|---|---|---|
| `mod.speech` | Speech and text announcements on/off | T | Top-bar toggle, View menu, palette |
| `mod.verbosity` | Verbosity terse ↔ verbose (A11Y-12) | Shift+T | View menu, palette |
| `mod.sonification` | Sonification (point cues and playback) on/off | S | Top-bar toggle, View menu, palette |
| `mod.braille` | Braille output on/off | B | View menu, palette |
| `mod.brailleMode` | Braille review line ↔ sparkline (A11Y-21) | Shift+B | View menu, palette |
| `mod.visual` | Visual highlight on/off | V | View menu, palette |
| `mod.haptics` | Haptics on/off (where hardware exists) | (unbound) | View menu, palette |

Turning speech off does not silence the screen reader; it stops SONARITHM sending announcements. Every toggle is announced with the mode-change earcon and written to the log.

### 5.6 Table view

The Table tab is a React Aria `Table` following the APG grid pattern (A11Y-03).

| Key | Action |
|---|---|
| Arrows | Move between cells; row and column headers are announced |
| Home / End | First / last cell in the row |
| Primary+Home / Primary+End | First / last cell of the whole table, loading that window |
| PageUp / PageDown | Previous / next page of rows; on the first or last loaded row, the previous or next window of rows (B04-T05) |
| Space | Play or pause from the focused row |
| Enter (on a column header) | Sort |
| Primary+F | Search the table |
| Go to row… (command palette) | Load the window that holds a row number and focus that row, in the same column |

The table holds a window of rows. Its first column, a row header, is each row's number in the whole table, and a line above the grid gives the loaded range ("Rows 1 to 1,000 of 1,000,000"), announced politely when the window changes (09 §3, B04-T05). The focused row is the shared cursor (A11Y-11): moving it moves the plot highlight and the explorer cursor, and plays the point cue if sonification is on. The move is not announced, because the screen reader reads the focused row (ADR-0017 "Cursor", INFER, OD-30). The cursor's row is shown selected, and the Status region's readout and breadcrumb follow it. A cursor moved elsewhere never moves the focus (B04-T06).

### 5.7 Project tree

APG tree pattern via React Aria `Tree`.

| Key | Action |
|---|---|
| Up / Down | Previous / next visible item |
| Right / Left | Expand or enter / collapse or go to parent |
| Home / End | First / last item |
| Type characters | Type-ahead |
| Enter | Open the item in the Workspace (dataset → Plot; spec → Spec) |
| F2 | Rename |
| Delete | Remove from project (undoable; confirmation for data sources) |
| Shift+F10, Menu key | Context menu |
| Primary+F | Filter the tree |

### 5.8 Inspector

| Key | Action |
|---|---|
| Tab / Shift+Tab | Next / previous control |
| Enter or Space on a section header | Expand or collapse the engine section |
| Slider: Left/Down, Right/Up | Decrease / increase by a small step |
| Slider: PageDown / PageUp | Decrease / increase by a large step |
| Slider: Home / End | Minimum / maximum |
| NumberField: Up / Down | Step; typed values accept units ("1.2 kHz") |
| Space or Enter on a toggle | Toggle (for example polarity flip) |

Slider announcements always include value, unit and range ("Pitch range maximum, 1400 hertz, 65 to 1480") following Ableton Live's option to speak slider minimum and maximum ([notes 05 §8](../research/notes/05-desktop-frontend-stack.md)).

### 5.9 Spec editor

The Spec tab uses CodeMirror 6 with its standard editing keys.

| Key | Action |
|---|---|
| Primary+Enter | Apply the spec (validate and compile) |
| F8 / Shift+F8 | Next / previous diagnostic (CodeMirror lint default; verify in B05) |
| Primary+F | Search panel |
| Shift+Alt+F (macOS Shift+Option+F) | Format document |
| Completion | CodeMirror default; on macOS Ctrl+Space switches input source, so an alternative binding is used (verify in B05) |
| Tab | Moves focus out of the editor by default. Tab indentation is an opt-in preference; when it is on, Escape followed by Tab leaves the editor (CodeMirror behaviour; verify in B05) |

### 5.10 Status region

| Key | Action |
|---|---|
| Up / Down, Home / End | Move through log or diagnostics entries |
| Enter on an announcement | Move the cursor to the data node it describes |
| Enter on a diagnostic | Go to its source (spec range in the editor, or the data node) |
| Primary+C | Copy the entry text |

## 6. Where am I

W speaks a full orientation report regardless of the verbosity setting (A11Y-07). It reports position, level and neighbours:

> Weather 2026, Temperature, series 1 of 3; July, segment 7 of 12; 14 July, point 195 of 365: 28.7 degrees Celsius. Previous 27.9, next 28.1. No selection. Stopped. Analysis mode; speech and sonification on.

The visual equivalent is the breadcrumb above the plot and the persistent cursor readout. On a braille display the review line shows the same path in compact form.

## 7. Query commands

Q opens a query menu (a React Aria `Menu`; type-ahead works). Answers are announced, written to the log, and do not move the cursor unless the item says so. The scope is the selection if one exists, otherwise the series containing the cursor; the first menu item shows and changes the scope (A11Y-22).

| Item | Answer format (example) |
|---|---|
| Summary | L1/L2 summary: "Temperature: 365 points, 1 Jan to 31 Dec 2026, range 3.1 to 28.7 °C, mean 15.2 °C." |
| Minimum / Maximum | "Maximum of Temperature: 28.7 °C on 14 Jul." (menu item offers "Go to") |
| Mean | "Mean of Temperature, 365 points: 15.2 °C." |
| Median | "Median of Temperature, 365 points: 15.6 °C." |
| Mode | Reported only for discrete or binned data; otherwise "Not meaningful for continuous values." |
| Standard deviation | "Standard deviation of Temperature: 6.9 °C." |
| Sum, Count | "Sum of Rainfall: 812 mm." / "Count: 365 points (2 missing)." |
| Range | "Range of Temperature: 3.1 to 28.7 °C, span 25.6 °C." and the x range |
| Value at x | Opens the go-to field; answers "On 3 Mar: 9.4 °C" and offers to move there |

Statistics are computed by ECHOMETRIC from the data, never phrased by a language model; L3 trend phrasing appears only from verified facts ([engine catalogue](../product/engines.md); A11Y-02). Natural-language Q&A is a Stage D option (D04).

## 8. Announcement composition

### 8.1 Parts and order

An announcement is composed from parts. Users choose verbosity and reorder parts in Preferences (A11Y-12).

| Part | Example | Notes |
|---|---|---|
| Value | "28.7 °C" | Always with unit |
| X | "14 Jul" | Label, date or category |
| Series | "Temperature" | Omitted when unchanged since the last announcement, because redundant near-identical utterances slowed users (Zong et al. 2022) |
| Position | "195 of 365" | — |
| Level | "segment" | Spoken on level change |
| Tags | "global maximum" | Insights and landmarks at this node |
| Provenance | "estimated" | **Mandatory** when the value is an estimate; can be moved but not removed ([architecture P6](../architecture/01-overview.md)) |
| Selection | "selected" | — |
| Hint | "Press Down to explore points" | Verbose only, first visit to a node type only |

Default order. Terse: value, x, tags, provenance. Verbose: series, x, value, position, tags, provenance, hint. B03 puts the level part first in both, so a level change, or the segment a step enters, is heard before the value. The name of a node that is not a point (a series, an axis, a filter) always leads. In B03 the parts are joined with a full stop and a space (`announce-join`), so speech pauses between them.

Speech text uses spoken unit names from the unit table ("°C" → "degrees Celsius" in verbose, "degrees C" in terse; "dB" → "decibels"); symbols stay in visual text and braille (DL-P12). Numbers use the user's locale.

### 8.2 Templates

Example dataset: Weather 2026, series Temperature (°C), Humidity (%), Pressure (hPa). Spoken forms are shown with symbols for readability.

| Event | Terse | Verbose |
|---|---|---|
| Point | "28.7 °C, 14 Jul. Maximum." | "Temperature, 14 July 2026: 28.7 °C. Point 195 of 365. Global maximum." |
| Point, estimated (Stage C) | "21.4 °C, 12 Mar. Estimated." | "Temperature, 12 March: 21.4 °C, estimated from a chart image, confidence 0.82. Point 71 of 365." |
| Segment, estimated (Stage C) | "March, 31 points, 4.1 to 12.6 °C. Estimated." | "Segment 3 of 12: March 2026, 31 points. Range 4.1 to 12.6 °C. 31 of 31 points estimated from a chart image, median confidence 0.82." |
| Series, estimated (Stage C) | "Temperature, 365 points, 3.1 to 28.7 °C. Estimated." (some points only: "Partly estimated.") | "Series 1 of 3: Temperature, degrees Celsius. 365 points … 365 of 365 points estimated from a chart image, median confidence 0.82. Lowest confidence around 12 March, median 0.41." |
| Segment | "July, 31 points, 24.9 to 28.7 °C." | "Segment 7 of 12: July 2026, 31 points. Range 24.9 to 28.7 °C, mean 26.8 °C." |
| Series | "Temperature, 365 points, 3.1 to 28.7 °C." | "Series 1 of 3: Temperature, degrees Celsius. 365 points, 1 January to 31 December 2026. Range 3.1 to 28.7 °C, mean 15.2 °C. Press Down to explore, Space to play." |
| Lateral move | "Humidity: 61 %, 14 Jul." | "Series 2 of 3, Humidity, 14 July 2026: 61 %." |
| Lateral, nearest x | "Pressure: 1012 hPa, nearest 13 Jul." | "Series 3 of 3, Pressure: no value on 14 July; nearest 13 July, 1012 hectopascals." |
| Boundary | Earcon, then "Last." | Earcon, then "Last point of Temperature. Nothing further to the right." |
| Insight | "Insight: maximum, 28.7 °C, 14 Jul." | "Insight 2 of 6, extrema: global maximum of Temperature, 28.7 °C on 14 July 2026. Press Enter to go to the point." |
| Stream event (Monitoring) | "CPU load above 90 %, 14:02:11." | "Stream CPU load crossed the upper threshold of 90 % at 14:02:11; now 93 %. 3 threshold events in the last minute." |
| Pause during playback | "Paused. 26.3 °C, 2 Aug." | "Paused at 2 August 2026: 26.3 °C. Point 214 of 365." (A11Y-09) |
| Invalid action | Earcon, then "No other series." | Earcon, then "Not available: this dataset has one series." |
| Mode change | "Presentation mode." | "Presentation mode. Pitch is quantized to a scale; values are approximate." (SDR-10) |
| Query answer | "Mean 15.2 °C." | "Mean of Temperature, 365 points: 15.2 °C." |

The estimate wording is always the Provenance part, never a prefix on the value, so a listener can move it but not lose it (§8.1, A11Y-12); an earlier draft of the point example began with "About". A series or segment of estimates says so too (C02-T09): terse "estimated" or "partly estimated", verbose the count, the source and the median confidence, and for a series the stretch where confidence is lowest relative to the series (no absolute cut-off, OD-07).

## 9. Announcement queue and sequencing

### 9.1 Classes and policy

| Class | Examples | Priority | Policy |
|---|---|---|---|
| Navigation | Point, segment, series, lateral moves | High | A new navigation announcement supersedes any pending or speaking navigation announcement. Only the latest position is spoken (A11Y-17). |
| Response | Where am I, query answers, repeat, legend | High | Spoken after the current cue; superseded by the next user command. |
| Status | Mode and modality changes, export finished, errors | Normal | Queued after the current announcement; not superseded by navigation. Errors are never dropped. |
| Stream | Thresholds, anomalies, connect and disconnect | Low | Rate-limited and coalesced: proposal at most one per stream every 5 s, with a summary ("3 threshold events in 10 seconds"); dropped if older than 10 s. Pause, stop and hide are always available (A11Y-16). |
| Earcon text | Boundary, invalid action | Follows its earcon | Spoken in verbose; in terse the earcon alone is played and the text goes to the log (per-earcon override in Preferences). |

Every announcement, whatever its class, is written to the announcement history with a timestamp and a link to its data node. The log is the text equivalent for Deaf and hard-of-hearing users and the review tool for everyone (A11Y-24).

The announcer delivers through the user's screen reader by default (A11Y-19), using the platform mechanism chosen in [08](../architecture/08-interaction-and-accessibility.md): UIA notifications on Windows, `announcementRequested` on macOS, live regions on Linux (A11Y-17). Two platform facts constrain the design ([notes 03 §4](../research/notes/03-accessibility-hci.md)): VoiceOver is reported to sometimes skip queued announcements, and macOS gives no completion callback. Sequencing therefore cannot wait for speech to finish; it uses estimated durations and supersession instead. The exact UIA `NotificationProcessing` semantics were not verified in the research [CHECK].

### 9.2 Speech and tones

1. **Cue, then speech.** On each step the point cue starts first (key press to cue onset within the 50 ms p95 budget, [architecture §7](../architecture/01-overview.md)), and the announcement follows the cue. They are sequenced, not overlapped (A11Y-20; [architecture §6.2](../architecture/01-overview.md)).
2. **Key repeat.** While an arrow key auto-repeats, only cues play; the position is announced when the key is released (proposal).
3. **During playback** the cursor follows the playhead silently. Landmarks crossed during playback play the landmark earcon and are logged, not spoken. On pause, the current value is announced (A11Y-09).
4. **Responses during playback.** A user command that speaks (W, R, Q) pauses playback by default; "duck instead of pause" is a preference. When ducking, sonification sits about 20 dB below speech (A11Y-20).
5. **Self-voicing.** With optional self-voicing, SONARITHM controls both streams and enforces the same order directly.

## 10. Braille

With braille on, the review line (A11Y-21) shows the cursor's value, unit, x and position in compact form, sized to the user's cell count; Shift+B switches to a sparkline of the current segment using MAIDR-style 8-dot encodings ([notes 03 §1](../research/notes/03-accessibility-hci.md)). The line is persistent rather than a flash message, because MAIDR participants valued persistent review. Routing keys are covered in section 14.

**As built (B03-T10, Rust side).** AUDAXIS's `braille` module builds both lines; the app core sends the line with every `cursor` event ([ADR-0017](../decisions/0017-ipc-protocol-v1.md) "Braille"). The review line is the composer's parts in symbol form, in the listener's terse order with the position added, joined by spaces. The cell count is a setting (`settings.braille.cells`), because a web page cannot read it: without one nothing is fitted and the screen reader pans; with one, whole parts are kept from the front and a trailing "…" marks parts left out. No part is cut, so no number is shortened, and the first part is kept whole even on a display shorter than it. The sparkline uses MAIDR's four heights (dots 7–8, 3–6, 2–5 and 1–4), from the segment's smallest to its largest value in four equal bands, a blank cell for a missing value, and a window around the cursor when the segment is longer than the display. Widths count characters, which is exact for computer (8-dot) braille; a contracted table can shorten text further, to be checked on a display in B07 (HP-10). A routing-key move is not announced, as for the table (INFER, OD-30). **As built (B03-T10, desktop).** With braille on, the Plot tab shows the braille line under the chart explorer: a read-only text field named "Braille line" or "Braille sparkline", described by what it holds, which the screen reader shows on the display while the field has the focus. The caret sits on the cursor's cell. The field never takes the focus by itself. A routing key sets the caret through the accessibility API with no key event, so the field watches `selectionchange` and sends that cell as `braille_route`. B (`mod.braille`) and Shift+B (`mod.brailleMode`) in the explorer save the modality and the mode in the settings and say what they did. Preferences › Braille holds the cell count, empty by default.

## 11. Earcon catalogue

Earcons mark structure and events; they are designed to be distinct from data tones (A11Y-08). Acoustic descriptions are proposals; assets are produced in B03 under the rules in [sound design §8](sound-design.md). Every earcon has a text equivalent, written to the log and spoken according to section 9.1 (A11Y-08, A11Y-24).

| ID | Meaning | Acoustic description (proposal) | Text equivalent | Default |
|---|---|---|---|---|
| `E-BOUNDARY` | Edge of the structure in the direction moved | Dull low "bonk": about 60 ms of band-limited noise centred near 250 Hz, fast decay. Noise-based so it cannot be mistaken for a data tone (Chart Reader's boundary earcon is the precedent) | "First", "Last", "Top level", "No lower level" | On |
| `E-INVALID` | Command not available here | Soft "drip": two short pitched blips falling a minor third, about 40 ms each, below data level (Chart Reader's invalid-action earcon is the precedent) | "Not available: <reason>" | On |
| `E-LANDMARK` | Arrived at, or crossed, an insight, landmark or annotation | Bell-like inharmonic ping, about 120 ms, decaying | "Insight: <name>" | On |
| `E-THRESHOLD` | Crossed a user threshold or a y gridline | Wood-block "tock", about 30 ms, mid register | "Crossed <value> <unit>" | On for user thresholds; off for gridlines |
| `E-MODE` | Mode or modality change | Two-note dyad, about 60 ms per note: rising for on or enter, falling for off or exit | "<Modality> on/off", "<Mode> mode" | On |
| `E-STREAM` | Stream event: anomaly, threshold, connect, disconnect | Soft chime whose brightness scales with event class; never urgent | "<Stream>: <event>" | On in Monitoring |
| `E-ALARM` | User-configured true alarm only | Repeating pulsed pattern designed for urgency following Edworthy et al. 1991 (SDR-35) | "Alarm: <condition>" | Off; exists only when the user configures an alarm |
| `E-LEVEL` | Moved into a child level or out to the parent | Very short up or down glide, about 50 ms | None (the level is in the announcement) | Off (optional) |
| `E-GAP` | The cursor is on a missing value (`data.missing: "earcon"`, 03 §5) | Two soft noise taps 60 ms apart, low-passed near 3 kHz, about 70 ms in all (proposal, B03) | "No value" | On when the spec asks for it |

B03 (B03-T06) builds this set in the render engine (`EarconSound`; live with `Command::Earcon`) and lists it in AUDAXIS (`earcons::CATALOGUE`) with each text equivalent from the message catalogue. The frequencies and lengths are proposals until the listening check (HP-11).

**Function cues (C03-T04, [RFC-0007](../rfcs/0007-function-cues-and-audification.md), [ADR-0029](../decisions/0029-function-cues-and-audification.md); A11Y-28).** CURVASONA's `curvasona.cues` track plays these during playback of a function with `analysis.detect: ["critical-points"]`. Extrema, inflections, holes and jumps play `E-LANDMARK`. Cues closer than 150 ms merge into the highest-priority kind (asymptote, undefined, off-screen, zero, landmark), and the cue log names every feature. Sounds and timings are proposals (INFER) until the listening check (HP-21).

| ID | Meaning | Acoustic description (proposal) | Text equivalent | Default |
|---|---|---|---|---|
| `E-ZERO` | A function crosses or touches zero | Dry inharmonic tick, about 40 ms: partials at 700, 1932 and 3780 Hz (a struck bar), fast decay; every partial below 4 kHz | "Zero" | On with `critical-points` |
| `E-ASYMPTOTE` | A vertical asymptote: the glide breaks here | Band-passed noise near 1 kHz swelling in over 60 ms, about 120 ms in all ("whoosh") | "Vertical asymptote" | On with `critical-points` |
| `E-UNDEFINED` | An undefined span starts | Low-passed noise near 200 Hz with a soft 20 ms attack, about 100 ms: a dull hush | "Undefined" | On with `critical-points` |
| `E-OFFSCREEN` | The function leaves the y-window (an explicit pitch `scale.domain`) | Two taps of band-passed noise near 2 kHz, 30 ms apart, about 80 ms in all | "Off screen above" / "below" | On with `critical-points` and a window |

As built, the RFC's upward noise sweep for `E-ASYMPTOTE` is a swell of band-passed noise, because the render engine's noise filter is fixed per earcon; the listening check decides whether a swept filter is worth adding. `E-OFFSCREEN`'s taps are band-passed near 2 kHz rather than high-passed above 1.5 kHz: high-passed white noise put 88 % of its energy above 4 kHz, against ED-06 (test `ed_06_function_cue_earcons_keep_their_energy_below_4_khz`).

Reserved for the C02 experiment: `E-ESTIMATE` (low-confidence region).

## 12. Conflict analysis

### 12.1 Assistive-technology keys never bound by default

| AT | Reserved keys | Notes for the explorer |
|---|---|---|
| NVDA | NVDA key (Insert; CapsLock if enabled) with anything; Ctrl alone stops speech; Shift alone pauses speech; numpad review keys (desktop layout); single-letter quick navigation in browse mode; Ctrl+Alt+arrows (table navigation) | Explorer keys need focus mode. Escape may switch focus mode back to browse mode in some configurations [CHECK]. |
| JAWS | JAWS key (Insert or CapsLock) with anything; virtual-cursor quick keys in browse mode; Ctrl+Alt+arrows (table navigation) | Explorer keys need forms mode (auto forms mode). |
| Narrator | Narrator key (CapsLock or Insert) with anything; scan mode (Narrator+Space) captures arrows and letters; Ctrl+Win+Enter toggles Narrator | Scan mode must be off in the explorer; automatic behaviour [CHECK]. |
| VoiceOver | VO keys (Ctrl+Option, or CapsLock if set) with anything; Quick Nav (Left+Right together) captures arrows; single-key Quick Nav letters when enabled; Cmd+F5 toggles VoiceOver | Quick Nav must be off in the explorer. Option and Option+Shift arrows (Jump) are not VO keys. |
| Orca | Orca key (Insert on desktop layout, CapsLock on laptop layout) with anything; browse-mode structural navigation letters; flat review on the numpad (desktop layout) | Explorer keys need focus mode. |

### 12.2 SONARITHM bindings against screen readers

| Binding (scope) | NVDA | JAWS | Narrator | VoiceOver | Orca | Mitigation |
|---|---|---|---|---|---|---|
| Arrows, Home/End, PageUp/PageDown, letters, Space (explorer) | Passed in focus mode; captured in browse mode | Passed in forms mode; captured by the virtual cursor | Captured in scan mode | Captured when Quick Nav is on | Passed in focus mode | Explorer role triggers automatic focus/forms mode where supported; help explains manual switching; verified in A02 and B07 |
| Shift+letter (explorer) | Shift alone pauses speech; the chord passes in focus mode | Passes in forms mode | As above | Passes | Passes | Counted as single-character shortcuts under KB-02 |
| Jump+arrows (explorer) | Ctrl+arrows: paragraph navigation in browse mode only | Same | Scan mode only | Option+arrows: not VO keys | Browse mode only | Same as above |
| Escape (explorer) | May leave focus mode [CHECK] | Usually passed in forms mode [CHECK] | — | Passed | — | Stop also on Ctrl+Period and toolbar |
| F6, Shift+F6, F1, Shift+F1 | No default conflict known | No default conflict known (JAWS help is Insert+F1) | No default conflict known | No default conflict known; Fn may be needed | No default conflict known | — |
| Ctrl+Shift+Space, Ctrl+Period, Ctrl+Shift+C/H/P | No default conflict known | No default conflict known [CHECK] | No default conflict known | Cmd variants [CHECK] | No default conflict known | Verified in B03 |

"No default conflict known" means no conflict was found in the research or the AT's well-known defaults; it is not a test result. B07 runs the full matrix ([conformance plan](accessibility-conformance-plan.md)).

### 12.3 Operating system, webview and input methods

| Source | Reserved | Consequence |
|---|---|---|
| Windows | Win+anything; Alt+Tab; Alt+F4; Alt+Space (window menu); Ctrl+Esc; Ctrl+Shift+Esc; Magnifier Win+Plus/Minus and Ctrl+Alt+arrows (panning) | None bound |
| macOS | Cmd+Space; Ctrl+Space and Ctrl+Option+Space (input sources); Ctrl+arrows (Mission Control, Spaces); Cmd+Tab; Cmd+H, Cmd+M, Cmd+Q; Cmd+Shift+3/4/5 (screenshots); Cmd+Shift+Q (log out); Option+Cmd+8/=/− (Zoom); Ctrl+F2/F3 (keyboard navigation) | Jump is Option on macOS; none of these bound |
| Linux desktops | Super+anything; Ctrl+Alt+arrows (workspace switching on some desktops); Super+Alt+S (screen reader toggle in GNOME) | None bound |
| WebView2 | Browser accelerator keys are enabled by default: reload (F5, Ctrl+R), print (Ctrl+P), find (Ctrl+F), caret browsing (F7), developer tools (F12, Ctrl+Shift+I), history (Alt+Left/Right), zoom (Ctrl+Plus/Minus) | The app disables browser accelerators and re-implements zoom and find. B01-T03: WebView2's `AreBrowserAcceleratorKeysEnabled` is off, read back by the Windows CI smoke test ([ADR-0025](../decisions/0025-unsafe-register-desktop-host.md)); a page-level guard (`apps/desktop/src/keyboard/accelerators.ts`) stops the default action of the same keys in every webview and leaves the event to app commands ([09 §4](../architecture/09-desktop-app.md#4-security-model)). |
| Input methods | Ctrl+Space and Win+Space (IME switching); Alt+Shift or Ctrl+Shift alone (layout switching, if configured); Shift alone (IME mode) | No Ctrl+Space bindings; text fields keep IME behaviour |
| Keyboard layouts | AltGr is Ctrl+Alt on Windows; AZERTY digits need Shift; brackets need AltGr on several layouts | No Ctrl+Alt+letter, no digit or bracket defaults for essential commands (KB-04, KB-08) |

## 13. Remapping and disabling (A11Y-06)

1. **Commands.** Every action has a stable command ID, a name, a description, a scope and default bindings per platform. Commands stay reachable from menus and the palette whatever their binding.
2. **Storage.** Bindings live in the user profile (`keymap` section, owned by `sonarithm-app`). Users can export, import and reset them per command or in full.
3. **Group switch.** Preferences › Keyboard › "Single-key commands in the chart explorer" (on by default; focus scoping already satisfies WCAG 2.1.4). Turning it off unbinds every single-character binding (letters, Shift+letters, punctuation), leaving menus, the palette and any user-assigned chords. Arrows, Home/End, PageUp/PageDown, Space and Escape are not character keys and stay.
4. **Alternative chord set (proposal for B07).** An option to give each explorer letter command a Primary+Shift+letter chord when single keys are off, skipping letters that collide with global commands.
5. **Recorder.** The keymap editor (a React Aria `Table`) records a combination on key-down and asks for confirmation. It refuses reserved combinations (rule 6), warns about conflicts in the same scope and about shadowing a global command, and names the screen reader a combination would collide with.
6. **Reserved, never bindable.** Tab, Shift+Tab, F6, Shift+F6, Escape (as dismiss), any combination with Insert or CapsLock, Ctrl+Option on macOS, Win and Super combinations, Alt+F4, Alt+Space, Cmd+Q.
7. **Display.** Menus, tooltips, the palette and the keyboard reference always show the current binding, never the default.
8. **Layouts.** Letter bindings match the produced character; punctuation and digits match the physical key and show the localised label (KB-08).
9. **Presets.** "Default" and "No single-key commands". More presets only with evidence from B07.
10. **Change control.** A change to a default binding needs a finding in the [findings log](../research-paper/findings-log.md) and a row in this document's change log.

## 14. Discrete-step input beyond the keyboard (A11Y-27)

Navigation is built on discrete commands (`nav.move(verb, direction)`), not on continuous pointer positions, so any input that can issue a step can drive it (Data Navigator; [notes 03 §2](../research/notes/03-accessibility-hci.md)).

| Input | Mapping (proposal) | Status |
|---|---|---|
| Braille display | The screen reader handles panning. With braille on, the review line is exposed as a read-only text field; a routing-key press moves the caret, and the caret position maps to the point under that cell in sparkline mode, or to the value field in review mode. | B03; behaviour per screen reader [CHECK] |
| Switch access | OS switch control (for example macOS Switch Control) issues the same key commands. A proposed in-app "explorer action ring" lets one or two switches cycle through Next, Previous, In, Out, Lateral, Play/Pause and Where am I, and select. | Proposal for B07 co-design |
| Gamepad | Gamepad API: D-pad Left/Right = spatial step, Up/Down = local out/in; shoulder buttons = lateral; triggers = jump ±10 %; A = play/pause; B = stop; X = where am I; Y = repeat; Start = query menu. DIXTRAL used keyboard and gamepad navigation in a small study. | Proposal; Gamepad API availability per webview [CHECK] |
| Voice control | OS voice control (Windows Voice Access, macOS Voice Control) can press keys and activate named controls; every command has a visible name in menus and the palette. | Supported by KB-01 |
| Pointer | Clicking a point moves the cursor there; drag scrubbing and drag zoom have keyboard and single-pointer alternatives (A11Y-26). | B04 |

## 15. Validation

- **A02 spike (5):** NVDA, VoiceOver and Orca walkthrough of React Aria Tree, Slider and Toolbar in a Tauri window ([roadmap](../planning/roadmap.md)); add the explorer role and Escape behaviour as [CHECK] items.
- **B03:** automated tests for scoping, the group switch, remapping, supersession and sequencing; scripted tasks (find max, compare series, read value at x) with NVDA, JAWS, VoiceOver and Orca (B03 exit criteria).
- **B07:** co-design round 1 tests the keymap with 10–15 BLV participants on their own AT (A11Y-34). Measures: task completion, errors, keys pressed that did nothing, requests for remapping, conflicts reported per AT. Each accepted change is a finding (`F-###`) and a change-log row here.
- **B07 [CHECK] items for the explorer:** whether Escape in the explorer makes NVDA leave focus mode (§12); and that Option+arrows (Jump on macOS, KB-07) reach the explorer with VoiceOver on. VoiceOver's own keys are Ctrl+Option combinations (§12), so no collision is expected, but it has not been tried.

## Change log

| Date | Change | By |
|---|---|---|
| 2026-10-18 | v0 keymap, NavTree verb mapping, announcement templates, queue policy and earcon catalogue created. Not yet validated. | Kickoff |
| 2026-11-01 | KB-04's "no Alt+arrows" scoped to Windows and Linux: Alt+arrows are webview history in WebView2 and WebKitGTK only (WKWebView uses Cmd), and on macOS Option+arrows are Jump (KB-07). The keymap lint follows. Option+arrows with VoiceOver added to the B07 [CHECK] items. No default binding changed. Approved by the coordinator (B03-T09). | Lane U |

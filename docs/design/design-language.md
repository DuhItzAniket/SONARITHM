# Design language

> **Status:** Baseline v1 (2026-10-18). **Owner:** Design lead.
> Related: [research notes 05 §8, §10, §11](../research/notes/05-desktop-frontend-stack.md) (source of this document), [design rules](../research/design-rules.md), [engine identity](engine-identity.md), [keyboard and navigation](keyboard-and-navigation.md), [sound design](sound-design.md), [accessibility conformance plan](accessibility-conformance-plan.md), [desktop app architecture](../architecture/09-desktop-app.md), [roadmap B02](../planning/roadmap.md).

This document is the visual design system for the SONARITHM desktop app. Phase B02 implements it in `@sonarithm/ui` and closes with a design review against the anti-pattern list ([roadmap](../planning/roadmap.md), B02 exit criteria). Interaction, keyboard and sound behaviour live in the companion documents linked above. Where this document gives values, they are **proposals** to be validated in B02 unless a rule ID or source says otherwise.

## 1. Positioning: a calibrated instrument

SONARITHM should look and behave like a calibrated instrument: neutral, dense, precise, keyboard-first and fully operable without sight. Every visual flourish must earn its place by encoding data or state ([05 §11](../research/notes/05-desktop-frontend-stack.md)).

The reference points are pro-audio and scientific software: neutral dark-gray canvases, colour reserved for data and state, hairline separators, compact controls, tabular readouts with explicit units, persistent transport and status bars, and docked panels instead of floating cards. That characterisation is an inference in the research, not a sourced survey of those products ([05 §8](../research/notes/05-desktop-frontend-stack.md), Gaps). The sourced anchors are Linear's three-variable theming and chrome/content separation, Carbon's layered gray surfaces, and Ableton Live 12 as proof that a dense music instrument can be screen-reader accessible.

The owner's brief is explicit: the frontend must look professional and must not look like generic AI-generated UI. Section 3 turns that brief into checkable anti-patterns.

## 2. Principles (do)

Each principle has an ID for design reviews, issues and PR descriptions. "Check" says how a reviewer verifies it.

| ID | Principle | Check | Basis |
|---|---|---|---|
| DL-P01 | **Instrument, not marketing page.** The app opens straight into the workspace (data tree, plot canvas, inspector, transport, status bar). No hero, splash screen, onboarding carousel or "welcome" page. | First launch lands in the workspace with focus on a defined control; nothing blocks it. | 05 §11 principle 1 |
| DL-P02 | **Neutral surfaces, one accent, reserved signals.** Three or four near-neutral surfaces, one restrained accent for focus and selection, and signal colours (playhead, warning, error, OK) used only for their meaning. | Every colour in a screen maps to a semantic token in §5.2. No signal token appears decoratively. | 05 §11 principle 2 |
| DL-P03 | **Density with alignment.** 4 px grid, 24–28 px control rows, 1 px low-contrast hairlines, radii 0–4 px. Numbers align on the decimal point. | Spacing and radius values come only from tokens (§7). Numeric columns are decimal- or right-aligned. | 05 §11 principle 3 |
| DL-P04 | **Numbers are first-class.** Tabular lining figures, slashed zero where the font supports it, a monospace face for readouts, units always (Hz, dB, s, samples, BPM, data units), precision suited to the data. | No readout without a unit. No proportional digits in values that change. | 05 §11 principle 4 |
| DL-P05 | **Visible state everywhere.** Focus rings are at least 2 px and at least 3:1 against adjacent colours. Selection, armed/playing state and the data cursor are unambiguous both visually and in speech. | Each state has a visual form and a spoken or text form. | 05 §11 principle 5; A11Y-05 |
| DL-P06 | **Keyboard-first and screen-reader-complete.** Every action is in a menu and the command palette, and frequent ones have a shortcut. F6 cycles regions. Widgets follow the WAI-ARIA APG patterns. Each plot has an explore mode and a table view. Slider announcements include value, unit and range. | Action inventory: every command lists its menu path, palette entry and binding ([keymap](keyboard-and-navigation.md)). | 05 §11 principle 6; A11Y-03, A11Y-05, A11Y-06 |
| DL-P07 | **Themes are generated, not hand-painted.** Dark, light and high-contrast themes come from base, accent and contrast parameters in OKLCH. `forced-colors` and `prefers-contrast` are honoured, and canvases redraw with system colours. | No colour literal in component CSS; components reference semantic tokens only (stylelint rule in B02). | 05 §5, §11 principle 7; A11Y-25 |
| DL-P08 | **Data colour discipline.** CVD-safe categorical palettes, perceptually uniform colormaps for spectrograms, and a redundant encoding (dash, marker, label) for every colour distinction. | No series, category or status is distinguishable by colour alone. | 05 §11 principle 8; WCAG 1.4.1 |
| DL-P09 | **Motion only for meaning.** The playhead, meters and short state transitions (at most 120 ms) move. `prefers-reduced-motion` is honoured. Nothing animates while idle. | Idle screen recording shows no movement. | 05 §11 principle 9 |
| DL-P10 | **Typography with a point of view.** IBM Plex Sans and IBM Plex Mono, 12–13 px UI text, a tight scale (11/12/13/15/18 px), weight contrast rather than size contrast, sentence-case labels. | Only scale tokens are used; no all-caps paragraphs. | 05 §11 principle 10 |
| DL-P11 | **Copy like a lab instrument.** Terse, precise, unit-bearing labels. Error messages state cause and fix. No exclamation marks, no chatty microcopy. | Copy review against §12. | 05 §11 principle 11 |
| DL-P12 | **One value, one formatter.** The visual readout, the announcement, the braille review line and the announcement log show the same value, unit and precision, produced by one shared formatter. | Snapshot tests compare the four outputs for a sample of points. | Derived from A11Y-10 and A11Y-24 |
| DL-P13 | **Label every estimate.** Values that are estimated (for example raster-digitised chart points) are marked visually and in speech, and the mark cannot be switched off. | Estimated points render a distinct marker and the "estimated" announcement part ([keymap §8](keyboard-and-navigation.md)). | [Architecture principle P6](../architecture/01-overview.md) |

## 3. Anti-patterns (don't)

A B02 exit criterion is zero violations of this list. "Detect" names the cheapest reliable check; most can be automated.

| ID | Do not | Detect | Instead | Basis |
|---|---|---|---|---|
| DL-X01 | Purple, indigo or blue gradients, gradient text, or "glow" accents. | Lint for `linear-gradient`, `radial-gradient`, `background-clip: text` and coloured `box-shadow` in chrome CSS. | Flat neutral surfaces; one flat accent. | 05 §11 anti-pattern 1 |
| DL-X02 | Glassmorphism, frosted blur panels, translucent layered cards. | Lint for `backdrop-filter` and translucent surface tokens. | Opaque surfaces separated by hairlines. | anti-pattern 2 |
| DL-X03 | Oversized rounded cards (radius 12 px or more) with soft drop shadows as the main layout unit. | Radius token above `radius-2` (4 px); `box-shadow` outside overlay tokens. | Docked panels and hairlines (§8). | anti-pattern 3 |
| DL-X04 | Centred hero layouts, "three feature boxes" rows, big empty-state illustrations. | Review of every empty state and first-run screen. | Empty states are one line of text plus the command that resolves them (§12). | anti-pattern 4 |
| DL-X05 | Emoji as icons or status indicators; icon-only controls without accessible names and tooltips. | Lint UI strings for emoji code points; axe `button-name`. | Labelled Lucide icons, used sparingly (§10). | anti-pattern 5 |
| DL-X06 | Untouched shadcn/Tailwind defaults: slate/zinc plus indigo palette, default `rounded-lg`/`shadow-md`, default Inter/Geist pairing. | Dependency review (no Tailwind default theme; no shadcn styles). | CSS Modules with SONARITHM tokens ([05 §10](../research/notes/05-desktop-frontend-stack.md)). | anti-pattern 6 |
| DL-X07 | Pure black (#000) backgrounds with pure white text in the default themes (halation); gray-on-gray text below 4.5:1. | Token contrast check in CI (§5.3). | Dark-gray surfaces (§5.2). The high-contrast theme may approach the extremes because the user chose maximum contrast. | anti-pattern 7 |
| DL-X08 | Colour-only encodings, such as red/green status without shape or text. | Review plus CVD simulation of every status and series view. | Icon, text or pattern alongside colour. | anti-pattern 8; WCAG 1.4.1 |
| DL-X09 | Decorative motion: parallax, floating blobs, shimmer skeletons on local data. | Idle and load recordings. | Instant render; a progress bar only for long jobs. | anti-pattern 9 |
| DL-X10 | Proportional digits in changing readouts (they jitter); unitless numbers. | Lint for readout components without the numeric class; copy review. | Plex Mono or `tabular-nums`, unit always. | anti-pattern 10 |
| DL-X11 | Hover-only affordances. | Every `:hover` rule has a `:focus-visible` counterpart; every tooltip trigger is focusable. | Keyboard and AT reach everything hover reaches. | anti-pattern 11 |
| DL-X12 | Free-floating draggable windows as the primary layout. | Layout review. | Fixed, learnable regions (§4). | anti-pattern 12 |
| DL-X13 | Engine identity colours as backgrounds, gradients, data colours or state colours. | Lint for engine colour tokens outside the chip and tile components. | Small chips and section headers only ([engine identity](engine-identity.md)). | [Engine catalogue](../product/engines.md), naming rule 2 |

## 4. Layout

### 4.1 Fixed regions

The workspace uses fixed regions with resizable splitters, not free docking. dockview's spatial keyboard navigation and keyboard docking are Enterprise-only, and free-floating panels are hard for screen-reader users, so fixed regions with F6 cycling are the more accessible choice ([05 §7](../research/notes/05-desktop-frontend-stack.md); [research report](../research/research-report.md), "React Aria is the decisive frontend dependency").

| F6 order | Region | Landmark (proposal) | Contents | Default size at 1440×900 | Limits |
|---|---|---|---|---|---|
| 1 | **Top bar** | `banner` containing labelled toolbars | Menu bar (native where the platform provides one, decided in [09](../architecture/09-desktop-app.md)); command bar (palette trigger); transport toolbar (play/pause, stop, loop, position readout, rate); mode switch (Analysis, Presentation, Monitoring; SDR-36); master volume with dB readout (A11Y-15) | Menu 28 px + toolbar 40 px | Toolbar wraps to two rows below 960 px |
| 2 | **Project** (left) | `navigation`, "Project" | Project and data tree: project, data sources, series, specs, exports | 264 px wide | 200–400 px; collapsible |
| 3 | **Workspace** (centre) | `main` | Tabs **Plot · Table · Spec**. Plot holds the canvas and the chart explorer; Table is the accessible data grid (A11Y-03); Spec is the CodeMirror 6 editor | Remaining width (856 px) | Never collapses |
| 4 | **Inspector** (right) | `complementary`, "Inspector" | Settings grouped by engine, under functional names with the engine as secondary label ([engine identity](engine-identity.md)) | 320 px wide | 280–480 px; collapsible |
| 5 | **Status** (bottom) | `contentinfo` with a `log` | Panel with tabs **Announcements · Diagnostics** (the announcement history is the text record of everything spoken or sounded; A11Y-24), and a 24 px status bar (mode, active modalities, audio device and buffer, lint count, save state) | Panel 144 px + status bar 24 px | Panel collapsible to the status bar |

Rules:

1. The F6 order is the same in every layout and theme. Shift+F6 reverses it. F6 lands on the last focused element in the region, or on its first control the first time ([keymap §3](keyboard-and-navigation.md)).
2. Regions never overlap content at the wide breakpoint. Splitters follow the APG window-splitter pattern (focusable, arrow keys resize, Enter collapses) and have a keyboard alternative to dragging (A11Y-26).
3. Collapsing a region keeps it in the F6 cycle as a single labelled "Expand" control.
4. Popovers, menus and dialogs are the only layers above the regions. They are anchored, never free-floating, and return focus on close.

### 4.2 Main workspace at 1440×900

The wireframe is for sighted readers. The region table in §4.1 is the normative description.

```text
+--------------------------------------------------------------------------------------------------+
| File  Edit  View  Navigate  Transport  Tools  Help                              menu bar 28 px   |
+--------------------------------------------------------------------------------------------------+
| Search commands  Ctrl+Shift+P | > || []  00:12.40 / 01:31.00 | Analysis v | Level -12.0 dB       |
+----------------------+--------------------------------------------------+------------------------+
| Project              | [Plot]  Table  Spec                              | Inspector              |
| v Weather 2026       | Temperature, 2026 (°C)                           | [tile] Mapping         |
|   v temps.csv        |  30 |                   |   .--.                 |   NUMISONA             |
|       Temperature    |     |              .-.  | ./    \                |   Channel     Pitch    |
|       Humidity       |  20 |          .--/   \_|/       \               |   Range  400-1400 Hz   |
|       Pressure       |     |       ./          |         \._            |   Scale  log (st)      |
|   > Specs            |  10 |   .--/            |            \           |   Polarity  + [Flip]   |
|   > Exports          |     | _/                |             \_         | [tile] Axes & nav.     |
|                      |   0 +-----+-----+-----+-|---+-----+-----+--      |   AUDAXIS              |
|                      |        Feb   Apr   Jun  ^    Aug   Oct   Dec     |   Ticks          Off   |
|                      |                      playhead                    |   Reference  Dynamic   |
|                      | Cursor  14 Jul 2026  28.7 °C  195 of 365         | [tile] Transport       |
|                      |                                                  |   SONOTRACE            |
|                      |                                                  |   Rate     3.0 pt/s    |
|                      |                                                  |                        |
| 264 px (200-400)     | 856 px (fills remaining width)                   | 320 px (280-480)       |
+----------------------+--------------------------------------------------+------------------------+
| [Announcements]  Diagnostics                                                    panel 144 px     |
| 14:02:11  28.7 °C, 14 Jul. Global maximum.                                                       |
| 14:02:09  Temperature. 365 points, 3.1 to 28.7 °C.                                               |
| 14:01:58  SDR-19: 12 points/s exceeds the 8 points/s guidance.                                   |
+--------------------------------------------------------------------------------------------------+
| Analysis | Speech: screen reader | Sound on | Braille off | 48 kHz, 256 frames | 1 warning       |
+--------------------------------------------------------------------------------------------------+
```

Notes on the wireframe:

- On macOS the menu bar is the system menu bar, so the in-window row is absent.
- The cursor readout under the plot is persistent and uses the shared formatter (DL-P12). It is not a tooltip.
- `[tile]` marks an engine tile in an inspector section header. The functional name ("Mapping") is the heading; the engine name is a secondary label.

### 4.3 Narrow-window variant

Three breakpoints, measured in CSS pixels after UI zoom (proposal):

| Breakpoint | Width | Behaviour |
|---|---|---|
| Wide | ≥ 1280 px | All five regions visible (§4.2). |
| Medium | 960–1279 px | Project and Inspector collapse to labelled rails; expanding one overlays the Workspace edge as an anchored side sheet, still a landmark, still in the F6 cycle. |
| Narrow | < 960 px | One of Project, Workspace or Inspector is visible at a time, chosen with a region switcher (tabs). The top bar wraps to two rows. The status panel collapses to its last announcement. This is the layout at 200–400 % zoom on common displays. |

At every breakpoint the F6 order from §4.1 is unchanged. In Narrow, moving to a region with F6 makes that region the visible one, so keyboard and screen-reader users do not need to learn a second model.

Narrow variant at 800×600 CSS px (for example 1600×1200 at 200 % zoom):

```text
+----------------------------------------------------------------------+
| Menu  |  Search  Ctrl+Shift+P  | > || []  00:12.40 | Level -12 dB    |
| Mode Analysis v | Rate 3.0 pt/s                                      |
+----------------------------------------------------------------------+
| [Project] [Plot] [Inspector]   region switcher (tabs)                |
+----------------------------------------------------------------------+
| [Plot]  Table  Spec                                                  |
| Temperature, 2026 (°C)                                               |
|  30 |              |  .--.                                           |
|  20 |        .-.   |./    \                                          |
|  10 |   .--./   \__|      \._                                        |
|   0 +-----+-----+--|--+-----+-----+--                                |
|        Mar   May  ^    Sep   Nov                                     |
| Cursor  14 Jul 2026  28.7 °C  195 of 365                             |
|                                                                      |
+----------------------------------------------------------------------+
| 14:02:11  28.7 °C, 14 Jul. Global maximum.     [Log] (collapsed)     |
+----------------------------------------------------------------------+
| Analysis | Sound on | 1 warning                                      |
+----------------------------------------------------------------------+
```

## 5. Colour

### 5.1 Generation model

Themes are generated from three inputs, following Linear's approach: a **base** (neutral hue and a very low chroma), an **accent** (hue and chroma) and a **contrast** level. The contrast input is what makes a high-contrast theme automatic rather than hand-maintained ([05 §5, §8](../research/notes/05-desktop-frontend-stack.md)). The generator works in OKLCH because equal lightness looks equally bright across hues.

Token architecture (05 §5):

1. **Primitives:** OKLCH ramps generated from base, accent and contrast, gamut-mapped to sRGB.
2. **Semantic tokens:** the table in §5.2. Themes swap only this layer.
3. **Component tokens:** aliases of semantic tokens inside `@sonarithm/ui`. Components never reference primitives.

The base hue is a warm neutral with chroma at or below 0.006, so surfaces read as gray. Linear moved from cool blue-grays toward warmer gray and warns that too warm looks muddy; the low chroma keeps the warmth below that point.

The accent is a teal-cyan (hue ≈ 195). It is deliberately not indigo or purple (DL-X01), and it stays clear of the reserved signal hues. The accent marks focus and selection only; it is never a series colour.

The generator lives in `packages/ui/tokens/` (B02-T01). `recipe.ts` holds the inputs (base hue 85 at chroma 0.004, accent hue 195 at chroma 0.095) and one lightness ladder per scheme and contrast level. The contrast level is what turns dark and light into their high-contrast themes: surfaces move toward the extreme, text and borders toward the other end, neutrals lose their chroma and the accent gains chroma. `pnpm -C packages/ui tokens` writes `src/generated/tokens.css` and `src/generated/tokens.json`; the package tests fail when the committed files differ from the generator's output, or when any pair in §5.3 is below its threshold.

### 5.2 Semantic tokens

**Generated values (B02-T01).** These are the generator's output, and [`tokens.json`](../../packages/ui/src/generated/tokens.json) is the record. They stay proposals until the B02 design review (B02-T09); the §5.3 pairs are the gate. Hex values are the sRGB colours a browser renders for the emitted OKLCH values. † marks a colour that was outside sRGB, which the generator gamut-maps with the CSS Color 4 algorithm (lightness and hue kept, chroma reduced) before emitting it.

| Token | Role | Dark (default) | Light | High contrast, dark | High contrast, light |
|---|---|---|---|---|---|
| `surface-0` | App background; plot canvas | `oklch(0.185 0.004 85)` ≈ #131311 | `oklch(0.97 0.004 85)` ≈ #F6F5F2 | `oklch(0.14 0 0)` ≈ #090909 | `oklch(1 0 0)` ≈ #FFFFFF |
| `surface-1` | Panels: tree, inspector, bars | `oklch(0.215 0.004 85)` ≈ #1A1917 | `oklch(0.99 0.004 85)` ≈ #FDFCF9 | `oklch(0.14 0 0)` ≈ #090909 | `oklch(1 0 0)` ≈ #FFFFFF |
| `surface-2` | Inputs, raised controls, table header, overlays | `oklch(0.255 0.004 85)` ≈ #242321 | `oklch(0.95 0.004 85)` ≈ #F0EEEB | `oklch(0.2 0 0)` ≈ #161616 | `oklch(0.96 0 0)` ≈ #F2F2F2 |
| `surface-3` | Hover, selected row (with an accent indicator) | `oklch(0.3 0.004 85)` ≈ #2F2E2C | `oklch(0.915 0.004 85)` ≈ #E4E3E0 | `oklch(0.28 0 0)` ≈ #292929 | `oklch(0.91 0 0)` ≈ #E1E1E1 |
| `text-1` | Primary text, values | `oklch(0.93 0.004 85)` ≈ #E9E8E5 | `oklch(0.23 0.004 85)` ≈ #1E1D1B | `oklch(0.99 0 0)` ≈ #FCFCFC | `oklch(0.14 0 0)` ≈ #090909 |
| `text-2` | Secondary text, labels | `oklch(0.79 0.004 85)` ≈ #BCBAB8 | `oklch(0.4 0.004 85)` ≈ #494745 | `oklch(0.95 0 0)` ≈ #EEEEEE | `oklch(0.22 0 0)` ≈ #1B1B1B |
| `text-3` | Tertiary text: units, hints, tick labels | `oklch(0.69 0.004 85)` ≈ #9C9B98 | `oklch(0.49 0.004 85)` ≈ #61605E | `oklch(0.9 0 0)` ≈ #DEDEDE | `oklch(0.3 0 0)` ≈ #2E2E2E |
| `border` | Hairline separators (decorative) | `oklch(0.33 0.004 85)` ≈ #363533 | `oklch(0.88 0.004 85)` ≈ #D9D7D4 | `oklch(0.82 0 0)` ≈ #C4C4C4 | `oklch(0.4 0 0)` ≈ #484848 |
| `border-strong` | Component boundaries needed to identify a control | `oklch(0.57 0.004 85)` ≈ #787775 | `oklch(0.58 0.004 85)` ≈ #7B7A78 | `oklch(0.95 0 0)` ≈ #EEEEEE | `oklch(0.22 0 0)` ≈ #1B1B1B |
| `accent` | Selection, active tab, checked state | `oklch(0.75 0.095 195)` ≈ #5BC1C0 | `oklch(0.5001 0.0853 195)` ≈ #007272 † | `oklch(0.87 0.1187 195)` ≈ #63EDEC | `oklch(0.4289 0.0731 195.71)` ≈ #005C5C † |
| `focus` | Focus ring (2 px, 2 px offset) | `oklch(0.83 0.1092 195)` ≈ #63DEDD | `oklch(0.4563 0.0778 195.44)` ≈ #006465 † | `oklch(0.94 0.15 105)` ≈ #FAF170 | `oklch(0.42 0.19 262)` ≈ #033FB2 |
| `signal-playhead` | Playhead line and handle | `oklch(0.96 0.02 95)` ≈ #F6F2E3 | `oklch(0.26 0.01 85)` ≈ #26241F | `oklch(0.99 0 0)` ≈ #FCFCFC (3 px) | `oklch(0.14 0 0)` ≈ #090909 (3 px) |
| `signal-warning` | Warnings, over-range, lint warnings | `oklch(0.8 0.14 70)` ≈ #F7AC4D | `oklch(0.57 0.115 70)` ≈ #A26A18 | `oklch(0.8524 0.1361 78.18)` ≈ #FFC35D † | `oklch(0.5011 0.1116 65.96)` ≈ #8E5400 † |
| `signal-error` | Errors, clipping | `oklch(0.7 0.17 25)` ≈ #F66D67 | `oklch(0.53 0.18 25)` ≈ #BE2E31 | `oklch(0.7966 0.1166 25.08)` ≈ #FF9E96 † | `oklch(0.48 0.18 25)` ≈ #AC1922 |
| `signal-ok` | Success, connected, valid | `oklch(0.77 0.14 150)` ≈ #6CCD83 | `oklch(0.53 0.125 150)` ≈ #277F43 | `oklch(0.86 0.16 150)` ≈ #7BEE98 | `oklch(0.461 0.1285 149.48)` ≈ #006B2D † |

Besides the colours, each theme sets `color-scheme` (so scrollbars and native controls match) and the overlay shadow (§8).

Design notes:

- **Changes from the baseline proposals.**
  - All neutrals share the base chroma 0.004. The baseline varied it from 0.002 to 0.006, which moved no colour by more than one step in any channel.
  - The light themes use the accent hue 195 instead of 200.
  - The light high-contrast column is new.
  - In the high-contrast light theme, focus is a deep blue (hue 262). That hue is not used by the accent or the signals, just as yellow is not in the dark high-contrast theme.
- **Surfaces.** The dark surfaces sit near the Carbon (#161616) and Material (#121212) dark grays rather than pure black ([05 §8](../research/notes/05-desktop-frontend-stack.md)).
- **Playhead.** The playhead is a near-neutral, maximum-luminance line with a handle, not a hue. Every hue is already taken by the categorical data palette (§6), so hue would collide with some series; position, shape and luminance identify the playhead instead. This is a proposal to confirm in B04.
- **Focus versus accent.** The focus ring and the accent are close in colour (about 1.2–1.3:1 against each other). That is acceptable only because the ring always has a 2 px offset in the surface colour, so the ring is judged against the surface. Never indicate focus by an accent fill alone.
- **Signals in plots.** Warning and error hues are close to the orange and vermillion of the data palette. Signal colours therefore appear in plots only together with an icon or text label (for example a clip marker with "Clip"), never as bare colour.

### 5.3 Contrast pairs

The generator emits a pair list with the WCAG 2.2 ratio of the rendered colours, and CI fails on any pair below its threshold (WCAG 2.2 ratios as pass/fail; APCA only as a supplementary design check for thin text on dark surfaces, since APCA is not in the WCAG 3 draft; [05 §8](../research/notes/05-desktop-frontend-stack.md)). The tests also check that a high-contrast theme never lowers a pair's ratio below its standard theme's.

| Foreground | Backgrounds | Threshold | Lowest ratio: dark · light · high contrast dark · high contrast light (B02-T01) |
|---|---|---|---|
| `text-1`, `text-2` | `surface-0..3` | 4.5:1 | 7.01 · 7.21 · 12.54 · 13.17 (`text-2` on `surface-3`) |
| `text-3` | `surface-0..2` (not used on `surface-3`) | 4.5:1 | 5.65 · 5.42 · 13.45 · 12.13 (on `surface-2`); dark `text-3` on `surface-3` would be 4.9:1, so it stays off it for margin |
| `border-strong` | `surface-0..3` | 3:1 | 3.03 · 3.34 · 12.54 · 13.17 (on `surface-3`); dark is at the limit |
| `focus` | `surface-0..3` | 3:1 | 8.41 · 5.43 · 12.39 · 6.81 |
| `accent` (as indicator) | `surface-0..3` | 3:1 | 6.35 · 4.48 · 10.29 · 5.98 |
| `signal-*` | `surface-0..3` | 3:1 (4.5:1 when used as text) | 4.72 · 3.55 · 7.33 · 4.70 (dark `signal-error`, light and high contrast light `signal-warning`, on `surface-3`). Light `signal-warning` is below 4.5:1 on every surface and light `signal-ok` on `surface-2..3`, so warning and success text uses `text-1` with an icon |
| `series-1..8` | plot background | 3:1 | See §6; the published palette does not pass on light surfaces (B04) |

The validation is the generator output ([F-131](../research-paper/findings-log.md)). Any change to a threshold or token role is a finding in the [findings log](../research-paper/findings-log.md).

### 5.4 Theme selection, forced colors and `prefers-contrast`

The tokens are CSS custom properties on the root element, in the `tokens` cascade layer. The app sets `data-theme` on the root element from the `theme` setting (`dark`, `light`, `highContrastDark`, `highContrastLight`). With `system`, or with no attribute, the OS decides:

| `data-theme` | No OS preference | OS prefers light | OS prefers more contrast | Both |
|---|---|---|---|---|
| `system`, absent or unknown | dark | light | high contrast dark | high contrast light |
| `dark` | dark | dark | high contrast dark | high contrast dark |
| `light` | light | light | high contrast light | high contrast light |
| `highContrastDark` / `highContrastLight` | that theme | that theme | that theme | that theme |

The rules apply in order, so a webview without a media feature keeps the theme chosen so far (dark by default). Forced colours and reduced motion sit in a later sublayer (`tokens.system`) and so override every theme.

- `forced-colors: active` (Windows High Contrast, via WebView2) maps tokens to system colours: surfaces → `Canvas`; text → `CanvasText`; disabled text → `GrayText`; borders → `CanvasText` or `ButtonBorder`; accent and selection → `Highlight` / `HighlightText`; focus → `Highlight` with the same 2 px geometry; buttons → `ButtonFace` / `ButtonText`. The generated tokens map surfaces to `Canvas`, text and both borders to `CanvasText`, accent, focus and the playhead to `Highlight`, the other signals to `CanvasText`, and the overlay shadow to `none`; `GrayText`, `HighlightText` and the button colours are for components (B02-T04).
- Signals lose their hue under forced colours. That is safe only because every signal already carries an icon or text (DL-X08).
- Canvas and WebGL content is not adjusted by the browser. Plots read the system colours with `getComputedStyle` and redraw: data in `CanvasText`, playhead and cursor in `Highlight`, series distinguished by dash and marker ([05 §5](../research/notes/05-desktop-frontend-stack.md)). `forced-color-adjust: none` is used only on the canvas element that does this.
- `prefers-contrast: more` (macOS Increase Contrast) selects the high-contrast theme generated from the user's base theme.
- Open item: no authoritative source was found on `forced-colors` support inside WKWebView or WebKitGTK ([05 §5](../research/notes/05-desktop-frontend-stack.md), Gaps). B02 tests both and records findings.

### 5.5 Engine identity colours

Engine colours are identity, not semantics. They appear only in small chips and section-header tiles, never as backgrounds, gradients, data colours or state colours (DL-X13). Values, tile construction and contrast notes are in [engine identity](engine-identity.md).

## 6. Data colour

| Use | Palette | Rules |
|---|---|---|
| Categorical series (`series-1..8`) | Starts from the Okabe–Ito palette (Okabe & Ito, *Color Universal Design*, 2008), the CVD-safe palette family named in [05 §8](../research/notes/05-desktop-frontend-stack.md): orange #E69F00, sky blue #56B4E9, bluish green #009E73, yellow #F0E442, blue #0072B2, vermillion #D55E00, reddish purple #CC79A7, black #000000 | Black is replaced by a neutral (`text-2`) in dark themes. Computed against the proposed plot backgrounds, the published values fall below 3:1 on light surfaces for orange, sky blue, yellow and reddish purple, and black fails on dark. B04 derives per-theme lightness-adjusted variants in OKLCH that keep hue order, then re-runs CVD simulation and contrast checks. |
| Redundant encoding | Dash pattern, marker shape, direct label | Every series gets all three (WCAG 1.4.1). Legends are also announced (SDR-18). More than 8 series switch to small multiples or grouping. |
| Spectrograms and density | viridis, magma, cividis (perceptually uniform; CVD-friendly) | Implemented as LUT textures in the WebGL2 layer. Every colormap view has a legend with units and a value readout at the cursor, plus a table or summary alternative ([05 §6](../research/notes/05-desktop-frontend-stack.md)). |
| Selection in plots | Accent as a band or outline | Data marks are never recoloured to show selection. |
| Estimates | Hollow marker plus "estimated" in the readout | DL-P13. |

A diverging map is not chosen yet; B04 selects one with the same CVD and perceptual-uniformity criteria.

## 7. Typography, spacing and shape

### 7.1 Typography

| Item | Specification | Notes |
|---|---|---|
| Families | IBM Plex Sans (UI), IBM Plex Mono (readouts, code, tables of numbers), both SIL OFL 1.1, bundled as WOFF2 | Bundling avoids per-platform font differences ([05 §2](../research/notes/05-desktop-frontend-stack.md)). The OFL texts ship with the app. Fallback alternative recorded in the research: Source Sans 3 + JetBrains Mono. |
| Numerals | `font-variant-numeric: tabular-nums lining-nums slashed-zero` on all numeric text | Plex's tabular-figure defaults and slashed-zero support were not verified in the research ([05 §8](../research/notes/05-desktop-frontend-stack.md), Gaps). B02 verifies them per weight. Readouts use Plex Mono, whose digits are fixed-width regardless. |
| Scale | `type-11` 11/16 px (tick labels, status bar) · `type-12` 12/16 px (dense tables, secondary UI) · `type-13` 13/20 px (default UI text and controls) · `type-15` 15/20 px (panel and dialog titles) · `type-18` 18/24 px (rare: about dialog, empty-state line) | Sizes are expressed in `rem` against the UI scale so OS text scaling (where the webview exposes it) and in-app zoom both apply (A11Y-25). |
| Weights | 400 regular (body), 500 medium (labels, emphasis, headers), 600 semibold (panel titles, sparingly) | No light or thin weights; they break up on dark surfaces. Hierarchy comes from weight and colour (`text-1/2/3`), not size jumps. |
| Case | Sentence case for every label, title and menu item | Engine names are the exception: capitals in headings and prose ([engine catalogue](../product/engines.md)). |
| Units | Unit follows the value with a space (`28.7 °C`, `-12.0 dB`) in `text-3` | Visual minus uses U+2212 in readouts; spoken text is generated separately ([keymap §8](keyboard-and-navigation.md)). |

### 7.2 Spacing and sizing

| Token | Value | Use |
|---|---|---|
| `space-1` … `space-6` | 4, 8, 12, 16, 24, 32 px (0.25–2 rem) | All padding, gaps and margins are multiples of 4 px. Sizes that hold text (spacing, rows, bars, targets) are in rem so text scaling and zoom apply (A11Y-25); radii, hairlines and focus rings are in px. |
| `row-compact` | 24 px | Tree rows, table rows, status bar. |
| `row-default` | 28 px | Buttons, inputs, inspector rows. |
| `bar-toolbar` | 40 px | Top toolbar. |
| Target size | At least 24×24 CSS px hit area | WCAG 2.5.8 (A11Y-26). A 16 px icon in a compact row still gets a 24 px hit area. |
| Readout precision | From data and units, never fixed 2 decimals | DL-P04. |

### 7.3 Radii

| Token | Value | Use |
|---|---|---|
| `radius-0` | 0 | Regions, panels, plot area, tables |
| `radius-1` | 2 px | Buttons, inputs, chips, toggles |
| `radius-2` | 4 px | Menus, popovers, tooltips, dialogs |

Nothing in the UI exceeds 4 px (DL-X03). Engine tiles are a brand mark, not UI chrome, and follow their own geometry in [engine identity](engine-identity.md).

## 8. Elevation and separators

- Regions and panels are separated by 1 px `border` hairlines, not shadows. Separators are used where they group; Linear found that dividing lines "quietly proliferated" and softened them ([05 §8](../research/notes/05-desktop-frontend-stack.md)), so a separator that only decorates is removed.
- Overlays (menus, popovers, tooltips, dialogs) are the only elevated layer: `surface-2`, a 1 px `border-strong` outline and one shadow token, `shadow-overlay` (proposal: `0 2px 8px` in black at 40 % opacity in dark, 15 % in light, none in high contrast and forced colours). No other element has a shadow; Stylelint allows `box-shadow` only as `none` or `var(--shadow-overlay)`.
- Modal dialogs use an opaque scrim. No blur (DL-X02).

## 9. Motion

| Rule | Specification |
|---|---|
| Allowed motion | Playhead, meters, and state transitions (open/close, expand/collapse, selection change). |
| Duration | `motion-fast` 80 ms, `motion-base` 120 ms; 120 ms is the maximum (DL-P09). |
| Idle | Nothing moves while the app is idle. No shimmer skeletons for local data (DL-X09). |
| Reduced motion | `prefers-reduced-motion: reduce` sets transitions to 0 ms and replaces continuous plot auto-scroll with page jumps. The playhead still moves during playback because it is content the user started. |
| Flashing | No element flashes more than three times per second (WCAG 2.3.1). Clip and over-range indicators latch until cleared instead of blinking. |
| Performance | The playhead lives on its own layer, extrapolated from audio-clock anchors in `requestAnimationFrame`, so it never re-renders the data layer ([05 §6](../research/notes/05-desktop-frontend-stack.md)). |

## 10. Iconography

| Rule | Specification |
|---|---|
| Set | Lucide (ISC; Feather-derived icons also MIT). Notices ship with the app ([05 §8](../research/notes/05-desktop-frontend-stack.md)). |
| Size and stroke | 16 px in toolbars and rows (stroke proposal 1.5 px), 20 px rarely. Icons inherit `currentColor` (`text-2` at rest, `text-1` when active). |
| Labelling | An icon is decorative (`aria-hidden`) when visible text accompanies it. Icon-only buttons are allowed only where the convention is strong (transport, panel collapse, splitter) and always have an accessible name and a tooltip that shows the name and shortcut. |
| Restraint | Icons mark actions and object types, not decoration. No icon in body text; no icon rows as ornaments. |
| Never | Emoji as icons or status (DL-X05); coloured icons outside engine tiles and status-with-text. |
| Transport glyphs | Lucide `play`, `pause`, `square` (stop), `repeat` (loop), with names "Play", "Pause", "Stop", "Loop". |

## 11. Component inventory

React Aria Components (RAC) is the single primitive layer; mixing Radix, Base UI and RAC is avoided because focus management and portal conventions conflict ([05 §4](../research/notes/05-desktop-frontend-stack.md)). The B02 deliverable list ([roadmap](../planning/roadmap.md)) is marked **B02**.

| SONARITHM component | RAC primitive | APG pattern | Notes |
|---|---|---|---|
| Button, IconButton **B02** | `Button` | Button | IconButton requires `aria-label` and a tooltip. |
| ToggleButton **B02** | `ToggleButton`, `ToggleButtonGroup` | Button (pressed) | Modality toggles, loop, polarity flip. |
| Toolbar **B02** | `Toolbar` | Toolbar | Transport toolbar; arrows move within, Tab leaves. |
| Menu, menu bar fallback **B02** | `Menu`, `MenuTrigger`, `SubmenuTrigger` | Menu, Menubar | In-window menu bar only where no native menu is used ([09](../architecture/09-desktop-app.md)). |
| Tabs **B02** | `Tabs`, `TabList`, `TabPanel` | Tabs | Plot · Table · Spec; Announcements · Diagnostics; region switcher in Narrow. |
| Tree **B02** | `Tree` | Tree view | Project tree. |
| Table **B02** | `Table`, `ResizableTableContainer`, `Virtualizer` | Grid | Data table (A11Y-03), keymap editor, diagnostics. Row and column headers exposed. |
| Slider **B02** | `Slider` | Slider | Announces value, unit and range; PageUp/PageDown large step; Home/End min/max. |
| NumberField | `NumberField` | Spinbutton | Accepts units in input ("1.2 kHz"); formats with the shared formatter. |
| TextField, SearchField | `TextField`, `SearchField` | — | — |
| Select, ComboBox **B02** | `Select`, `ComboBox` | Listbox, Combobox | Concept selector, scale selector. |
| ListBox, GridList | `ListBox`, `GridList` | Listbox, Grid | Announcement-part ordering uses `GridList` with keyboard-accessible drag and drop and explicit "Move up/down" actions. |
| Checkbox, Switch, RadioGroup | `Checkbox`, `Switch`, `RadioGroup` | Checkbox, Switch, Radio group | Mode switch is a radio group or toggle-button group (decided in B02). |
| Dialog, Modal **B02** | `Dialog`, `Modal`, `ModalOverlay` | Dialog (modal) | Focus returns to the invoker. |
| Popover, Tooltip | `Popover`, `TooltipTrigger`, `Tooltip` | Tooltip | Tooltips never hold information that is not also reachable by keyboard and AT. |
| Disclosure | `Disclosure`, `DisclosureGroup` | Disclosure | Engine sections in the inspector. |
| Meter, ProgressBar | `Meter`, `ProgressBar` | Meter, Progressbar | Level meter values are throttled for AT; render jobs report progress. |
| DropZone, FileTrigger | `DropZone`, `FileTrigger` | — | Drag-and-drop always has a FileTrigger alternative. |
| Command palette **B02** | `Autocomplete` or `ComboBox` with `Menu` | Combobox | Keep one primitive layer; evaluate cmdk only if the RAC composition fails, checking its dependencies against the single-layer rule. Results must be announced ([05 §7](../research/notes/05-desktop-frontend-stack.md)). Check the stability of `Autocomplete` at B02. |
| Splitter | Custom | Window splitter | Not provided by RAC. |
| Chart explorer | Custom | Custom composite (role decided in [08](../architecture/08-interaction-and-accessibility.md)) | The plot's keyboard explorer; behaviour in [keyboard and navigation](keyboard-and-navigation.md). |
| Plot canvas, playhead overlay | Custom (`@sonarithm/viz`) | — | Named, described object (A11Y-01); not itself focusable; the explorer is. |
| Announcement log | `ListBox` or `GridList` in a `log` region | Log | Entries link to the data node they describe. |
| Status bar | Custom | — | Read-only items with text; never colour alone. |
| Toast | RAC toast if stable at B02; otherwise none | Alert/status | Transient messages are also written to the announcement log; nothing important exists only in a toast. |

Every component ships with a Storybook story, the axe addon passing, keyboard and screen-reader notes, and a manual NVDA/VoiceOver check (B02 exit criteria).

## 12. Copywriting voice

The voice is a lab instrument: terse, exact, unit-bearing, calm. Rules:

1. Sentence case; no exclamation marks; no emoji; no "Oops", "Awesome" or "Let's".
2. Every number has a unit and sensible precision.
3. Labels name the thing or the action with a verb ("Export audio", not "Export").
4. Errors state cause, then fix. Warnings state the rule ID when a design rule triggered them.
5. Confirmations use verb buttons ("Discard", "Cancel"), never "Yes" and "No".
6. Empty states are one line plus the command that resolves them.
7. Estimated values say so.
8. Copy is written to be heard: no meaning carried only by symbols, punctuation or layout.

| Context | Bad | Good |
|---|---|---|
| Error | "Oops, something went wrong." | "Could not open temps.csv: row 214 has 5 fields, expected 4. Fix the row or enable Allow ragged rows in Import options." |
| Success | "Awesome, your sonification is ready." | "Exported temps.flac: 1:31, 48 kHz, −16.1 LUFS." |
| Control label | "Volume: 75" | "Sonification level −12.0 dB" |
| Control label | "Speed" | "Rate (points/s)" |
| Lint warning | "Too fast." | "SDR-19: 12 points/s exceeds the 8 points/s guidance; detail may be lost. Lower the rate or switch to a glide mark." |
| Empty state | Illustration plus "Let's get started." | "No dataset open. Open a file (Ctrl+O) or drop a CSV, Parquet or XLSX file here." |
| Tooltip | "Click to play" | "Play from cursor (Space)" |
| Confirmation | "Are you sure? Yes / No" | "Discard unsaved changes to Weather 2026? Discard / Cancel" |
| Estimate | "21.4" | "≈21.4 °C (estimated)" |
| Mode change | "Fun mode on" | "Presentation mode. Pitch is quantized to a scale; values are approximate." |
| Polarity | "Inverted" | "Polarity negative: higher values sound lower." |

Spoken strings follow the same rules and are specified, with terse and verbose templates, in [keyboard and navigation §8](keyboard-and-navigation.md).

## 13. Design review checklist

Use this list for every design review and for the B02 exit review. Each item cites the rule it enforces. A review passes only with zero DL-X violations.

**Structure**
- [ ] Opens into the workspace; no hero, splash or carousel (DL-P01, DL-X04).
- [ ] Fixed regions in the §4.1 order; no free-floating windows (DL-X12); F6 order unchanged at every breakpoint.
- [ ] Narrow layout works at 200 % and 400 % zoom without loss of function (A11Y-25).

**Colour**
- [ ] Only semantic tokens; no colour literals (DL-P07).
- [ ] One accent; signals used only for their meaning (DL-P02).
- [ ] No gradients, glow, blur or translucent cards (DL-X01, DL-X02).
- [ ] Contrast pairs from §5.3 pass in dark, light and high contrast; forced-colors rendering checked, including plots (A11Y-25).
- [ ] No colour-only meaning anywhere, including series and status (DL-P08, DL-X08).
- [ ] Engine colours only in chips and tiles (DL-X13).

**Type and numbers**
- [ ] Plex Sans and Mono; scale tokens only; sentence case (DL-P10).
- [ ] Every value has a unit; tabular figures in all changing readouts (DL-P04, DL-X10).
- [ ] Visual readout, announcement, braille and log agree (DL-P12); estimates are labelled (DL-P13).

**Shape and density**
- [ ] 4 px grid; 24–28 px rows; radii 0–4 px; hairlines, not shadows (DL-P03, DL-X03).
- [ ] Targets at least 24×24 px (A11Y-26).

**Interaction and accessibility**
- [ ] Visible focus, 2 px with offset, at least 3:1 (DL-P05, A11Y-05).
- [ ] Every action in a menu and the palette; shortcuts shown in tooltips and menus (DL-P06).
- [ ] No hover-only affordance (DL-X11).
- [ ] Every icon-only control has an accessible name and tooltip; no emoji (DL-X05).
- [ ] Each plot has the explorer and the Table view (A11Y-01, A11Y-03).

**Motion**
- [ ] Nothing moves at idle; transitions at most 120 ms; reduced motion honoured; no flashing (DL-P09, DL-X09).

**Copy**
- [ ] Copy follows §12; errors give cause and fix (DL-P11).

## 14. Open items for B02

| Item | Resolution path |
|---|---|
| Validate every token pair (§5.3) in all themes; generate the light high-contrast variant | Done in B02-T01: generator output committed, every pair passes ([F-131](../research-paper/findings-log.md)) |
| Verify Plex `tnum` and `zero` features per weight | Font test page in Storybook |
| Forced colors in WKWebView and WebKitGTK | Manual test per webview; finding |
| OS text scaling per webview | Manual test; finding |
| Native versus in-window menu bar | Decided in [09](../architecture/09-desktop-app.md) |
| Command palette implementation | RAC composition prototype; cmdk only as fallback |
| Per-theme categorical palette variants | B04, with CVD simulation |
| Dark-theme engine tile treatment | [Engine identity §5](engine-identity.md) |

## Change log

| Date | Change | By |
|---|---|---|
| 2026-10-18 | Baseline created from research notes 05 §8, §10, §11. | Kickoff |
| 2026-10-25 | B02-T01: §5.2 and §5.3 replaced with the generator's output, including the light high-contrast theme; §5.4 theme selection; spacing tokens are `space-1` to `space-6`; the overlay shadow token. | AI agent (Claude Code), for review |

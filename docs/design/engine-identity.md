# Engine identity

> **Status:** Baseline v1 (2026-10-18). **Owner:** Design lead.
> Related: [engine catalogue](../product/engines.md) (canonical names, roles, icons, OD-03), [design language](design-language.md) (DL-X13 and colour system), [roadmap B02](../planning/roadmap.md), [research notes 05 §8](../research/notes/05-desktop-frontend-stack.md) (Lucide licence).

SONARITHM is one platform with nine engines that plug into one pipeline ([engine catalogue](../product/engines.md)). This document defines how the platform and the engines are shown: names, icons, identity colours, the tile, where tiles may appear, accessible naming, and the SVG assets that B02 produces. The engine catalogue stays canonical for names, roles and icon choices; this document does not restate engine scope.

## 1. Identity rules

1. **Text carries the identity.** Engine names are written in capitals in prose and UI headings (NUMISONA) and in lowercase as machine identifiers (`numisona`) ([engine catalogue](../product/engines.md), naming rule 1). Icons and colours are recognition aids for sighted users; screen-reader users never depend on them.
2. **Functional label first.** Ordinary users see what a group does ("Mapping", "Axes & navigation"); the engine name is the secondary label in headers and advanced views ([engine catalogue](../product/engines.md), naming rule 2).
3. **Identity is not semantics.** An engine colour never signals state, severity, selection or data. It appears only in chips and tiles (DL-X13 in the [design language](design-language.md)).
4. **No tenth identity without an ADR.** The set below mirrors the catalogue's rule that a tenth engine needs an ADR.

## 2. Catalogue

Icons are Lucide icon IDs as listed in the [engine catalogue](../product/engines.md). Colours are the hex approximations sampled from the owner's naming mockup (section 3). Functional labels other than "Mapping" and "Axes & navigation" (which the catalogue gives) are proposals for B02.

| Name | Machine ID | Etymology | Role (one line) | Lucide icon | Glyph colour (≈) | Tile tint (≈) | Functional label |
|---|---|---|---|---|---|---|---|
| **SONARITHM** (platform) | `sonarithm` | SONic + ARITHMetic / algorithm | Spec language, compiler, Score IR, render engine, data layer, app shell | `audio-lines` | teal #2E7D6B | #DDEBE7 | — (product name) |
| **NUMISONA** | `numisona` | NUMerical + SONic | Maps any numeric field to auditory channels | `waves` | indigo #5B5FC7 | #E6E7F5 | Mapping |
| **AUDAXIS** | `audaxis` | AUDitory + AXIS | Auditory axes, navigation structure, speech and spatial cues | `move-down-left` | blue #3B6EA5 | #DCE6F2 | Axes & navigation |
| **SONOTRACE** | `sonotrace` | SONic + TRACE | Time-series transport, live streams and monitoring | `spline` | purple #7B5EA7 | #E8E1F0 | Transport & streams (proposal) |
| **PLOTSONA** | `plotsona` | PLOT + SONification | Chart ingestion and chart-type templates | `chart-no-axes-combined` | amber #B07A35 | #F3E6D3 | Chart import (proposal) |
| **CURVASONA** | `curvasona` | CURVE + SONic | Math functions, critical-point cues and audification | `square-activity` | steel-teal #2F6E8A | #DDEAF0 | Functions & audification (proposal) |
| **AXONOTE** | `axonote` | AXis + NOTE | Musical mapping, instruments and MIDI/OSC output | `audio-waveform` (recommended; see OD-03) | gold #A0814A | #EFE8DA | Music & MIDI (proposal) |
| **ECHOMETRIC** | `echometric` | ECHO + METRIC | Pattern analytics, descriptions, perceptual lint, evaluation harness | `ear` | graphite #5A5F66 | #E4E5E7 | Analysis & lint (proposal) |
| **TONALENS** | `tonalens` | TONAL + LENS | Multivariate exploration lens with linked selection | `scan-eye` | rose #A8506A | #F2E0E6 | Multivariate lens (proposal) |
| **RESONUM** | `resonum` | RESONance + NUMbers | Model-based sonification: data as an instrument | `circle-dot` | green #2F6B4A | #DDEBE3 | Model-based sound (proposal) |

## 3. Identity colours

### 3.1 Status of the values

The hex values in section 2 are **approximations sampled from the naming mockup**. They are not tokens. During B02 each pair is converted to OKLCH tokens (`engine-<id>-glyph`, `engine-<id>-tint`) by the same generator as the semantic colours, then contrast-checked in the dark, light and high-contrast themes.

### 3.2 Preliminary observations

The figures below are WCAG 2.2 contrast ratios computed from the approximate hexes, as input to B02. They are arithmetic, not a validation, and they change when the tokens are generated.

| Name | Glyph ≈ OKLCH | Glyph on tint | Glyph alone on dark `surface-1` |
|---|---|---|---|
| SONARITHM | `oklch(0.536 0.082 175)` | 4.0:1 | 3.6:1 |
| NUMISONA | `oklch(0.537 0.159 278)` | 4.4:1 | 3.3:1 |
| PLOTSONA | `oklch(0.622 0.108 70)` | 3.0:1 | 4.7:1 |
| AUDAXIS | `oklch(0.528 0.103 252)` | 4.2:1 | 3.3:1 |
| SONOTRACE | `oklch(0.542 0.114 301)` | 4.1:1 | 3.3:1 |
| RESONUM | `oklch(0.478 0.083 157)` | 5.1:1 | 2.8:1 |
| AXONOTE | `oklch(0.620 0.082 81)` | 3.0:1 | 4.8:1 |
| TONALENS | `oklch(0.548 0.119 3)` | 4.1:1 | 3.4:1 |
| CURVASONA | `oklch(0.509 0.078 230)` | 4.6:1 | 3.1:1 |
| ECHOMETRIC | `oklch(0.484 0.013 257)` | 5.1:1 | 2.7:1 |

What follows for B02:

- **PLOTSONA and AXONOTE** sit at the 3:1 non-text threshold on their tints. Their glyphs need slightly lower lightness, or their tints slightly higher, to keep a margin.
- **RESONUM and ECHOMETRIC** glyphs fall below 3:1 on the dark surface without their tint. In the dark theme a tile either keeps its light tint as a badge or uses a lightened glyph on a dark tint (section 5).
- **Hue neighbours.** SONARITHM teal (hue ≈ 175), RESONUM green (≈ 157), CURVASONA steel-teal (≈ 230) and AUDAXIS blue (≈ 252) are close, as are PLOTSONA amber (≈ 70) and AXONOTE gold (≈ 81). Colour alone would not separate them, which is acceptable only because the name and glyph always accompany the colour (rule 1).
- **Indigo and purple.** NUMISONA and SONOTRACE use the indigo and purple hues that DL-X01 bans as accents. Confining identity colours to small tiles and chips keeps them from reading as an accent; they must never be promoted to buttons, links, focus or selection.

## 4. Tile style

The tile is the mockup's rounded-square badge: a tinted background with a saturated glyph.

| Property | Specification (proposal, to match the mockup in B02) |
|---|---|
| Shape | Square with rounded corners, corner radius ≈ 22 % of the tile side (7 units on a 32-unit tile). This exceeds the UI radius limit on purpose: the tile is a brand mark, not chrome ([design language §7.3](design-language.md)). |
| Grid | 32×32 viewBox. The Lucide glyph (24-unit grid) is scaled to 20 units and centred with 6 units of padding. |
| Glyph | Lucide outline, round caps and joins, stroke 2 at the 24-unit grid, no fill; stroke colour `engine-<id>-glyph`. |
| Background | Flat `engine-<id>-tint`. No gradient, no shadow, no border in the light theme. |
| Sizes | 16 px (inline chip), 24 px (inspector section header), 32 and 48 px (docs, about dialog), 128 and 256 px (README, release pages). Below 16 px the glyph is shown without the tile. |
| Chip | A text chip ("AUDAXIS") may carry the tint as background with `text-1`-equivalent dark text, verified at 4.5:1; the engine glyph colour is never used for text. |

## 5. Theme variants

| Variant | Treatment |
|---|---|
| Light | Tint background, saturated glyph (as in the mockup). |
| Dark | Decided in B02 by contrast check. Option A: keep the light tint tile as a badge on dark surfaces. Option B: dark-tinted tile (`engine-<id>-tint` regenerated at low lightness) with a lightened glyph. Either option must give the glyph at least 3:1 against its tile. |
| High contrast | Glyph in `text-1` on `surface-2`, 1 px `border-strong` outline; identity hue dropped. |
| Forced colors | Glyph in `CanvasText`, outline in `CanvasText`, no tint. Implemented with `currentColor` so the browser's system colours apply. |
| Print and monochrome | Glyph only, single colour. |

## 6. Where tiles appear

| Context | Allowed | Notes |
|---|---|---|
| About dialog | Yes | Platform tile plus the nine engine tiles with names and one-line roles. |
| Inspector section headers | Yes, 24 px | Header text is the functional label; the engine name follows as a secondary label. |
| Advanced views (engine settings, diagnostics source column) | Yes, 16 px chip | Always next to the engine name. |
| Documentation and docs site | Yes | Engine pages, architecture diagrams, the engine catalogue. |
| README and release pages | Yes | With alt text (section 7). |
| Data canvas (plot, table, explorer) | **No** | Never as decoration, watermark or marker in the data area. |
| Transport, status bar, toasts | **No** | State there uses semantic tokens and text. |
| Backgrounds, gradients, illustrations | **No** | DL-X13. |
| Lint diagnostics severity | **No** | Severity uses signal tokens; the engine appears only as a text source label. |

## 7. Accessible names

1. **Icons are never the only label.** In the UI, tiles beside a visible name are decorative (`aria-hidden="true"`). A tile that stands alone (rare, for example a collapsed header rail) has an accessible name such as "AUDAXIS, Axes & navigation".
2. **Documentation images.** When the name is printed next to the tile, use an empty alt attribute. When it is not, use alt text in the form "AUDAXIS engine icon".
3. **Standalone SVG files** carry a `<title>` with the engine name so they are labelled when opened directly; inline SVG in the app omits it and relies on rule 1.
4. **Pronunciation risk [CHECK].** Some speech synthesizers spell out short all-capital words letter by letter. B07 checks how NVDA, JAWS, Narrator, VoiceOver and Orca, with their default voices, read the engine names in headings. If any spells them out, the accessible name uses mixed case ("Audaxis") while the visible text stays in capitals, and the change is logged as a finding. A pronunciation lexicon for optional self-voicing is a B03 task.
5. **Functional labels are spoken first.** Inspector headings are announced as "Mapping, NUMISONA", not the reverse, so users who do not know the engine names still understand the group.

## 8. Open decision OD-03: AXONOTE icon

**Problem.** In the naming mockup, AXONOTE's glyph is the same "audio bars" shape as the SONARITHM platform icon (`audio-lines`). Two identical icons with different meanings hurt recognition. Screen-reader users are unaffected because text labels carry the meaning ([engine catalogue](../product/engines.md), icon note).

**Recommendation.** Use Lucide `audio-waveform` for AXONOTE and keep `audio-lines` for the platform. The silhouettes differ (a continuous waveform versus discrete vertical bars), and the waveform still reads as "sound shaped into music".

**Constraints on any alternative.** It must exist in the pinned Lucide version, differ in silhouette from `audio-lines` at 16 px, and not reuse another engine's glyph.

**Decision.** Owner: project lead. Due: before B02 asset production. Record the decision in the [engine catalogue](../product/engines.md) and in this document's change log. Until it is decided, B02 assets use `audio-waveform`.

## 9. Assets (produced in B02)

### 9.1 File naming

| File | Content |
|---|---|
| `assets/brand/engine-<id>.svg` | Tile, light variant, for each engine (`<id>` is the machine ID, for example `assets/brand/engine-numisona.svg`). |
| `assets/brand/engine-<id>-dark.svg` | Tile, dark variant (proposal; depends on the section 5 decision). |
| `assets/brand/engine-<id>-glyph.svg` | Glyph only, `currentColor` stroke, for monochrome, forced-colors and inline use (proposal). |
| `assets/brand/platform-sonarithm.svg` | Platform tile (proposal: the platform is not an engine, so it does not use the `engine-` prefix). Variants follow the same suffixes. |

`assets/` is the existing target directory for earcons, instrument samples and fonts with their licences ([repository layout](../architecture/02-repository-and-crate-layout.md)); `assets/brand/` is a subdirectory, not a new top-level directory.

### 9.2 SVG requirements

- `viewBox="0 0 32 32"` for tiles, `0 0 24 24` for glyph-only files; no fixed `width` or `height`.
- Paths only: no embedded raster, no text elements, no fonts, no scripts, no external references.
- Colours from the generated tokens, written as literal values at export, with the token name in a comment.
- Optimised with SVGO, keeping `viewBox` and `<title>`.
- Each file has an SPDX header comment (REUSE).

### 9.3 Licence notices

The glyphs are Lucide icons, licensed under ISC; icons derived from Feather are additionally under MIT, © Cole Bemis ([05 §8](../research/notes/05-desktop-frontend-stack.md); [Lucide licence](https://github.com/lucide-icons/lucide/blob/main/LICENSE)). B02 adds:

- `assets/brand/LICENSE-LUCIDE` with the ISC text and the Feather MIT notice (both included for every glyph, since the per-icon origin is not tracked here);
- REUSE metadata assigning each SVG both the Lucide notices (for the glyph) and the project licence (for the tile composition), pending the project licence decision (ADR-0010);
- a line in the app's third-party notices.

## 10. Change control

Changes to names, icons or identity colours start in the [engine catalogue](../product/engines.md) (owned by the project lead) and are mirrored here in the same PR. Changes that affect contrast or the tile geometry need the B02 contrast check to pass again.

## Change log

| Date | Change | By |
|---|---|---|
| 2026-10-18 | Baseline created: catalogue, colour status, tile, placement, accessible naming, OD-03 recommendation, asset naming. | Kickoff |

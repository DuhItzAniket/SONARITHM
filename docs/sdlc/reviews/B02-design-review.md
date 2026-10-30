# B02 design review: components and frame against the design language

> **Status:** Draft (2026-10-30). **Phase:** B02 (task B02-T09). **Author:** written by an AI agent (Claude Code, Lane U session). **Reviewers:** project lead; accessibility owner.
> **Related:** [design language §3 and §13](../../design/design-language.md#13-design-review-checklist) · [engine identity](../../design/engine-identity.md) · [implementation plan B02-T09](../../planning/implementation-plan.md) · [F-173](../../research-paper/findings-log.md)

## Summary

| Item | Value |
|---|---|
| Scope | Every `@sonarithm/ui` component (B02-T04, B02-T05) and the desktop frame, announcer log, command palette and keyboard reference (B02-T06, B03-T04, B02-T07) |
| Revision | `main` at 1353b76 (after #394, B02-T07) |
| Method | Read every component's CSS and markup against design-language §3 and §13; automate the detectable anti-patterns as lints; simulate colour-vision deficiencies for every status colour |
| Outcome | **Zero DL-X violations.** Four checklist items are open; none is a DL-X rule (see Open items) |

This review is not a sign-off of any component. Sign-off needs the NVDA and Narrator walkthroughs (B02-T10, still blocked under the provisional Gate U).

## Anti-patterns (§3)

Every rule is either a lint that runs in CI or, where no lint can decide it, a review result recorded here.

| ID | Check | Result | Evidence |
|---|---|---|---|
| DL-X01 | No gradients, gradient text, glow or coloured shadows | Pass | Stylelint `function-disallowed-list` (gradients, `blur`, `drop-shadow`), `background-clip: text` rejected, `box-shadow` only `none` or `var(--shadow-overlay)`; seeded test `dl_x01` |
| DL-X02 | No glassmorphism or translucent cards | Pass | `backdrop-filter` rejected; surface tokens are opaque (token tests); seeded test `dl_x02` |
| DL-X03 | No radius over 4 px; no shadow outside overlays | Pass | Radius allowed only as `0` or `--radius-0..2`; seeded test `dl_x03`. Two uses of `--shadow-overlay`: overlays (menus, popovers, dialogs, tooltips) and the Medium-layout side rail while it opens over the workspace (B02-T06), which is an overlay in that layout |
| DL-X04 | No hero, feature boxes or empty-state illustrations | Pass | The app opens into the workspace frame. Empty states are one line of text: "No dataset is open.", "Nothing is selected.", "No diagnostics." (see Open item 1 for the missing command) |
| DL-X05 | No emoji; every icon-only control is named and has a tooltip | Pass | **New:** `dl_x05 no catalogue contains emoji` scans every Fluent catalogue (`packages/i18n/test/no-emoji.test.ts`). `IconButton` does not compile without a name, and its tooltip opens on focus (B02-T04). The field error mark (`fields.module.css`) is U+26A0 with VS15 (text presentation), next to the error text, not an emoji (see Open item 2) |
| DL-X06 | No Tailwind, shadcn, Radix or Material defaults | Pass | **New:** `dl_x06 no workspace package depends on a ready made look` (every `package.json`) and `dl_x06 the lockfile has no ready made look even transitively` (`packages/ui/test/dependencies.test.ts`). Widgets are React Aria Components; styles are CSS Modules with tokens |
| DL-X07 | No #000 on #FFF in default themes; text pairs ≥ 4.5:1 | Pass | Token tests (`a11y_25 text pairs meet 4.5 to 1 in every theme`); dark `surface-0` is #131311 |
| DL-X08 | No colour-only meaning | Pass | Two places use a signal colour: an invalid field (red border, plus the error text in `text-1` with a warning mark) and the start failure ("SONARITHM could not start.", `role="alert"`). Both carry the meaning in words. The CVD pass below shows why colour alone would fail |
| DL-X09 | No decorative motion | Pass | **New:** Stylelint rejects `@keyframes`, `animation` and `animation-name` (`dl_x09 keyframe animation is rejected`). The only transitions are background and expand state changes on `--motion-fast`, which reduced motion sets to 0 s. Nothing moves at idle; there are no skeletons |
| DL-X10 | Tabular figures in changing readouts; units always | Pass | Stylelint `sonarithm/readout-numeric`; the Slider's value comes from the Fluent unit formatter ("440 Hz", F-171). No other changing readout exists yet |
| DL-X11 | No hover-only affordance | Pass | Stylelint `sonarithm/hover-focus-visible`: every `:hover` or `[data-hovered]` rule has a focus-visible counterpart. Tooltips open on keyboard focus |
| DL-X12 | No free-floating windows | Pass | Fixed regions in the §4.1 order; F6 order is the same at 1440, 1100 and 800 px (`e2e/frame.spec.ts`) |
| DL-X13 | Engine colours only in chips and tiles | Pass | **New:** Stylelint plugin `sonarithm/engine-colours` rejects `var(--engine-…)` in any file whose name does not contain "chip" or "tile" (`dl_x13 engine colours only in chips and tiles`). No engine colour is used yet |

## Colour-vision deficiency pass (DL-X08, A11Y-25)

`packages/ui/tokens/cvd.ts` simulates protanopia, deuteranopia and tritanopia with the severity-1.0 matrices of Machado, Oliveira and Fernandes (2009) on linear sRGB, and achromatopsia as relative luminance. `packages/ui/test/cvd.test.ts` runs it on the rendered token colours of all four themes.

**Indicators.** `focus`, `accent` and `border-strong` stay at or above 3:1 against `surface-0..3` for every deficiency in every theme (`a11y_25 indicators keep 3 to 1 under every cvd type`). The lowest is 3.03:1, dark `border-strong` on `surface-3`, the same pair that is lowest without simulation (§5.3).

**Signals.** The closest pair of signal colours, as OKLab distance (ΔE_OK; about 0.02 is just noticeable):

| Theme | No CVD | Protanopia | Deuteranopia | Tritanopia | Achromatopsia |
|---|---|---|---|---|---|
| Dark | 0.156 warning/error | 0.064 warning/ok | 0.062 error/ok | 0.134 warning/error | **0.006** warning/ok |
| Light | 0.135 warning/error | 0.041 warning/ok | 0.039 error/ok | 0.112 warning/error | 0.024 warning/ok |
| High contrast dark | 0.128 warning/error | 0.087 warning/ok | 0.058 error/ok | 0.074 warning/error | 0.031 warning/ok |
| High contrast light | 0.121 warning/error | 0.030 warning/ok | 0.023 warning/error | 0.090 warning/error | **0.018** error/ok |

Warning, error and success cannot be told apart by colour alone for some viewers in every theme: in dark under achromatopsia warning and success are the same grey (0.006). This is the expected result for any three saturated hues of similar lightness, and is why §5.3 and DL-X08 already require text or an icon with every signal colour. The tokens are not changed; the rule stands, and every future status view must pass the DL-X08 row of this review ([F-173](../../research-paper/findings-log.md)). `dl_x08 some signal pairs collapse under cvd so signals need text` keeps this table honest: it fails if the closest pair ever moves above 0.02, which means the table should be regenerated.

Series colours (`series-1..8`) do not exist yet; B04 derives them and runs the same simulation (§6).

## Checklist (§13)

| Item | Result | Evidence or note |
|---|---|---|
| Opens into the workspace; no hero, splash or carousel | Pass | `App.tsx`; "Starting SONARITHM…" is one line of text while the app core starts |
| Fixed regions in §4.1 order; F6 order unchanged at every breakpoint | Pass | `e2e/frame.spec.ts` at three widths, three engines |
| Narrow layout works at 200 % and 400 % zoom | **Open** (item 3) | Narrow is tested at 800 px CSS width (200 % of 1600). 400 % zoom (320 px) is not tested |
| Only semantic tokens; no colour literals | Pass | Stylelint `color-no-hex`, `color-named`, colour functions rejected |
| One accent; signals only for their meaning | Pass | `accent` marks selection, the selected tab and the splitter on focus; `signal-error` only for errors |
| No gradients, glow, blur or translucent cards | Pass | DL-X01, DL-X02 |
| §5.3 pairs pass in all themes; forced colours checked | Pass for the pairs; **open** for webviews | Token tests. Every component CSS that sets its own colours has a `forced-colors` block, and the tokens map to system colours. The forced-colours check in WebView2, WKWebView and WebKitGTK is a §14 open item |
| No colour-only meaning | Pass | DL-X08 and the CVD pass |
| Engine colours only in chips and tiles | Pass | DL-X13 |
| Plex Sans and Mono; scale tokens; sentence case | Pass | Stylelint `font-family`, `font-size`, `line-height`, `font-weight` allow-lists; `text-transform: uppercase` rejected; catalogue strings are sentence case |
| Every value has a unit; tabular figures | Pass | DL-X10 |
| Readout, announcement, braille and log agree; estimates labelled | Not applicable yet | No data readout exists before B04; the announcer writes each announcement to the log as spoken (B03-T04) |
| 4 px grid; 24–28 px rows; radii 0–4 px; hairlines | Pass | Spacing only from `--space-*` (Stylelint unit rule); rows `--row-default` 28 px; region borders are 1 px `--border` |
| Targets at least 24 × 24 px | Pass | Controls `min-block-size: --row-default` (28 px), `min-inline-size: --target-min` (24 px). The splitter is 4 px wide and meets WCAG 2.5.8 by the spacing exception: open regions pad their content by 12 px, so a 24 px circle centred on the splitter reaches 10 px past its edge and touches no other target. It is also fully keyboard-operable |
| Visible focus, 2 px with offset, ≥ 3:1 | Pass | `--focus-ring-width` 2 px, `--focus-ring-offset` 2 px (inset for regions, tabs and rows, so the ring is not clipped); `focus` ≥ 5.43:1, and ≥ 3:1 under every CVD |
| Every action in a menu and the palette; shortcuts in tooltips and menus | Pass for the palette | Every registered command runs from the palette (`kb_01_every_registered_command_runs_from_the_palette`); menu items and palette results show their binding. Native menus come later from the same registry (09 §3) |
| No hover-only affordance | Pass | DL-X11 |
| Icon-only controls named with a tooltip; no emoji | Pass | DL-X05 |
| Each plot has the explorer and the Table view | Not applicable yet | No plot before B04 |
| Nothing moves at idle; ≤ 120 ms; reduced motion; no flashing | Pass | DL-X09; `--motion-base` is 120 ms; token test `a11y_25 reduced motion sets every duration to zero`; nothing blinks |
| Copy follows §12; errors give cause and fix | Pass with **open** item 1 | No exclamation marks, "Oops", "Awesome" or "Let's" in any catalogue. The start error gives the app core's message as its cause; its fix depends on the cause |

## Open items

None of these is a DL-X violation. Each has an owner task.

| # | Item | Rule | Owner |
|---|---|---|---|
| 1 | "No dataset is open." names no command, because opening a file is not a command yet. Add "Open a file (Ctrl+O)" with the command that opens datasets | §12 rule 6 | The task that registers the open-dataset command (B04) |
| 2 | The field error mark is a Unicode symbol drawn in CSS, which screen readers may read as "warning sign" before the message. Replace it with the Lucide `triangle-alert` icon, `aria-hidden`, when the icon set is added | §10 | The first task that adds Lucide icons; B02-T10 walkthrough confirms what NVDA and Narrator read |
| 3 | Test the Narrow layout at 320 px CSS width (400 % zoom) | A11Y-25, WCAG 1.4.10 | B02-T10 |
| 4 | Forced colours in WebView2, WKWebView and WebKitGTK | A11Y-25 | §14 open item; B02-T10 for WebView2 |

## Lints added by this review

| Rule | Where | Test |
|---|---|---|
| DL-X05 no emoji in any catalogue | `packages/i18n/test/no-emoji.test.ts` | `dl_x05 emoji detector finds emoji`, `dl_x05 no catalogue contains emoji` |
| DL-X06 no ready-made look | `packages/ui/test/dependencies.test.ts` | `dl_x06 no workspace package depends on a ready made look`, `dl_x06 the lockfile has no ready made look even transitively` |
| DL-X09 no keyframe animation | `packages/ui/stylelint/config.js` | `dl_x09 keyframe animation is rejected` |
| DL-X13 engine colours in chips and tiles only | `packages/ui/stylelint/engine-colours.js` | `dl_x13 engine colours only in chips and tiles` |
| A11Y-25 indicators under CVD | `packages/ui/test/cvd.test.ts` | `a11y_25 indicators keep 3 to 1 under every cvd type` |

All run in CI through the existing `pnpm -r lint` and `pnpm -r test` steps.

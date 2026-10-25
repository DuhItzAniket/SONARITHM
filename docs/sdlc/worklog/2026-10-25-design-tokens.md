# 2026-10-25: Design tokens (B02-T01) and bundled fonts (B02-T02)

> **Date(s):** 2026-10-25. **Phase:** B02 (T01, T02). **Author:** AI agent (Claude Code, Claude Opus 5.5) in a session started by the coordinator session, to be reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [design-language §5, §7–§9](../../design/design-language.md#5-colour) · [implementation plan B02](../../planning/implementation-plan.md#b02--design-system-and-application-frame)

## Context and phase

- Phase: B02, Lane R (no owner step needed).
- Issue(s) / PR(s): B02-T01 [#200](https://github.com/DuhItzAniket/SONARITHM/issues/200); B02-T02 [#201](https://github.com/DuhItzAniket/SONARITHM/issues/201).
- Starting point: `@sonarithm/ui` was an empty placeholder; no Stylelint; the design-language colour table held baseline proposals for three themes. The AUDAXIS lane (B03-T06, #275) had just merged, and this session's worktree was cleaned (`cargo clean`) before starting.

## Goals

- B02-T01: generate the semantic colour tokens in OKLCH for four themes, handle forced colours, `prefers-contrast` and reduced motion, add spacing, radius and motion tokens, validate every §5.3 pair, and lint the detectable anti-patterns.
- B02-T02: bundle IBM Plex with its OFL notices, add the type scale and tabular figures, verify `tnum` and `zero` per weight, and lint readouts (DL-X10). The coordinator's licensing rule applied: stop if OFL-1.1 is not yet in `REUSE.toml` or `deny.toml`.

## What was done

- `packages/ui/tokens/`:
  - `color.ts`: OKLab with Ottosson's matrices, CSS Color 4 gamut mapping, the WCAG 2.2 contrast ratio.
  - `recipe.ts`: the inputs, lightness ladders, the pair list, the forced-colours mapping and the static tokens; the only file with raw values.
  - `build.ts`: resolution, pair checks, the theme-selection rules as data, and CSS and JSON rendering.
  - `generate.ts`: `pnpm -C packages/ui tokens`.
- The generator writes `packages/ui/src/generated/tokens.css` (exported as `@sonarithm/ui/tokens.css`) and `tokens.json`, which records every pair's ratio.
- Stylelint 17.16: the config in `packages/ui/stylelint/config.js`, re-exported by the root `stylelint.config.js`; `@sonarithm/ui`'s lint script runs it.
- Tests, 26 in all:
  - colour arithmetic against published reference values;
  - the committed files are current;
  - every pair passes in every theme, recomputed from the emitted CSS;
  - high contrast never lowers a pair;
  - no pure black or white in the default themes;
  - opaque colours, radii at most 4 px, and a neutral shadow;
  - theme selection over every combination of setting and OS preferences;
  - the theme IDs match the IPC `Theme` type;
  - forced colours and reduced motion;
  - seeded Stylelint violations for DL-X01, DL-X02, DL-X03 and colour literals.
- Docs:
  - design-language §5.1–§5.4, §7.2, §8 and §14 (the generated table with the light high-contrast column, the measured ratios, the selection table);
  - coding-standards §12.6, testing-strategy §13.1, tech-stack, 09;
  - status, traceability (FR-117), CHANGELOG and F-131.
  - status also records B03-T06 as merged (#275).
- B02-T02, first pass: stopped before any change. OFL-1.1 was in neither `REUSE.toml` nor `deny.toml`, and `LICENSES/` had no `OFL-1.1.txt`. The coordinator then confirmed that the owner-approved supply-chain policy already allows OFL-1.1 for fonts, and that adding the text and annotation implements it.
- B02-T02 (merged into this worklog after #281):
  - `packages/ui/fonts/`: IBM's six WOFF2 files (Sans and Mono at 400, 500 and 600) and IBM's `LICENSE.txt`, copied byte for byte from `@ibm/plex-sans` 1.1.0 and `@ibm/plex-mono` 2.5.0, plus a README with their provenance and how to update them.
  - Licensing: `LICENSES/OFL-1.1.txt` (from `reuse download`); a `REUSE.toml` annotation for those seven files only; `.gitattributes` keeps their exact bytes.
  - `tokens/fonts.ts` (the font table with upstream SHA-256 values) writes `src/generated/fonts.css` (the @font-face rules and the `snr-numeric` class), `src/generated/licenses.json` (the font notices for the licences view) and `font-test/index.html`.
  - `tokens/woff2.ts`: a small WOFF2 reader (Brotli from `node:zlib`) for GSUB features, names, the weight class, cmap and advance widths.
  - Type, weight and numeric tokens in the recipe.
  - Stylelint: type-scale, family and weight rules, no uppercase, and the `sonarithm/readout-numeric` plugin rule (DL-X10).
  - Nine more tests (35 in the package), and a rendered check of the test page in Chromium.
  - Docs: design-language §7.1 and §14, supply-chain, tech-stack, 12 §9, coding-standards §12.6, status, traceability, CHANGELOG, F-132 and F-133.

## Decisions

| Decision | Why | Record |
|---|---|---|
| `data-theme` on the root element takes the app's `theme` setting values; `system`, no attribute or an unknown value follows the OS | The app core already has the `Theme` setting (`system`, `dark`, `light`, `highContrastDark`, `highContrastLight`); one attribute maps it directly, and a test keeps the IDs in step with the generated IPC type | design-language §5.4 |
| `prefers-contrast: more` turns an explicit dark or light theme into its high-contrast variant | §5.4: it "selects the high-contrast theme generated from the user's base theme"; A11Y-25 says platform preferences are respected | design-language §5.4 |
| Selection rules apply in order with zero specificity (`:where`), dark first | A webview without a media feature keeps the theme chosen so far instead of losing every token | `tokens/build.ts` |
| Forced colours and reduced motion in a later sublayer (`tokens.system`) | A later cascade layer wins regardless of selector specificity, so no theme rule can override them | design-language §5.4 |
| Emitted colours are gamut-mapped and rounded inside sRGB | Browsers clip out-of-gamut OKLCH; emitting in-gamut values makes the checked colour the rendered colour | design-language §5.2 |
| One base chroma (0.004) and one accent hue (195) in every theme | The baseline's per-token chroma (0.002–0.006) and light hue 200 changed no colour by more than one step per channel; one input per property is what a generator is for | design-language §5.2 design notes |
| High-contrast light focus is a deep blue (hue 262) | Mirrors the dark high-contrast choice: a hue nothing else uses, 6.81:1 or more on every surface | design-language §5.2; proposal for B02-T09 |
| Spacing tokens are `space-1` to `space-6` (4, 8, 12, 16, 24, 32 px) | §7.2 named `space-1 … space-8` but listed six values; the values are the substance | design-language §7.2 |
| Spacing, rows, bars and targets in rem; radii, hairlines and focus rings in px | Sizes that hold text must grow with text scaling (A11Y-25); lines should stay thin | design-language §7.2 |
| The CSS lint uses Stylelint's built-in rules only (B02-T01) | Every check in B02-T01's acceptance criteria maps onto a built-in rule | coding-standards §12.6 |
| The fonts are vendored, not an npm dependency | IBM's packages run an install-time telemetry script, and pnpm 12 fails the install when it is blocked | F-133; `packages/ui/fonts/README.md` |
| Unmodified upstream files only, checked by SHA-256 and by the font names | The OFL reserves the name "Plex" for IBM's own font; subsetting or renaming would need a new name | supply-chain policy (fonts row); `fonts.test.ts` |
| The complete WOFF2 files, not IBM's split subsets | One file per weight covers every Plex script, including the second locale (OD-12), and the six files total about 347 KB | `tokens/fonts.ts` |
| `font-display: swap` | The files are local, so the swap is not visible in practice, and text is never hidden if a file fails to load | `fonts.css` |
| A readout is a CSS class whose name contains "readout", and it must compose `snr-numeric` | Readout components do not exist yet; a naming rule in CSS Modules is what a lint can check, and composing a global class works in CSS Modules | coding-standards §12.6 |
| The font test page is a generated static HTML page | Storybook arrives with B02-T03 (Gate U); the page needs no build and covers every weight from the font table | design-language §14 |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| An unknown `data-theme` value ignored the OS contrast preference | The first draft matched only `system` and no attribute for "follow the OS" | "Follow the OS" is now `:not()` of the four explicit themes; the selection test covers an unknown value |
| The sRGB → OKLab → sRGB round trip missed a 1e-6 tolerance | The published matrices have ten digits, so they are not exact inverses | Tolerance 1e-5, far below one 8-bit step |
| `pnpm install` failed after adding the IBM packages | Their `postinstall` telemetry script was blocked, and pnpm 12 treats ignored builds as an error | Copied the files, removed the dependency (F-133) |
| IBM's `LICENSE.txt` would change on commit | It has CRLF line endings and `.gitattributes` normalises text to LF | `-text` for the font files |
| Plex has no `tnum` feature | Its default figures are already tabular | The test checks equal digit widths instead of the feature (F-132) |
| The first test-page render used fallback fonts | The pane shows `file://` pages outside the project as static snapshots without their stylesheets | Served `packages/ui` from a local static server for the check |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| Lowest text contrast | 5.42:1 (light `text-3` on `surface-2`) | Generator output, 8-bit sRGB | F-131 |
| Lowest non-text contrast | 3.03:1 (dark `border-strong` on `surface-3`) | Generator output, 8-bit sRGB | F-131 |
| Plex digit advance width | 600 units for 0–9 in all six files | WOFF2 hmtx | F-132 |
| Rendered tabular check | `1111.11`, `8888.88` and `0000.00` equal width in every weight (system sans fallback: 59.7 vs 65.1 px) | Chromium pane, 18 px, HW-01 | F-132 |

## Findings to log

- F-131: the generated tokens meet all 156 contrast pairs in four themes; light warning and success colours stay below 4.5:1 for text — [findings log](../../research-paper/findings-log.md)
- F-132: IBM Plex needs no `tnum` feature; its default figures are tabular in every bundled weight, and every weight has `zero`
- F-133: IBM's Plex npm packages send install telemetry, and pnpm 12 fails the install when their script is blocked

## Next steps

- [ ] Owner, with the WebKit checks: open `packages/ui/font-test/index.html` in WKWebView and WebKitGTK (served over HTTP) to see the figures there.
- [ ] When the npm licence report arrives: merge `@sonarithm/ui/licenses.json` into the app's licences view (12 §9), and review install scripts beside licences (F-133).
- [ ] B02-T04 (Lane U): components consume the tokens; `GrayText`, `HighlightText` and button system colours in forced colours.
- [ ] Owner, with B02-T09: forced colours in WKWebView and WebKitGTK (design-language §14).

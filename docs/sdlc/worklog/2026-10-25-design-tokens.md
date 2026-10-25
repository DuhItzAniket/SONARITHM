# 2026-10-25: Design tokens (B02-T01) and the font licence stop (B02-T02)

> **Date(s):** 2026-10-25. **Phase:** B02 (T01, T02). **Author:** AI agent (Claude Code, Claude Opus 5.5) in a session started by the coordinator session, to be reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [design-language §5, §7–§9](../../design/design-language.md#5-colour) · [implementation plan B02](../../planning/implementation-plan.md#b02--design-system-and-application-frame)

## Context and phase

- Phase: B02, Lane R (no owner step needed).
- Issue(s) / PR(s): B02-T01 [#200](https://github.com/DuhItzAniket/SONARITHM/issues/200); B02-T02 [#201](https://github.com/DuhItzAniket/SONARITHM/issues/201) (stopped).
- Starting point: `@sonarithm/ui` was an empty placeholder; no Stylelint; the design-language colour table held baseline proposals for three themes. The AUDAXIS lane (B03-T06, #275) had just merged, and this session's worktree was cleaned (`cargo clean`) before starting.

## Goals

- B02-T01: generate the semantic colour tokens in OKLCH for four themes, handle forced colours, `prefers-contrast` and reduced motion, add spacing, radius and motion tokens, validate every §5.3 pair, and lint the detectable anti-patterns.
- B02-T02: bundle IBM Plex, but only if OFL-1.1 is already in `REUSE.toml` or `deny.toml` (the coordinator's licensing rule).

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
- B02-T02: stopped before any change. IBM Plex is OFL-1.1. ADR-0010 and the supply-chain policy allow OFL-1.1 for fonts, but `REUSE.toml` has no annotation for font files, `LICENSES/` has no `OFL-1.1.txt` and `deny.toml` covers Rust crates only. Reported to the coordinator for the owner.

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
| The CSS lint uses Stylelint's built-in rules only | Every check in the acceptance criteria maps onto a built-in rule, so no plugin is needed | coding-standards §12.6 |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| An unknown `data-theme` value ignored the OS contrast preference | The first draft matched only `system` and no attribute for "follow the OS" | "Follow the OS" is now `:not()` of the four explicit themes; the selection test covers an unknown value |
| The sRGB → OKLab → sRGB round trip missed a 1e-6 tolerance | The published matrices have ten digits, so they are not exact inverses | Tolerance 1e-5, far below one 8-bit step |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| Lowest text contrast | 5.42:1 (light `text-3` on `surface-2`) | Generator output, 8-bit sRGB | F-131 |
| Lowest non-text contrast | 3.03:1 (dark `border-strong` on `surface-3`) | Generator output, 8-bit sRGB | F-131 |

## Findings to log

- F-131: the generated tokens meet all 156 contrast pairs in four themes; light warning and success colours stay below 4.5:1 for text — [findings log](../../research-paper/findings-log.md)

## Next steps

- [ ] Owner: decide B02-T02's licence step. Adding `LICENSES/OFL-1.1.txt` and a `REUSE.toml` annotation for the font files would unblock it (checklist #129).
- [ ] B02-T02 once unblocked (agent): IBM Plex WOFF2, type tokens and the `font-size` lint, the font test page, DL-X10.
- [ ] B02-T04 (Lane U): components consume the tokens; `GrayText`, `HighlightText` and button system colours in forced colours.
- [ ] Owner, with B02-T09: forced colours in WKWebView and WebKitGTK (design-language §14).

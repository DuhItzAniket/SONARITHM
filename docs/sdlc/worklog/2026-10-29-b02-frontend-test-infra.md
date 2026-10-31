# 2026-10-29: frontend test infrastructure

> **Date(s):** 2026-10-29. **Phase:** B02. **Author:** written by an AI agent (Claude Code, Lane U session), for review by @DuhItzAniket.
> **Related:** [worklog conventions](README.md), [B01 host entry](2026-10-29-b01-desktop-host.md)

## Context and phase

- Phase: B02 ([roadmap](../../planning/roadmap.md)); task B02-T03 ([#202](https://github.com/DuhItzAniket/SONARITHM/issues/202)), stacked on the Tauri host (B01-T03), whose UI the E2E tests drive.
- Under the provisional Gate U, every component must ship with axe and keyboard tests, and the A02 UIA tree dump must run in CI (OD-15 as amended on 2026-10-29).

## What was done

- `packages/ui`: Storybook 10.6 (React + Vite) with the a11y and Vitest addons; a theme toolbar and decorator for the four themes; a foundations story (surfaces, text roles, a numeric readout) in each theme; Vitest 4.1 browser projects `storybook` and `visual`; the seeded-violation story and `test:seeded`.
- `apps/desktop`: `e2e.html` with Tauri's `mockIPC` over `@sonarithm/ipc`'s mock host; Playwright 1.64 in three engines; `e2e/axe.ts`, `e2e/keyboard.ts` and their negative tests; `scripts/uia-check.ps1` with `test/uia/workspace.json`.
- CI: the `frontend-browser` job (pinned Playwright image) and the UIA step in the Windows `desktop` job.

## Decisions

| Decision | Why | Record |
|---|---|---|
| Vitest 4.1, not 5 | testing-strategy §12 names Vitest 4; Storybook's addon supports both | no ADR needed: version choice |
| Every axe violation fails a story, not only serious and critical | Stricter than §13.1 and simpler: Storybook's `a11y.test: "error"` has no impact filter. Playwright filters to serious and critical, as §13.1 says | testing-strategy §12 |
| Screenshot baselines come from CI, not a laptop | Fonts render differently per OS; the job writes missing baselines to an artifact for review | testing-strategy §12 |
| esbuild's install script stays off (`allowBuilds`) | It only re-checks the binary that its optional package already provides | pnpm-workspace.yaml |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| Storybook story tests | 4 stories pass axe | HW-01, Windows 11, Chromium (Playwright 1.64.1) | not a budget |
| Playwright E2E | 5 tests pass | HW-01, Chromium only (WebKit and Firefox in CI) | not a budget |
| UIA tree check | 22 elements, 3 expected found | HW-01, WebView2 156 | not a budget |

## B02-T04: core components (same day)

- `@sonarithm/ui` components on React Aria Components 1.22: Button, IconButton, ToggleButton, ToggleButtonGroup, Toolbar, MenuButton (MenuItem, Submenu, MenuSeparator), ModalDialog with DialogTrigger and DialogActions, Tabs, Tooltip, Disclosure and DisclosureGroup. Every string is a prop; 26 new `ui-*` messages for commands and controls.
- The story tests now run once per theme. Each project has its own Vite cache folder, because parallel projects that share one fail on Windows with EPERM.
- Locally: 60 story tests (15 stories × 4 themes) pass axe and their keyboard tests; the IconButton type test and the DL-X11 Stylelint rule pass.

## B02-T05: data components (2026-10-30)

- Tree, DataTable (virtualized), Slider (React Aria hooks), NumberField, Select, ComboBox, ListBox and ReorderableList; 23 new `ui-*` messages; `react-aria` 3.53.1 and `react-stately` 3.51.0 added as direct dependencies at the versions React Aria Components already uses.
- #383 fixed on the DOM side (F-172). Of F-045's three points: the tree row name and the `SliderOutput` live region are resolved; React Aria's own announcer is filed for B03-T04 (#388).
- The story runs crashed and hung on Windows: four parallel theme projects shared Storybook's dependency cache, and the failed runs left 53 headless browsers running, which hurts on the shared 16 GB laptop. The themes now run one after another (`test/run-stories.ts`), and a run ends by checking that no browser is left. That fix and the Menu and Dialog focus waits reached main with this PR, since #386 merged first.
- Locally: 104 story tests (26 stories × 4 themes), the seeded negative test and the package tests pass.

## B02-T06: application frame (2026-10-30)

- `apps/desktop/src/app/frame`: regions, F6 cycle, region focus memory, Medium and Narrow layouts; `@sonarithm/ui` `Splitter`; 11 new `ui-*` messages. The region placeholders say which task fills them.
- `e2e/keyboard.ts` `tabThrough` now counts a composite widget (toolbar, radio group, tab list, grid, tree, listbox, menu) as one Tab stop, as APG intends, and names any element it never reached.
- The `hidden` attribute lost to the regions' own `display`, so Narrow first showed every region. A CSS rule restores it.
- Decision: native menus on every OS (09 §3); built with the command registry (B02-T07).
- Locally: 19 Playwright tests in Chromium (landmarks, F6 order, no trap, axe at three widths, splitter keyboard, collapse and expand, Narrow F6) and the Splitter story pass.

## B03-T04: announcer (2026-10-30, after the owner's pause)

- `apps/desktop/src/announcer`: the routed announcer (live regions by default, UIA notifications by setting, fallback), the live-region pair, the feed from the `announcements` channel and the Status log. The host gained `announce_native` in `platform/windows.rs` (ADR-0025 scope). `sonarithm-app` gained the setting `speech.announcer` (additive, `#[serde(default)]`; coordinator OK).
- #388 decided: React Aria's widget announcements stay; documented in 08 §4 and added to the Narrator walkthrough (WALKTHROUGH.md §2).
- Built while free RAM stayed near 0.5–1.2 GB (the owner's other work): the TypeScript was written and unit-tested first, and the Rust build and browser tests waited for at least 2 GB free.

## B02-T07: command registry and palette (2026-10-30)

- `apps/desktop/src/commands`: chords (letters by character, punctuation by physical key, KB-08), the v0 keymap as data, the lint, scopes, the registry and dispatcher, the palette and the Shift+F1 reference; 31 new messages. The frame's F6 handling moved into the registry.
- React Aria's Autocomplete must do the filtering itself (its `filter` prop). With an externally filtered list, its active result pointed at a removed option and arrows stopped working.
- axe's `label-content-name-mismatch` flagged menu items whose visible shortcut was not in their name, because React Aria labels the item from its label slot. Items are now named from their content, with the shortcut hidden from the name and kept as the description. This also applies to `@sonarithm/ui`'s `MenuItem`.
- Stacked on #393 (the announcer), whose announcer the palette uses for the result count.

## B02-T09: design review (2026-10-30)

- [Review record](../reviews/B02-design-review.md): every component and the frame against design-language §3 and §13. Zero DL-X violations; four open items, each with an owner task (the empty state's missing command, the CSS warning mark to become a Lucide icon, 400 % zoom, forced colours per webview).
- New lints: Stylelint rejects `@keyframes` and `animation` (DL-X09) and `--engine-*` outside chip and tile files (DL-X13, plugin `sonarithm/engine-colours`); node tests for emoji in the catalogues (DL-X05) and for Tailwind, shadcn, Radix or Material packages in any manifest or the lockfile (DL-X06).
- `packages/ui/tokens/cvd.ts`: CVD simulation (Machado 2009, severity 1.0; achromatopsia as luminance). Indicators keep 3:1 for every deficiency; signal pairs collapse (F-173), so the DL-X08 rule of text or an icon with every signal colour stands.
- No cargo build: PLOTSONA held the Rust build slot. Locally: the `@sonarithm/ui` and `@sonarithm/i18n` node tests, lint and typecheck. The writing tools turned Unicode escapes in a test file into literal invisible characters; the file now spells them as escapes.

## B04-T01: plots and palettes (2026-10-30)

- `series-1..8` tokens derived from Okabe–Ito by a reproducible loop (`tokens/series.ts`, printed by `tokens/series-print.ts`). A test checks that the recipe matches the derivation. Two fixes came from the token tests: black became `text-2` everywhere (DL-X07), and high-contrast series take their standard counterpart's ratio as a floor.
- The light theme's warm hues turn brown to stay apart under deuteranopia (F-174).
- `@sonarithm/viz`: `Plot` on uPlot 1.6.32 (MIT), the dash patterns, and viridis and magma lookup tables from BIDS/colormap (CC0). cividis is OD-29. The Plot stories live in `@sonarithm/ui`'s Storybook (`src/viz`), so they run in all four themes with axe.
- No cargo build. Locally: `@sonarithm/ui` (48) and `@sonarithm/viz` (3) node tests, lint and typecheck, and the Plot story tests in four themes. The shell kept collapsing backslashes, so regex work moved to `sed` and to the editor tool.

## B04-T02: dense layer (2026-10-30)

- `DenseLayer`, `createRenderer` (WebGL2 or Canvas2D), `cssColor`. The shaders are adapted from the A02 spike's `webgl.ts`.
- A first draft drew twice on mount: the ResizeObserver reported the same size, and a new size object redrew. An unchanged size now keeps its object; the story test catches a regression.
- **Fallback path.** A draft fell back by swapping the canvas from inside an effect (setState in an effect, which lint rejects). A one-time probe on a throwaway canvas picks the context up front instead.
- **perf project.** The test was run once locally to check the plumbing, and its numbers are not recorded. CI reports them; the trace is written to `packages/ui/.perf/`, which git ignores.
- **F-044 GPU setting.** Asked the coordinator to approve a `gpu` settings field; this PR holds that part.

## B04-T04: playhead, UI side (2026-10-30)

- `PlayheadClock` and `Playhead` in `@sonarithm/viz`. The anchor's `hostTimeS` is on the device clock (cpal), which the page cannot map onto `performance.now()`, so the clock works on receive times with a minimum-delay filter.
- The jump threshold is 50 ms of position: a 250 ms draft would have ignored a short backward seek while playing.
- `ClockAnchor.outputLatencyS` was approved as an additive field. The Rust type, codegen and the ADR-0017 "Clock anchors" record with a change-history line are done, and the null device's anchors carry `null`. `@sonarithm/viz/playhead` is a subpath export, so Node tests can import the clock without TSX or CSS. Stored offsets (approved as option b) get their own PR after this one.

## B04-T05: data table (2026-10-31)

- **Design.** React Aria's Table cannot hold a million rows or take an overridden `aria-rowcount`, so the table holds windows of 1,000 rows. The coordinator chose this over patching ARIA (rule 6) and added conditions: the true row number as the row header, whole-table jumps, "Go to row…", the focus kept in its column, and the range announced only when the window changes. F-177.
- **`encodeView`** in `@sonarithm/ipc` writes binary views for mocks and tests, byte for byte as Rust does (checked against the shared fixture). Mock binary replies can now depend on the request.
- **`GoToRow`.** NumberField commits a typed number during the Enter keydown that also submits the form, so the submit reads the latest value from a ref.
- Locally: desktop 37 unit tests and all 37 E2E tests in Chromium, plus `table.spec.ts` (4) in WebKit; ipc 16, i18n 9 and ui 48 node tests; the DataTable stories in two themes. No cargo: the new Fluent messages reuse sample arguments the Rust catalogue tests already have, and CI runs those tests.

## PB-08 perf trace on slow runners (2026-10-31)

- #419's frontend-browser job failed in `dense.perf.ts`: the test ran past Vitest's 15 s browser timeout, because 60 software-rendered redraws of a million points were too slow on that runner, and no trace was written. No budget check failed; the test only records. Measuring now stops after 8 s (`cutShort` in the report), the test timeout is 120 s, and it still fails if nothing draws. Not run locally (the builder and browser slots were others'); CI runs it.

## B04-T06 (1): the one cursor (2026-11-01)

- **Contract, approved with five conditions.**
  1. One owner; B03-T09's verbs use this cursor.
  2. `origin` decides speech (table moves are not announced; INFER, OD-30).
  3. A structured path with levels.
  4. The cursor is cleared or clamped when its data changes, with a test for each case.
  5. A `cursor` event never moves the focus, and the readout is not live.
- **App core.** `app/cursor.rs`. `App::dispatch` now runs the command, then `sync_cursor`. That one check covers a closed dataset, corrections, undo and a compile, and a project load clears the cursor first. A replaced dataset can keep its ID (projects reuse them), so clearing on load is explicit.
- **Gap.** The app core has no point-cue builder, so a cursor move plays no cue yet (B03-T09, B03-T12).
- **UI.** `useCursor` (`cursor_get`, then events, newest `seq` wins). `CursorReadout` with `@sonarithm/ui`'s new `Breadcrumbs`: only the current level is a link, with `aria-current="page"`, and the separators are CSS. The table sends `cursor_set` on `focusin` and shows the cursor row selected. A cursor in another window loads it only when the focus is outside the table. Table rows now carry `__snr_row`, so a sorted table still names the dataset row.
- Locally, between low-memory pauses: codegen, clippy and all 134 `sonarithm-app` tests, `sonarithm-i18n` 25, the frontend typecheck, lint and unit tests, the Breadcrumbs and DataTable stories, and all 39 Chromium E2E tests. The first cursor test caught an empty `cursor` event on the session's first command; the model now starts as "no cursor, already sent". The E2E caught that a cursor's window never loaded after the focus left the table: whether the focus is inside is now state, so leaving re-runs the load.

## B04-T06 (2): the plot and its crosshair (2026-11-01)

- `Plot` gains `cursor` and `onPick`. The crosshair and the ring live in uPlot's overlay, placed with `valToPos` and moved by transform, so a cursor move redraws nothing. The click is a native listener on the overlay, so the image gets no JSX handler. The design system has no full radius, so the ring is a square with `--radius-1`.
- `DatasetPlot` with `axesOf` (the first temporal or numeric field, then the next numeric one) and `nearest` (binary search over the tile's x). M4 keeps some rows only, so a cursor row outside the tile reads its values from a one-row `table_page`.
- **What the E2E caught.** In the virtualized table, the cursor's row could be loaded but scrolled out of the DOM. A cursor moved elsewhere now scrolls its row into view (never the focus) while the focus is outside the table.
- **Pseudo-locale.** The test forbids unaccented letters in the output, and the `axis` sample is the letter "x", so the plot's label takes the x header as `text`.
- Locally: desktop 43 unit tests, all 40 Chromium E2E tests, `cursor.spec.ts` in WebKit (3), the Plot stories, `sonarithm-i18n` 25 and the TypeScript i18n 9.

## Smoke run watchdog (2026-11-01)

- #436's Linux smoke step timed out after 5 minutes without a report. The log ends at WebKit's usual libEGL warnings, so nothing showed whether a check hung or the page never loaded.
- `public/smoke-guard.js` (classic, before the modules) reports a page error before the run starts, or no start within 60 s. `runSmoke` gains a 120 s watchdog that names the check still waiting. The normal path is unchanged; CI exercises it, since a smoke run needs the desktop build.

## Next steps

- [ ] Commit the screenshot baselines from the first CI run's `visual-baselines` artifact.
- [ ] B02-T04 and T05: components with stories, keyboard tests, and UIA expectations.

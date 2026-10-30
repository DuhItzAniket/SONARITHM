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

## Next steps

- [ ] Commit the screenshot baselines from the first CI run's `visual-baselines` artifact.
- [ ] B02-T04 and T05: components with stories, keyboard tests, and UIA expectations.

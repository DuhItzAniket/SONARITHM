# 2026-11-28: OD-23 and OD-29 recorded, Dependabot alerts triaged

> **Date(s):** 2026-11-28. **Phase:** maintenance (C02, B04). **Author:** coordinator lane, written by an AI agent.
> **Related:** [worklog conventions](README.md) · [open decisions](../../planning/open-decisions.md) · [#129](https://github.com/DuhItzAniket/SONARITHM/issues/129)

## Context and phase

- Phase: maintenance beside C02 ([roadmap](../../planning/roadmap.md)) and B04-T02 (the spectrogram layer)
- Starting point: the owner gave the coordinator full decision authority for the night. They decided OD-23 themselves and delegated OD-29. Four moderate Dependabot alerts were open.

## Goals

- Record OD-23 (OCR deferred to D04) in the plan, the status page and #129.
- Research the licence of cividis for OD-29, from primary sources, and record the coordinator's decision.
- Fix the four Dependabot alerts where a compatible bump exists, and record why the others stay.

## What was done

- **OD-23:** closed as option (d). C02-T07 is deferred to D04 and C02's exit does not need it. The change touched open-decisions, status, the implementation plan (task row, HP-18, the Stage C dependency graph, the licence table), the roadmap (C02 row and size note, the D04 scope), the risk register (R-32), supply-chain-and-licensing and 10 §3. The OD-23 item on #129 is ticked.
- **OD-29:** the paper and its S2 table are CC0, and matplotlib's table is a different revision under a licence outside the allowlist ([F-250](../../research-paper/findings-log.md#f-250--the-cividis-paper-and-its-256-entry-table-are-cc0-not-cc-by-40-and-matplotlibs-cividis-differs-from-the-papers-table)). The coordinator decided (under the owner's delegation): ship cividis from S2 with credit in B04-T02, not matplotlib's table and not a re-derivation. OD-29 is closed; design-language §6, supply-chain-and-licensing and `colormaps.ts` no longer say it waits.
- **npm alerts 4 (glob, GHSA-3g3m-2fjr-6pw4) and 5 (minimatch, GHSA-g3hv-75wg-m249):** typescript-eslint moves to 8.73.0, which takes minimatch ^11.0.1. A scoped pnpm override gives Storybook's `@joshwooding/vite-plugin-react-docgen-typescript` (still `^0.7` in Storybook 10.6.9) glob ^14.0.2. Both are development-only (BlueOak-1.0.0). The lockfile now holds glob 14.0.6 and minimatch 11.0.9 (plus 3.1.5, outside the advisory's range).
- **glib alerts 1 and 3 (GHSA-wrw7-89jp-8q8g):** no compatible bump exists, and the unsound API is not reachable ([F-251](../../research-paper/findings-log.md#f-251--glib-018s-unsound-variantstriter-has-no-fix-reachable-from-tauri-2-on-linux-and-nothing-in-the-tree-calls-it)). The spike lock stays because HP-01 to HP-03 still build the spike. With the coordinator's approval, both alerts were dismissed as `tolerable_risk`, citing [#514](https://github.com/DuhItzAniket/SONARITHM/issues/514), which tracks the risk and when to revisit it. The matching item on #129 is ticked.

## Decisions

| Decision | Why | Record |
|---|---|---|
| OD-23: defer OCR to D04 | The owner's decision (2026-11-28); no OCR model with a verified licence exists, and manual tick entry meets C02's exit | open-decisions (closed entry) |
| OD-29: cividis from the paper's S2 table (CC0) | Same terms as viridis and magma, so no allowlist change; matplotlib's copy is another revision under a licence outside the allowlist | open-decisions (closed entry) |
| Dismiss glib alerts 1 and 3 as tolerable risk | No compatible bump exists, the unsound API is not called in the tree, and #514 tracks it | [#514](https://github.com/DuhItzAniket/SONARITHM/issues/514) |
| Fix glob through a scoped override, not a Storybook bump | No Storybook release moves the docgen plugin past ^0.7; the plugin only calls `globSync` with options glob 14 keeps (checked by resolving it from the plugin's own directory) | `pnpm-workspace.yaml` comment; no ADR needed |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| typescript-eslint 8.74.0 has the minimatch fix but is 5 days old | `minimumReleaseAge` is 7 days | 8.73.0 (12 days old) already takes minimatch ^11.0.1 |

## Findings to log

- F-250: the cividis paper and its S2 table are CC0, and matplotlib's cividis differs from it.
- F-251: glib 0.18's unsound `VariantStrIter` has no fix reachable from Tauri 2 on Linux, and nothing in the tree calls it.

## Next steps

- [ ] B04-T02: add the cividis lookup table from S2, with credit in the source file and the third-party notices.
- [ ] On every Tauri bump: re-check whether Linux still pins gtk 0.18 and re-run #514's grep (F-251).
- [ ] Drop the glob override once Storybook's docgen plugin range moves past ^0.7.

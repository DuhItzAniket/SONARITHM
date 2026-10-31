# 2026-10-31: C02-T06 part 4, chart corrections in the app core

> **Date(s):** 2026-10-30–2026-10-31. **Phase:** C02. **Author:** project lead, written by an AI agent (Claude Code), reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [RFC-0005](../../rfcs/0005-chart-correction-file.md) · [ADR-0026](../../decisions/0026-chart-correction-file.md)

## Context and phase

- Phase: C02, Lane R ([roadmap entry](../../planning/roadmap.md)).
- Issue(s) / PR(s): [#308](https://github.com/DuhItzAniket/SONARITHM/issues/308); #406 (the correction file, spec 0.3, PLOTSONA applying edits), merged; this PR (the app part).
- Starting point: #406 merged the reader and writer, spec 0.3 and PLOTSONA's application of edits. RFC-0005 §9–§11 (app commands, undo, export, import, the project member) were left for this part.

## What was done

- App core (`sonarithm-app`):
  - Each dataset opened from a chart keeps a correction document. The undo history holds spec snapshots and correction snapshots, so one undo covers either kind.
  - IPC commands, all additive: `corrections_get`; `corrections_apply` (a JSON Patch, one undoable edit, after which the chart is re-extracted); `corrections_export`; and `corrections_import`.
  - Export leaves notes out on request. It includes a PNG chart's image only on request, with metadata chunks dropped.
  - Import previews first, then applies as one undoable edit. Edits are renumbered, `+e<n>` references are rewritten, duplicates are skipped, and failed guards are listed as conflicts.
  - A project keeps each chart's corrections as `annotations/corrections-<dataset-id>.json`.
- Codes `SNR-APP-084` to `086`. The TypeScript protocol was regenerated, and the facade re-exports `CorrectionSet` and `CorrectionRecord`.

## Decisions

| Decision | Why | Record |
|---|---|---|
| After a reopen, a chart's corrections are read-only until the chart is opened again | The project holds the correction file but not the chart bytes, and editing needs a fresh extraction to check `was` guards against | 10 §3 "As built (C02-T06, corrections)"; `SNR-APP-085` |
| Undo snapshots the whole correction document | Correction files are small (the file limits of RFC-0005), and snapshots make undo of an import exact | `app.rs` `Snapshot::Corrections` |

## Next steps

- [ ] C02-T10, T11 (Lane U, after Gate U): the correction UI on top of these commands.

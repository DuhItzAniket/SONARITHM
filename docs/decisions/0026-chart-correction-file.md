---
status: accepted
date: 2026-10-30
decision-makers: ["coordinator session, under @DuhItzAniket's delegation"]
---

# 0026 · The chart correction file per RFC-0005

## Context and problem statement

Raster extraction is an estimate (10 §3), and 10 §5 promises that a person can correct it, alone or with a helper, by exporting and importing a correction file. [RFC-0004](../rfcs/0004-spec-v0.2.md) §6 defined inline corrections in a spec and reserved `corrections.path` for a file format of its own. A persisted format that is shared between people and opened from untrusted sources is a contract (AGENTS.md rule 11), so [RFC-0005](../rfcs/0005-chart-correction-file.md) proposed one, with eight open questions, each with a proposal.

## Decision outcome

**RFC-0005 is accepted with two amendments** (coordinator under the owner's delegation, 2026-10-30; 7-day FCP waived; the owner can reopen it). The most consequential points:

- **One correction model, two homes.** The file `sonarithm-corrections` 1.0 carries RFC-0004 §6's edits (`add`, `move`, `delete`, `retype`) with stable edit IDs (`e<n>`), stable point IDs (`<series>#<index>` for extracted points, `+e<n>` for added ones) and `was` guards; a spec has inline edits or `corrections.path`, never both.
- **History without personal data.** Each edit records the point's class and confidence `before` it, and chains of edits keep a value's whole past, so a `user_edited` value can say what it was (03 §4). No paths, names, handles or times are written (ADR-0012); notes are the person's own text and export can leave them out.
- **Bound to the chart by content.** `chart.sha256`, the extractor and its version; a file made for another chart applies nothing (`SNR-COMP-037`). Re-binding needs an explicit action (Q3).
- **Calibration travels with the edits** (Q1); the spec's own calibration wins when both exist.
- **Image pixels only on request**, PNG only, metadata chunks stripped, decoded under the raster budgets.
- **In a project** the file is the member `annotations/corrections-<dataset-id>.json`, inside RFC-0002's reserved folder, so a 1.0 reader keeps it and the project format version does not change; `corrections.path` resolves against the bundle there (Q2).
- **Strict reading** with `SNR-SPEC-090` to `094` and stated limits; the reader lives in `sonarithm-spec` (Q7); fuzz target `corrections_read`.
- **Undo** through B05-T01's JSON Patch model: each correction is one undoable patch on the correction document.
- **Import** previews before it changes anything. Every imported edit is renumbered, and references to added points are rewritten to the new IDs (amendment 1); conflicts are failed guards only, and duplicates match on `op`, point, `was` and `to`, ignoring IDs (amendment 2). Export then import into an empty session is the identity on the file's bytes.
- **Spec 0.3** defines `corrections.path` (Q6); the 0.2 → 0.3 migration is the identity.

### Consequences

- Good: C02-T06, T10 and T11 build on one fixed format, and corrections survive save, reopen and sharing.
- Good: the `before` history makes it possible to measure how much correction raster readings need, evidence the uncalibrated confidence model (C02-T05) lacks.
- Neutral: point identity is index-based per extractor version; a new extractor version may make some edits' guards fail, which is reported, never misapplied.
- Bad: collaborators cannot see who made an edit (Q4), by design.

### Confirmation

C02-T06 (issue [#308](https://github.com/DuhItzAniket/SONARITHM/issues/308)): the reader and writer in `sonarithm-spec`, spec 0.3 with its schema, migration and doc 04, the compiler's binding check, the app commands with undo, export and import, and the tests of RFC-0005 §12, including the renumbering test and the fuzz target. No code reads `corrections.path` before then.

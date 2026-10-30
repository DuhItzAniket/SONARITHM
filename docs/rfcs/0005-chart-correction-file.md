# RFC 0005: The chart correction file

> **Status:** In discussion (proposed for acceptance by the coordinator under the owner's delegation). **Author(s):** project lead (drafted by an AI coding agent, Claude Code).
> **Related:** [RFC process](README.md) · [RFC-0004 §5, §6 (calibration and inline corrections)](0004-spec-v0.2.md#6-datasourcecorrections-chart-sources) · [RFC-0002 (project format v1)](0002-project-format-v1.md) · [ADR-0018](../decisions/0018-project-format-v1.md) · [ADR-0012](../decisions/0012-privacy-and-telemetry.md) · [ADR-0013](../decisions/0013-determinism-and-provenance.md) · [ADR-0017](../decisions/0017-ipc-protocol-v1.md) · [03 §4](../architecture/03-data-model.md#4-provenance-and-confidence) · [10 §3.1, §5](../architecture/10-ingestion-and-extraction.md#5-accessibility-of-extraction) · [12 §4, §6](../architecture/12-cross-cutting-concerns.md#4-project-file-format)

| Field | Value |
|---|---|
| RFC | 0005 |
| Start date | 2026-10-30 |
| RFC PR | (this PR) |
| Affects | Persisted formats (a new file, `sonarithm-corrections` v1); spec language (a minor version defines `data.source.corrections.path`, reserved by RFC-0004 §6); IPC protocol (additive commands, ADR-0017 §5); the project bundle only through its reserved `annotations/` folder |
| Phase | C02 (task C02-T06, issue [#308](https://github.com/DuhItzAniket/SONARITHM/issues/308)) |
| Related ADR(s) | [ADR-0012](../decisions/0012-privacy-and-telemetry.md), [ADR-0013](../decisions/0013-determinism-and-provenance.md), [ADR-0017](../decisions/0017-ipc-protocol-v1.md), [ADR-0018](../decisions/0018-project-format-v1.md), [ADR-0021](../decisions/0021-spec-v0.2.md); a new ADR records this RFC's outcome |
| Supersedes | none (fills the `corrections.path` slot RFC-0004 §6 reserved) |

## Summary

This RFC defines a portable **correction file** for an extracted chart: the points a person added, moved, deleted or gave to another series, the axis calibration they set, and enough of each edit's past to show where a corrected value came from. It uses the edit vocabulary RFC-0004 §6 already defines for inline corrections, so there is one correction model with two homes: inline in a spec, or in a file that a spec names, that a project bundle carries, and that people pass to each other to correct a chart together (10 §5). The file is bound to its chart by the chart's content hash, holds no paths and no personal data, carries image pixels only when the person asks, and is read strictly with registered codes. In the app, each correction is an undoable patch on the B05-T01 command bus.

## Motivation

Extraction from a raster chart is an estimate (10 §3, F-180, F-182). The people who need extraction most, blind and low-vision users who were sent an image without its data, also find some corrections hard to make alone: a missed point in a dense cluster is easier to see than to hear. 10 §5 therefore promises collaborative correction, "export and import of a correction file". RFC-0004 §6 defined inline edits in a spec and reserved `corrections.path` for this RFC (amendment 3), because inline edits do not travel well:

- **They live in a spec.** A helper who only has the chart image, or a different spec for the same chart, cannot take them.
- **A spec holds no history.** RFC-0004 left "who changed what" to this RFC; 03 §4 says corrected values become `user_edited`, and a reviewer of a sonification needs to know what each corrected value was before.
- **Calibration and edits belong together.** Edits are in data units after calibration (RFC-0004 §6, A11Y-29), so a correction set is only meaningful with the calibration it was made against.
- **A project needs a place for them.** RFC-0002 reserved `annotations/` for "the PLOTSONA corrections log" without defining it.

Without this RFC, C02-T06 (corrections as app commands), C02-T10 (calibration dialog) and C02-T11 (correction view) have no persisted form, and corrections made in the app would be lost on reopen or could only be shared by copying spec fragments.

## Guide-level explanation

**A correction file** is a small JSON file, conventionally named after its chart: `rain.png.corrections.json`. It looks like this:

```json
{
  "format": "sonarithm-corrections",
  "version": "1.0",
  "chart": {
    "sha256": "9f2c…e41a",
    "name": "rain.png",
    "extractor": "sonarithm-plotsona/raster",
    "extractorVersion": "0.1.0"
  },
  "calibration": {
    "axes": {
      "x": { "kind": "category", "anchors": [{ "at": [77.5, 0], "value": "Jan" }, { "at": [811.5, 0], "value": "Dec" }] },
      "y": { "kind": "linear", "anchors": [{ "at": [0, 507.5], "value": 0 }, { "at": [0, 27.5], "value": 110 }] }
    }
  },
  "edits": [
    {
      "id": "e1",
      "op": "move",
      "series": "Series 2",
      "point": 4,
      "was": { "x": "May", "y": 61.2 },
      "to": { "y": 64 },
      "before": { "class": "cv_estimate", "confidence": 0.41 }
    },
    { "id": "e2", "op": "delete", "series": "Series 1", "point": 11, "was": { "x": "Dec", "y": 3.0 },
      "before": { "class": "cv_estimate", "confidence": 0.22 }, "note": "legend swatch, not data" }
  ]
}
```

**In a spec**, a chart source names it instead of writing edits inline (the minor spec version that follows acceptance; §7):

```json
"data": { "source": { "kind": "chart", "path": "rain.png", "corrections": { "path": "rain.png.corrections.json" } } }
```

The compiler extracts the chart, checks that the file was made for this chart (same SHA-256), applies the calibration and then the edits in order, and reports each edit it could not apply. Corrected rows are `user_edited`. The extraction report lists every corrected point with its value before and after and its class and confidence before.

**In the desktop app,** opening a chart creates an empty correction set. Each correction (in the C02-T11 correction view, or by keyboard in the point table) is one undoable step: Ctrl+Z takes it back and the point returns to its extracted value and class. Saving the project stores the set in the bundle; reopening it restores the corrections and each value's history, though the undo stack itself starts empty, as for specs (B05-T01).

**Collaborating.** *Export corrections…* writes the file. Nothing in it identifies the person or the computer: no names, no times, no paths. It holds the chart image only if the person ticks "Include the chart image" (off by default), so a helper who was sent only the file can see the chart. *Import corrections…* reads a file, checks that it was made for the chart that is open, and announces what it would do ("12 corrections: 11 apply, 1 conflicts with a point you already moved") before the person confirms. Imported corrections are added after the person's own and can be undone as one step.

**On the command line**, `sonarithm chart inspect rain.png --corrections rain.png.corrections.json` shows the corrected chart and the edits that did not apply (C01-T12's `inspect`, extended by C02-T06).

## Reference-level explanation

### 1. File, media type, encoding

- UTF-8 JSON (RFC 8259), no byte-order mark, one top-level object. Conventional name `<chart file name>.corrections.json`; media type `application/vnd.sonarithm.corrections+json` (not registered; used for the desktop file association only).
- Written canonically (§9): keys in the order of this RFC's tables, two-space indentation, `\n` line ends, a final newline, numbers in Rust's shortest round-trip form.

### 2. Top-level object

| Key | Type | Required | Meaning |
|---|---|---|---|
| `format` | `"sonarithm-corrections"` | Yes | Identifies the file; anything else is `SNR-SPEC-090` |
| `version` | `"<major>.<minor>"` | Yes | `"1.0"` in this RFC; §8 |
| `chart` | object, §3 | Yes | The chart the corrections were made on |
| `calibration` | RFC-0004 §5 calibration | No | The calibration the edits were made against |
| `edits` | array of edits, §4 | Yes (may be empty) | In the order they apply |
| `image` | object, §6 | No | The chart's pixels, only on explicit request |

Unknown keys in a version the reader knows are `SNR-SPEC-091` (strict reading, as for specs); §8 covers newer versions.

### 3. `chart`: binding by content

| Key | Type | Required | Meaning |
|---|---|---|---|
| `sha256` | 64 lowercase hex digits | Yes | SHA-256 of the chart file's bytes, as the compile report already records it (RFC-0004 §6, Hashes) |
| `name` | string ≤ 255 chars | No | The chart's **file name only**, for display ("made for rain.png"); never a path. A name containing `/`, `\`, `:` or a control character is `SNR-SPEC-091` |
| `page` | integer ≥ 1 | No | The PDF page, when the chart came from a PDF; default 1 |
| `extractor` | string | Yes | The extractor that produced the points the edits refer to, as in the extraction report (`sonarithm-plotsona/raster`) |
| `extractorVersion` | string | Yes | Its version; point indices are deterministic per extractor version (RFC-0004 §6) |

- **Applying to another chart.** When the chart's SHA-256 differs from `chart.sha256`, no edit is applied and the compile reports `SNR-COMP-037` (warning: "these corrections were made on a different chart file"). There is no partial or fuzzy match: the `was` guards protect a point, not a whole chart. A host may offer to re-bind the file after the person confirms (Unresolved question Q3).
- **Another extractor version.** When `extractor` matches but `extractorVersion` differs, edits still apply through their guards (§4), and each edit whose guard fails is reported as RFC-0004 §6 defines (`SNR-COMP-036`). A different `extractor` (say the file was made on an ML extraction, the chart is now read by the raster tier) is `SNR-COMP-037` too.

### 4. Edits and stable point identity

An edit is RFC-0004 §6's edit (`add`, `move`, `delete`, `retype`, with `series`, `point`, `was`, `to` as defined there), with three keys added for the file:

| Key | Type | Required | Meaning |
|---|---|---|---|
| `id` | `e<n>`, n ≥ 1, unique in the file | Yes | The edit's stable ID: history, undo and import refer to it. Assigned in sequence by the writer; never reused within a file |
| `before` | `{ "class": <class>, "confidence": number? }` | Yes for `move`, `delete`, `retype`; absent for `add` | The point's provenance class and confidence when the edit was made (03 §4): the history a `user_edited` value keeps |
| `note` | string ≤ 1 000 chars | No | The person's own words about the edit. User content: §5 |

- **Stable point ID.** A point the extractor produced is identified by `(series, point)`: its series name as extracted and its 0-based index in that series in the extractor's output order, which is deterministic for an extractor and version (RFC-0004 §6, ADR-0013). Its text form, used in reports and the IPC protocol, is `<series>#<point>` (`Series 2#4`). A point an `add` created is identified by its edit, `+<id>` (`+e7`). Later edits may refer to an added point with `"point": "+e7"`; an integer `point` always means the extractor's index.
- **Guards.** `was` is required on every edit that refers to an existing point and is checked as in RFC-0004 §6: if the point at that ID no longer has that value (within the extractor's stated tolerance), the edit is not applied and `SNR-COMP-036` names it by `id`. An edit is never applied to a point other than the one the person saw.
- **Chains are history.** A value moved twice has two `move` edits; the second's `was` is the first's `to`. The file therefore keeps a corrected value's whole history: its extracted value and provenance (`was` and `before` of the first edit), every intermediate value, and the current one. Nothing is overwritten in place.
- **Order.** Edits apply in array order, after extraction and calibration and before `transform`, exactly as inline edits do. Inline `edits` and `path` are exclusive (RFC-0004 §6: "exactly one of").

### 5. Privacy (ADR-0012, AGENTS.md rule 9)

A writer never writes, and the format has no field for:

- a path of any kind (absolute, relative, UNC, home shorthand, URL) — `chart.name` is a file name and is checked as in §3;
- a person's name, handle, e-mail, machine name or locale;
- a time or date (ADR-0018's rule for projects: same content, same bytes).

`note` is the person's own text and may contain anything they type. The export dialog says so ("Notes are included as you wrote them"), and *Export corrections…* offers "Leave out notes". Calibration anchors and edit values are chart content, not personal data.

### 6. Image pixels only on request

| Key | Type | Meaning |
|---|---|---|
| `image.media` | `"image/png"` | Only PNG in v1 |
| `image.data` | base64 (RFC 4648 §4) | The pixels: the whole chart, or the plot area when the person chooses "plot area only" |
| `image.region` | `{ "min": [x, y], "max": [x, y] }` | Where the pixels sit in the chart's coordinates, so anchors and edits line up |

- Written only when the person turns on "Include the chart image" in the export dialog; never by save, autosave or the CLI without `--include-image`.
- PNG metadata chunks other than `IHDR`, `PLTE`, `tRNS`, `IDAT` and `IEND` are stripped on write (text, time and EXIF chunks can carry personal data); readers ignore the image's own metadata.
- On read the image is decoded under the raster tier's budgets (C02-T02: header checked against the pixel budget before decoding, `SNR-ENG-306`); an `image.data` longer than 4 MiB of base64 text is `SNR-SPEC-092`.
- The image is for people. It never replaces the chart for extraction: corrections bind to `chart.sha256`, and an image-only file applies to nothing until the chart is opened (Unresolved question Q5).

### 7. Where the file lives

- **Beside a spec.** A chart source's `corrections` may be `{ "path": "<relative path>" }`, resolved like the source's own `path` (relative to the spec's base directory; P8). This is the form RFC-0004 §6 reserved; the next minor spec version defines it and stops rejecting it with `SNR-SPEC-056`. A path that is absolute, contains `..` past the base directory, or names something other than a regular file is `SNR-SPEC-093`, before anything is read. Inline edits stay valid; a spec has one form or the other.
- **Inside a project bundle (RFC-0002).** The file is the member `annotations/corrections-<dataset-id>.json`, where `<dataset-id>` is the dataset the chart was opened as. That name satisfies RFC-0002 §2's `annotations/<id>.json` rule, so a 1.0 reader keeps it byte for byte, and it is listed in the manifest with its length and SHA-256 like every member. The project format version does not change. Within a project, the spec's `corrections.path` resolves against the bundle (Unresolved question Q2).
- **Hashes.** The correction file's SHA-256 joins the chart's in the compile report and the render manifest (RFC-0004 §6).

### 8. Versions and strict reading

- `version` is `"<major>.<minor>"`. A reader refuses a newer **major** version (`SNR-SPEC-090`, "made by a newer SONARITHM"). It reads a newer **minor** version, ignores keys it does not know, and warns (`SNR-SPEC-094`) that saving writes its own version and drops them, as RFC-0002 §5 does for projects. Older versions are migrated in memory one step at a time; 1.0 is the first.
- **Limits** (12 §6): the file ≤ 16 MiB; ≤ 100 000 edits; strings ≤ 1 KiB except `note` (1 000 characters) and `image.data`; JSON nesting ≤ 32 (the format itself nests at most 5 levels); duplicate keys are an error. Over a limit: `SNR-SPEC-092`.
- **Shape errors** (a missing required key, a wrong type, an edit ID that repeats or does not match `e<n>`, a `point` that names an added point not defined earlier) are `SNR-SPEC-091` with a JSON pointer. A `calibration` is validated as RFC-0004 §5 defines (`SNR-SPEC-054`), an edit as §6 does (`SNR-SPEC-055`).
- Reading never fails partially: the file is accepted whole or refused with diagnostics; a refused file applies nothing.

### 9. The app: corrections as undoable patches (B05-T01)

- The session holds the open chart's correction set as a JSON document of §2's shape. Each correction is a **JSON Patch (RFC 6902) on that document**, B05-T01's `PatchOp` model: a `move` appends `{ "op": "add", "path": "/edits/-", "value": { … } }`; undoing removes it. A patch applies atomically; every applied patch is one undoable step with an undo token on the command bus (FR-111), and its label names the point (`Move Series 2#4 to 64`).
- Calibration changes in the C02-T10 dialog are patches on `/calibration`.
- New IPC commands, additive under ADR-0017 §5, defined with their payloads by C02-T06: `corrections_apply(dataset, patch)`, `corrections_export(dataset, options)`, `corrections_import(dataset, file)` (with a preview step), sharing `history_undo` and `history_redo`. The extraction result is recomputed from the extracted points plus the edits, never by mutating the extracted table, so undo is exact.
- **Inline or file.** A spec that has inline `edits` keeps them; the app writes corrections to the inline form when the spec already uses it, otherwise to the correction file, and never to both.

### 10. Provenance (03 §4)

- An added, moved or retyped point's row is `user_edited`, with no confidence, as RFC-0004 §6 says. A deleted point's row is removed and counted.
- The extraction report lists each corrected point by its stable ID with its first `was`, its current value, and the first edit's `before` (class and confidence), so announcements and the correction view can say "edited; was 61.2, estimated with confidence 0.41". This is the history 03 §4 keeps; the data table's provenance columns are unchanged (one class and confidence per row).
- 10 §5's summary counts `user_edited` points as exact (C01-T12) and states how many were corrected.

### 11. Import and merge (10 §5)

1. Read the file strictly (§8). Refused files change nothing.
2. Check `chart.sha256` against the open chart; a mismatch is `SNR-COMP-037` and the import stops, unless the person re-binds it (Q3).
3. **Preview, then confirm.** Compute which imported edits would apply. An imported edit conflicts when it refers to a point the session's own edits already changed (its `was` no longer matches) or when its `id` already exists with different content; conflicts are listed, never applied. Edits identical to ones already present (same `op`, point, `was`, `to`) are skipped as duplicates.
4. The edits that apply are appended after the session's own, renumbered `e<n>` from the next free number (their `note` and `before` kept), as **one** undoable patch.
5. An imported `calibration` replaces the session's only if the person chooses it in the preview; otherwise it is reported as differing.

Export is the reverse: write the session's set (§1, §5, §6). Export then import into an empty session is the identity on the file's bytes (§12).

### 12. Testing (C02-T06)

- **Round trip:** export then import gives byte-identical files; save and reopen of a project keeps the set and every edit's history (`before`, chains).
- **Undo:** property test — any sequence of correction patches, undone in full, returns the canonical document bytes (as B05-T01's spec patches).
- **Privacy:** no path, time or name is ever written (a test that scans written files for path separators beyond `chart.name`, drive letters, and digits in date shapes); PNG metadata chunks stripped.
- **Binding:** a correction file applied to a different chart applies nothing and reports `SNR-COMP-037`; a guard that fails reports `SNR-COMP-036` with the edit's `id`.
- **Strict reading:** one test per code (`SNR-SPEC-090` to `094`), unknown keys, newer major and minor versions, limits.
- **Fuzz target:** `corrections_read` in `sonarithm-spec`'s fuzz crate (nightly matrix): arbitrary bytes through the reader, every error a registered code, with seeds from the tests' files.

### 13. Codes

New: `SNR-SPEC-090` not a correction file, or a newer major version (error); `091` a shape error (error); `092` over a limit (error); `093` an unsafe `corrections.path` (error); `094` a newer minor version (warning); `SNR-COMP-037` corrections made on a different chart or extractor (warning). Reused: `SNR-SPEC-054`, `055`, `056` (until the spec minor version), `SNR-COMP-036`, `SNR-ENG-306`.

### 14. Determinism, real-time safety, performance, security

- **Determinism:** the same set gives the same bytes (§1); IDs are assigned in sequence; no times. A spec with `corrections.path` compiles to the same Score for the same chart and file bytes; both hashes are in the manifest (ADR-0013).
- **Real time:** nothing reaches the audio thread.
- **Performance:** applying 100 000 edits is linear in their number plus the extraction's points; no budget in 01 §7 changes.
- **Security:** a new untrusted input, read with the limits of §8, decoded images under C02-T02's budgets, no paths followed except the spec's `corrections.path` under §7's checks, no network (P8). No new licence: base64 decoding uses the `base64` crate, already in the tree as a transitive dependency (0.21.7, MIT or Apache-2.0); C02-T06 adds it as a direct dependency only after `cargo deny check`.

## Drawbacks

- **Two homes for one model.** Inline edits and the file must stay in step; the reader is shared and the vocabulary is RFC-0004's, so the cost is one more wrapper, not a second model.
- **Index-based identity is brittle across extractor versions.** A new extractor version may reorder points; the guards stop wrong edits, but a person may have to redo some. Content-based identity (nearest point to `was`) would survive more changes but could silently pick the wrong point, which RFC-0004 §6 rules out.
- **History without authorship.** Collaborators cannot see who made an edit. Notes can say so if people choose; the format will not.
- **Images make files large** when included; they are opt-in and capped.

## Rationale and alternatives

- **Corrections inside the spec only** (status quo): no sharing without a spec, no history, nowhere in a project. Rejected; it is why RFC-0004 reserved `path`.
- **A sidecar keyed by file path** instead of content hash: breaks when a file is renamed or moved and leaks folder names. Rejected.
- **An append-only event log** (apply, undo, redo events with sequence numbers): more history, but undo events in a shared file are noise to a collaborator and make the file grow with every slip. The chain of edits (§4) keeps each value's history without recording keystrokes.
- **Storing corrected values instead of edits** (a corrected table): simpler to read, but loses what each value was and cannot be re-applied to a new extraction. Rejected for 03 §4.
- **A new top-level `corrections/` folder in the project** instead of `annotations/`: clearer, but a 1.0 reader drops unknown folders on save (RFC-0002 §5), losing corrections in a mixed-version team. The reserved folder keeps them.
- **Authorship fields** (`by`, a handle): useful for teams, but personal data in a file built to be shared, against ADR-0012's default. Left to a later minor version behind an explicit opt-in if co-design asks for it (Q4).

## Prior art

- **WebPlotDigitizer** (user documentation, register CR-01) saves a project JSON with calibration and points and lets people re-open and adjust them; points are stored as corrected coordinates, not as edits, so the history of a value is lost. SONARITHM keeps edits for 03 §4.
- **JSON Patch (RFC 6902)** as the edit model for undo, as B05-T01 already uses for specs.
- **Reproducible archives** (fixed timestamps, canonical JSON): the same practice RFC-0002 follows.
- **Accessible chart work** (MAIDR, Chart Reader, Umwelt): they present data a person can query but do not define a shared correction file; 10 §5's collaborative workflow has no direct precedent we found (TODO(verify) during review).

## Accessibility impact

- **A11Y-29 (values in data units):** edits and announcements use data units, as RFC-0004 §6 did.
- **A11Y-24 (non-audio equivalents):** the file is plain JSON a screen-reader user can read; the extraction report states every correction in text.
- **A11Y-26, A11Y-05 (keyboard and focus):** export and import are dialogs C02-T11 builds with React Aria; the import preview is announced (count applied, conflicts) before confirming, so nothing changes silently.
- **10 §5:** this RFC is what makes "collaborative correction" possible for a blind user working with a sighted helper: the helper can work from the file (and the image, if included) without the user's project.
- Manual checks: the C02-T11 NVDA and Narrator check (HP-22) covers export, import preview and undo announcements.

## Research impact

- No design rule or default changes.
- The `before` history makes measurable how much correction raster readings need: edits per chart, by class and confidence. That evidence can calibrate the confidence model C02-T05 documents as uncalibrated (notes/06 §8 gaps). A finding is logged when C02-T08's corpus has corrections to count.
- Hypotheses: none directly; H-06 (rendering uncertainty in sound) gains a ground truth when corrected values are compared with their estimates.

## Unresolved questions

Each has a proposal, which C02-T06 follows unless the approver decides otherwise.

| # | Question | Proposal |
|---|---|---|
| Q1 | Should calibration live in the correction file? | Yes, optional (§2): edits are only meaningful with the calibration they were made against. When the spec also has `calibration`, the spec's wins, and the extraction report notes that the file's differs (information, not a warning) |
| Q2 | How `corrections.path` resolves inside a project | Against the bundle root, so `annotations/corrections-d1.json` names the member; outside a project, against the spec's base directory (§7) |
| Q3 | Re-binding a file to another chart (a re-export, a cropped copy) | Allowed only by an explicit person's action in the app (or `--rebind` on the CLI), which rewrites `chart.sha256` and relies on the guards; never automatic |
| Q4 | Authorship | Not in v1 (§5); revisit in a minor version only with an explicit opt-in and co-design evidence |
| Q5 | An image-only file (no chart available) | Shown for viewing only; it applies to nothing until the chart is opened. Extracting from the embedded image would make the file a second chart source and is out of scope |
| Q6 | Which spec minor version defines `corrections.path` | The next minor after acceptance (`0.3`), with only this change unless other accepted RFCs are ready; the 0.2 → 0.3 migration is the identity |
| Q7 | Codes area for the reader | `SNR-SPEC-09x`: the reader lives in `sonarithm-spec` beside RFC-0004's `Corrections` types (wasm-safe, no I/O); the compiler reports binding (`SNR-COMP-037`) |
| Q8 | FCP | The coordinator accepts under the owner's delegation, with the 7-day FCP waived, as for RFC-0002 and RFC-0004; the owner can reopen it |

Out of scope: an ML or OCR suggestion of corrections (C02-T07, D04), merging two people's conflicting edits automatically, a server or cloud sync (P8), and polar or ternary calibrations (10 §3.1 later).

## Future possibilities

- Opt-in authorship and per-edit reasons as a controlled vocabulary ("legend glyph", "occluded", "misread colour") for research on extraction errors.
- A `sonarithm chart correct` CLI that applies a file and writes the corrected data as CSV with provenance.
- Correction files for vector and exact tiers (a wrong Office cache), already allowed inline by RFC-0004 §6.
- A diff view between two correction files for review.

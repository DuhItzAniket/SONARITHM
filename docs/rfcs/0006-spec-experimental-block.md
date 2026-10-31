# RFC 0006: An `experimental` block in the spec, first for low-confidence timbre

> **Status:** **Accepted** (2026-10-31, coordinator under the owner's delegation; 7-day FCP waived; the owner can reopen it). Q1–Q5 as proposed; Q6: manifest version 2. Conditions: the timbre parameters are a new `INFER` rule (SDR-43) cited in doc comments and test names, and the Prior-art citations are resolved. Recorded in [ADR-0027](../decisions/0027-spec-experimental-block.md); implemented by C02-T09b. **Author(s):** project lead (drafted by an AI coding agent, Claude Code).
> **Related:** [RFC process](README.md) · [RFC-0004 (spec v0.2)](0004-spec-v0.2.md) · [RFC-0005 (spec v0.3 via the correction file)](0005-chart-correction-file.md) · [ADR-0013](../decisions/0013-determinism-and-provenance.md) · [03 §4](../architecture/03-data-model.md#4-provenance-and-confidence) · [04](../architecture/04-sonification-spec.md) · [open-decisions OD-07](../planning/open-decisions.md) · [hypotheses H-06](../research-paper/hypotheses.md) · [design-rules](../research/design-rules.md)

| Field | Value |
|---|---|
| RFC | 0006 |
| Start date | 2026-10-31 |
| RFC PR | (this PR) |
| Affects | Spec language (a minor version, `specVersion: "0.4"`, one new top-level key); the compile report (one Info code); the render manifest (format version 2, one optional field) |
| Phase | C02 (task C02-T09b, split from C02-T09 by the coordinator on 2026-10-30) |
| Related ADR(s) | [ADR-0013](../decisions/0013-determinism-and-provenance.md), [ADR-0021](../decisions/0021-spec-v0.2.md); [ADR-0027](../decisions/0027-spec-experimental-block.md) records this RFC's outcome |
| Supersedes | none |

## Summary

This RFC adds one top-level key to the spec, `experimental`, an object of named switches for features whose sound design is not yet backed by evidence (design-rule strength `INFER`, or a hypothesis still to be tested). Every switch is off unless the spec turns it on. Every switch that is on is reported in the compile report and recorded in the render manifest, and the UI and descriptions say that an experimental feature shaped the sound. The first switch is `lowConfidenceTimbre`, which renders estimated values in a distinct timbre: the treatment H-06 proposes and OD-07 leaves open. The block is the one place future sound experiments go. It is separate from `lint.experimental`, which only turns on experimental lint rules (RFC-0004 §4).

## Motivation

- **C02-T09** labels every estimate in speech, text and descriptions. OD-07 also asks whether a **sound** treatment helps blind listeners notice estimated regions (H-06, EXP-005, P08). To run that study, and to let co-designers try it, the treatment must exist. It must also be off by default and clearly marked, because no evidence supports it yet (AGENTS rule 5: an `INFER` default must stay flagged as experimental).
- Anything that changes sound must come from the spec (AGENTS rule 1). A setting in the app or a profile would make the same spec sound different on two machines. So the switch belongs in the spec, and adding a spec key is a contract change (AGENTS rule 11).
- **One place for experiments.** Without a common block, each experiment would add its own top-level key or overload an unrelated one. `lint.experimental` already exists, but it changes which advice the lint gives, not the sound. Mixing the two would make "experimental" mean two things.

## Guide-level explanation

A spec author who wants to hear estimated values in a distinct timbre writes:

```json
{
  "specVersion": "0.4",
  "data": { "source": { "kind": "chart", "path": "rain.png", "calibration": { … } } },
  "template": { "id": "chart-line" },
  "experimental": { "lowConfidenceTimbre": true }
}
```

- **Compiling.** The compiler reports `SNR-COMP-037` (Info): "Experimental: low-confidence timbre is on. Estimated values sound different; speech still says which values are estimates." The summary and the text description carry the same note.
- **Exporting.** The render manifest's `spec` section lists `"experimental": ["lowConfidenceTimbre"]`, so anyone who receives the audio knows.
- **In the app.** The spec editor shows the switch with the label "Experimental" and the help text from the catalogue. The transport's status line says "Experimental sound on". It is never turned on by a profile, a template or a preset.
- **Without the key, or with `false`**, nothing changes: the Score is byte-identical to one compiled without the block (ADR-0013).

## Reference-level explanation

### 1. The key

| Key | Type | Required | Default | Owner |
|---|---|---|---|---|
| `experimental` | object | no | none (every switch off) | the engine named per switch |

Each member is a switch: a key named in camelCase, with a boolean value. A later RFC or normal PR may give a switch an object value with parameters. The table of known switches lives in `sonarithm_spec::v04::EXPERIMENTS`, each entry giving its key, the engine that implements it, its design-rule or hypothesis IDs and its help message ID.

| Switch | Engine | Evidence | Effect when `true` |
|---|---|---|---|
| `lowConfidenceTimbre` | NUMISONA | H-06 (proposed), OD-07 (open); a new `INFER` rule added by C02-T09b under the design-rules change control | Notes of rows whose provenance class is an estimate (03 §4) play with a distinct timbre (§3) |

### 2. Validation and versions

- **Version gate.** `experimental` is a spec 0.4 key. In a document that declares `"0.3"` or older it is unknown: `SNR-SPEC-004`, with the hint to set `"specVersion": "0.4"`, as RFC-0004 does for v0.2 keys (end of its §8). The 0.3 → 0.4 migration is the identity, so 0.1 to 0.3 documents read as they are and their spec hashes do not change.
- **Unknown switch.** A switch this version does not know is a warning, `SNR-SPEC-047` ("not known to this SONARITHM; ignored"), and has no effect, so a spec written for a newer version still compiles, as with unknown lint rule IDs (RFC-0004 Q7).
- **Wrong type.** A switch whose value is not a boolean is `SNR-SPEC-002` at its pointer.
- **Canonical form.** The data-independent defaults (04 §7) never add `experimental`. The canonical form drops switches set to `false` and drops the block when it is then empty, so `{}`, all-`false` and no block share one spec hash. Unknown switches are kept, so a newer reader sees what the author wrote. Existing hashes stay valid.
- **Templates and profiles** cannot set switches. Templates are built into NUMISONA as code (`templates::Template`) and have no such field, and profiles change speech and navigation, not the spec. A future user-template format must not carry the block either. The spec's own block is the only source.

### 3. `lowConfidenceTimbre`

- **What changes.** For a track whose notes come from rows with an estimate class (`cv_estimate`, `ml_estimate`, `cloud_estimate`), NUMISONA plays those notes with a timbre variant of the track's instrument.
- **The parameters are not fixed here.** C02-T09b proposes them as a new `INFER` rule under the design-rules change control (rule 5). The candidate the rule must justify or replace is a brightness reduction proportional to 1 − confidence, held within the track's brightness range. The rule cites H-06 and EXP-005, and the treatment stays flagged as experimental until EXP-005 reports. This RFC fixes only that the treatment is per note, deterministic, and keyed on provenance class and confidence.
- **Pitch, timing and loudness do not change,** so trend reading is untouched (H-06's non-inferiority margin). Hearing-safety protections apply as always (RFC-0004 amendment 2).
- **Speech stays mandatory.** Estimates are still announced as such (03 §4 rule 3, C02-T09). The timbre is an additional cue, never the only one (A11Y-24).

### 4. Reporting

- **Compile report:** one `SNR-COMP-037` Info per switch that is on, naming it and its evidence status.
- **Render manifest:** `spec.experimental`, a sorted array of the switch keys that were on, left out when none is on. The manifest types refuse unknown fields and `MANIFEST_VERSION` must change with any change to the serialized form, so the manifest becomes **version 2** with schema `sonarithm-render-manifest-2.schema.json`. A manifest with no switch on serializes as before apart from its version number. The reader accepts versions 1 and 2, and the version 1 schema stays as written (Q6).
- **Summary and text description:** a line "Experimental: …" per switch, from the same catalogue message as the UI.

### 5. Codes

New: `SNR-SPEC-047` unknown experimental switch (warning); `SNR-COMP-037` an experimental switch is on (Info). `SNR-COMP-035` is not used: RFC-0004 reserved it for instruments that hide a preset. Reused: `SNR-SPEC-002`, `SNR-SPEC-004`.

### 6. Determinism, real-time safety, performance, security

- **Determinism:** a switch is part of the spec hash. With every switch off or absent, the Score is byte-identical to today's. With `lowConfidenceTimbre` on, the Score is deterministic for the same data (ADR-0013); golden tests cover both states (§7).
- **Real-time:** the timbre variant is chosen at compile time and written into the Score; nothing new runs on the audio thread.
- **Performance:** none measurable.
- **Security and privacy:** no new input beyond a boolean.

### 7. Testing (C02-T09b)

- **Validation:** the version gate, an unknown switch and a wrong type (one test per code).
- **Hashes:** every committed spec keeps its hash, and a spec with `"experimental": {}` or every switch `false` compiles to the same Score bytes as without the block.
- **Golden audio:** a chart with estimated regions, with the switch on, and a null test with it off.
- **Reporting:** the manifest array and the Info diagnostic are present exactly when a switch is on.
- **Schemas:** `sonarithm-spec-0.4.schema.json` and `sonarithm-render-manifest-2.schema.json` are generated, and the 0.3 spec and version 1 manifest schemas stay as written. A version 1 manifest fixture still reads.

## Drawbacks

- **One more top-level key** for authors to learn; mitigated by keeping it out of templates and presets and by help text in the editor.
- **Experiments can linger.** A switch whose study never runs could stay forever. Each switch names its evidence (§1), and the design-rules review revisits `INFER` entries; a switch is removed or promoted when its study reports.

## Rationale and alternatives

- **Reuse `lint.experimental`:** it changes advice, not sound; one flag meaning both would be confusing and would turn on all experimental lint rules with the timbre. Rejected.
- **A per-track field** (`tracks[].uncertainty: "timbre"`): finer-grained, but scatters experiments across the spec, and the study needs a whole-chart condition. It can come later as a switch parameter.
- **An app or profile setting:** would make one spec sound different on two machines, against AGENTS rule 1 and ADR-0013. Rejected.
- **Turn it on by default:** no evidence supports it (H-06 is proposed, EXP-005 planned); AGENTS rule 5 forbids an unevidenced default. Rejected.

## Prior art

- Opt-in feature gates: Rust's `#![feature]` gates and Cargo's `-Z` flags are opt-in, named and removable, and a build that uses one says so. Within SONARITHM, `lint.experimental` (RFC-0004 §4) already gates the `INFER` lint rules the same way.
- Uncertainty in sound: granular synthesis renders distributions and uncertainty as grain density ([Williamson & Murray-Smith 2005](https://doi.org/10.1109/MMUL.2005.37); SDR-32), a different treatment from a per-note timbre. The project's literature review found no study that measures how blind or low-vision listeners notice, or trust, extraction uncertainty conveyed in audio ([notes/06 §7–§8](../research/notes/06-chart-extraction-ai.md)); the research report lists that as an open study (accessible uncertainty, P08). So SDR-43's values are a starting point to test (H-06, EXP-005), not a result taken from the literature.

## Accessibility impact

- **A11Y-24:** the timbre is never the only cue; speech, text and the table keep labelling every estimate.
- **A11Y-14:** the treatment uses brightness, not pan, so it survives mono output and NVDA's sound split.
- **Announcements:** turning the switch on is reported and shown; the status line and description say experimental sound is on, so no listener hears a changed timbre without being told why.
- Manual check: the NVDA and Narrator walkthrough of C02-T11 includes a chart with the switch on.

## Research impact

- **H-06 / EXP-005** become runnable: the condition "timbre change" is this switch.
- No default changes. A new `INFER` rule is proposed by C02-T09b under the change control, with this RFC's limits (per note, provenance-keyed, no pitch, timing or loudness change).
- Findings: C02-T09b logs one when the golden and null tests are in.

## Unresolved questions

Each has a proposal, which C02-T09b follows unless the approver decides otherwise.

| # | Question | Proposal |
|---|---|---|
| Q1 | Spec version | `0.4`, a minor version with only this key, unless another accepted RFC is ready at the same time |
| Q2 | Switch value type | Boolean in 0.4; an object with parameters may come in a later minor version, keeping `true` as its default parameters |
| Q3 | May the CLI or Python turn a switch on without editing the spec? | No; they edit the spec, so the switch is in the hash and the manifest |
| Q4 | Which timbre parameters | Left to C02-T09b's `INFER` rule under the design-rules change control (§3), not fixed by this RFC |
| Q5 | FCP | The coordinator accepts under the owner's delegation with the 7-day FCP waived, as for RFC-0002, RFC-0004 and RFC-0005; the owner can reopen it |
| Q6 | Manifest version 2, or no manifest change? | Version 2 as above, so a listener who receives the audio sees the experiment. The alternative is no manifest change, with the switch visible only in the embedded canonical spec (when included) and through the spec hash, which an audio recipient cannot read |

## Future possibilities

- A switch for an earcon marker at the start of estimated regions (H-06's third condition).
- Switches for other `INFER` sound designs as they are proposed (SDR-20's glide switch-over, SDR-26's stream cap), each removed or promoted when its evidence reports.

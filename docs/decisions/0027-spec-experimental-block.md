---
status: accepted
date: 2026-10-31
decision-makers: ["coordinator session, under @DuhItzAniket's delegation"]
---

# 0027 · The spec `experimental` block per RFC-0006

## Context and problem statement

C02-T09 labels every estimate in speech, text and descriptions. OD-07 and H-06 also ask whether a **sound** treatment helps blind listeners notice estimated regions. That study (EXP-005, P08) needs the treatment to exist. But no evidence backs it yet, so it cannot be a default (AGENTS.md rule 5). Anything that changes sound must come from the spec (rule 1), and adding a spec key is a contract change (rule 11). [RFC-0006](../rfcs/0006-spec-experimental-block.md) proposed one place for such experiments, with six open questions, each with a proposal.

## Decision outcome

**RFC-0006 is accepted** (coordinator under the owner's delegation, 2026-10-31; 7-day FCP waived; the owner can reopen it). Q1–Q5 stand as proposed, and Q6 is manifest version 2:

- **Spec 0.4 adds `experimental`,** an object of named boolean switches. Each is off unless the spec turns it on; templates, profiles and presets never set one. The 0.3 → 0.4 migration is the identity.
- **Validation.** The key in a document declaring 0.3 or older is `SNR-SPEC-004`. A switch this version does not know is a warning, `SNR-SPEC-047`, and has no effect. A known switch that is not a boolean is `SNR-SPEC-002`.
- **Spec hashes.** The canonical form drops switches set to `false` and an empty block, so `{}`, all-`false` and no block share one spec hash.
- **Reporting.** A switch that is on gives `SNR-COMP-037` (Info), a line "Experimental: …" in the text description, and `spec.experimental` in the render manifest.
- **Manifest version 2** (Q6). The manifest types refuse unknown fields and `MANIFEST_VERSION` changes with any change to the serialized form, so the field needs a new version. It is left out when no switch is on, and version 1 manifests still read.
- **The first switch is `lowConfidenceTimbre`** (NUMISONA). Its parameters are a new `INFER` rule, SDR-43, under the design-rules change control: an estimated value plays 0.25 of the brightness scale darker, plus 0.25 × (1 − confidence), kept within [0, 1]. Pitch, timing and loudness do not change. Speech, text and the table keep labelling every estimate (A11Y-24). With brightness mapped from data, NUMISONA warns (`SNR-ENG-057`).
- **Separate from `lint.experimental`,** which only turns on experimental lint rules.

### Consequences

- Good: H-06 / EXP-005 can run, and co-designers can try the treatment, without any default changing.
- Good: one place, with one reporting path, for later sound experiments; each names its evidence and is removed or promoted when its study reports.
- Neutral: the brightness step rests on A05's provisional brightness mapping; if that mapping changes, SDR-43's step must be re-checked.
- Bad: one more top-level key for authors to learn, and manifest readers built before version 2 refuse manifests with an experiment on.

### Confirmation

C02-T09b: spec 0.4 with its schema, migration, validation and doc 04; `TrackRequest.experiments` in `sonarithm-core`; NUMISONA's SDR-43 treatment for notes and glides; the compiler's `SNR-COMP-037`; manifest version 2; the facade's description line. Tests: one per validation code, the spec-hash identity for switches that are off, a byte-identical Score with the switch off, the darkening per confidence, the reporting, and a version 1 manifest that still reads against its schema.

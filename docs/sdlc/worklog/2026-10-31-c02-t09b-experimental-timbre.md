# 2026-10-31: C02-T09b, the spec `experimental` block and low-confidence timbre

> **Date(s):** 2026-10-31. **Phase:** C02. **Author:** project lead, written by an AI agent (Claude Code), reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [RFC-0006](../../rfcs/0006-spec-experimental-block.md) · [ADR-0027](../../decisions/0027-spec-experimental-block.md)

## Context and phase

- Phase: C02, Lane R ([roadmap entry](../../planning/roadmap.md)).
- Issue(s) / PR(s): RFC-0006 ([#418](https://github.com/DuhItzAniket/SONARITHM/pull/418)); this task's PR.
- Starting point: C02-T09 labels every estimate in speech and text (#413). The coordinator split the timbre flag off as C02-T09b, behind an RFC. RFC-0006 was accepted on 2026-10-31: Q1–Q5 as proposed, Q6 manifest version 2. The conditions were: parameters as a new `INFER` rule cited in code and test names, real Prior-art citations, and ADR-0027.

## Goals

- Spec 0.4 with the `experimental` block; `lowConfidenceTimbre` in NUMISONA, off by default and reported wherever it is on.

## What was done

- **Spec 0.4:**
  - `experimental` with `lowConfidenceTimbre`; validation `SNR-SPEC-004` (the key in a document declaring 0.3 or older), `SNR-SPEC-047` (unknown switch) and `SNR-SPEC-002` (not a boolean).
  - The canonical form drops switches that are off.
  - Identity migration 0.3 → 0.4; schema `sonarithm-spec-0.4.schema.json`.
- **Core:** `TrackRequest.experiments`, the switches that are on, so each engine reads only its own.
- **NUMISONA (SDR-43):**
  - Estimated notes and glide points play darker: 0.25 of the brightness scale plus 0.25 × (1 − confidence).
  - Exact and user-edited values do not change. `SNR-ENG-057` warns when brightness also follows data.
- **Compiler:** one `SNR-COMP-037` Info per switch that is on; `Compiled.experiments`.
- **Render manifest version 2:** `spec.experimental`, left out when empty; the version 1 schema stays as written.
- **Facade:** an "Experimental: …" line in the text description; `Compiled::experiments`.
- **Docs:** SDR-43 in design-rules (`INFER`), H-06 linked to it, doc 04 §5.12 and §7, doc 06's manifest note, ADR-0027.

## Decisions

| Decision | Why | Record |
|---|---|---|
| A fixed step for being an estimate, plus a step proportional to 1 − confidence | A step proportional to 1 − confidence alone would leave an estimate with confidence near 1 sounding like exact data, and H-06 asks whether listeners can tell estimates apart | SDR-43, `ESTIMATE_DARKENING` |
| Darken by brightness, not by changing the instrument | RFC-0006 §3 names brightness as the candidate. It survives mono output and the NVDA sound split (A11Y-14), and keeps pitch, timing and loudness | RFC-0006 §3 |
| Engines get the switches that are on as a set of keys in `TrackRequest` | Engines must not read each other's settings; each acts only on the keys it owns, and the compiler passes known switches only | ADR-0027 |
| Warn, not refuse, when brightness also follows data | The spec is valid; the listener should know the two mix | `SNR-ENG-057` |

## Findings to log

- F-185: the render manifest refuses unknown fields, so even an optional field needs a new manifest version.

## Next steps

- [ ] Lane U: show "Experimental sound on" in the transport status line and the switch in the spec editor (RFC-0006 guide-level).
- [ ] C02-T11: include a chart with the switch on in the NVDA and Narrator walkthrough.
- [ ] H-06 / EXP-005: test SDR-43 with blind listeners before any default changes.

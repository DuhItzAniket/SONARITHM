---
status: proposed
date: 2026-10-21
decision-makers: ["@DuhItzAniket"]
consulted: ["coordinator agent (Claude Code), under the owner's delegation of contract approvals"]
---

# 0020 · Profile format v1 and the layers of effective settings

## Context and problem statement

B03-T08 makes listener preferences portable: pitch range, polarity flips, timbre, mark preference, pace, verbosity, part order, modalities and keymap. They are stored as **profiles**, with shipped presets ([08 §6](../architecture/08-interaction-and-accessibility.md#6-modalities-profiles-and-settings), [12 §3](../architecture/12-cross-cutting-concerns.md#3-configuration-and-settings); SDR-15, SDR-38, A11Y-10, A11Y-12).

Two things make this a contract:

* Profiles are files that users import, export and share between machines and versions.
* [RFC-0002](../rfcs/0002-project-format-v1.md) reserves `profiles/project-profile.json` in project bundles and leaves its schema to B03-T08.

Two questions also need one recorded answer:

* How each layer (defaults, settings, profile, project, spec, UI action) reaches the sound.
* Which precedence order holds, since 08 §6 and 12 §3 list slightly different layers.

## Decision drivers

* Spec in, Score out (AGENTS.md rule 1, ADR-0005). A Score must stay reproducible from its recorded canonical spec and data (ADR-0013), whatever profile was active.
* Research-backed defaults keep their rule IDs. Presets are proposals until co-design (B07).
* Profiles move between machines: no paths, no device IDs, no personal data (ADR-0012, AGENTS.md rule 9).
* Forward compatibility like the project format: a newer minor version opens, and a newer major version is refused (RFC-0002 §5).

## Considered options

* **A. One profile document for user profiles and the project profile.** Every field is optional. Sound preferences reach the compiler only as spec defaults that the app core fills in.
* **B. Profiles as a compiler input.** A new "profile" parameter to `compile`, read by each engine.
* **C. Preferences folded into `settings.json`,** with no separate profile files.

## Decision outcome

Chosen option: **A**.

### 1. The document

A profile is a UTF-8 JSON object:

```json
{
  "format": "sonarithm-profile",
  "formatVersion": "1.0",
  "name": "BLV chart reading",
  "sound": {
    "pitchRange": { "hz": [400, 1400] },
    "timbre": "square",
    "marks": "discrete",
    "pace": { "pointsPerSecond": 3 },
    "polarity": { "concepts": { "cost": "negative" }, "flippedFields": ["temp"] },
    "context": { "ticks": true, "referenceTone": true, "spokenLegend": true }
  },
  "interaction": {
    "verbosity": "terse",
    "partOrder": { "terse": ["level", "value", "x"], "verbose": ["level", "series", "x", "value"] },
    "modalities": { "sonification": true, "speech": true, "braille": false, "visual": true },
    "speech": { "route": "screenReader", "duckInsteadOfPause": false, "rateWpm": 180 },
    "keymap": { "nav.whereAmI": "W" }
  }
}
```

* Every field under `sound` and `interaction` is optional. A profile sets only what it changes.
* Value forms reuse the spec's: `pitchRange` is a spec channel range, and `pace` is the spec's `substrate.time.pace`.
* Rust types are the source of truth. `cargo xtask codegen` generates `schemas/sonarithm-profile-1.schema.json` and the TypeScript types (AGENTS.md rule 10).
* Versions follow RFC-0002 §5:
  * same major: read;
  * newer minor: unknown fields are dropped with a warning (`SNR-APP`), and saving writes this version;
  * newer major: refused.
* Limits: at most 1 MiB, and no absolute paths. Import validates against the schema before anything changes.

### 2. Where profiles live

| Profile | Location |
|---|---|
| User profiles | `profiles/<id>.json` in the app-config folder (09 §5). `settings.json` names the active one. |
| Presets | Built into the app and read-only. "BLV chart reading", "Analyst", "Classroom", "Presentation" and "Monitoring", with values from [sound-design §13](../design/sound-design.md). They are proposals pending B07 co-design and are flagged as such. Editing a preset saves a copy. |
| Project profile | `profiles/project-profile.json` in a `.sonarithm` bundle, using the same document. It replaces RFC-0002's byte-for-byte reservation for this member. Other members under `profiles/` stay reserved. |

### 3. Layers and precedence

12 §3's order holds; 08 §6 is updated to match, adding the user-settings layer:

**built-in defaults < user settings < active profile < project profile < spec < explicit UI action**

* **Interaction preferences** (verbosity, part order, modalities, speech, keymap) are resolved field by field in the app core. A UI action is a session override: it applies at once and persists only if the listener saves it to a profile.
* **Sound preferences reach the compiler only through the spec.**
  * Before compiling, the app core fills spec fields that the spec leaves unset with the profile's values: pitch range, pace, instrument, context cues. This produces the effective spec, which is what gets compiled, and its canonical form is what the render manifest records. The compiler and the Score IR do not change, and a Score is reproducible from its manifest without the profile (rule 1, ADR-0013).
  * A spec value always wins.
  * `marks` applies only where the spec leaves the choice: templates and quick sonifications.
* **Polarity** uses NUMISONA's existing `PolarityProfile`: spec explicit > profile concept > concept table, then the flip (SDR-15). The one-key flip toggles the field in the active profile, saves it, and recompiles (FR-032).
* **The effective-settings view.**
  * A new `settings_effective` command returns every setting with its value and the layer it came from. The CLI's `inspect --settings` uses the same function (12 §3).
  * The value of any setting is announced on request.
* **IPC.** The new commands are additive (ADR-0017 §5): `profile_list`, `profile_get`, `profile_activate`, `profile_save`, `profile_import`, `profile_export`, `profile_flip_polarity` and `settings_effective`.

### Consequences

* Good, because the compiler, the engines and the Score IR are untouched, and every Score stays reproducible from its manifest.
* Good, because one document serves user profiles, presets and projects, and a project can carry a class's or a study's settings.
* Good, because each value's source is inspectable and can be announced (12 §3).
* Bad, because the effective spec differs from the spec text the user typed. The spec editor must show which values came from the profile (B05 inspector).
* Bad, because preset values are proposals until B07. Changing them later follows design-rules change control.

### Confirmation

B03-T08's tests:

* Schema tests, and an import–export round trip that is byte-identical after canonicalisation.
* Precedence unit tests over all six layers.
* `sdr_15_*`: the flip persists and survives a restart.
* `sdr_38_*`: one test per personalization field.
* The project profile round-trips through a `.sonarithm` bundle.
* The existing project tests keep passing.

## Pros and cons of the options

### A. One document; sound preferences as spec defaults

* Good, because it keeps "spec in, Score out" and the reproducibility of Scores.
* Neutral, because the app core needs a spec-defaults merge step. It works on the typed spec, so the merge needs no new contract.
* Bad, because a value can be set in two places (profile and spec). The effective-settings view shows which one wins.

### B. Profiles as a compiler input

* Good, because engines would see preferences directly.
* Bad, because the Score would depend on an input that the manifest does not record, unless the profile were also added to the Score IR and the manifest. That would be a contract change to both (RFC territory) for little gain.

### C. Everything in `settings.json`

* Good, because there is one file.
* Bad, because settings hold machine-specific values (device, buffer, consent) that must not travel, and presets and projects could not carry preferences.

## More information

* RFC-0002 §2 (reserved `profiles/`) and §5 (versions); ADR-0017 §5 (additive commands).
* Sound-design §13 (preset table); keyboard-and-navigation §5.5 and §13 (toggles, keymap).
* Revisit if a preference cannot be expressed as a spec default. That would need an engine input and therefore an RFC.

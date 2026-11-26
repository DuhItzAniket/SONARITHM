# RFC 0009: ECHOMETRIC insights in the Score, and detector parameters in annotations

> **Status:** Proposed (2026-11-26). **Author(s):** project lead (drafted by an AI coding agent, Claude Code, for review by the coordinator under the owner's delegation). **Decision:** an ADR, or acceptance recorded here, with C05-T01 and C05-T04.
> **Related:** [RFC process](README.md) · [RFC-0001 §8](0001-spec-v0.1.md) · [RFC-0007 §2](0007-function-cues-and-audification.md) · [05 §4](../architecture/05-compiler-and-score-ir.md#4-score-ir) · [07 ECHOMETRIC](../architecture/07-engines.md) · [ADR-0013](../decisions/0013-determinism-and-provenance.md) · [ADR-0031](../decisions/0031-echometric-numerics.md) · [ADR-0032](../decisions/0032-the-lint-pass.md)

| Field | Value |
|---|---|
| RFC | 0009 |
| Start date | 2026-11-26 |
| RFC PR | (this PR) |
| Affects | Core `Annotation` (a field; not persisted); the Score's `markers`, filled for the first time (IR v1 type, no IR change); a context track from ECHOMETRIC; spec v0.1's `analysis.cue` semantics for ECHOMETRIC's detectors (no new key) |
| Phase | C05 (tasks C05-T01 and C05-T04) |
| Related ADR(s) | ADR-0031 (the detectors), ADR-0032 (the lint pass) |
| Supersedes | none |

## Summary

1. ECHOMETRIC's detectors record the parameters they used in every annotation they produce. This is a new field on the core `Annotation`; annotations are not part of the Score.
2. When a spec's `analysis.detect` names an ECHOMETRIC detector, the Score gains one `Insight` marker per annotation. ECHOMETRIC then adds an `echometric.insights` cue track: `E-LANDMARK` earcons, or speech, as `analysis.cue` says, each with its text equivalent.
3. No new Score IR type, earcon kind or IR version. Specs without ECHOMETRIC detectors compile to exactly the Scores they do today.

## Motivation

- **C05-T01's acceptance:** "annotations carry detector parameters (snapshot)". A listener told "outlier at row 42" should be able to find out what made it one: the method, the threshold, the window. ADR-0031 makes every threshold INFER, so the parameters must travel with the result. Core `Annotation` (`sonarithm-core`, `engine.rs`) has no field for them, and `analysis` is `deny_unknown_fields` (RFC-0001 §8), so the spec has none either.
- **C05-T04's acceptance:** every insight is reachable and linked to its rows (A11Y-23), and every insight earcon has a text equivalent (A11Y-24, A11Y-08).
  - `Score.markers` and `MarkerKind::Insight` exist in IR v1 for exactly this ("A labelled point in time, used by navigation and descriptions"), but nothing fills them.
  - RFC-0007 §2 did the same for CURVASONA's critical points with the `curvasona.cues` track.
  - Filling markers changes the Score content and hash of the specs that ask for detectors. That is a change to what the Score records, so this RFC (AGENTS rule 11).
- **If we do nothing:** insights reach the NavTree (AUDAXIS builds insight nodes from annotations already) but have no time in the Score. Playback cannot cue them, and descriptions cannot say where they are.

## Guide-level explanation

A spec asks for insights as RFC-0001 §8 already allows:

```json
"analysis": { "detect": ["extrema", "outliers", "changepoints"], "cue": "earcon" }
```

While the series plays, a landmark earcon sounds at each maximum, minimum, outlier and change point. The explorer's insight keys (N, Shift+N, I) move between them, and each announces its text, such as "outlier, 3.9 robust z, at row 42". With `"cue": "speech"`, the text is spoken instead of the earcon. With `"cue": "none"`, the insights are still listed and navigable, but nothing sounds.

`sonarithm inspect` lists each insight with the parameters that produced it.

## Reference-level explanation

### 1. Detector parameters in annotations

Core `Annotation` gains one field (additive; the type is `#[non_exhaustive]`):

```rust
/// The parameters the analyzer used, by name (ADR-0031): what a listener can ask for
/// to understand why the annotation exists. Ordered, so its serialization is stable.
pub params: BTreeMap<String, AnnotationParam>,

#[non_exhaustive]
pub enum AnnotationParam { Number(f64), Text(String) }
```

- Every ECHOMETRIC annotation records its method and every parameter:
  - `method: "mad"`, `threshold: 3.5` for outliers;
  - `method: "pelt"`, `penalty`, `minSegment` for change points.
- Other analyzers may leave `params` empty. CURVASONA's `critical-points` does, until it has a reason not to.
- **Annotations stay out of the Score.** The facade exposes them as `Compiled::annotations()`, a public API that is not persisted. `sonarithm inspect` and the app's insight list print them.
- The parameters themselves are fixed per ECHOMETRIC version and listed in 07. The engine versions in the Score header therefore identify them, and a render stays reproducible (SDR-01).
- **No spec key.** A spec cannot set detector parameters in v1. A later `analysis.params` would be its own RFC.
- The data layer's `Annotation` (`sonarithm-data`, `bundle.rs`) is not on the compile path and is unchanged.

### 2. Insight markers

When `analysis.detect` names at least one of ECHOMETRIC's detectors (`extrema`, `outliers`, `changepoints`, `periodicity`, `trend-segments`), the compiler appends one `Marker` per annotation from those detectors, after the shapers:

| Field | Value |
|---|---|
| `t` | The time of the annotation's first row in the first data track that plays that row. This is the time of the event whose `src` contains the row, at the breakpoint for that row (RFC-0007 §2's rule; a glide's last breakpoint is a hold, F-203). A row no event plays (filtered, or outside the timed rows) takes its time from the nearest two timed rows (F-203). |
| `kind` | `Insight` |
| `label` | The annotation's label: a stable English data name such as `maximum`, `outlier`, `change point`, `period` or `rising`. It is read through the catalogue's `insight-<analyzer>-<label>` when that message exists (F-191, C03-T08 i18n). |
| `src` | The annotation's row range |

- **Order:** markers are sorted by `t`, then analyzer ID, then label, so the Score is deterministic (ADR-0013).
- **`critical-points`:** the marker rule applies only to ECHOMETRIC's five detectors, so CURVASONA's function Scores and their replication records do not move (see Unresolved question 1).

### 3. The `echometric.insights` track

ECHOMETRIC registers a context provider that joins a compile only when the spec names one of its detectors (as the facade adds CURVASONA only where a spec uses it, F-197). When `analysis.cue` is not `none`, it adds one track, `echometric.insights` (role `context`), after the data tracks and after `curvasona.cues`:

| `analysis.cue` | Event at each marker's `t` |
|---|---|
| `earcon` | `E-LANDMARK` (an existing earcon kind), with `src` the marker's rows and its text equivalent the marker's label |
| `speech` | a speech event with the label's text |
| `none` | no track |

- **Merging:** cues within 150 ms of each other merge, as RFC-0007 §2 amendment 1 does (ED-02, INFER). The merged cue plays at the first onset, its `src` spans every merged insight's rows, and the text log names every one, so none disappears from the insight list.
- **Earcon kind:** one kind, `E-LANDMARK`, for every insight type. Distinct earcons per insight type would be new earcon kinds and an IR version bump (RFC-0007 §4). They wait for HP-21's listening check and a later RFC.

### 4. Navigation

- AUDAXIS already makes an insight group per analyzer and an insight node per annotation (`InsightGroupNode`, `InsightNode`, ADR-0017 "Navigation"). No change.
- An insight node's rows are the annotation's range, and its time is the marker's `t`, so a move to it seeks there.

### 5. What does not change

- No new Score IR type, field, earcon kind or event type, and no IR version change.
- No spec key.
- A spec without ECHOMETRIC detectors compiles to a byte-identical Score.

## Drawbacks

- Golden records and Score snapshots of specs that name ECHOMETRIC detectors gain markers and a track when T04 lands. Today these are only specs written for C05.
- One earcon for every insight type: a listener tells an outlier from a change point by its text, not its sound, until HP-21.

## Rationale and alternatives

- **Parameters in the Score's markers:** they would make every Score larger to serve a question asked rarely. Marker labels are text equivalents, not reports.
- **A spec key for parameters now:** unneeded until a user needs to tune a detector, and every threshold is INFER, so exposing knobs before evidence invites unsupported settings.
- **New earcon kinds now:** without HP-21's listening check, nobody can say which sounds tell the types apart (RFC-0007's caution for its own earcons).

## Prior art

- RFC-0007 §2: CURVASONA's cue track, its timing and merge rules, which this follows.
- Chart Reader and Siu et al. 2022 (A11Y-23): navigable insights linked to data ranges.

## Accessibility impact

- A11Y-23: every insight is in the Score with its rows and in the NavTree.
- A11Y-24 and A11Y-08: every cue has its text equivalent (the marker label).
- A11Y-02: the labels are statistics, never interpretation.
- Rule 7: an annotation's value carries the confidence and provenance of its rows, and its text qualifies estimates (C05-T05).

## Research impact

- Every detector parameter is INFER (ADR-0031). It is listed in 07, recorded in every annotation, and changed only through design-rules change control. C05-T02 and T03 measure precision and recall on labelled synthetic sets, reported in the experiments README, and note the parameters used.

## Unresolved questions

1. **Should `critical-points` gain markers too?** This RFC limits markers to ECHOMETRIC's detectors, so CURVASONA's function Scores and the 40-curve gallery records do not move. Giving CURVASONA's features markers as well would be consistent, but needs a re-record. Proposed: not in C05, revisit with the C03 pilot (C03-T09).
2. **Insight earcons per type:** after HP-21 (above).

## Future possibilities

- An `analysis.params` key per detector, once a study shows which knobs listeners need.
- Per-type insight earcons, with a new IR version.

## Testing

- **C05-T01:** a snapshot of each detector's annotations with their `params`; a determinism property test (byte-identical annotation JSON across runs).
- **C05-T04:**
  - `a11y_23_*`: each insight has a marker, is reachable in the NavTree and is linked to its rows;
  - `a11y_24_*`: every cue has a text equivalent;
  - Score snapshots for one spec per detector;
  - a test that a spec without ECHOMETRIC detectors compiles to the same Score hash as before;
  - the golden case, with its HP-21 note.

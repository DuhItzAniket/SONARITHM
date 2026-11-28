# RFC 0009: ECHOMETRIC insights in the Score, and detector parameters in annotations

> **Status:** **Accepted** (2026-11-26, by the coordinator under the owner's delegation; FCP waived; Q1 as proposed; with the §1 amendment on the analyzed field, below). **Author(s):** project lead (drafted by an AI coding agent, Claude Code). **Decision:** recorded here; the analyzer change amends [ADR-0014](../decisions/0014-capability-trait-signatures.md).
> **Related:** [RFC process](README.md) · [RFC-0001 §8](0001-spec-v0.1.md) · [RFC-0007 §2](0007-function-cues-and-audification.md) · [05 §4](../architecture/05-compiler-and-score-ir.md#4-score-ir) · [07 ECHOMETRIC](../architecture/07-engines.md) · [ADR-0013](../decisions/0013-determinism-and-provenance.md) · [ADR-0031](../decisions/0031-echometric-numerics.md) · [ADR-0032](../decisions/0032-the-lint-pass.md)

| Field | Value |
|---|---|
| RFC | 0009 |
| Start date | 2026-11-26 |
| RFC PR | (this PR) |
| Affects | Core `Annotation` (two fields; not persisted); the `Analyzer` call (an `AnalysisRequest` with the analyzed fields, ADR-0014 amended); the Score's `markers`, filled for the first time (IR v1 type, no IR change); a context track from ECHOMETRIC; spec v0.1's `analysis.cue` semantics for ECHOMETRIC's detectors (no new key) |
| Phase | C05 (tasks C05-T01 and C05-T04) |
| Related ADR(s) | ADR-0031 (the detectors), ADR-0032 (the lint pass) |
| Supersedes | none |

## Summary

1. ECHOMETRIC's detectors record the parameters they used, and the data field they analyzed, in every annotation they produce. The compiler tells each analyzer which fields the spec's tracks play. This is a new field on the core `Annotation`; annotations are not part of the Score.
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

**Amendment (2026-11-26, with acceptance): the analyzed field.** An analyzer used to get only the data and the `analysis` object, so nothing told it which field a track plays, and an annotation did not say which field it was about. AUDAXIS linked each insight to the first series whose points cover its rows, so on a table with two series an outlier in the second would be linked to the first, and announced as the wrong series. Three changes:

1. Core `Annotation` gains `field: Option<String>`: the data field it is about, or `None` for an analyzer of whole rows. CURVASONA's `critical-points` sets its table's `y`.
2. The `Analyzer` call takes a request instead of the bare `params` (an [ADR-0014](../decisions/0014-capability-trait-signatures.md) amendment):

   ```rust
   fn analyze(&self, data: &D, request: &AnalysisRequest<'_>, cx: &mut CompileContext)
       -> Result<Vec<Annotation>, CompileError>;

   #[non_exhaustive]
   pub struct AnalysisRequest<'a> {
       pub params: &'a SpecFragment,        // the `analysis` object, as before
       pub fields: &'a [AnalyzedField],     // what to analyze, in track order
   }

   #[non_exhaustive]
   pub struct AnalyzedField { pub field: String, pub track: TrackId, pub rows: Option<Vec<u64>> }
   ```

   - The compiler fills `fields` with one entry per data track that maps a field: its pitch channel's field, else its first channel mapped to a field, timbre aside.
   - Context tracks add nothing.
   - `rows` are the rows the track plays, in the order it plays them. They are the rows its `filter` keeps, without those that have no time value, stably ordered by the time field. The time field is the substrate's, else the spec's field with role `x`, else the data's own role-`x` field.
   - `sonarithm_data::play_rows` computes `rows`. A test holds them equal to the source rows of every note NUMISONA encodes, so what a detector reads is what the listener hears. `None` means every row in table order, for a data type that cannot tell.
   - ECHOMETRIC's detectors analyze exactly these series, so every insight belongs to a series that sounds.
3. Core `Annotation` also gains `track: Option<TrackId>`, set by an analyzer that read one track's series. AUDAXIS links an annotation to that track's series first, then to a series of its `field`, then by the old rule (the first series whose points cover the rows).
   - A track's series may skip rows (its filter) and run out of table order (its time field). An annotation's `range` therefore spans its first and last rows in table order, whichever comes first, and `track` says which of those rows belong to it.
   - `field` and `track` are `Option`s, serialised with `default` and `skip_serializing_if = "Option::is_none"`, as `params` is when empty. An annotation without them serialises exactly as before, and one written before them still reads. Annotations are not part of the Score (above), so no Score's bytes change either.
   - The compiler gathers `rows` only when `analysis.detect` names at least one detector, so a large table carries no row list for a compile that analyzes nothing.
   - `play_rows` sorts stably, so rows with equal time values keep their table order. A row with no time value is left out, as the encoders leave it out (`SNR-ENG` "rows without time" in NUMISONA). The function reads no clock and iterates no hash map.

   *Amended again on 2026-11-26 (#506's review).* `AnalyzedField` was first one per field, listing every track that played it, and its series was the whole column in table order. On DS-003, whose three tracks each filter one region of a month-major table, the detectors then read a series that zig-zagged between regions. So `AnalyzedField` is now one per track, with the rows it plays, and annotations carry their track.

*Amendment note (2026-11-28, #506 review, approved by the coordinator): one play order for every encoder.* Every encoder of data tracks selects and orders its rows through `sonarithm_data::play_rows`: the rows the track's `filter` keeps that have a time value, stably sorted by time. NUMISONA's encoders do, and since this batch CURVASONA's `audify` does too. Before, it ignored a track `filter` and played every timed row (a bug, now fixed; a filter it cannot apply is refused, `SNR-ENG-466`). An `audify` track has no field channel, so `analyzed_fields` gives it the field `audify` plays, `sonarithm_data::default_series` (the field with role `y`, else the first numeric field that is not the time field), and the detectors analyse exactly the series that sounds.

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

- AUDAXIS already makes an insight group per analyzer and an insight node per annotation (`InsightGroupNode`, `InsightNode`, ADR-0017 "Navigation"). The only change is how a node finds its series: by the annotation's `field` (§1 amendment).
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
  - `a11y_23_*` on a two-series table: an outlier in the second series links to that series' node, not the first one's (§1 amendment);
  - Score snapshots for one spec per detector;
  - a test that a spec without ECHOMETRIC detectors compiles to the same Score hash as before;
  - the golden case, with its HP-21 note.

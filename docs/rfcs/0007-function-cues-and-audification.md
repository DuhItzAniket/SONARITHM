# RFC 0007: Function cues and audification in the Score IR

> **Status:** In discussion (proposed for acceptance by the coordinator under the owner's delegation). **Author(s):** project lead (drafted by an AI coding agent, Claude Code).
> **Related:** [RFC process](README.md) · [05 §4](../architecture/05-compiler-and-score-ir.md#4-score-ir) · [06 §3](../architecture/06-audio-engine.md) · [07 CURVASONA](../architecture/07-engines.md#curvasona-sonarithm-curvasona-c03) · [ADR-0013](../decisions/0013-determinism-and-provenance.md) · [ADR-0028](../decisions/0028-function-expression-language.md) · [sound design §8](../design/sound-design.md#8-earcon-design-rules) · [keyboard and navigation §11](../design/keyboard-and-navigation.md#11-earcon-catalogue) · [design rules](../research/design-rules.md)

| Field | Value |
|---|---|
| RFC | 0007 |
| Start date | 2026-10-31 |
| RFC PR | (this PR) |
| Affects | Score IR (four earcon kinds, one event type, the IR version rule); render engine (four earcon sounds, one voice); CURVASONA (a context track); spec v0.1's `analysis.cue` semantics for `critical-points` (no new key) |
| Phase | C03 (tasks C03-T04 and C03-T05) |

## Summary

Adds what C03-T04 (function cues, A11Y-28) and C03-T05 (audification, SDR-31) need from the Score IR, which AGENTS.md rule 11 puts under change control:

1. Four `EarconKind`s, the ones reserved in keyboard and navigation §11: `zero`, `asymptote`, `undefined` and `offscreen`, with render sounds that follow sound design §8.
2. A CURVASONA context track, `curvasona.cues`, that plays the critical points of a function source as earcons (or as speech) where the spec already asks for cues: `analysis.detect` containing `critical-points` and `analysis.cue` set to `earcon` (the default) or `speech`.
3. The `Audify` event of 05 §4: the samples of a series, played at `k` times their own rate by resampling.
4. A rule for the IR version: a Score records the lowest IR version that can read it, so Scores without the additions stay at version 1 and keep their hashes.

## Motivation

A11Y-28 (SHOULD) asks that function plots have "derivative and sign cues, zero and asymptote markers, and undefined and off-screen cues". C03-T03 finds those points and lists them as insight nodes, but a listener who just presses Play hears none of them: no Score event marks them. SDR-31 asks for audification by resampling, with k, the duration T/k and the audible band shown. The Score IR has no event that plays samples. 05 §4 planned both (`Audify`, and earcon kinds for CURVASONA) as additive changes in C03. This RFC fixes their form before code depends on it.

## Guide-level explanation

A spec like 04 §6.3's

```json
{ "specVersion": "0.1",
  "data": { "source": { "kind": "function", "expr": "1/(x^2 - 4)", "domain": [-5, 5] } },
  "tracks": [{ "id": "f", "mark": "glide", "channels": { "pitch": { "field": "y" } } }],
  "analysis": { "detect": ["critical-points"], "cue": "earcon" } }
```

now sounds its features while it plays: a short `E-ASYMPTOTE` as the glide reaches each pole (where the glide breaks, C03-T02), `E-LANDMARK` at the maximum at x = 0, and `E-ZERO` wherever the curve crosses zero. With `"cue": "speech"`, each feature is spoken instead ("vertical asymptote", "local maximum"). With `"cue": "none"`, the cues are not played, and the insight list and navigation tree still carry them.

A track with the `audify` mark plays a series as a waveform:

```json
{ "id": "quake", "mark": "audify", "audify": { "k": 2200 } }
```

The compile reports k, the output duration T/k and the audible band as Info diagnostics (SDR-31), and the Score holds one `Audify` event with the samples.

## Reference-level explanation

### 1. Earcon kinds

`EarconKind` gains `zero`, `asymptote`, `undefined` and `offscreen` (serialized in lowercase, like the existing kinds). The render engine gets four `EarconSound`s. The descriptions below are **proposals** without study evidence (INFER), flagged experimental until the listening check (HP-21). Each follows sound design §8: noise-based or inharmonic (ED-01), at most about 150 ms (ED-02), energy mainly below 4 kHz (ED-06), with SDR-22 ramps.

| ID | Meaning | Proposed sound | Text equivalent |
|---|---|---|---|
| `E-ZERO` | The function crosses or touches zero | A dry inharmonic tick, about 40 ms: partials at 900, 2484 and 4860 Hz (ratios 1 : 2.76 : 5.4, a struck bar), fast decay | "Zero" |
| `E-ASYMPTOTE` | A vertical asymptote: the glide breaks here | Band-passed noise sweeping up from 500 to 2000 Hz over about 120 ms ("whoosh") | "Vertical asymptote" |
| `E-UNDEFINED` | An undefined span starts | Low-passed noise near 200 Hz, about 100 ms, soft attack: a dull hush, the glide is silent after it | "Undefined" |
| `E-OFFSCREEN` | The function leaves the y-window | Two quick high-passed noise taps above about 1.5 kHz, 30 ms apart, about 80 ms in all | "Off screen above" / "below" |

The four text equivalents become catalogue messages in `sonarithm-i18n` (`earcon-zero`, …), like the B03 set.

### 2. The `curvasona.cues` track

CURVASONA registers a context provider. When the spec's `analysis.detect` contains `critical-points` and its source is a function, the provider adds one track, `curvasona.cues` (role `context`), after the data tracks:

| Feature (C03-T03) | `cue: "earcon"` | `cue: "speech"` |
|---|---|---|
| zero | `E-ZERO` | its label |
| vertical asymptote | `E-ASYMPTOTE` | its label |
| undefined span (at its start) | `E-UNDEFINED` | its label |
| off-screen span (at its start), when a window is given | `E-OFFSCREEN` | its label |
| extremum, inflection, hole, jump | `E-LANDMARK` | its label |
| horizontal asymptote | none (it is a limit, not a point on the glide) | none |

The time of a cue is the time of the feature's row in the first data track that plays it: CURVASONA adds a row at each feature (C03-T02, C03-T03), and a glide has one breakpoint per row of its `src` range. Each event's `src` is that row. Earcons whose onsets fall within 150 ms of each other (ED-02) are merged into the first, so a dense function does not turn into a buzz. Speech events use the en-US labels, as AUDAXIS's legend does; the app speaks them in the listener's locale. `cue: "none"` adds no track.

**The sign cues of A11Y-28.** A glide already carries the sign of `f'` in its pitch direction, and `E-ZERO` marks every change of sign of `f`. The 04 §6.3 example maps `brightness` to `dy`, which adds a derivative cue in timbre on top of pitch. This RFC adds **no automatic timbre change**: a default that changes how data sounds needs evidence, and A11Y-28's sources (SonicFunction, AudioFunctions 2.0) are not specific enough to set one (TODO(verify)). Mapping a channel to `dy` stays the way to ask for it, and the sign is then never carried by timbre alone (A11Y-14).

### 3. The `Audify` event

```rust
pub struct Audify {
    pub t: Seconds,          // onset
    pub rate: f64,           // playback samples per second: k × the series' own rate
    pub samples: Vec<f32>,   // the series, normalized to [-1, 1]
    pub gain: Db,
    pub src: Option<RowRange>,
}
```

- **Inline samples.** The Score stays self-contained and its hash still covers everything audible (ADR-0013). 05 §4 sketched a `BufferRef`. A reference would need the render engine to read data, which it never does. The samples are capped at 2^22 (about 4.2 million, 16 MB as f32): a longer series is refused with a diagnostic. The review form shows a count, the extremes and a hash instead of the samples.
- **Duration.** `samples.len() / rate`, which counts for invariant 2 and the header's duration.
- **Rendering.** The plan builder resamples the samples from `rate` to the output rate with rubato (SDR-31: resampling, not pitch shifting) on a worker thread. The audifier voice plays the result from a preallocated `Arc<[f32]>`, with SDR-22 ramps at both ends, and allocates nothing on the audio thread (AGENTS.md rule 2).
- **Normalization.** Each series is scaled by its largest absolute value. Undefined samples are 0.

### 4. The IR version

`IR_VERSION` stays the version a reader understands (2 after this RFC). A Score's `header.ir_version` is the **lowest** version that can read it: 1 when it uses none of the additions, 2 when it has an `Audify` event or a new earcon kind. Existing Scores, snapshots and golden hashes are unchanged. A version 1 reader refuses a version 2 Score rather than misreading it.

## Drawbacks

- Four more earcons to learn (ED-03); the tutorial (A11Y-30) has to teach them.
- Inline samples make audification Scores large.
- A cue track that depends on rows CURVASONA adds is a convention between CURVASONA and NUMISONA's glide encoding (one breakpoint per row), checked by a test rather than by a type.

## Rationale and alternatives

- **Reuse `E-LANDMARK` for everything.** Fewer sounds, but a listener cannot tell a zero from an asymptote without stopping to read, and A11Y-28 names zeros and asymptotes separately.
- **Score markers instead of a track.** `Score.markers` hold labelled times (the cue sidecar exports them), but a context provider returns tracks, not markers, and markers make no sound. A later change could let context providers add markers too, for the sidecar.
- **Bump `ir_version` for every Score.** Simpler to state, but it would change every golden hash and snapshot without any audible change.
- **Automatic sign timbre.** Rejected for now (§2): no evidence for a default.

## Prior art

SonicFunction and AudioFunctions 2.0 (A11Y-28's sources) mark zeros, extrema and asymptotes with distinct sounds. Dombois (2001) and Walker and Nees (2011) describe audification by resampling (SDR-31).

## Accessibility impact

Positive: a listener hears where the features are without leaving playback. Every cue has a text equivalent in the catalogue, in the insight list and as speech with `cue: "speech"` (A11Y-24, ED-05). No information is in pan (ED-06). The new earcons are proposals until HP-21 and the co-design rounds.

## Research impact

The four earcons are candidates for the earcon identification study (sound design §15 step 5). The audification event lets the Dombois gallery item (C03-T06) render from a spec.

## Unresolved questions

For the coordinator, each with a proposal:

| Q | Question | Proposal |
|---|---|---|
| Q1 | IR version per Score (lowest reader) or a global bump? | Per Score (§4) |
| Q2 | Are cues on whenever `critical-points` is detected, or does a function spec need `cue` written? | On, since `cue` defaults to `earcon` (RFC-0001) |
| Q3 | Merge window for dense cues | 150 ms, the ED-02 earcon length (proposal, INFER) |
| Q4 | Inline sample cap | 2^22 samples |
| Q5 | Should the y-window for `E-OFFSCREEN` come from the pitch channel's explicit `scale.domain`? | Yes when one is written; otherwise no off-screen cues. A spec key for a window waits for a later RFC |

## Future possibilities

- Letting context providers add `Score.markers`, so cues reach the WebVTT sidecar.
- A `sign` channel preset once a study supports one.
- `Grains` (TONALENS) and `Excite` (RESONUM) follow the same version rule.

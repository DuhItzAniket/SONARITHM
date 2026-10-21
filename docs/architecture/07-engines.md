# 07 · Engines: Technical Contracts

> **Status:** Baseline v1 (2026-10-18). Product view: [product/engines.md](../product/engines.md). Decision: [ADR-0007](../decisions/0007-engines-as-pipeline-capabilities.md).
> Capability traits are defined in [05-compiler-and-score-ir.md §3](05-compiler-and-score-ir.md#3-engine-registry-and-capability-traits).

Each engine is one crate under `crates/engines/` and implements `Engine`. This page specifies **what each engine registers, which spec keys it owns, its main algorithms and its tests**. Engines never depend on each other ([02 §3](02-repository-and-crate-layout.md#3-dependency-rules-enforced)).

## Summary matrix

| Engine | Source | Analyzer | Encoder | Context | Shaper | Voice | Sink | Interactive | Spec keys owned |
|---|---|---|---|---|---|---|---|---|---|
| NUMISONA | | | `tone`, `glide`, `auto` | | | | | | `tracks[].channels`, `tracks[].redundant`, `concepts` |
| AUDAXIS | | | | ticks, reference, legend, earcons | `Spatial` | earcon set | | navigation cursor | `context`, `navigation` |
| SONOTRACE | `stream` | | incremental encoding hooks | | | monitor soundscape | | transport, scrub | `playback`, `data.source.kind=stream` |
| PLOTSONA | `chart` | | chart templates (via NUMISONA marks) | | | | | correction loop (UI) | `data.source.kind=chart`, `template` |
| CURVASONA | `function` | `critical-points` | `audify` | | | audifier | | | `data.source.kind=function`, `tracks[].audify` |
| AXONOTE | | | | | `Quantize` | instrument library | MIDI file, MIDI live, OSC | | `music`, `instruments`, `output.sinks` |
| ECHOMETRIC | | `extrema`, `outliers`, `changepoints`, `periodicity`, `trend-segments` | | insight markers | `Lint` | | | queries | `analysis`, `describe`, `lint` |
| TONALENS | | `pca` (transform helper) | `grain`, multivariate presets | | | granular | | lens | `lens` |
| RESONUM | | | `model` | | | modal/resonator models | | excitation | `models` |

---

## NUMISONA (`sonarithm-numisona`), A06

**Registers:** Encoder for `tone`, `glide` and `auto`; the concept table; the template registry.

| Module | Responsibility |
|---|---|
| `scale` | Linear, log, symlog, sqrt, pow, psychophysical (SDR-11), quantile, band, ordinal; `domain: auto/zero-based/explicit` |
| `pitch` | Hz ↔ MIDI ↔ cents; mel, ERB-rate and Bark spaces (SDR-09); range resolution with defaults (SDR-08) |
| `polarity` | Concept lookup (SDR-12..14); profile overrides (SDR-15); ambiguity diagnostics |
| `encode` | Data rows → `Note`/`Glide` events with `RowRef`; redundant `duration` and `taps` (A11Y-13); categorical → timbre (SDR-25) |
| `density` | Events-per-second estimate. `auto` mark switching above about 10 events/s (SDR-20, flagged experimental) |
| `templates` | Reusable designs ("sonecules", SDR-03): `line-overview`, `bar-discrete`, `scatter-xy`, `histogram`, `box-five-number`, `heatmap-rowscan`. Spec v0.1 reserves `template`, so templates are spec builders with named slots bound to a dataset's fields; `box-five-number` computes its quartiles itself (no v0.1 aggregate has them) and marks them `derived` |
| `import::erie` | Erie spec subset → SONARITHM spec (A06), broader in C05 |

**Tests:** one named conformance test per owned rule (`sdr_08_default_range`, `sdr_15_polarity_flip_symmetry`, …); property tests on scale monotonicity and range bounds; snapshots of each template on the example datasets.

## AUDAXIS (`sonarithm-audaxis`), B03 (HRTF in D03)

**Registers:** ContextProvider (ticks, reference tone, legends, earcons, insight markers rendered as earcons; and `nav_tree`, B03-T01); Shaper `Spatial`; earcon VoiceProvider; the navigation model.

| Module | Responsibility |
|---|---|
| `nav::tree` | Builds `NavTree` from Score, data and annotations ([08 §2](08-interaction-and-accessibility.md#2-the-navigation-model)). Points are the rows a track reads, ordered as the encoder orders them; without a time field, a `category` or `label` field names them. Segments come from trend-segment annotations or even bins (OD-18) (B03-T01) |
| `nav::cursor` | Cursor state and the verbs local/lateral/spatial/targeted; "where am I". Each verb returns an outcome: moved (with level change, "nearest", the segment entered and landmark flags), boundary (the cursor stays, `E-BOUNDARY`) or invalid with a reason (`E-INVALID`) (B03-T02) |
| `announce` | Composes announcements from templates (verbosity, part order, units, provenance qualifiers); queue with a supersede policy. Each part is a catalogue message and `announce-join` combines them, so every announcement is one message; speech uses spoken unit names and the display text symbols; the queue keeps a history linked to nodes (B03-T03) |
| `context` | Tick track (SDR-16, auto off for glides); dynamic extreme reference tone (SDR-17); spoken legend (SDR-18). B03-T05: tracks `audaxis.ticks` (render instrument `click`), `audaxis.reference` (overlapping notes at the series maximum or minimum, distinct timbre) and `audaxis.legend` (speech in `en-US`; the app speaks it in the listener's locale with `Composer::legend`); codes `SNR-ENG-104` to `107` |
| `earcons` | Earcon catalogue (boundary, invalid, landmark, threshold, mode change, stream event). Each has a text equivalent (A11Y-08) |
| `spatial` | Constant-power stereo; ITD/ILD; binaural via SOFA HRTF (D03); guard so pan is never the sole carrier (A11Y-14). B03-T11: the `Spatial` shaper keeps positions in [−1, 1] and warns when only pan (`SNR-ENG-100`) or only timbre (`SNR-ENG-101`) varies along a data track, or when data pitches leave 65–1480 Hz (`SNR-ENG-102`, SDR-08); binaural and multichannel fall back to stereo with `SNR-ENG-103` |
| `braille` | Review-line text and compact sparkline encoding (A11Y-21) |

**Tests:** NavTree snapshots per chart type; verb-sequence property tests (no dead ends, every node reachable, "where am I" always defined); announcement snapshot tests (terse and verbose, several locales from B07); scripted screen-reader task suites (B03 exit).

## SONOTRACE (`sonarithm-sonotrace`), B06

**Registers:** Source `stream` (adapters live in `sonarithm-stream`); incremental-encoding orchestration; monitor soundscape voice; transport helpers.

| Module | Responsibility |
|---|---|
| `live` | Subscribes to a `StreamTable`; drives `encode_incremental` of the track's encoder; sliding-window shapers |
| `monitor` | Ambient soundscape with salience modulation on detector events (SDR-35); alarm tier only for `salience: high` |
| `rate_limit` | Announcement rate limits; coalescing; pause/stop/hide of streams and announcements (A11Y-16) |
| `transport` | Loop, speed, scrub-grain generation; long-series windowing |

**Tests:** a deterministic stream replay (recorded input → identical event sequence); a 1-hour soak with a 1 kHz synthetic source (B06 exit); rate-limit property tests.

**As built (B06-T03).** `sonarithm-sonotrace` registers the `stream` source: a spec's `{"kind": "stream", …}` reads the window of the `StreamTable` the host connected (`Sonotrace::with_window`), and fails with `SNR-ENG-200` when none is. `live::Live` follows one track: each `advance` encodes the stream's current window with the track's encoder from the registry, so scales and shapers see the sliding window, and keeps only the events of rows past its cursor, mapped to sequence numbers through each event's source row; rows the stream dropped first are reported as skipped. Every row's event is emitted once, and a recorded input replays to the same event sequence (`sdr_01_recorded_input_replays_to_an_identical_event_sequence`). **Deviation from 05 §6:** there is no `encode_incremental` on the core `Encoder` trait; adding one is a cross-engine contract change that needs an RFC, and the PB-11 soak (B06-T07) decides whether window encoding is fast enough. `transport::scrub_grain` gives a track's preview at the cursor: an 80 ms note (provisional, the low end of SDR-32's grain range; TODO(verify) with HP-11) that starts at the cursor with the local pitch, level, pan and brightness, interpolated along glides. `transport::window` and `windows` cut a long series into sections that start at 0.
**Stream adapters as built (B06-T02, `sonarithm-stream`).** `connect(consent, options)` is the only way to open a stream, and a `Consent` exists only through `Consent::user_approved(endpoint)`, which hosts call from the user's own confirmation after showing `Endpoint::describe()` (ADR-0012, A11Y-16); `Endpoint::from_parts` turns a spec's stream source into an endpoint without opening anything. Each connection reads on its own thread with synchronous clients, so WebSocket (`ws://` only; TLS is a follow-up), OSC over UDP (listening on every interface, as the prompt says), serial (one message per line) and file tail need no async runtime; MQTT (behind the `mqtt` feature) uses rumqttc. Messages become rows through a `MessageSchema`: a spec's `schema` object is read as a JSON Schema subset (`properties` with `number`, `integer`, `string` or `boolean`, and `required`), and without one the first valid message decides the fields. Rows are appended to a `StreamTable` every 256 rows or 50 ms, so memory stays bounded by its capacity; messages that do not fit the schema are counted and dropped; the thread reconnects with exponential backoff (0.25 s doubling to 30 s); `pause` keeps the connection and discards messages, `stop` ends the thread within 100 ms. The JSON, OSC and line decoders are fuzzed nightly.

## PLOTSONA (`sonarithm-plotsona` + optional `sonarithm-plotsona-ml`), C01/C02 (ML in D04)

**Registers:** Source `chart` with extractor tiers; chart-type templates (expressed as NUMISONA specs).

| Module | Responsibility |
|---|---|
| `detect` | Container sniffing: XLSX/PPTX/DOCX chart parts, SVG, HTML with Vega/Plotly/Highcharts JSON, PDF, raster |
| `office` | `c:numCache` / `c:numLit` / embedded workbook extraction → `exact` |
| `svg` | Mark geometry + axis tick labels → calibrated values → `vector` |
| `pdf_vector` | Drawing operators → paths → axis detection → calibration (ChartDetective approach) → `vector` |
| `raster` | Axis calibration (linear/log/date), colour masks, line tracing, bar and point detection → `cv_estimate` + confidence |
| `ml` (crate `-ml`) | tract/ort models; self-consistency confidence; `ml_estimate` |
| `templates` | `chart_type → spec` (line, bar, grouped/stacked, scatter, histogram, box, heat map, pie) |

**Tests:** extraction corpora with ground truth (exact ≤ 0.5% error; raster median ≤ 2%, C02 exit); fuzzing of every parser (untrusted input); provenance-class assertions.

## CURVASONA (`sonarithm-curvasona`), C03

**Registers:** Source `function`; Analyzer `critical-points`; Encoder `audify`; audifier VoiceProvider.

| Module | Responsibility |
|---|---|
| `expr` | Facade over exmex: parse, evaluate, symbolic derivative; a sandboxed variable set. **Never evalexpr (AGPL)** |
| `sample` | Adaptive domain sampling (denser near high curvature and discontinuities); undefined-region detection |
| `critical` | Zeros, extrema, inflections, asymptotes, discontinuities, off-screen spans → annotations (A11Y-28) |
| `cues` | Derivative-sign timbre, zero-crossing earcons, undefined/off-screen cues |
| `audify` | Speed-up calculator: k from the data band to a target band, output duration T/k, audible band (SDR-31); presets (seismic k≈2200, EEG) |

**Tests:** a 40-curve suite with analytically known critical points; audification round-trip (sine in → expected tone out); expression fuzzing.

## AXONOTE (`sonarithm-axonote`), C04

**Registers:** Shaper `Quantize` (presentation only); VoiceProvider for the instrument library; Sinks for MIDI file, live MIDI and OSC.

| Module | Responsibility |
|---|---|
| `scales` | Scale and mode tables; tuning systems (12-TET, just intonation, custom cents); snapping with a reported resolution loss |
| `instruments` | Synth presets (mallet, clarinet-like, sine, square-BLV per SDR-29, soft pad); SF2 via rustysynth |
| `midi` | SMF export (midly); live out (midir); glide → pitch-bend or note policy |
| `osc` | OSC out (rosc) with an address scheme `/sonarithm/<track>/<param>` |
| `guard` | Refuses to run in `analysis` mode (SDR-10), with an explicit diagnostic |

**Tests:** quantization never runs in analysis mode (invariant); MIDI round-trip (export → parse → compare); OSC message snapshot.

## ECHOMETRIC (`sonarithm-echometric`), harness A08, analytics C05

**Registers:** Analyzers (extrema, outliers, change points, periodicity, trend segments); Shaper `Lint`; insight markers; the query engine; the description generator; the evaluation harness (library + `sonarithm study` CLI).

| Module | Responsibility |
|---|---|
| `detect` | Detectors (statrs, augurs); parameters recorded in annotations |
| `describe` | Deterministic L1 (encoding facts) and L2 (statistics) text; the L3 fact list for optional LLM phrasing with numeric verification (D04); never automatic L4 (A11Y-02) |
| `query` | min, max, mean, median, mode, sd, sum, range, value-at-x (A11Y-22) |
| `lint` | Rules: SDR-14 (flagged concept), SDR-17 (static reference), SDR-19 (rate), SDR-23 (loudness carrying data), SDR-26 (stream count), SDR-27 (redundant spatial), SDR-28 (critical-band clash), SDR-39 (comfort), A11Y-14 (pan-only information) |
| `harness` | Stimulus generators (Nees & Walker reversals × density; Smith & Walker price series; SNR transits; correlation triplets; `harness::generators`, A08); the replication gallery; the trial runner; the log schema. A generator returns a `Stimulus`: a numeric table, its ground truth, notes on departures from the published design, and a manifest with the table's SHA-256 |

**Study runner (A08).** `harness::study` is the runner as a library; the `sonarithm study check` and `sonarithm study run` subcommands are its front end. Every prompt of `study run` is one plain line, every answer a typed line sent with Enter (no raw keyboard mode or redraws), so screen readers read the dialogue in order; the question comes before the sound, as in the published studies. The pilot protocol is `examples/studies/pilot-001`. The library does no I/O and reads no clock: the front end passes the monotonic time of each action, renders and plays each trial's stimulus, and appends each returned record to the log at once, so the log is complete up to the last action.

- **Protocol** (`sonarithm-study/0`, TOML): `exp-id`, `protocol-version`, `seed`, instructions, a task (`point-estimation`, `trend-identification`, `count` or `choice`) with its response format, conditions (a spec each, relative to the protocol file, data-agnostic: the trial's stimulus replaces its `data`), and blocks (conditions, `repeats`, `order`, practice and feedback flags, generator parameters). Problems are diagnostics `SNR-ENG-600`–`607`.
- **Schedule:** each block draws from a stream keyed by the protocol seed, the session's order seed and the block ID; first the shuffle, then one stimulus seed per trial. The order seed is logged, so a session can be rebuilt exactly.
- **Log** (`sonarithm-study-log/0`, one JSON object per line): every record has `event`, `t_mono_ms` (monotonic, since the start of the run), `run`, `exp_id`, `session_id` and `participant_code`. `session_start` adds `log_schema`, `protocol_version`, `protocol_sha256`, `engine_version`, `order_seed`, `trials` and any audio context the researcher entered; trial records add `trial`, `block`, `condition` and `practice`; `trial_start` adds the stimulus provenance (`stimulus_id`, `generator_version`, `seed`, `stimulus_sha256`, `spec_hash`, `data_fingerprint`, `manifest_ref`); `response` adds `response`, `correct`, `true_value`, `true_answer`, `error`, `rt_ms` and `plays`. Events: `session_start`, `trial_start`, `play`, `replay`, `pause`, `resume`, `trial_abort`, `response`, `trial_end`, `note`, `session_end`. This is the normative form of the evaluation plan's proposed schema.
- **Resume:** trials with a `trial_end` are done; a run that stopped mid-trial repeats that trial. A changed protocol file (another SHA-256) cannot resume a session (`SNR-ENG-621`).
- **Privacy (ADR-0012):** session IDs and participant codes must be short codes, never names; specs are named relative to the protocol; audio context is recorded only as entered.
- **Research use (OD-04):** no participant data is collected as research until an ethics route exists; the pilot runs with scripted responses.

**Tests:** detector precision/recall on labelled synthetic sets; positive and negative tests for each lint rule; generators are deterministic given a seed; scripted study sessions with a pinned log.

## TONALENS (`sonarithm-tonalens`), D01

**Registers:** Encoder `grain` and multivariate presets; granular VoiceProvider; an Interactive lens.

| Module | Responsibility |
|---|---|
| `lens` | Lens geometry (point, radius, rectangle) over 2-D views; hit-testing against spatial indexes (k-d tree); sweep along x, y or a free path |
| `multistream` | Timbre + pan + staggered-onset separation; enforces a 3–4 stream cap with a lint override (SDR-26, H-04) |
| `quicklook` | PCA → harmonic partial amplitudes (SDR-34) |
| `granular` | Density-weighted grains, 80–300 ms (SDR-32) |
| `link` | Selection linking across visual, audio and text (A11Y-11) |

## RESONUM (`sonarithm-resonum`), D02

**Registers:** Encoder `model`; modal/resonator VoiceProviders; an Interactive excitation handler.

| Module | Responsibility |
|---|---|
| `models::sonogram` | Data-sonogram: a spherical wavefront from the excited point; each data point "rings" when the front reaches it (distance → onset; local density → timbre) |
| `models::crystal` | Crystallization growth from a seed through k-NN (clustering and intrinsic dimension cues) |
| `models::modal` | Each datum is a resonator (frequency from value, damping from uncertainty or weight) |
| `excite` | Keyboard, pointer and gamepad excitation; silence without interaction (SDR-33) |

**Status:** research-grade until a pilot study (D06) supports its defaults. Graph and network use needs H-05 polarity data first.

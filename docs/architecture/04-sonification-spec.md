# 04 · Sonification Spec Language (v0.3)

> **Status:** **Frozen as v0.1** by [RFC-0001](../rfcs/0001-spec-v0.1.md), accepted 2026-10-19 ([ADR-0015](../decisions/0015-spec-v0.1-frozen.md)). The RFC is normative: it holds the exact key set, defaults, validation codes and the 21 resolutions, and it wins wherever this overview is less precise. Changes follow the versioning rules in §7. **Spec v0.2** ([RFC-0004](../rfcs/0004-spec-v0.2.md), [ADR-0021](../decisions/0021-spec-v0.2.md), accepted 2026-10-26) adds `template`, `instruments`, `lint`, a typed chart calibration, chart corrections and AXONOTE's scale and tuning names. It is implemented (C01-T01), and this document describes v0.2; RFC-0004 is normative for what it adds.
> **Decision:** [ADR-0005 Declarative spec and Score IR](../decisions/0005-declarative-spec-and-score-ir.md). **Owner:** Spec lead.
> **Evidence:** [SDR-01..06](../research/design-rules.md#a-specification-and-reproducibility).

## 1. Design goals

| Goal | Why | Source |
|---|---|---|
| **Declarative and complete.** Data plus spec fully determine the Score. | Reproducibility | SDR-01, SDR-02 |
| **Shared vocabulary with visualization** (substrate, mark, channel) | Interop with visual grammars; the same mental model as Vega-Lite and Erie | SDR-04, Enge et al. 2023 |
| **Erie-compatible concepts** | Import the only existing declarative sonification grammar | SDR-05, Kim et al. 2024 |
| **Data-agnostic templates** | Reusable designs ("sonecules") | SDR-03 |
| **Mode is explicit** | Separates the analytic goal from the aesthetic goal | SDR-36 |
| **Evidence-backed defaults; everything overridable** | Defaults from the design rules; user ownership | P7 |
| **Human-writable JSON**, with a schema for editors | Authoring in the CodeMirror editor, CLI, notebooks | — |

The canonical serialization is **JSON**. Builders exist in Rust and Python and produce the same JSON. The JSON Schema is generated from the Rust types (schemars) into `schemas/sonarithm-spec-0.3.schema.json`, and CI fails if it is stale. The v0.1 and v0.2 schemas stay in `schemas/` for editors of older files.

## 2. Top-level structure

```jsonc
{
  "specVersion": "0.2",          // required; see §7
  "name": "Global temperature anomaly",
  "description": "Optional free text, spoken as the title",
  "mode": "analysis",            // analysis | presentation | monitoring   (SDR-36)
  "seed": 1729,                  // all randomness derives from this     (SDR-02)
  "data": { },                   // §3
  "transform": [ ],              // §4
  "substrate": { },              // §5.1  time (and optional frequency) substrate
  "tracks": [ ],                 // §5.2  marks + channels (optional when `template` gives them)
  "context": { },                // §5.4  AUDAXIS: ticks, reference tones, legends, earcons
  "analysis": { },               // §5.5  ECHOMETRIC: detectors, cues, descriptions
  "music": { },                  // §5.6  AXONOTE: presentation-mode quantization, instruments
  "playback": { },               // §5.7  SONOTRACE: speed, loop, monitoring
  "output": { },                 // §5.8  sample rate, channels, loudness target, sinks
  "template": { },               // §5.9  v0.2: a reusable design bound to the data
  "instruments": { },            // §5.10 v0.2: named instruments (SoundFonts)
  "lint": { }                    // §5.11 v0.2: lint rule levels and suppressions
}
```

Unknown top-level keys are an **error**. The exception is the `x-` vendor prefix (for example `x-myplugin`), which is preserved and ignored.

## 3. `data`

```jsonc
"data": {
  "source": { "kind": "file", "path": "data/gistemp.csv", "format": "csv" },
  "fields": {
    "year":    { "type": "temporal",     "unit": "year" },
    "anomaly": { "type": "quantitative", "unit": "°C", "concept": "temperature", "label": "Temperature anomaly" }
  },
  "missing": "silence"            // silence | earcon | interpolate
}
```

| `source.kind` | Engine | Parameters |
|---|---|---|
| `file` | platform | `path`, `format` (csv, tsv, json, ndjson, parquet, arrow, xlsx, ods), `sheet`, `options` |
| `inline` | platform | `values` (array of records). For examples and tests |
| `chart` | PLOTSONA | `path`, `extractor` (`auto`, `office`, `svg`, `vega`, `plotly`, `pdf-vector`, `raster`, `ml`), `calibration` (§3.1), `corrections` (§3.1, v0.2; a file in v0.3) |
| `function` | CURVASONA | `expr`, `domain`, `samples` or `step`, `variables` |
| `stream` | SONOTRACE | `transport` (`websocket`, `mqtt`, `serial`, `osc`, `tail`), `url`/`port`/`topic`, `schema`, `window` |
| `dataset` | platform | A reference to a dataset the host already holds: a dataset loaded in a project (desktop/app core), or data handed to the facade or the Python package (A07) |

`fields` overrides inference ([03-data-model.md §3](03-data-model.md#3-field-semantics)). Any field that is not mentioned keeps its inferred metadata.

### 3.1 Chart calibration and corrections (v0.2)

A chart source can carry the axis calibration a person confirmed and the corrections they made, so the same spec and chart file always give the same table (SDR-01, [RFC-0004 §5–§6](../rfcs/0004-spec-v0.2.md)):

```jsonc
"data": {
  "source": {
    "kind": "chart", "path": "figure3.png", "extractor": "raster",
    "calibration": {
      "system": "cartesian",                 // default; polar and ternary later
      "axes": {                              // anchors: a point in the source's coordinates and its value
        "x": { "kind": "linear", "anchors": [ { "at": [112, 940], "value": 0 }, { "at": [1210, 940], "value": 50 } ] },
        "y": { "kind": "log",    "anchors": [ { "at": [112, 940], "value": 1 }, { "at": [112, 120],  "value": 1000 } ] }
      }
    },
    "corrections": { "edits": [               // in data units; `was` guards the point the person saw
      { "op": "move",   "series": "Series 1", "point": 12, "was": { "x": 24.1, "y": 210.4 }, "to": { "y": 230 } },
      { "op": "delete", "series": "Series 1", "point": 30, "was": { "x": 48.7, "y": 3.2 } }
    ] }
  }
}
```

- An axis `kind` is `linear`, `log` (values above 0), `date` (ISO 8601 strings) or `category` (labels), with at least two anchors at distinct points and values; more than two are fitted by least squares and the residual is reported. `page` (PDF, default 1) and `plotArea` (`{ "min": [x, y], "max": [x, y] }`) are optional. An invalid calibration is `SNR-SPEC-054`.
- Without `calibration`, the extractor calibrates on its own and reports the calibration it used in this shape.
- Edits are `add`, `move`, `delete` and `retype`, applied in order after extraction and before `transform`; edited rows become `user_edited` (03 §4). An edit whose `was` no longer matches is not applied and is reported (`SNR-ENG-386`). Shape errors are `SNR-SPEC-055`.
- **v0.3:** `corrections` may instead be `{ "path": "<relative path>" }`, a correction file ([RFC-0005](../rfcs/0005-chart-correction-file.md)) resolved like the source's own `path`. A path that is absolute, uses `..`, a drive letter or a URL is `SNR-SPEC-093`; in a document that declares `"0.2"` or older, `path` is `SNR-SPEC-056`. A file made on another chart applies nothing (`SNR-ENG-387`).

## 4. `transform`

An ordered list. Each item has an `op` plus parameters ([03-data-model.md §6](03-data-model.md#6-transforms)).

```jsonc
"transform": [
  { "op": "filter", "expr": "year >= 1880" },
  { "op": "smooth", "field": "anomaly", "method": "ema", "alpha": 0.3, "as": "anomaly_smooth" },
  { "op": "aggregate", "groupby": ["decade"], "ops": [{ "op": "mean", "field": "anomaly", "as": "mean_anomaly" }] }
]
```

## 5. Sonification

### 5.1 `substrate`

The temporal canvas onto which marks are placed (Enge et al.). Exactly one **pace** is required:

```jsonc
"substrate": {
  "time": {
    "field": "year",                         // ordering field; defaults to role=x or row order
    "pace": { "pointsPerSecond": 3 }         // or { "duration": { "s": 20 } } or { "secondsPerUnit": 0.1 }
  }
}
```

The default pace is **3 points/s** ([SDR-19](../research/design-rules.md#e-timing-and-density)). Paces above 8 points/s produce lint `SDR-19`. Above about 10 events/s, `mark: "auto"` switches to `glide` ([SDR-20](../research/design-rules.md#e-timing-and-density), experimental).

### 5.2 `tracks`

Each track is one stream of auditory marks. Tracks overlay in time by default.

```jsonc
"tracks": [
  {
    "id": "anomaly",
    "engine": "numisona",             // encoder engine; default numisona
    "mark": "tone",                   // see §5.3
    "instrument": "mallet",           // AXONOTE preset or platform instrument
    "channels": {
      "pitch": {
        "field": "anomaly",
        "scale": { "type": "linear", "domain": "auto", "nice": false },
        "range": { "hz": [400, 1400] },          // SDR-08
        "space": "log-frequency",                // SDR-09: log-frequency | linear-frequency | mel | erb | bark
        "polarity": "auto"                       // auto (from concept) | positive | negative  (SDR-12..15)
      },
      "duration": { "value": { "s": 0.25 } },
      "loudness": { "value": { "db": -18 } },    // constant by default (SDR-23)
      "pan":      { "value": 0 },
      "timbre":   { "field": "region" }          // nominal → timbre (SDR-25)
    },
    "redundant": ["taps"],                       // A11Y-13: optional redundant encodings
    "filter": { "expr": "region != 'Antarctica'" }
  }
]
```

### 5.3 Marks and channels

| Mark | Technique label (SDR-06) | Meaning | Valid channels | Engine |
|---|---|---|---|---|
| `tone` | PMSon | One discrete note per datum | pitch, duration, loudness, pan, timbre, brightness, taps | NUMISONA |
| `glide` | PMSon | A continuous contour through the data | pitch, loudness, pan, timbre, brightness, rate | NUMISONA |
| `grain` | PMSon (granular) | A grain cloud whose density follows the data (SDR-32) | density, pitch, duration, pan, brightness | TONALENS |
| `audify` | Audification | The data series played as a waveform (SDR-31) | loudness only; speed is set by `tracks[].audify` (`k` or `band`) | CURVASONA |
| `sample` | Auditory icon | A recorded sound per event | loudness, pan, rate | platform |
| `model` | MBS | An excitable data model (SDR-33). Stage D | model-specific | RESONUM |
| `auto` | — | Chooses tone, glide or audify from density and size | — | NUMISONA |

| Channel | Units accepted | Default | Notes |
|---|---|---|---|
| `pitch` | `hz`, `midi`, `cents` (relative to A4 = 440 Hz) | 400–1400 Hz, log-frequency | SDR-07..11 |
| `duration` | `s`, `ms` | 0.25 s | Redundant precise channel (A11Y-13) |
| `taps` | count (1–8) | 1 | Tap-count redundancy (Wang et al. 2022) |
| `loudness` | `db` (relative) | −18 dB, constant | Not for precise values (SDR-23) |
| `pan` | −1…1 | 0 | Never the sole carrier (A11Y-14) |
| `timbre` | preset name, or a field (nominal) | instrument default | Categorical (SDR-25) |
| `brightness` | 0…1 (filter cutoff and spectral tilt) | 0.5 | Secondary quantitative |
| `rate` | `hz` (tremolo/AM) or events/s | — | Glide modulation |
| `density` | grains/s | — | Grain mark only |

Every channel takes **either** `field` (+ `scale` + `range`) **or** `value` (a constant), never both.

**Scale types:** `linear`, `log`, `symlog`, `sqrt`, `pow` (`exponent`), `psychophysical` (`exponent` m and a reference; SDR-11), `quantile`, `band`, `ordinal`. `domain` is `"auto"` (data extent), `[min, max]`, or `"zero-based"`.

### 5.4 `context` (AUDAXIS)

```jsonc
"context": {
  "ticks":     { "axis": "x", "every": 10, "minor": false, "enabled": "auto" },   // SDR-16; auto = on for tone, off for glide
  "reference": { "kind": "dynamic-extreme", "track": "anomaly", "timbre": "triangle" },  // SDR-17
  "legend":    { "speak": "before-play", "parts": ["title", "x-range", "y-range", "units", "polarity"] }, // SDR-18
  "earcons":   { "boundary": true, "landmarks": true, "thresholds": [{ "track": "anomaly", "value": 0 }] },
  "spatial":   { "mode": "stereo" }      // stereo | binaural (D03) | multichannel (D03)
}
```

`reference.kind`: `none`, `dynamic-extreme` (the default when context is on), `static-value` (allowed, but lint warns per SDR-17), or `zero-line`.

### 5.5 `analysis` (ECHOMETRIC)

```jsonc
"analysis": {
  "detect": ["extrema", "outliers", "changepoints"],     // also: periodicity, trend-segments, critical-points (CURVASONA)
  "cue": "earcon",                                       // earcon | speech | none
  "describe": { "levels": ["L1", "L2"] }                 // L3 phrasing is opt-in; L4 never automatic (A11Y-02)
}
```

### 5.6 `music` (AXONOTE)

Allowed only when `mode` is `presentation` (SDR-10). In `analysis` mode it is a validation error.

```jsonc
"music": { "quantize": { "scale": "C major pentatonic", "tuning": "12-TET" }, "tempo": { "bpm": 96 }, "voicing": "none" }
```

`scale` is a name, `"<tonic> <mode>"`: a tonic `A`–`G` with an optional `#` or `b`, and one of the modes `major`, `minor`, `harmonic minor`, `melodic minor`, `dorian`, `phrygian`, `lydian`, `mixolydian`, `locrian`, `major pentatonic`, `minor pentatonic`, `blues`, `whole tone`, `chromatic` (degrees in [RFC-0004 §7](../rfcs/0004-spec-v0.2.md)). Or it is custom: `{ "tonic": "D", "cents": [0, 150, 350, 500, 700, 850, 1050] }`, degrees in cents from 0, rising, below 1200. `tuning` applies to named scales: `12-TET` (default) or `{ "cents": [c0, …, c11] }`; `just` is reserved until AXONOTE cites its ratio table (RFC-0004 Q13). An unknown scale is `SNR-SPEC-043`; an invalid tuning, or a tuning on a custom scale, `SNR-SPEC-044`. Pitches snap to the nearest degree, ties to the lower one.

### 5.7 `playback` (SONOTRACE)

```jsonc
"playback": { "speed": 1.0, "loop": false, "monitor": { "soundscape": "forest", "alarms": [{ "detector": "outliers", "salience": "high" }] } }
```

### 5.8 `output`

```jsonc
"output": {
  "sampleRate": 48000, "channels": 2, "blockSize": 256,
  "loudness": { "targetLufs": -16, "truePeakDbtp": -1 },     // proposed export defaults (see design/sound-design.md)
  "sinks": [{ "kind": "midi-file", "path": "out.mid" }, { "kind": "osc", "address": "127.0.0.1:57120" }]
}
```

### 5.9 `template` (v0.2)

A template is a reusable design bound to the dataset's fields (SDR-03): NUMISONA's six (`line-overview`, `bar-discrete`, `scatter-xy`, `histogram`, `box-five-number`, `heatmap-rowscan`) and PLOTSONA's chart-type templates (`chart-line`, `chart-bar`, and the experimental `chart-stacked-bar` and `chart-pie`; [07 PLOTSONA](07-engines.md#plotsona-sonarithm-plotsona--optional-sonarithm-plotsona-ml-c01c02-ml-in-d04)). With `template`, `tracks` is optional.

```jsonc
"template": { "id": "bar-discrete", "bind": { "category": "quarter", "value": "revenue" } }   // unnamed slots bind by role, then by kind
```

A section written in the spec replaces the template's section of the same name, as a whole; the spec's `transform` runs before the template's. The compiler reports the binding (`SNR-COMP-033`) and which sections came from the template (`SNR-COMP-034`) ([RFC-0004 §2](../rfcs/0004-spec-v0.2.md)). An unknown template is `SNR-COMP-030`; a binding to an unknown slot or field, or a field of the wrong kind, is `SNR-COMP-031`; a required slot no field fits is `SNR-COMP-032`. A template that computes values from the data (`box-five-number`'s quartiles) replaces the data read with those `derived` rows.

### 5.10 `instruments` (v0.2)

```jsonc
"instruments": { "piano": { "kind": "soundfont", "path": "sounds/piano.sf2", "bank": 0, "program": 0 } }   // a track's "instrument": "piano" plays it
```

`path` is relative to the spec; `bank` is 0 … 16383 and `program` 0 … 127 (both default 0). A defined name that equals a built-in preset hides it, with an `Info`. Instruments are allowed in every mode; the default instrument stays mallet (SDR-29).

### 5.11 `lint` (v0.2)

```jsonc
"lint": {
  "rules": { "SDR-19": "error" },                                      // off | info | warning | error
  "suppress": [ { "rule": "SDR-17", "reason": "The 0 °C line is the reference the class is learning." } ],
  "experimental": false                                                // INFER rules (SDR-26, SDR-28, SDR-39) when true
}
```

Rules are named by design-rule ID; one the lint does not implement is a warning (`SNR-SPEC-045`). A suppression needs a reason (`SNR-SPEC-046`) and may name a `track`. Turning a rule off is always reported. Validation errors (such as `music` outside Presentation, SDR-10) and hearing safety are not lint and cannot be turned off.

## 6. Complete examples

### 6.1 Minimal (inferred defaults)

```json
{ "specVersion": "0.1", "data": { "source": { "kind": "file", "path": "temps.csv" } },
  "tracks": [{ "id": "y", "mark": "tone", "channels": { "pitch": { "field": "anomaly" } } }] }
```

The compiler fills in the substrate from `role=x` (or row order), pace 3 points/s, range 400–1400 Hz, polarity from the concept, and context `auto`. Every filled default is reported as an `Info` diagnostic with its rule ID.

### 6.2 Two series, separated (SDR-25/26)

```json
{
  "specVersion": "0.1", "mode": "analysis", "seed": 7,
  "data": { "source": { "kind": "file", "path": "co2_temp.csv" } },
  "substrate": { "time": { "field": "year", "pace": { "duration": { "s": 24 } } } },
  "tracks": [
    { "id": "co2",  "mark": "tone", "instrument": "mallet",
      "channels": { "pitch": { "field": "co2_ppm" }, "pan": { "value": -0.5 } } },
    { "id": "temp", "mark": "tone", "instrument": "clarinet-like",
      "channels": { "pitch": { "field": "anomaly", "polarity": "positive" }, "pan": { "value": 0.5 } },
      "redundant": ["taps"] }
  ],
  "context": { "ticks": { "axis": "x", "every": 10 }, "reference": { "kind": "dynamic-extreme", "track": "temp" } }
}
```

### 6.3 Function with critical-point cues (CURVASONA)

```json
{
  "specVersion": "0.1",
  "data": { "source": { "kind": "function", "expr": "sin(x) / x", "domain": [-20, 20], "samples": 2000 } },
  "substrate": { "time": { "pace": { "duration": { "s": 8 } } } },
  "tracks": [{ "id": "f", "mark": "glide", "channels": { "pitch": { "field": "y" }, "brightness": { "field": "dy" } } }],
  "analysis": { "detect": ["critical-points"], "cue": "earcon" }
}
```

### 6.4 Live monitoring (SONOTRACE)

```json
{
  "specVersion": "0.1", "mode": "monitoring",
  "data": { "source": { "kind": "stream", "transport": "websocket", "url": "ws://127.0.0.1:9000/cpu", "window": { "s": 300 } } },
  "tracks": [{ "id": "cpu", "mark": "glide", "channels": { "pitch": { "field": "load", "range": { "hz": [300, 900] } } } }],
  "analysis": { "detect": ["outliers"], "cue": "earcon" },
  "playback": { "monitor": { "soundscape": "forest" } }
}
```

### 6.5 From a template (v0.2)

```json
{ "specVersion": "0.2",
  "data": { "source": { "kind": "file", "path": "sales.csv" } },
  "template": { "id": "bar-discrete", "bind": { "category": "quarter", "value": "revenue" } },
  "lint": { "suppress": [ { "rule": "SDR-17", "reason": "Quarterly totals have no reference level." } ] } }
```

## 7. Versioning and compatibility

| Rule | Detail |
|---|---|
| Version format | `specVersion: "MAJOR.MINOR"`, independent of crate versions |
| Minor bump | Adds optional fields or enum values. Older readers reject unknown fields, so a document declares the minimum version it needs |
| Major bump | Renames, removes or changes semantics. Needs an RFC and a migration |
| RFC scope | An RFC is required for a major bump, a new top-level section, a new substrate, mark or channel kind, or any reinterpretation of existing fields. Adding an optional field or enum value inside an existing construct needs only a normal PR with schema regeneration |
| Migrations | `sonarithm spec migrate` rewrites documents explicitly. The compiler may read old versions through in-memory migration, with a `Warning` diagnostic. It never silently rewrites files |
| v0.1 and v0.2 | The 0.1 → 0.2 migration changes `specVersion` only, so a v0.2 reader reads `"0.1"` documents **as they are**: no migration, no warning, and their canonical form and spec hash do not change ([RFC-0004 §9](../rfcs/0004-spec-v0.2.md)). A v0.2 key in a `"0.1"` document is unknown (`SNR-SPEC-004`). v0.2 defaults (instrument bank and program, lint settings, the `12-TET` tuning, the calibration's `system` and `page`) fill only v0.2 documents |
| v0.3 | Adds only `corrections.path` ([RFC-0005](../rfcs/0005-chart-correction-file.md), [ADR-0026](../decisions/0026-chart-correction-file.md)). The 0.2 → 0.3 migration changes `specVersion` only, so `"0.1"` and `"0.2"` documents are read as they are; `sonarithm spec migrate` writes `"0.3"` |
| Pre-1.0 | Until SONARITHM 1.0, `0.x` minor versions may break, with a migration provided |
| Canonical form | For hashing, specs are canonicalized per RFC 8785 (JSON Canonicalization Scheme) after the data-independent defaults are applied. The **spec hash** in manifests is SHA-256 of that form ([ADR-0013](../decisions/0013-determinism-and-provenance.md)). Implemented by `sonarithm_spec::prepare` (A03-T08); every applied default is reported as `SNR-SPEC-080` (Info) with its rule ID |

## 8. Validation layers

| Layer | Implemented by | Output |
|---|---|---|
| 1. Syntax | serde | `Error` with JSON pointer and span |
| 2. Schema | Generated JSON Schema (editors) and the serde types (runtime) | `Error` |
| 3. Semantics | `sonarithm-spec` validators: fields exist; channel and field types are compatible; ranges are valid; `music` only in presentation mode; polarity for flagged concepts | `Error` or `Warning` with code `SNR-SPEC-nnn` |
| 4. Perceptual lint | ECHOMETRIC on the compiled Score: rate, stream count, critical-band spacing, loudness on precise data, pan as sole carrier | `Warning`/`Info` tagged with SDR/A11Y IDs |

## 9. Importers

| Format | Phase | Scope |
|---|---|---|
| Erie spec | A06 (subset), C05 (broader) | Tone definitions, encodings (time, duration, tapSpeed, pitch, detune, pan, loudness, timbre), sequence and overlay |
| Vega-Lite | C01 | Data plus x/y/color encodings → fields, roles and a default spec |
| MAIDR JSON | C01 | Data and chart type → default spec |
| Apple `AXChartDescriptor` semantics | Reference only | Its axis model informs the default mapping (A11Y-13) |

Importers emit a report of every construct they could not map. They never drop one silently. The A06 Erie importer is `sonarithm_numisona::import::erie`; what spec v0.1 cannot express is listed in [F-081](../research-paper/findings-log.md#f-081--spec-v01-can-express-8-of-eries-15-documented-encoding-channels-and-neither-sequential-composition-nor-intervals).

The Vega-Lite and MAIDR importers (C01-T10) are `sonarithm_plotsona::import::{vegalite, maidr}`. Each writes one v0.2 spec: the chart's data as an `inline` or `file` source (a URL on the network is reported and never fetched, ADR-0012), field types and labels in `data.fields`, supported transforms (Vega-Lite field-predicate filters, the six aggregates, bins), and the chart-type template its mark or layer type calls for, bound to the chart's fields (x → time, y → pitch, colour → timbre; §5.9). Unmapped constructs are `SNR-ENG-381` warnings and approximated ones `SNR-ENG-382` notes, located in the imported document. Vega-Lite's expression strings are JavaScript and are reported, never run (OD-25). MAIDR's schema states no version (TODO(verify), [F-166](../research-paper/findings-log.md#f-166--maidrs-data-schema-states-no-version-and-leaves-stacked-bars-undocumented)).

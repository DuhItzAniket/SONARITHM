# 04 · Sonification Spec Language (v0.1 draft)

> **Status:** Draft under [RFC-0001](../rfcs/0001-spec-v0.1.md) (in discussion). Field names may change before `specVersion: "0.1"` is frozen at the end of A03; the RFC holds the precise key set, defaults, codes and open questions.
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

The canonical serialization is **JSON**. Builders exist in Rust and Python and produce the same JSON. The JSON Schema is generated from the Rust types (schemars) into `schemas/sonarithm-spec-0.1.schema.json`, and CI fails if it is stale.

## 2. Top-level structure

```jsonc
{
  "specVersion": "0.1",          // required; see §7
  "name": "Global temperature anomaly",
  "description": "Optional free text, spoken as the title",
  "mode": "analysis",            // analysis | presentation | monitoring   (SDR-36)
  "seed": 1729,                  // all randomness derives from this     (SDR-02)
  "data": { },                   // §3
  "transform": [ ],              // §4
  "substrate": { },              // §5.1  time (and optional frequency) substrate
  "tracks": [ ],                 // §5.2  marks + channels
  "context": { },                // §5.4  AUDAXIS: ticks, reference tones, legends, earcons
  "analysis": { },               // §5.5  ECHOMETRIC: detectors, cues, descriptions
  "music": { },                  // §5.6  AXONOTE: presentation-mode quantization, instruments
  "playback": { },               // §5.7  SONOTRACE: speed, loop, monitoring
  "output": { }                  // §5.8  sample rate, channels, loudness target, sinks
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
| `chart` | PLOTSONA | `path`, `extractor` (`auto`, `office`, `svg`, `vega`, `plotly`, `pdf-vector`, `raster`, `ml`), `calibration` |
| `function` | CURVASONA | `expr`, `domain`, `samples` or `step`, `variables` |
| `stream` | SONOTRACE | `transport` (`websocket`, `mqtt`, `serial`, `osc`, `tail`), `url`/`port`/`topic`, `schema`, `window` |
| `dataset` | platform | A reference to a dataset already loaded in a project (desktop/app core) |

`fields` overrides inference ([03-data-model.md §3](03-data-model.md#3-field-semantics)). Any field that is not mentioned keeps its inferred metadata.

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
| `audify` | Audification | The data series played as a waveform (SDR-31) | gain only; speed is set by `audify.k` or a target band | CURVASONA |
| `sample` | Auditory icon | A recorded sound per event | gain, pan, rate | platform |
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

## 7. Versioning and compatibility

| Rule | Detail |
|---|---|
| Version format | `specVersion: "MAJOR.MINOR"`, independent of crate versions |
| Minor bump | Adds optional fields or enum values. Older readers reject unknown fields, so a document declares the minimum version it needs |
| Major bump | Renames, removes or changes semantics. Needs an RFC and a migration |
| RFC scope | An RFC is required for a major bump, a new top-level section, a new substrate, mark or channel kind, or any reinterpretation of existing fields. Adding an optional field or enum value inside an existing construct needs only a normal PR with schema regeneration |
| Migrations | `sonarithm spec migrate` rewrites documents explicitly. The compiler may read old versions through in-memory migration, with a `Warning` diagnostic. It never silently rewrites files |
| Pre-1.0 | Until SONARITHM 1.0, `0.x` minor versions may break, with a migration provided |
| Canonical form | For hashing, specs are canonicalized per RFC 8785 (JSON Canonicalization Scheme) after defaults are applied. The **spec hash** in manifests is SHA-256 of that form ([ADR-0013](../decisions/0013-determinism-and-provenance.md)) |

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

Importers emit a report of every construct they could not map. They never drop one silently.

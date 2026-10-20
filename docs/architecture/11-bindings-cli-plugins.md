# 11 · CLI, Language Bindings, WebAssembly and Plugins

> **Status:** Baseline v1 (2026-10-18). CLI and Python in A07; WASM and plugins in D05.
> **Decision:** [ADR-0009 WASM plugins at compile time](../decisions/0009-wasm-plugins-compile-time.md).

## 1. Rust facade (`sonarithm` crate)

The facade is the **stable public API**. Internal crates may change freely between minor versions until 1.0; the facade changes only with a CHANGELOG entry and semver-checks.

```rust
use sonarithm::{Sonarithm, Spec, RenderOptions};

let snr = Sonarithm::builder().with_default_engines().build()?;
let spec = Spec::from_file("temps.spec.json")?;            // file sources resolve next to it
let compiled = snr.compile(&spec)?;                        // Score + diagnostics
for d in compiled.diagnostics() { eprintln!("{d}"); }
println!("{}", compiled.describe());                       // L1/L2 text, deterministic
snr.render_to_file(&compiled, "temps.flac", RenderOptions::default())?; // + temps.render.json
```

A quick-start builder covers the common case:

```rust
let compiled = snr.quick()
    .data_file("temps.csv")?                               // any supported format
    .x("year").y("anomaly")
    .compile()?;
```

**As built (A07-T01).** `Spec` holds the JSON text, a name for messages (a file name, never a path) and the base directory of its `file` sources; `Spec::validate` and `Spec::migrate` back `sonarithm validate` and `sonarithm spec migrate`. `compile_with(&spec, &Datasets)` compiles specs whose `dataset` sources name data the caller holds (the Python package passes DataFrames this way). `Compiled` gives the Score, its diagnostics, the canonical spec and Score hash, a `Summary` for `inspect` and a `Description`: L1 encoding facts and L2 facts about the Score (where the extremes are, how many notes are estimates). Data statistics and trends join the description with ECHOMETRIC (C05), and the NavTree with AUDAXIS (B03). `quick` applies NUMISONA's `line-overview` template and accepts a data file or a `DataBundle`; `--mark`, `--range` and a concept for polarity are builder options. `render_to_file(&compiled, path, &RenderOptions)` writes the audio (`.wav` 24-bit or `.flac`), `<stem>.render.json` and `<stem>.cues.vtt` through A05's export crate; sample rate, block size and the loudness target come from the spec's `output` section unless the options override them, and the manifest embeds the canonical spec, the data source (a name, never a path) and the provenance mix. `render_wav` renders the same bytes into memory for notebook playback.

## 2. CLI (`sonarithm`)

| Command | Purpose | Key flags | Phase |
|---|---|---|---|
| `sonarithm render <spec> [--data <file>] -o <out>` | Offline render to WAV/FLAC/MIDI + manifest (+ `.cues.vtt`) | `--format`, `--sample-rate`, `--seed`, `--deny-warnings`, `--stems` | A07 |
| `sonarithm play <spec>` | Real-time playback in the terminal (space = pause, q = quit, arrows = step) | `--device`, `--speed` | A07 |
| `sonarithm quick <data> --x <f> --y <f>` | Default spec plus playback or render | `-o`, `--mark`, `--range` | A07 |
| `sonarithm validate <spec>` | Schema and semantic validation | `--json` | A07 |
| `sonarithm lint <spec>` | Compile plus perceptual lint (rule IDs) | `--json`, `--deny-warnings` | A07 (stub), C05 |
| `sonarithm inspect <spec>` | Print the Score summary, NavTree and diagnostics; `--describe` prints the L1/L2 text | `--json`, `--tree` | A07 |
| `sonarithm schema` | Emit the JSON Schema (spec, manifest, profile) | `--kind` | A03 |
| `sonarithm spec migrate <spec>` | Explicit spec version migration | `--to`, `--in-place` | A07 |
| `sonarithm study run <protocol>` | Run an evaluation protocol (ECHOMETRIC harness) | `--participant`, `--out` | A08 |
| `sonarithm gallery render` | Render the replication gallery | `--out` | A08 |
| `sonarithm devices` | List audio devices | — | A07 |
| `sonarithm extract <chart>` | Chart → data with provenance (PLOTSONA) | `--tier`, `--json` | C01 |

**Conventions.** Exit codes: `0` success, `1` errors, `2` usage, `3` warnings denied. `--json` gives machine-readable output with stable fields. Diagnostics are rendered by `miette` with codes and rule IDs. Output is **screen-reader friendly**: `--plain` disables colours and box drawing, and progress is reported as text lines, not spinners.

**As built (A07-T02).** Results go to standard output as one fact per sentence; diagnostics go to standard error, drawn by miette's graphical handler on a terminal and by its narratable handler with `--plain` or when standard error is not a terminal. Colour follows `NO_COLOR` and never carries meaning alone (severities are words). `--json` prints one object on standard output (`spec`, `ok`, `counts`, `diagnostics`, plus `summary` and `description` for `inspect` and `quick`, the Score for `inspect --tree`) and nothing on standard error. `Info` diagnostics (applied defaults with their rule IDs) are counted and listed with `--verbose`. A spec of `-` is read from standard input. A diagnostic whose pointer names a missing top-level key gets its location in the help instead of a snippet of the whole spec. `lint` is the stub: it compiles and lists the compiler's warnings with their rules until ECHOMETRIC's lint (C05). `quick --spec-out` writes the generated spec with the data path relative to the spec. `render <spec> -o FILE` writes the audio, manifest and cues (`--format`, `--sample-rate`, `--seed`, `--data`, `--preserve-level`, `--no-cues`; `--deny-warnings` writes nothing and exits 3) and reports progress as a text line; `quick -o FILE` does the same for the generated spec. The CLI's files are byte-identical to the facade's for the same spec and seed. `play`, `devices` and `quick`'s playback arrive with A05's audio backend; `--stems` and MIDI output are later work.

## 3. Python package (`sonarithm`)

| Aspect | Design |
|---|---|
| Build | PyO3 0.30 + maturin; **abi3** wheels (one wheel per OS/arch for all supported Pythons); `sonarithm._core` native module plus a thin pure-Python layer |
| Data in | pyarrow tables, pandas and polars DataFrames, NumPy arrays, via the Arrow C Data Interface / PyCapsule (zero-copy where possible) |
| API style | Mirrors the facade with Pythonic builders. Specs round-trip as dicts or JSON |
| Notebook | `Compiled._repr_html_()` shows an accessible summary and an `<audio>` element (rendered in memory). `snr.play()` returns an `IPython.display.Audio` |
| Typing | Shipped `.pyi` stubs; `py.typed` |
| Threading | Long renders release the GIL |

```python
import sonarithm as snr
import pandas as pd

df = pd.read_csv("temps.csv")
c = snr.quick(df, x="year", y="anomaly", concept="temperature")
c.describe()            # L1/L2 text (deterministic)
c.play()                # notebook audio
c.render("temps.flac")  # + temps.render.json

spec = snr.Spec.from_dict({...})          # full control
c2 = snr.compile(spec, data={"main": df})
```

## 4. WebAssembly (`sonarithm-wasm`), D05

- Compiles the compiler, engines, DSP and render crates to `wasm32-unknown-unknown`. CI keeps them building from A01, so D05 is packaging, not porting.
- Audio runs in an **AudioWorklet** through cpal's WebAudio host. The render graph is the same code.
- Built with `wasm-bindgen-cli` directly. wasm-pack is optional, given maintainer churn after the rustwasm organization was sunset ([notes/04 §9](../research/notes/04-rust-core-stack.md)).
- The npm package `@sonarithm/wasm` has an API mirroring Python's.
- Interop demos: an MAIDR and Erie bridge (import their specs and data, render with SONARITHM, compare Score IR with the desktop render (H-08)).

## 5. Plugin system (D05)

### 5.1 Model

Third-party **engines** (Encoders, Analyzers, Shapers, Sources) ship as **WebAssembly components** and run in `sonarithm-plugin-host` (wasmtime + WIT; WASI 0.3 shipped June 2026).

| Property | Decision |
|---|---|
| When plugins run | **Only at compile time** (and in live encoding on the control thread). **Never on the audio thread.** Plugins cannot supply Voice DSP in v1 |
| Sandboxing | No file system or network capabilities by default. Data is passed in as Arrow IPC buffers. Fuel and epoch limits bound CPU. Memory is capped |
| Interface | A WIT world `sonarithm:engine@0.1.0` with interfaces `encoder`, `analyzer`, `shaper`, `source` (source receives bytes provided by the host) |
| Distribution | `.sonarithm-plugin` = component `.wasm` + manifest (id, version, capabilities, licence, checksum). Installed per user. Signature verification planned for v1.1 |
| Versioning | The WIT world is semver-versioned. Breaking changes need an RFC |
| Why not native dylibs | No stable Rust ABI; unsafe loading; abi_stable is dormant ([notes/04 §10](../research/notes/04-rust-core-stack.md)) |

### 5.2 WIT sketch (illustrative)

```wit
package sonarithm:engine@0.1.0;

interface types {
  record diagnostic { severity: u8, code: string, message: string, rules: list<string> }
  record note-event { t: f64, dur: f64, cents: f64, gain-db: f32, pan: f32, row: option<u64> }
  variant score-event { note(note-event), earcon(tuple<f64, string>) }
  record score-track { id: string, role: u8, instrument: string, events: list<score-event> }
}

interface encoder {
  use types.{score-track, diagnostic};
  /// track-spec is JSON; data is an Arrow IPC stream
  encode: func(track-spec: string, data: list<u8>) -> result<tuple<score-track, list<diagnostic>>, string>;
}

world engine-plugin {
  export encoder;
}
```

## 6. C ABI and other languages (post-1.0)

A `cbindgen`-generated C header over the facade (compile, render-to-file, free) enables Julia, R and MATLAB bridges. An R package would follow the Python design. It is demand-driven and not on the 1.0 path.

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

**As built (C03-T07): functions.** `read_function(&FunctionSpec)` samples an expression with CURVASONA and returns a `Function`. Its `critical_points()` are the zeros, extrema, inflections, asymptotes, holes, jumps and undefined spans in increasing `x`, plus, with a pitch window, the off-screen spans. Each entry has its label, `x`, `end` for spans, `y` where there is one, and a `text` such as "local maximum at x = 1.5708, y = 1": the text equivalent of the function cues (A11Y-24, A11Y-28; tested on the 40-curve suite). `summary()` is one sentence with the expression, the domain and the count of each feature. `spec()` (or `function_spec(&FunctionSpec, seconds)` for a source that has not been read) is the shape of 04 §6.3: one glide over `FUNCTION_DURATION_S` = 8 s (the example's length, not a research default, TODO(verify) with HP-21), y as pitch, and `analysis.detect: ["critical-points"]` with earcons. `Function::from_spec` reads a spec's function source and its pitch window. The calculator is re-exported as `audify_speedup` with `Speedup` and `SEISMIC_K`, and `TrackSummary.audifications` counts audified series (left out of `--json` when there are none). CURVASONA joins only the compiles that use it (F-197). The text is English, like the labels (F-191); it moves to the catalogue with the desktop function input (C03-T08).

## 2. CLI (`sonarithm`)

| Command | Purpose | Key flags | Phase |
|---|---|---|---|
| `sonarithm render <spec> [--data <file>] -o <out>` | Offline render to WAV/FLAC/MIDI + manifest (+ `.cues.vtt`) | `--format`, `--sample-rate`, `--seed`, `--deny-warnings`, `--stems` | A07 |
| `sonarithm play <spec>` | Real-time playback in the terminal (space = pause, q = quit, arrows = step) | `--device`, `--speed` | A07 |
| `sonarithm quick <data> --x <f> --y <f>`, `sonarithm quick --function <expr> --domain <min,max>` | Default spec plus playback or render; for a function, a glide with critical-point earcons, its critical points printed as text first (C03-T07) | `-o`, `--mark`, `--range`, `--samples`, `--duration` | A07, C03 |
| `sonarithm validate <spec>` | Schema and semantic validation | `--json` | A07 |
| `sonarithm lint <spec>` | Compile plus perceptual lint (rule IDs) | `--json`, `--deny-warnings` | A07 (stub), C05 |
| `sonarithm inspect <spec>` | Print the Score summary, NavTree and diagnostics; `--describe` prints the L1/L2 text. Given a chart file, print its extraction report, the summary sentence of 10 §5 and the Score of its chart-type template (C01-T12). For a function source, also print its summary and critical points; for an audification, the calculator's readout (`SNR-ENG-460`) as a result line (C03-T07) | `--json`, `--tree` | A07, C01, C03 |
| `sonarithm schema` | Emit the JSON Schema (spec, manifest, profile) | `--kind` | A03 |
| `sonarithm spec migrate <spec>` | Explicit spec version migration | `--to`, `--in-place` | A07 |
| `sonarithm study check <protocol>` | Check a study protocol, compile its conditions with their stimuli, summarize the schedule | `--order-seed`, `--json` | A08 |
| `sonarithm study run <protocol>` | Run a session of an evaluation protocol (ECHOMETRIC harness): one plain line per prompt, typed answers, a JSONL log | `--participant`, `--session`, `--order-seed`, `--log`, `--resume`, `--audio`, `--device` | A08 |
| ~~`sonarithm gallery render`~~ | Not built: the replication gallery is checked with `cargo xtask replication` (a developer task), and any of its specs renders with `sonarithm render` | — | A08 |
| `sonarithm devices` | List audio devices | — | A07 |
| `sonarithm extract <chart>` | Chart → data with provenance (PLOTSONA) | `--tier`, `--json` | C01 |

**Conventions.** Exit codes: `0` success, `1` errors, `2` usage, `3` warnings denied. `--json` gives machine-readable output with stable fields. Diagnostics are rendered by `miette` with codes and rule IDs. Output is **screen-reader friendly**: `--plain` disables colours and box drawing, and progress is reported as text lines, not spinners.

**As built (A07-T02).** Results go to standard output as one fact per sentence; diagnostics go to standard error, drawn by miette's graphical handler on a terminal and by its narratable handler with `--plain` or when standard error is not a terminal. Colour follows `NO_COLOR` and never carries meaning alone (severities are words). `--json` prints one object on standard output (`spec`, `ok`, `counts`, `diagnostics`, plus `summary` and `description` for `inspect` and `quick`, the Score for `inspect --tree`) and nothing on standard error. `Info` diagnostics (applied defaults with their rule IDs) are counted and listed with `--verbose`. A spec of `-` is read from standard input. A diagnostic whose pointer names a missing top-level key gets its location in the help instead of a snippet of the whole spec. `lint` is the stub: it compiles and lists the compiler's warnings with their rules until ECHOMETRIC's lint (C05). `quick --spec-out` writes the generated spec with the data path relative to the spec. `render <spec> -o FILE` writes the audio, manifest and cues (`--format`, `--sample-rate`, `--seed`, `--data`, `--preserve-level`, `--no-cues`; `--deny-warnings` writes nothing and exits 3) and reports progress as a text line; `quick -o FILE` does the same for the generated spec. The CLI's files are byte-identical to the facade's for the same spec and seed. `play <spec>` plays through A05's `Player` (start-up ramp, limiter, recovery from device loss, paused on a device change) and announces every change as one text line: what plays and for how long, pauses with their position, steps, device changes and the end. When standard input is a terminal, space pauses and resumes, Left and Right step one second (data-point steps arrive with AUDAXIS navigation, B03), Home returns to the start, and q, Escape or Ctrl+C stop; otherwise the Score plays to its end. `--device` picks an output by name or ID, `--speed` 0.25–4 keeps pitch, and `--backend null` (or `SONARITHM_BACKEND=null`) is a silent timer-driven device; the CLI tests set it for every run, so no test ever plays through a real device. `devices` lists the outputs with the default marked. `quick` without `-o` plays the generated spec after its summary, description and diagnostics, so a screen reader hears them first. `--stems` and MIDI output are later work.

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

**As built (A07-T03).** The package is `bindings/python`: the native module `sonarithm._core` (PyO3 0.29.3, not 0.30, because `arrow-pyarrow` 60 is built on it: F-101) and a pure-Python layer. Wheels are abi3 for CPython 3.10 and later, built by maturin, with no required dependencies: pyarrow, pandas and polars are used when installed. Data comes in through the Arrow PyCapsule interface (any object with `__arrow_c_stream__` or `__arrow_c_array__`, column buffers shared), pandas through pyarrow or, without it, pandas' own JSON writer; dicts of columns, lists of records, 1-D arrays and file paths are accepted too. The data layer's `read_batches` gives in-memory Arrow the same treatment as a file and normalizes layouts and library metadata, so the same values give the same fingerprint from every library (F-100). `compile` and `quick` release the GIL. Errors raise `SonarithmError` with `diagnostics` as dicts (the CLI's `--json` fields). `render(path)` writes the audio, manifest and cues like `sonarithm render`; `wav()` renders the same bytes in memory and `play()` wraps them in `IPython.display.Audio`. `Compiled._repr_html_()` is text first: the L1/L2 description as a list, an `<audio>` element labelled with its length for Scores up to 30 s (about 290 kB per second of WAV; longer ones point at `play()`), then the diagnostic counts. Parity tests compare Python's Score with the facade's value for value, and its WAV with the checked-in SHA-256 (`tests/golden/parity`), on every OS. `read_chart(path)` (C01-T12) returns a `Chart` with `summary` (the sentence of 10 §5), `report()` (the dict `inspect --json` prints), `template`, `spec()`, `dataset()` and `compile()`. `read_function(expr, domain, *, samples, step, variables)` (C03-T07) returns a `Function` with `expr`, `summary`, `critical_points()` (dicts with `kind`, `x`, `end`, `y` and `text`, as `inspect --json` lists them), `spec(duration)`, `dataset()` and `compile()`; an expression that cannot be read raises `SonarithmError` with its `SNR-ENG-40x` diagnostic.

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

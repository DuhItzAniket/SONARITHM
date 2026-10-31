// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask replication`: the replication gallery (A08-T02; evaluation plan
//! "Replication gallery"; replication/README.md).
//!
//! The gallery re-creates published designs from committed specs. Some of its files are
//! generated, and this task keeps them honest:
//!
//! 1. **Stimulus data** comes from ECHOMETRIC's generators with a fixed seed: the Smith &
//!    Walker series (GEN-002) and the sixteen Nees & Walker cells (GEN-001), each with its
//!    stimulus manifest.
//! 2. **Generated specs**: one spec per Nees & Walker cell from one template, and the
//!    Erie gallery subset imported by NUMISONA from its fixtures, with a synthetic
//!    stand-in for the one external file they name.
//! 3. **Renders**: every spec under `replication/` compiles with the real compiler and
//!    NUMISONA (and CURVASONA where the spec uses it: the Dombois audifications and the
//!    40-curve suite, C03-T06) and is exported as 24-bit WAV at the rendered level (no normalization, as
//!    for research exports) into `target/replication/`, with its render manifest. The
//!    reproducible part of each manifest (everything except the platform and the
//!    software versions) is committed in the group's `manifests.json`.
//!
//! `cargo xtask replication` regenerates everything in memory and fails on any difference
//! from the committed files; `--update` writes them. Audio is never committed (OD-08):
//! the manifests carry each file's SHA-256.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use serde_json::{Value, json};
use sonarithm_compiler::{Compiler, Platform};
use sonarithm_core::engine::CancelToken;
use sonarithm_core::score::{Note, Score, ScoreEvent};
use sonarithm_curvasona::Curvasona;
use sonarithm_curvasona::suite::CURVES;
use sonarithm_echometric::harness::generators::prices::PriceParams;
use sonarithm_echometric::harness::generators::reversals::{ReversalParams, Start};
use sonarithm_echometric::harness::generators::transits::{STUDIED_SNR, TransitParams};
use sonarithm_echometric::harness::{Params, Stimulus, generate};
use sonarithm_export::{
    AudioFormat, DataSource, ExportOptions, ManifestInputs, Normalization, WavEncoding, export,
};
use sonarithm_numisona::Numisona;
use sonarithm_numisona::import::erie;

/// The gallery folder, relative to the workspace root.
pub(crate) const DIR: &str = "replication";

/// The seed of every generated stimulus in the gallery.
pub(crate) const SEED: u64 = 20_261_019;

/// The Nees & Walker (2008) cells: densities in points per second × reversals.
const NEES_WALKER_RATES: [u32; 4] = [1, 2, 4, 8];
const NEES_WALKER_REVERSALS: [u32; 4] = [0, 1, 2, 3];

/// The Tucker Brown et al. (2022) set: every studied SNR with 0, 1 and 2 transits.
const TRANSIT_COUNTS: [u32; 3] = [0, 1, 2];

/// The Erie fixtures NUMISONA imports (A06-T08), relative to the workspace root.
const ERIE_FIXTURES: &str = "crates/engines/sonarithm-numisona/tests/erie";
const ERIE_NAMES: [&str; 8] = [
    "time-series",
    "histogram",
    "sparsity-simple",
    "sparsity-and-length",
    "interval",
    "model-fit-overlay",
    "residuals-sequence",
    "frequency",
];

/// Whether to check the committed files or rewrite them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Fail on any difference.
    Check,
    /// Write the generated files and manifests.
    Update,
    /// Render every spec one at a time and print compile and export times (A08-T04). Run
    /// it in a release build: `cargo run --release -p xtask -- replication --timings`.
    Timings,
}

/// Runs the gallery task. `out` receives the rendered audio and full manifests.
pub(crate) fn run(root: &Path, out: &Path, mode: Mode) -> Result<()> {
    let gallery = root.join(DIR);
    let mut files = generated(root)?;
    let mut jobs = Vec::new();
    for group in groups(&gallery, &files)? {
        // Data files are read from disk, so each group's data, committed or generated, is
        // staged once before the renders start.
        stage_data(&gallery.join(&group), &group, &out.join("data"), &files)?;
        for spec in specs(&gallery.join(&group), &files)? {
            jobs.push((group.clone(), spec));
        }
    }
    let rendered = jobs.len();
    if mode == Mode::Timings {
        return timings(&gallery, out, &jobs, &files);
    }
    // Renders are independent and deterministic, so they run on all cores; the results
    // are collected into sorted maps.
    let threads = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    let chunk = jobs.len().div_ceil(threads).max(1);
    let results: Vec<Result<(String, String, Value)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = jobs
            .chunks(chunk)
            .map(|batch| {
                let (gallery, files) = (&gallery, &files);
                scope.spawn(move || {
                    batch
                        .iter()
                        .map(|(group, spec)| {
                            let manifest =
                                render(&gallery.join(group), spec, &out.join(group), files)?;
                            Ok((group.clone(), spec.clone(), manifest))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| vec![Err(anyhow::anyhow!("a render thread panicked"))])
            })
            .collect()
    });
    let mut manifests: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
    for result in results {
        let (group, spec, manifest) = result?;
        manifests.entry(group).or_default().insert(spec, manifest);
    }
    for (group, records) in &manifests {
        let text = serde_json::to_string_pretty(records)? + "\n";
        files.insert(format!("{group}/manifests.json"), text);
    }
    let mut stale = Vec::new();
    for (path, text) in &files {
        let target = gallery.join(path);
        let current = std::fs::read_to_string(&target).ok();
        if current.as_deref() == Some(text.as_str()) {
            continue;
        }
        match mode {
            Mode::Check | Mode::Timings => stale.push(format!("{DIR}/{path}")),
            Mode::Update => {
                if let Some(dir) = target.parent() {
                    std::fs::create_dir_all(dir)
                        .with_context(|| format!("creating {}", dir.display()))?;
                }
                std::fs::write(&target, text)
                    .with_context(|| format!("writing {}", target.display()))?;
            }
        }
    }
    if !stale.is_empty() {
        bail!(
            "{} replication-gallery file(s) differ from what the generators and the renderer produce:\n  {}\n\
             Run `cargo xtask replication --update` and review the diff; a change to a manifest means \
             a render changed (state why in the PR).",
            stale.len(),
            stale.join("\n  ")
        );
    }
    match mode {
        Mode::Check | Mode::Timings => println!(
            "replication: {rendered} specs rendered; all {} generated files match",
            files.len()
        ),
        Mode::Update => println!(
            "replication: {rendered} specs rendered; {} generated files written",
            files.len()
        ),
    }
    Ok(())
}

/// Every generated file, by path relative to `replication/`, except the manifests.
fn generated(root: &Path) -> Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    // Smith & Walker (2005): the reconstructed series, all 11 hours queried.
    let prices = generate(&Params::PriceSeries(PriceParams::default()), SEED)?;
    files.insert("smith-walker-2005/data.csv".to_owned(), prices.csv());
    files.insert(
        "smith-walker-2005/stimulus.json".to_owned(),
        pretty(&prices.manifest())?,
    );
    // Nees & Walker (2008): sixteen cells, initially rising, schematic shapes.
    let mut stimuli = BTreeMap::new();
    for rate in NEES_WALKER_RATES {
        for reversals in NEES_WALKER_REVERSALS {
            let name = format!("d{rate}-r{reversals}");
            let params = ReversalParams {
                start: Start::Up,
                ..ReversalParams::cell(f64::from(rate), reversals)
            };
            let stimulus = generate(&Params::TrendReversals(params), SEED)?;
            files.insert(format!("nees-walker-2008/data/{name}.csv"), stimulus.csv());
            files.insert(
                format!("nees-walker-2008/{name}.json"),
                pretty(&nees_walker_spec(rate, reversals, &stimulus))?,
            );
            stimuli.insert(name, stimulus.manifest());
        }
    }
    files.insert(
        "nees-walker-2008/stimuli.json".to_owned(),
        pretty(&stimuli)?,
    );
    // Tucker Brown et al. (2022): 6 SNR levels × 0, 1, 2 transits = the study's 18 curves.
    // Each curve gets its own seed (SEED plus its position), so no two share their noise.
    let mut curves = BTreeMap::new();
    let mut position = 0;
    for snr in STUDIED_SNR {
        for transits in TRANSIT_COUNTS {
            let name = format!("snr{snr}-n{transits}");
            let stimulus = generate(
                &Params::Transits(TransitParams::curve(snr, transits)),
                SEED + position,
            )?;
            position += 1;
            files.insert(format!("tucker-brown-2022/data/{name}.csv"), stimulus.csv());
            files.insert(
                format!("tucker-brown-2022/{name}.json"),
                pretty(&transit_spec(&name, snr, transits, &stimulus))?,
            );
            curves.insert(name, stimulus.manifest());
        }
    }
    files.insert(
        "tucker-brown-2022/stimuli.json".to_owned(),
        pretty(&curves)?,
    );
    // The Erie gallery subset, imported from NUMISONA's fixtures.
    for name in ERIE_NAMES {
        let path = root.join(ERIE_FIXTURES).join(format!("{name}.json"));
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let import = erie::import(&text).with_context(|| format!("importing {name}"))?;
        for (index, spec) in import.specs.iter().enumerate() {
            let file = if import.specs.len() == 1 {
                format!("erie/{name}.json")
            } else {
                format!("erie/{name}-{}.json", index + 1)
            };
            files.insert(file, pretty(spec)?);
        }
    }
    files.insert("erie/data/penguins.json".to_owned(), pretty(&penguins())?);
    // CURVASONA's 40-curve suite (C03-T06): one spec per curve and the README listing
    // each curve's critical points, generated from the suite and the engine.
    for (index, curve) in CURVES.iter().enumerate() {
        files.insert(
            format!("{SUITE}/{}.json", suite_name(index, curve.name)),
            pretty(&suite_spec(curve))?,
        );
    }
    files.insert(format!("{SUITE}/README.md"), suite_readme()?);
    Ok(files)
}

/// The folder of the 40-curve suite.
const SUITE: &str = "curvasona-suite";

/// How long each suite curve plays: the length of 04 §6.3's example, as `quick --function`
/// (TODO(verify), HP-21).
const SUITE_SECONDS: f64 = 8.0;

/// `01-parabola` and so on: the position in the suite, then the name in lower case with
/// hyphens.
fn suite_name(index: usize, name: &str) -> String {
    let slug: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    format!("{:02}-{slug}", index + 1)
}

/// One suite curve as a spec: a glide over the domain with its critical points as earcons
/// (the shape of 04 §6.3).
fn suite_spec(curve: &sonarithm_curvasona::suite::Curve) -> Value {
    json!({
        "specVersion": "0.4",
        "name": format!("CURVASONA suite: {}", curve.name),
        "description": format!(
            "f(x) = {} over [{}, {}], one curve of the 40-curve suite with analytically known critical points (C03-T03), as a glide with its critical points as earcons. Curve gallery, C03-T06.",
            curve.expr, curve.domain[0], curve.domain[1]
        ),
        "data": { "source": { "kind": "function", "expr": curve.expr, "domain": curve.domain } },
        "substrate": { "time": { "pace": { "duration": { "s": SUITE_SECONDS } } } },
        "tracks": [{ "id": "f", "mark": "glide", "channels": { "pitch": { "field": "y" } } }],
        "analysis": { "detect": ["critical-points"], "cue": "earcon" }
    })
}

/// A number for the README: at most four decimals, no trailing zeros, never `-0`.
fn suite_number(value: f64) -> String {
    let mut text = format!("{value:.4}");
    if text.contains('.') {
        let kept = text.trim_end_matches('0').trim_end_matches('.').len();
        text.truncate(kept);
    }
    if text == "-0" { "0".to_owned() } else { text }
}

/// The suite's README: what the group is and, per curve, its critical points as CURVASONA
/// finds them at 1000 samples (the text equivalent of the earcons, A11Y-24, A11Y-28).
fn suite_readme() -> Result<String> {
    use std::fmt::Write as _;
    let mut text = String::from(
        "# CURVASONA's 40-curve suite\n\n\
         > **Generated** by `cargo xtask replication` from `sonarithm_curvasona::suite`; do not edit by hand.\n\n\
         Not a published design: this group renders the 40 curves whose critical points are known analytically, \
         the suite CURVASONA is tested against (07 CURVASONA, C03-T02 and C03-T03). Each spec plays its curve as one \
         8-second glide, y as pitch with the default range, and its critical points as earcons (`analysis.detect: \
         [\"critical-points\"]`, RFC-0007 §2). The list below is what CURVASONA finds at the default 1000 samples, \
         the text equivalent of each render's cues. Horizontal asymptotes are listed but have no earcon.\n\n\
         | Spec | f(x) | Domain | Critical points |\n|---|---|---|---|\n",
    );
    for (index, curve) in CURVES.iter().enumerate() {
        let source = sonarithm_curvasona::FunctionSpec::new(curve.expr, curve.domain);
        let sampled = sonarithm_curvasona::sample_function(
            &source,
            &sonarithm_curvasona::Budgets::DEFAULT,
            (sonarithm_data::BundleId(0), sonarithm_curvasona::FUNCTION_TABLE),
        )
        .map_err(|err| anyhow::anyhow!("{}: {}", curve.name, err.diagnostic.message))?;
        let points: Vec<String> = sampled
            .features
            .iter()
            .map(|feature| {
                let label = feature.kind.label();
                if !feature.is_point() {
                    format!(
                        "{label} from {} to {}",
                        suite_number(feature.x),
                        suite_number(feature.end)
                    )
                } else if label.starts_with("horizontal asymptote") {
                    feature
                        .y
                        .map_or_else(|| label.to_owned(), |y| format!("{label}, y = {}", suite_number(y)))
                } else {
                    format!("{label} at {}", suite_number(feature.x))
                }
            })
            .collect();
        let name = suite_name(index, curve.name);
        let listed = if points.is_empty() { "none".to_owned() } else { points.join("; ") };
        let _ = writeln!(
            text,
            "| [{name}]({name}.json) | `{}` | [{}, {}] | {listed} |",
            curve.expr.replace('|', "\\|"),
            suite_number(curve.domain[0]),
            suite_number(curve.domain[1]),
        );
    }
    text.push_str(
        "\nThe reproducible part of each render manifest is in [manifests.json](manifests.json). \
         Specs and records are CC BY 4.0, like the rest of the gallery.\n",
    );
    Ok(text)
}

/// The spec of one Nees & Walker cell: 125 ms tones, $6–$106 linear on MIDI G2–B6 in
/// log frequency (the study's chromatic "exact" scale), positive polarity, one second per
/// hour.
fn nees_walker_spec(rate: u32, reversals: u32, stimulus: &Stimulus) -> Value {
    let plural = if reversals == 1 { "" } else { "s" };
    json!({
        "specVersion": "0.1",
        "name": format!("Nees & Walker (2008): {rate} points/s, {reversals} reversal{plural}"),
        "description": "A stock price over a ten-hour day at one second per hour, as 125 ms tones from $6 (G2) to $106 (B6). Replication gallery, A08.",
        "mode": "analysis",
        "seed": stimulus.seed,
        "data": {
            "source": { "kind": "file", "path": format!("data/d{rate}-r{reversals}.csv"), "format": "csv" },
            "fields": {
                "t": { "type": "quantitative", "role": "x", "unit": "s", "label": "Time" },
                "hour": { "type": "quantitative", "unit": "h", "label": "Hour of the day" },
                "price": { "type": "quantitative", "unit": "$", "concept": "money", "label": "Price" }
            }
        },
        "substrate": { "time": { "field": "t", "pace": { "secondsPerUnit": 1 } } },
        "tracks": [{
            "id": "price",
            "mark": "tone",
            "channels": {
                "pitch": {
                    "field": "price",
                    "scale": { "type": "linear", "domain": [6, 106] },
                    "range": { "midi": [43, 95] },
                    "space": "log-frequency",
                    "polarity": "positive"
                },
                "duration": { "value": { "s": 0.125 } },
                "loudness": { "value": { "db": -18 } }
            }
        }]
    })
}

/// The spec of one light curve, as astronify sounded it in the study: one tone per point
/// every 0.01 s, 0.2 s notes, each curve's flux range mapped linearly onto 440–880 Hz with
/// positive polarity.
fn transit_spec(name: &str, snr: f64, transits: u32, stimulus: &Stimulus) -> Value {
    let plural = if transits == 1 { "" } else { "s" };
    json!({
        "specVersion": "0.1",
        "name": format!("Tucker Brown et al. (2022): SNR {snr}, {transits} transit{plural}"),
        "description": "A 1000-point light curve with box-shaped transits in Gaussian noise, one tone per point every 0.01 s, flux mapped linearly onto 440-880 Hz. Replication gallery, A08.",
        "mode": "analysis",
        "seed": stimulus.seed,
        "data": {
            "source": { "kind": "file", "path": format!("data/{name}.csv"), "format": "csv" },
            "fields": {
                "t": { "type": "quantitative", "role": "x", "label": "Time" },
                "flux": { "type": "quantitative", "label": "Brightness" }
            }
        },
        "substrate": { "time": { "field": "t", "pace": { "secondsPerUnit": 0.01 } } },
        "tracks": [{
            "id": "flux",
            "mark": "tone",
            "instrument": "sine",
            "channels": {
                "pitch": {
                    "field": "flux",
                    "scale": { "type": "linear", "domain": "auto" },
                    "range": { "hz": [440, 880] },
                    "space": "linear-frequency",
                    "polarity": "positive"
                },
                "duration": { "value": { "s": 0.2 } },
                "loudness": { "value": { "db": -24 } }
            }
        }]
    })
}

/// A synthetic stand-in for the penguins file the Erie histogram fixture names, as in
/// NUMISONA's conformance test: 60 body masses from 3000 to 5499 g.
fn penguins() -> Value {
    Value::Array(
        (0..60)
            .map(|i| json!({ "body_mass": 3000 + (i * 37) % 2500 }))
            .collect(),
    )
}

fn pretty<T: serde::Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)? + "\n")
}

/// The gallery's groups: its subfolders and those of the generated files, sorted.
fn groups(gallery: &Path, files: &BTreeMap<String, String>) -> Result<Vec<String>> {
    let mut groups: std::collections::BTreeSet<String> = files
        .keys()
        .filter_map(|path| path.split_once('/').map(|(group, _)| group.to_owned()))
        .collect();
    for entry in
        std::fs::read_dir(gallery).with_context(|| format!("reading {}", gallery.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            groups.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(groups.into_iter().collect())
}

/// A group's specs: its top-level `*.json` files except the generated records, plus the
/// generated specs not yet written. Names are sorted.
fn specs(dir: &Path, files: &BTreeMap<String, String>) -> Result<Vec<String>> {
    const RECORDS: [&str; 3] = ["manifests.json", "stimulus.json", "stimuli.json"];
    let group = dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut names = std::collections::BTreeSet::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type()?.is_file()
                && has_extension(&name, "json")
                && !RECORDS.contains(&name.as_str())
            {
                names.insert(name);
            }
        }
    }
    for path in files.keys() {
        if let Some(name) = path.strip_prefix(&format!("{group}/"))
            && !name.contains('/')
            && has_extension(name, "json")
            && !RECORDS.contains(&name)
        {
            names.insert(name.to_owned());
        }
    }
    Ok(names
        .into_iter()
        .map(|name| name.trim_end_matches(".json").to_owned())
        .collect())
}

/// Renders every spec on one thread, timing the compile and the export of each, and prints
/// a table: spec, audio seconds, compile ms, export ms, and the real-time factor of the
/// whole (audio seconds over wall seconds). Wall-clock reads are fine here: xtask is not a
/// compile or render path, and the numbers are never compared for determinism.
fn timings(
    gallery: &Path,
    out: &Path,
    jobs: &[(String, String)],
    files: &BTreeMap<String, String>,
) -> Result<()> {
    println!("spec\taudio_s\tcompile_ms\texport_ms\trtf");
    let (mut audio, mut wall) = (0.0, 0.0);
    for (group, spec) in jobs {
        let mut clock = Clock::default();
        let manifest = render_timed(
            &gallery.join(group),
            spec,
            &out.join(group),
            files,
            &mut clock,
        )?;
        let seconds = manifest
            .pointer("/render/durationS")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let total = clock.compile + clock.export;
        audio += seconds;
        wall += total;
        println!(
            "{group}/{spec}\t{seconds:.3}\t{:.1}\t{:.1}\t{:.0}",
            clock.compile * 1e3,
            clock.export * 1e3,
            seconds / total
        );
    }
    println!(
        "total\t{audio:.3}\t\t{:.1}\t{:.0}",
        wall * 1e3,
        audio / wall
    );
    Ok(())
}

/// Seconds spent compiling and exporting one spec.
#[derive(Debug, Default)]
struct Clock {
    compile: f64,
    export: f64,
}

/// Compiles and exports one spec; returns the reproducible part of its manifest.
fn render(dir: &Path, name: &str, out: &Path, files: &BTreeMap<String, String>) -> Result<Value> {
    render_timed(dir, name, out, files, &mut Clock::default())
}

/// [`render`], recording how long the compile and the export took.
fn render_timed(
    dir: &Path,
    name: &str,
    out: &Path,
    files: &BTreeMap<String, String>,
    clock: &mut Clock,
) -> Result<Value> {
    let group = dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    // Render from the generated text when there is one, so a check never reads stale files.
    let spec_text = match files.get(&format!("{group}/{name}.json")) {
        Some(text) => text.clone(),
        None => std::fs::read_to_string(dir.join(format!("{name}.json")))
            .with_context(|| format!("reading {group}/{name}.json"))?,
    };
    // `run` staged the group's data under the output folder's parent.
    let base = out
        .parent()
        .map_or_else(|| out.join("data"), |parent| parent.join("data"))
        .join(&group);
    let platform = Platform::with_base_dir(base);
    let numisona = Numisona::new();
    let curvasona = Curvasona::new();
    let spec: Value = serde_json::from_str(&spec_text)?;
    // CURVASONA only where the spec uses it: registered for every spec, it would enter
    // every Score's header and change every record (F-197).
    let engines: Vec<&dyn sonarithm_core::engine::Engine<sonarithm_data::DataBundle>> =
        if sonarithm_curvasona::is_used_by(&spec) {
            vec![&platform, &numisona, &curvasona]
        } else {
            vec![&platform, &numisona]
        };
    let pipeline =
        Compiler::new(&engines).map_err(|err| anyhow::anyhow!("{}", err.message))?;
    let started = std::time::Instant::now();
    let compiled = pipeline
        .compile(&spec_text, &CancelToken::new())
        .map_err(|failure| anyhow::anyhow!("{group}/{name}: compile failed: {failure:?}"))?;
    let source = spec
        .pointer("/data/source/path")
        .and_then(Value::as_str)
        .map(|path| DataSource::new("file", path))
        .or_else(|| Some(DataSource::new("inline", "inline")));
    let mut options = ExportOptions::new(AudioFormat::Wav(WavEncoding::Pcm24));
    options.normalization = Normalization::Preserve;
    options.cues = false;
    let mut inputs = ManifestInputs::default();
    inputs.data_source = source;
    std::fs::create_dir_all(out)?;
    clock.compile = started.elapsed().as_secs_f64();
    let started = std::time::Instant::now();
    let exported = export(&compiled.score, &options, &inputs, out, name)
        .with_context(|| format!("{group}/{name}: export failed"))?;
    clock.export = started.elapsed().as_secs_f64();
    let mut manifest = serde_json::to_value(&exported.manifest)?;
    if let Some(object) = manifest.as_object_mut() {
        object.remove("platform");
        object.remove("sonarithm");
        // The Score header names the engine versions, so the raw Score hash changes with
        // every release while no sound does (F-102). The record keeps the hash with every
        // engine version written as `*`; the full manifest in target/ has the real one.
        let independent = release_independent(&compiled.score)
            .canonical_hash()
            .map_err(|err| anyhow::anyhow!("{group}/{name}: no canonical Score: {err}"))?;
        object.insert(
            "score".to_owned(),
            json!({
                "irVersion": compiled.score.header.ir_version,
                "releaseIndependentHash": format!("sha256:{independent}"),
            }),
        );
        object.insert("tracks".to_owned(), tracks(&compiled.score));
        // Third-party data names its source and licence in the spec (CC BY 4.0 asks for
        // attribution); the record repeats it, so the credit travels with the render.
        if let Some(attribution) = spec.get("x-attribution") {
            object.insert("attribution".to_owned(), attribution.clone());
        }
    }
    Ok(manifest)
}

/// The Score with every engine version written as `*` (F-102).
fn release_independent(score: &Score) -> Score {
    let mut score = score.clone();
    for version in score.header.engines.values_mut() {
        *version = sonarithm_core::score::EngineVersion("*".to_owned());
    }
    score
}

/// What each track sounds, for comparing with the published design: notes, onsets,
/// durations and the pitch range in hertz (rounded to 0.01 Hz), and the instrument.
fn tracks(score: &Score) -> Value {
    let round = |value: f64| (value * 100.0).round() / 100.0;
    let summary: Vec<Value> = score
        .tracks
        .iter()
        .map(|track| {
            let notes: Vec<&Note> = track
                .events
                .iter()
                .filter_map(|event| match event {
                    ScoreEvent::Note(note) => Some(note),
                    _ => None,
                })
                .collect();
            let hz: Vec<f64> = notes.iter().map(|note| note.pitch.to_hz().0).collect();
            let low = hz.iter().copied().fold(f64::INFINITY, f64::min);
            let high = hz.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let durations: std::collections::BTreeSet<String> = notes
                .iter()
                .map(|note| format!("{}", (note.dur.0 * 1000.0).round() / 1000.0))
                .collect();
            let mut record = json!({
                "id": track.id.as_str(),
                "instrument": track.instrument.0,
                "events": track.events.len(),
                "notes": notes.len(),
                "firstOnsetS": notes.first().map(|note| note.t.0),
                "lastOnsetS": notes.last().map(|note| note.t.0),
                "noteDurationsS": durations.into_iter().collect::<Vec<_>>(),
                "pitchHz": if notes.is_empty() { Value::Null } else { json!([round(low), round(high)]) },
            });
            // Audified series (C03-T05): their sample count, playback rate and length.
            // Only tracks that have one carry the key, so older records are unchanged.
            let audified: Vec<Value> = track
                .events
                .iter()
                .filter_map(|event| match event {
                    ScoreEvent::Audify(audify) => Some(json!({
                        "samples": audify.samples.len(),
                        "rateHz": audify.rate,
                        "durationS": (audify.duration().0 * 1e6).round() / 1e6,
                    })),
                    _ => None,
                })
                .collect();
            if let (false, Some(object)) = (audified.is_empty(), record.as_object_mut()) {
                object.insert("audified".to_owned(), Value::Array(audified));
            }
            record
        })
        .collect();
    Value::Array(summary)
}

/// Copies the group's data, generated or committed, into a staging folder that mirrors
/// the group, and returns it as the base folder for `file` sources.
fn stage_data(
    dir: &Path,
    group: &str,
    staging: &Path,
    files: &BTreeMap<String, String>,
) -> Result<PathBuf> {
    let base = staging.join(group);
    std::fs::create_dir_all(&base)?;
    // Committed data first, then generated data over it.
    copy_tree(dir, &base, &|name| {
        has_extension(name, "csv") || has_extension(name, "parquet") || name == "penguins.json"
    })?;
    for (path, text) in files {
        if let Some(rest) = path.strip_prefix(&format!("{group}/"))
            && (has_extension(rest, "csv") || rest.ends_with("/penguins.json"))
        {
            let target = base.join(rest);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&target, text)?;
        }
    }
    Ok(base)
}

/// Whether `name` has the extension `extension`, in any case.
fn has_extension(name: &str, extension: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|found| found.eq_ignore_ascii_case(extension))
}

fn copy_tree(from: &Path, to: &Path, keep: &dyn Fn(&str) -> bool) -> Result<()> {
    if !from.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let source = entry.path();
        if entry.file_type()?.is_dir() {
            copy_tree(&source, &to.join(&name), keep)?;
        } else if keep(&name) {
            std::fs::create_dir_all(to)?;
            std::fs::copy(&source, to.join(&name))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A08-T02 acceptance: every gallery item renders reproducibly from its committed spec,
    /// with its manifest, on every CI platform.
    #[test]
    fn the_replication_gallery_renders_reproducibly() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let out = root.join("target").join("xtask-replication-test");
        run(&root, &out, Mode::Check).unwrap();
    }
}

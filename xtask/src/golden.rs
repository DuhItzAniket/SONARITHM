// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask golden`: the golden-audio harness (testing-strategy §5, A05-T11).
//!
//! Each case lives in `tests/golden/<case>/`: a `case.json` manifest and, unless it uses
//! a generated Score, a `score.json` (Score IR). **Storage (OD-08, decided in A05):** no
//! reference audio is committed. A05 renders are bit-identical across platforms (F-039), so
//! the manifest records the reference render's SHA-256 and metrics: length, integrated
//! loudness, peaks, RMS and the spectral centroid per second. CI renders every case on
//! Linux, Windows and macOS, checks each against its manifest, and compares the three sets
//! with each other (PB-06); the renders are kept as workflow artifacts for listening.
//!
//! | Command | Does |
//! |---|---|
//! | `cargo xtask golden` | Render every case and check it against its manifest and at other block sizes |
//! | `cargo xtask golden --update --reason "<why>" [case…]` | Re-render and rewrite the reference sections, printing the deltas |
//! | `cargo xtask golden --render-set <dir>` | Write each case as 32-bit float WAV plus `summary.json` |
//! | `cargo xtask golden --compare <reference> <set>…` | Null residual and ΔLUFS between render sets (PB-06) |

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use sonarithm_core::{SampleRate, Score};
use sonarithm_dsp::meter;
use sonarithm_render::fixtures::benchmark_score;
use sonarithm_render::{OfflineRender, PlanConfig, RenderPlan};

/// Where the cases live, relative to the workspace root.
const CASES: &str = "tests/golden";
/// Block sizes that must give the same audio as the case's own (testing-strategy §5.2).
const INVARIANCE_BLOCKS: [usize; 2] = [64, 1_024];

/// A case manifest, `case.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    description: String,
    source: Source,
    sample_rate: u32,
    block_size: usize,
    tolerances: Tolerances,
    reference: Option<Reference>,
    /// Why the reference last changed.
    reason: String,
}

/// Where the Score comes from.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind", deny_unknown_fields)]
enum Source {
    /// `score.json` in the case directory.
    Score,
    /// The DS-006 benchmark Score generator.
    Benchmark { seconds: f64, seed: u64 },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Tolerances {
    /// Largest allowed peak of the difference between renders, dBFS (PB-06: −90).
    null_residual_dbfs: f64,
    /// Largest allowed integrated-loudness difference, LU (PB-06: 0.1).
    lufs_delta: f64,
    /// Largest allowed relative change of the per-second spectral centroid (provisional
    /// default 1 %, testing-strategy §5.2).
    centroid_relative: f64,
}

/// The reference render's fingerprint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Reference {
    /// The OS and architecture that rendered it.
    platform: String,
    /// SHA-256 of the samples as little-endian f32, interleaved stereo.
    sha256: String,
    frames: u64,
    integrated_lufs: Option<f64>,
    true_peak_dbtp: f64,
    peak_dbfs: f64,
    rms_dbfs: f64,
    /// Mean spectral centroid of each second's non-silent frames, Hz.
    centroid_hz: Vec<Option<f64>>,
}

/// A rendered case.
struct Render {
    samples: Vec<f32>,
    reference: Reference,
}

pub(crate) fn run(root: &Path, args: &[String]) -> Result<()> {
    match args {
        [] => check_all(root),
        [flag, rest @ ..] if flag == "--update" => update(root, rest),
        [flag, dir] if flag == "--render-set" => render_set(root, Path::new(dir)),
        [flag, reference, sets @ ..] if flag == "--compare" && !sets.is_empty() => {
            compare_sets(Path::new(reference), sets)
        }
        _ => bail!(
            "usage: cargo xtask golden [--update --reason <why> [case...] | --render-set <dir> | --compare <reference> <set>...]"
        ),
    }
}

fn cases(root: &Path) -> Result<BTreeMap<String, (PathBuf, Case)>> {
    let dir = root.join(CASES);
    let mut cases = BTreeMap::new();
    for entry in std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        let manifest = path.join("case.json");
        if !manifest.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&manifest)
            .with_context(|| format!("reading {}", manifest.display()))?;
        let case: Case = serde_json::from_str(&text)
            .with_context(|| format!("parsing {}", manifest.display()))?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        cases.insert(name, (path, case));
    }
    if cases.is_empty() {
        bail!("no golden cases under {}", dir.display());
    }
    Ok(cases)
}

fn score(dir: &Path, case: &Case) -> Result<Score> {
    Ok(match case.source {
        Source::Benchmark { seconds, seed } => benchmark_score(seconds, seed),
        Source::Score => {
            let path = dir.join("score.json");
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?
        }
    })
}

fn render(dir: &Path, case: &Case, block: usize) -> Result<Render> {
    let rate = SampleRate::new(case.sample_rate).context("the sample rate is zero")?;
    let plan = RenderPlan::build(&score(dir, case)?, &PlanConfig::new(rate))
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .plan;
    let samples = OfflineRender::new(plan, block).render_all();
    let reference = fingerprint(&samples, case.sample_rate)?;
    Ok(Render { samples, reference })
}

fn db(x: f64) -> f64 {
    20.0 * sonarithm_core::math::log10(x.max(1e-12))
}

fn fingerprint(samples: &[f32], rate: u32) -> Result<Reference> {
    let mut hash = Sha256::new();
    for sample in samples {
        hash.update(sample.to_le_bytes());
    }
    let sha256 = hash
        .finalize()
        .iter()
        .fold(String::from("sha256:"), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        });
    let loudness = meter::measure(samples, SampleRate::new(rate).context("zero rate")?)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let peak = samples.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
    #[expect(clippy::cast_precision_loss, reason = "sample counts below 2^53")]
    let rms = (samples
        .iter()
        .map(|s| f64::from(*s) * f64::from(*s))
        .sum::<f64>()
        / samples.len().max(1) as f64)
        .sqrt();
    Ok(Reference {
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        sha256,
        frames: (samples.len() / 2) as u64,
        integrated_lufs: loudness.integrated.map(|l| round(l.0)),
        true_peak_dbtp: round(loudness.true_peak.0),
        peak_dbfs: round(db(f64::from(peak))),
        rms_dbfs: round(db(rms)),
        centroid_hz: centroids(samples, rate),
    })
}

fn round(x: f64) -> f64 {
    (x * 1_000.0).round() / 1_000.0
}

/// The mean spectral centroid of each second: 2048-point Hann frames, hop 1024, mono
/// mix, frames quieter than −80 dBFS RMS left out.
fn centroids(samples: &[f32], rate: u32) -> Vec<Option<f64>> {
    const N: usize = 2_048;
    let mono: Vec<f64> = samples
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&[l, r]| f64::midpoint(f64::from(l), f64::from(r)))
        .collect();
    let window: Vec<f64> = (0..N)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "small indices")]
            let x = i as f64 / N as f64;
            0.5 - 0.5 * sonarithm_core::math::cos(core::f64::consts::TAU * x)
        })
        .collect();
    let per_second = rate as usize;
    let seconds = mono.len().div_ceil(per_second);
    let mut sums = vec![(0.0, 0_u32); seconds];
    let mut start = 0;
    while start + N <= mono.len() {
        let frame = &mono[start..start + N];
        #[expect(clippy::cast_precision_loss, reason = "N is small")]
        let rms = (frame.iter().map(|x| x * x).sum::<f64>() / N as f64).sqrt();
        if db(rms) > -80.0 {
            let windowed: Vec<f64> = frame.iter().zip(&window).map(|(x, w)| x * w).collect();
            let magnitudes = fft_magnitudes(&windowed);
            let (weighted, total) =
                magnitudes
                    .iter()
                    .enumerate()
                    .fold((0.0, 0.0), |(w, t), (k, m)| {
                        #[expect(clippy::cast_precision_loss, reason = "bin indices are small")]
                        let hz = k as f64 * f64::from(rate) / N as f64;
                        (w + hz * m, t + m)
                    });
            if total > 0.0 {
                let second = (start + N / 2) / per_second;
                if let Some(slot) = sums.get_mut(second) {
                    slot.0 += weighted / total;
                    slot.1 += 1;
                }
            }
        }
        start += N / 2;
    }
    sums.into_iter()
        .map(|(sum, count)| (count > 0).then(|| (sum / f64::from(count) * 10.0).round() / 10.0))
        .collect()
}

/// Magnitudes of the first half of a radix-2 DFT.
fn fft_magnitudes(input: &[f64]) -> Vec<f64> {
    let n = input.len();
    let (mut re, mut im) = (input.to_vec(), vec![0.0; n]);
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        #[expect(clippy::cast_precision_loss, reason = "small lengths")]
        let angle = -core::f64::consts::TAU / len as f64;
        for start in (0..n).step_by(len) {
            for k in 0..len / 2 {
                #[expect(clippy::cast_precision_loss, reason = "small indices")]
                let (wr, wi) = (
                    sonarithm_core::math::cos(angle * k as f64),
                    sonarithm_core::math::sin(angle * k as f64),
                );
                let (a, b) = (start + k, start + k + len / 2);
                let (tr, ti) = (re[b] * wr - im[b] * wi, re[b] * wi + im[b] * wr);
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
            }
        }
        len <<= 1;
    }
    // `sqrt` is exact in IEEE 754, unlike `hypot`, so the metric is the same everywhere.
    (0..n / 2)
        .map(|k| (re[k] * re[k] + im[k] * im[k]).sqrt())
        .collect()
}

/// Peak and RMS of the difference between two renders, in dBFS. Renders of different
/// lengths compare as a full-scale difference.
fn residual(a: &[f32], b: &[f32]) -> (f64, f64) {
    if a.len() != b.len() {
        return (0.0, 0.0);
    }
    let (peak, sum) = a.iter().zip(b).fold((0.0_f64, 0.0_f64), |(p, s), (x, y)| {
        let d = f64::from(x - y).abs();
        (p.max(d), s + d * d)
    });
    #[expect(clippy::cast_precision_loss, reason = "sample counts below 2^53")]
    let rms = (sum / a.len().max(1) as f64).sqrt();
    (db(peak), db(rms))
}

/// How a render compares with its reference.
fn verdict(
    reference: &Reference,
    actual: &Reference,
    tolerances: Tolerances,
) -> Result<String, String> {
    if actual.frames != reference.frames {
        return Err(format!(
            "length {} frames, reference {}",
            actual.frames, reference.frames
        ));
    }
    if actual.sha256 == reference.sha256 {
        return Ok("identical".to_owned());
    }
    let lufs = match (actual.integrated_lufs, reference.integrated_lufs) {
        (Some(a), Some(r)) => (a - r).abs(),
        (None, None) => 0.0,
        _ => f64::INFINITY,
    };
    if lufs > tolerances.lufs_delta {
        return Err(format!("ΔLUFS {lufs:.3} > {}", tolerances.lufs_delta));
    }
    let mut worst = 0.0_f64;
    for (a, r) in actual.centroid_hz.iter().zip(&reference.centroid_hz) {
        match (a, r) {
            (Some(a), Some(r)) if *r > 0.0 => worst = worst.max((a - r).abs() / r),
            (None, None) => {}
            _ => worst = f64::INFINITY,
        }
    }
    if worst > tolerances.centroid_relative {
        return Err(format!(
            "centroid change {:.2} % > {:.2} %",
            100.0 * worst,
            100.0 * tolerances.centroid_relative
        ));
    }
    Ok(format!(
        "hash changed within tolerance: ΔLUFS {lufs:.3}, centroid {:.3} %",
        100.0 * worst
    ))
}

fn check_all(root: &Path) -> Result<()> {
    let mut failures = Vec::new();
    for (name, (dir, case)) in cases(root)? {
        let rendered = render(&dir, &case, case.block_size)?;
        let Some(reference) = &case.reference else {
            failures.push(format!(
                "{name}: no reference yet (run `cargo xtask golden --update --reason ...`)"
            ));
            continue;
        };
        match verdict(reference, &rendered.reference, case.tolerances) {
            Ok(note) => println!("golden {name}: {note}"),
            Err(problem) => failures.push(format!("{name}: {problem}")),
        }
        for block in INVARIANCE_BLOCKS {
            let other = render(&dir, &case, block)?;
            let (peak, _) = residual(&rendered.samples, &other.samples);
            if other.reference.sha256 == rendered.reference.sha256 {
                println!("golden {name}: block {block} identical");
            } else if peak > case.tolerances.null_residual_dbfs {
                failures.push(format!(
                    "{name}: block {block} differs, residual {peak:.1} dBFS"
                ));
            } else {
                println!("golden {name}: block {block} within tolerance ({peak:.1} dBFS)");
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        bail!(
            "{} golden failure(s):\n  {}",
            failures.len(),
            failures.join("\n  ")
        )
    }
}

fn update(root: &Path, args: &[String]) -> Result<()> {
    let (reason, names) = match args {
        [flag, reason, names @ ..] if flag == "--reason" && !reason.trim().is_empty() => {
            (reason, names)
        }
        _ => bail!("--update needs --reason \"<why the output changed>\" (testing-strategy §5.3)"),
    };
    for (name, (dir, mut case)) in cases(root)? {
        if !names.is_empty() && !names.contains(&name) {
            continue;
        }
        let rendered = render(&dir, &case, case.block_size)?;
        match &case.reference {
            Some(old) => println!(
                "golden {name}: {}",
                verdict(old, &rendered.reference, case.tolerances).unwrap_or_else(|e| e)
            ),
            None => println!("golden {name}: new reference"),
        }
        case.reference = Some(rendered.reference);
        case.reason.clone_from(reason);
        let mut text = serde_json::to_string_pretty(&case)?;
        text.push('\n');
        std::fs::write(dir.join("case.json"), text)?;
    }
    Ok(())
}

/// One render set's summary: per case, its fingerprint.
type Summary = BTreeMap<String, Reference>;

fn render_set(root: &Path, out: &Path) -> Result<()> {
    std::fs::create_dir_all(out).with_context(|| format!("creating {}", out.display()))?;
    let mut summary = Summary::new();
    for (name, (dir, case)) in cases(root)? {
        let rendered = render(&dir, &case, case.block_size)?;
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: case.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut wav = hound::WavWriter::create(out.join(format!("{name}.wav")), spec)?;
        for sample in &rendered.samples {
            wav.write_sample(*sample)?;
        }
        wav.finalize()?;
        summary.insert(name, rendered.reference);
    }
    let mut text = serde_json::to_string_pretty(&summary)?;
    text.push('\n');
    std::fs::write(out.join("summary.json"), text)?;
    println!(
        "golden: {} case(s) written to {}",
        summary.len(),
        out.display()
    );
    Ok(())
}

fn read_wav(path: &Path) -> Result<Vec<f32>> {
    let mut reader =
        hound::WavReader::open(path).with_context(|| format!("reading {}", path.display()))?;
    reader
        .samples::<f32>()
        .map(|s| s.map_err(Into::into))
        .collect()
}

fn compare_sets(reference: &Path, sets: &[String]) -> Result<()> {
    let tolerances = Tolerances {
        null_residual_dbfs: -90.0,
        lufs_delta: 0.1,
        centroid_relative: 0.01,
    };
    let base: Summary =
        serde_json::from_str(&std::fs::read_to_string(reference.join("summary.json"))?)?;
    let mut table = String::from(
        "| Case | Set | Identical | Peak residual (dBFS) | RMS residual (dBFS) | ΔLUFS |\n|---|---|---|---|---|---|\n",
    );
    let mut failures = Vec::new();
    for set in sets {
        let dir = Path::new(set);
        let other: Summary =
            serde_json::from_str(&std::fs::read_to_string(dir.join("summary.json"))?)?;
        for (name, expected) in &base {
            let Some(actual) = other.get(name) else {
                failures.push(format!("{set}: {name} missing"));
                continue;
            };
            let a = read_wav(&reference.join(format!("{name}.wav")))?;
            let b = read_wav(&dir.join(format!("{name}.wav")))?;
            let (peak, rms) = residual(&a, &b);
            let lufs = match (expected.integrated_lufs, actual.integrated_lufs) {
                (Some(x), Some(y)) => (x - y).abs(),
                _ => 0.0,
            };
            let identical = expected.sha256 == actual.sha256;
            let label = dir
                .file_name()
                .map_or_else(|| set.clone(), |n| n.to_string_lossy().into_owned());
            let _ = writeln!(
                table,
                "| {name} | {label} | {identical} | {peak:.1} | {rms:.1} | {lufs:.4} |"
            );
            if !identical && (peak > tolerances.null_residual_dbfs || lufs > tolerances.lufs_delta)
            {
                failures.push(format!(
                    "{set}: {name} residual {peak:.1} dBFS, ΔLUFS {lufs:.3}"
                ));
            }
        }
    }
    println!("{table}");
    if failures.is_empty() {
        Ok(())
    } else {
        bail!("PB-06 not met:\n  {}", failures.join("\n  "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(sha: &str, lufs: f64, centroid: f64) -> Reference {
        Reference {
            platform: "test".to_owned(),
            sha256: sha.to_owned(),
            frames: 10,
            integrated_lufs: Some(lufs),
            true_peak_dbtp: -3.0,
            peak_dbfs: -3.0,
            rms_dbfs: -20.0,
            centroid_hz: vec![Some(centroid), None],
        }
    }

    const TOLERANCES: Tolerances = Tolerances {
        null_residual_dbfs: -90.0,
        lufs_delta: 0.1,
        centroid_relative: 0.01,
    };

    #[test]
    fn verdicts_follow_the_tolerances() {
        let base = reference("a", -20.0, 1_000.0);
        assert_eq!(
            verdict(&base, &base, TOLERANCES).as_deref(),
            Ok("identical")
        );
        assert!(verdict(&base, &reference("b", -20.05, 1_005.0), TOLERANCES).is_ok());
        assert!(verdict(&base, &reference("b", -20.2, 1_000.0), TOLERANCES).is_err());
        assert!(verdict(&base, &reference("b", -20.0, 1_020.0), TOLERANCES).is_err());
    }

    #[test]
    fn residuals_measure_the_difference() {
        let a = [0.5_f32, 0.5, 0.0, 0.0];
        assert_eq!(residual(&a, &a).0, db(0.0));
        let b = [0.5_f32, 0.4, 0.0, 0.0];
        let (peak, _) = residual(&a, &b);
        assert!((peak - db(0.1)).abs() < 1e-3);
        assert_eq!(
            residual(&a, &b[..2]),
            (0.0, 0.0),
            "length mismatch is full scale"
        );
    }

    #[test]
    fn a_sine_has_its_frequency_as_centroid() {
        let samples: Vec<f32> = (0..48_000)
            .flat_map(|i| {
                #[expect(clippy::cast_possible_truncation, reason = "test signal")]
                let s = sonarithm_dsp::sine_turns(f64::from(i) * 1_000.0 / 48_000.0) as f32 * 0.5;
                [s, s]
            })
            .collect();
        let centroid = centroids(&samples, 48_000)[0].unwrap();
        assert!((centroid - 1_000.0).abs() < 20.0, "{centroid}");
    }
}

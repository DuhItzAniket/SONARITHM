// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A02-T04: the offline determinism check across operating systems (PB-06, ADR-0013).
//! `render-set` writes every variant of the benchmark render plus a summary; `compare`
//! checks render sets from several machines against the first one.

use std::error::Error;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::voice::MathMode;
use crate::{Rendered, render_offline, write_wav};

/// (graph, oscillator maths). `std` sines call the platform maths library; `poly` sines and
/// everything else are plain IEEE 754 arithmetic plus `libm`.
const VARIANTS: [(&str, MathMode); 3] = [
    ("own", MathMode::Poly),
    ("own", MathMode::Std),
    ("firewheel", MathMode::Poly),
];
/// PB-06 tolerances (12 §8): peak null residual ≤ −90 dBFS and ΔLUFS ≤ 0.1.
const MAX_RESIDUAL_DBFS: f64 = -90.0;
const MAX_DELTA_LUFS: f64 = 0.1;

fn file_name(graph: &str, math: MathMode) -> String {
    format!(
        "{graph}-{}.wav",
        if math == MathMode::Std { "std" } else { "poly" }
    )
}

pub fn render_set(dir: &Path, seconds: u32) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(dir)?;
    let mut summary = format!(
        "os={} arch={}\n",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    for (graph, math) in VARIANTS {
        let r: Rendered = render_offline(graph, 256, seconds, math)?;
        let name = file_name(graph, math);
        write_wav(&dir.join(&name), &r.samples)?;
        let _ = writeln!(
            summary,
            "{name} lufs={:.4} audio_sha256={} score_sha256={}",
            r.lufs, r.audio_hash, r.score_hash
        );
    }
    std::fs::write(dir.join("summary.txt"), &summary)?;
    print!("{summary}");
    Ok(())
}

/// The render set's folder name only: no absolute user paths in output (ADR-0012).
fn label(dir: &Path) -> String {
    dir.file_name().map_or_else(
        || dir.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

fn read_wav(path: &Path) -> Result<Vec<f32>, Box<dyn Error>> {
    Ok(hound::WavReader::open(path)?
        .samples::<f32>()
        .collect::<Result<_, _>>()?)
}

fn summary_field<'a>(summary: &'a str, file: &str, key: &str) -> Option<&'a str> {
    let line = summary.lines().find(|l| l.starts_with(file))?;
    line.split(' ')
        .find_map(|kv| kv.strip_prefix(key)?.strip_prefix('='))
}

/// Compares every render set in `dirs` with the first. Prints a Markdown table and fails if
/// any variant misses the PB-06 tolerances or the Score bytes differ.
pub fn compare(dirs: &[PathBuf]) -> Result<(), Box<dyn Error>> {
    let reference = dirs
        .first()
        .ok_or("compare needs at least two render sets")?;
    let ref_summary = std::fs::read_to_string(reference.join("summary.txt"))?;
    println!(
        "Reference: `{}` ({})\n",
        label(reference),
        ref_summary.lines().next().unwrap_or_default()
    );
    println!(
        "| Variant | Render set | Platform | Bit-identical | Peak null residual | ΔLUFS | Score bytes identical | PB-06 |"
    );
    println!("|---|---|---|---|---|---|---|---|");
    let mut failures = 0;
    for (graph, math) in VARIANTS {
        let name = file_name(graph, math);
        let a = read_wav(&reference.join(&name))?;
        for dir in &dirs[1..] {
            let summary = std::fs::read_to_string(dir.join("summary.txt"))?;
            let b = read_wav(&dir.join(&name))?;
            let peak = a
                .iter()
                .zip(&b)
                .fold(0.0f64, |m, (x, y)| m.max(f64::from((x - y).abs())));
            let residual = if a.len() != b.len() {
                f64::INFINITY
            } else if peak == 0.0 {
                f64::NEG_INFINITY
            } else {
                20.0 * peak.log10()
            };
            let lufs = |s: &str| {
                summary_field(s, &name, "lufs")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(f64::NAN)
            };
            let delta = (lufs(&ref_summary) - lufs(&summary)).abs();
            let score_same = summary_field(&ref_summary, &name, "score_sha256")
                == summary_field(&summary, &name, "score_sha256");
            let pass = residual <= MAX_RESIDUAL_DBFS && delta <= MAX_DELTA_LUFS && score_same;
            failures += usize::from(!pass);
            let residual_text = if residual == f64::NEG_INFINITY {
                "−∞ (identical)".to_owned()
            } else {
                format!("{residual:.1} dBFS")
            };
            println!(
                "| {name} | `{}` | {} | {} | {residual_text} | {delta:.4} | {} | {} |",
                label(dir),
                summary.lines().next().unwrap_or_default(),
                if peak == 0.0 && a.len() == b.len() {
                    "yes"
                } else {
                    "no"
                },
                if score_same { "yes" } else { "no" },
                if pass { "pass" } else { "FAIL" },
            );
        }
    }
    if failures > 0 {
        Err(format!("{failures} comparison(s) outside PB-06").into())
    } else {
        Ok(())
    }
}

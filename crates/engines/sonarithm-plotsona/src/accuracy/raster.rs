// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The C02 accuracy harness (C02-T08, DS-007's raster part): the line and bar charts,
//! rendered at 1x, 1.5x and 2x and without anti-aliasing (`generate/raster.py`), are read
//! through anchors a person would click (the SVG's outermost ticks and category labels,
//! scaled to the image) and scored against their ground truth. Each PNG is also re-encoded
//! as JPEG at quality 90 and 60 in memory, with the `image` crate the raster tier already
//! uses, so JPEG artefacts are measured without committing more files.
//!
//! Raster series are named `Series 1`, `Series 2`… (no legend OCR), so each is first
//! paired with the truth series it is closest to. Errors are relative to the chart's
//! largest absolute truth value, as for the vector tiers (a digitizing error is measured
//! against the axis); a truth value with no matching point counts as a full error.
//!
//! The test fails if a **clean** render (PNG, any resolution, with or without
//! anti-aliasing) has the wrong class, a median error above the C02 bar of 2 %
//! (notes/06 §8, SDR-42), or an RMS-F1 below 0.95. JPEG renders are reported, not held to
//! the bar. The report goes to `SONARITHM_ACCURACY_REPORT` with `-raster` before its
//! extension (the CI job `chart-accuracy` uploads both).

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::PathBuf;

use serde_json::Value;

use super::{Entry, error, found_entries, percent, rms_f1, truth_entries};
use crate::corpus::folder;
use crate::{ExtractOptions, extract};

/// The C02 bar: a median relative error of 2 % (notes/06 §8).
const BAR: f64 = 0.02;
/// The structure bar for clean renders.
const F1_BAR: f64 = 0.95;
/// The chart types the C02 bar covers.
const KINDS: [&str; 2] = ["line", "bar"];
/// The JPEG qualities measured.
const QUALITIES: [u8; 2] = [90, 60];

/// One render's scores.
#[derive(Debug)]
struct Score {
    name: String,
    variant: String,
    clean: bool,
    class_ok: bool,
    points: usize,
    median: f64,
    worst: f64,
    rms_f1: f64,
}

/// `vl-line@1.5x-crisp` → (`vl-line`, 1.5).
fn parse_name(stem: &str) -> Option<(&str, f64)> {
    let (svg, rest) = stem.split_once('@')?;
    let scale = rest.split_once('x')?.0.parse().ok()?;
    Some((svg, scale))
}

fn jpeg(png: &[u8], quality: u8) -> Option<Vec<u8>> {
    let image = image::load_from_memory(png).ok()?.to_rgb8();
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality)
        .encode_image(&image)
        .ok()?;
    Some(out)
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    values
        .get(values.len() / 2)
        .copied()
        .unwrap_or(f64::INFINITY)
}

/// Renames each found series to the truth series it is closest to (median error over the
/// x values both have), each truth series used once, in order of closeness.
fn pair_series(truth: &[Entry], found: &[Entry], top: f64) -> Vec<Entry> {
    let truth_names: BTreeSet<&str> = truth.iter().map(|e| e.0.as_str()).collect();
    let found_names: BTreeSet<&str> = found.iter().map(|e| e.0.as_str()).collect();
    let mut pairs: Vec<(f64, &str, &str)> = Vec::new();
    for f in &found_names {
        for t in &truth_names {
            let mut errors: Vec<f64> = truth
                .iter()
                .filter(|e| e.0 == *t)
                .filter_map(|(_, x, y)| {
                    found
                        .iter()
                        .find(|g| g.0 == *f && g.1 == *x)
                        .map(|g| error(g.2, *y, Some(top)))
                })
                .collect();
            pairs.push((median(&mut errors), f, t));
        }
    }
    pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut used_found = BTreeSet::new();
    let mut used_truth = BTreeSet::new();
    let mut rename = std::collections::BTreeMap::new();
    for (_, f, t) in pairs {
        if used_found.contains(f) || used_truth.contains(t) {
            continue;
        }
        used_found.insert(f);
        used_truth.insert(t);
        rename.insert(f, t);
    }
    found
        .iter()
        .map(|(s, x, y)| {
            let name = rename
                .get(s.as_str())
                .map_or_else(|| s.clone(), |t| (*t).to_owned());
            (name, x.clone(), *y)
        })
        .collect()
}

fn score(stem: &str, variant: &str, clean: bool, bytes: &[u8], truth: &Value) -> Score {
    let mut out = Score {
        name: stem.to_owned(),
        variant: variant.to_owned(),
        clean,
        class_ok: false,
        points: 0,
        median: f64::INFINITY,
        worst: f64::INFINITY,
        rms_f1: 0.0,
    };
    let Some((svg, scale)) = parse_name(stem) else {
        return out;
    };
    let Ok(drawing) = std::fs::read(folder("svg").join(format!("{svg}.svg"))) else {
        return out;
    };
    let Some(hand) = crate::svg::hand_calibration(&drawing, scale) else {
        return out;
    };
    let Ok(calibration) = serde_json::from_value(hand) else {
        return out;
    };
    let options = ExtractOptions::new().with_calibration(calibration);
    let Ok(read) = extract(bytes, stem, &options) else {
        return out;
    };
    let truth_entries = truth_entries(truth);
    let top = truth_entries.iter().fold(0.0_f64, |m, e| m.max(e.2.abs()));
    let found = pair_series(&truth_entries, &found_entries(&read.chart), top);
    let mut errors: Vec<f64> = truth_entries
        .iter()
        .map(|(s, x, y)| {
            found
                .iter()
                .find(|g| g.0 == *s && g.1 == *x)
                .map_or(f64::INFINITY, |g| error(g.2, *y, Some(top)))
        })
        .collect();
    let (f1, worst) = rms_f1(&truth_entries, &found, Some(top));
    out.class_ok = read.chart.series.iter().all(|s| {
        sonarithm_data::provenance::class_name(s.provenance.class) == "cv_estimate"
            && s.provenance.confidence.is_some()
    });
    out.points = found.len();
    out.median = median(&mut errors);
    out.worst = worst;
    out.rms_f1 = f1;
    out
}

fn scores() -> Vec<Score> {
    let dir = folder("raster");
    let mut truths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    truths.retain(|p| p.to_string_lossy().ends_with(".truth.json"));
    truths.sort();
    let mut out = Vec::new();
    for path in truths {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(truth) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        if !KINDS.contains(&truth["chart_type"].as_str().unwrap_or_default()) {
            continue;
        }
        let file = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        let stem = file.trim_end_matches(".truth.json").to_owned();
        let Ok(png) = std::fs::read(dir.join(format!("{stem}.png"))) else {
            continue;
        };
        let crisp = stem.ends_with("-crisp");
        out.push(score(
            &stem,
            if crisp { "PNG, crisp" } else { "PNG" },
            true,
            &png,
            &truth,
        ));
        if crisp {
            continue;
        }
        for quality in QUALITIES {
            if let Some(bytes) = jpeg(&png, quality) {
                out.push(score(
                    &stem,
                    &format!("JPEG q{quality}"),
                    false,
                    &bytes,
                    &truth,
                ));
            }
        }
    }
    out
}

fn report(scores: &[Score]) -> String {
    let mut text = String::from(
        "# C02 raster chart accuracy (DS-007, raster part)\n\nClean renders are held to a median error of 2 % and an RMS-F1 of 0.95; JPEG renders are reported only.\n\n| Chart | Variant | Class | Points | Median error | Worst error | RMS-F1 |\n|---|---|---|---|---|---|---|\n",
    );
    for s in scores {
        let _ = writeln!(
            text,
            "| {} | {} | {} | {} | {} | {} | {:.4} |",
            s.name,
            s.variant,
            if s.class_ok { "cv_estimate" } else { "wrong" },
            s.points,
            percent(s.median),
            percent(s.worst),
            s.rms_f1,
        );
    }
    text.push_str(
        "\n| Variant | Renders | Worst median error | Lowest RMS-F1 |\n|---|---|---|---|\n",
    );
    let variants: BTreeSet<&str> = scores.iter().map(|s| s.variant.as_str()).collect();
    for variant in variants {
        let of: Vec<&Score> = scores.iter().filter(|s| s.variant == variant).collect();
        let worst = of.iter().map(|s| s.median).fold(0.0_f64, f64::max);
        let f1 = of.iter().map(|s| s.rms_f1).fold(1.0_f64, f64::min);
        let _ = writeln!(
            text,
            "| {variant} | {} | {} | {f1:.4} |",
            of.len(),
            percent(worst)
        );
    }
    text
}

/// C02-T08 acceptance (C02 exit, SDR-42): every clean raster line and bar render, at 1x,
/// 1.5x and 2x and without anti-aliasing, reads as `cv_estimate` with a confidence, a
/// median error of at most 2 % and an RMS-F1 of at least 0.95; JPEG renders are reported.
#[test]
fn the_raster_corpus_meets_the_c02_accuracy_bar() {
    let scores = scores();
    let text = report(&scores);
    if let Ok(path) = std::env::var("SONARITHM_ACCURACY_REPORT") {
        let path = PathBuf::from(path);
        let raster = path.with_file_name(format!(
            "{}-raster.md",
            path.file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        ));
        let _ = std::fs::write(raster, &text);
    }
    println!("{text}");
    let clean: Vec<&Score> = scores.iter().filter(|s| s.clean).collect();
    assert!(clean.len() >= 24, "only {} clean renders", clean.len());
    for s in clean {
        assert!(s.class_ok, "{} ({}): provenance class", s.name, s.variant);
        assert!(
            s.median <= BAR,
            "{} ({}): median error {}",
            s.name,
            s.variant,
            s.median
        );
        assert!(
            s.rms_f1 >= F1_BAR,
            "{} ({}): RMS-F1 {}",
            s.name,
            s.variant,
            s.rms_f1
        );
    }
}

#[test]
fn render_names_give_their_svg_and_scale() {
    assert_eq!(parse_name("vl-line@1x"), Some(("vl-line", 1.0)));
    assert_eq!(parse_name("d3-bar@1.5x"), Some(("d3-bar", 1.5)));
    assert_eq!(
        parse_name("plotly-line@1x-crisp"),
        Some(("plotly-line", 1.0))
    );
    assert_eq!(parse_name("no-scale"), None);
}

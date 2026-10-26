// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The C01 accuracy harness (C01-T11, DS-007): every corpus chart is extracted and scored
//! against its ground truth with three metrics.
//!
//! - **Relative error.** For exact tiers each value's error relative to its own size; for
//!   calibrated (`vector`) tiers relative to the chart's largest absolute value, since a
//!   digitizing error is measured against the axis. The C01 bar is 0.5 % (notes/06 §8).
//! - **RMS-F1** for table structure (after the Relative Mapping Similarity of `DePlot`): each
//!   (series, x, y) entry of the truth is matched to the extracted entry with the same
//!   series and x (numbers and times within 0.5 % of the x span), scoring
//!   `1 − min(1, relative error)`; precision and recall divide the summed scores by the
//!   extracted and true entry counts. Keys must match exactly, where `DePlot` uses a
//!   normalised edit distance, so this is the stricter form (TODO(verify) against
//!   the reference implementation of `DePlot` before comparing numbers across papers).
//! - **Resampled curve error** for lines and areas on a numeric or time axis: each series is
//!   resampled at 101 evenly spaced x positions over the true range by linear interpolation,
//!   and the largest difference is taken relative to the chart's largest value.
//!
//! The test fails if a chart has the wrong provenance class, an error above 0.5 % or an
//! RMS-F1 below 0.99, and writes a Markdown report to `SONARITHM_ACCURACY_REPORT` when that
//! variable is set (the CI job `chart-accuracy` uploads it).

use std::fmt::Write as _;
use std::path::PathBuf;

use serde_json::Value;

use crate::calibrate::ticks::parse_date;
use crate::model::{ChartData, XValue};
use crate::{ExtractOptions, extract};

/// The C01 bar: 0.5 % relative error.
const BAR: f64 = 0.005;

/// One chart's scores.
#[derive(Debug)]
struct Score {
    name: String,
    tier: &'static str,
    class: String,
    class_ok: bool,
    points: usize,
    max_error: f64,
    rms_f1: f64,
    curve: Option<f64>,
}

/// One entry of a table: series, x, y.
type Entry = (String, XValue, f64);

fn x_number(x: &XValue) -> Option<f64> {
    match x {
        XValue::Number(n) => Some(*n),
        #[expect(
            clippy::cast_precision_loss,
            reason = "milliseconds since 1970 fit in f64"
        )]
        XValue::Time(t) => Some(*t as f64),
        XValue::Category(_) => None,
    }
}

fn truth_entries(truth: &Value) -> Vec<Entry> {
    let mut out = Vec::new();
    for series in truth["series"].as_array().into_iter().flatten() {
        let name = series["name"].as_str().unwrap_or_default().to_owned();
        let xs = series["x"].as_array().cloned().unwrap_or_default();
        let ys = series["y"].as_array().cloned().unwrap_or_default();
        for (x, y) in xs.iter().zip(&ys) {
            let x = match truth["x_kind"].as_str() {
                Some("number") => XValue::Number(x.as_f64().unwrap_or(f64::NAN)),
                Some("date") => XValue::Time(x.as_str().and_then(parse_date).unwrap_or(i64::MIN)),
                _ => XValue::Category(x.as_str().map_or_else(|| x.to_string(), str::to_owned)),
            };
            out.push((name.clone(), x, y.as_f64().unwrap_or(f64::NAN)));
        }
    }
    out
}

fn found_entries(chart: &ChartData) -> Vec<Entry> {
    chart
        .series
        .iter()
        .flat_map(|s| {
            s.points
                .iter()
                .map(|p| (s.name.clone(), p.x.clone(), p.y.unwrap_or(f64::NAN)))
        })
        .collect()
}

/// The error of `found` against `truth`, relative to `scale` (or to `truth` itself when
/// `scale` is `None`); a missing value counts as a full error.
fn error(found: f64, truth: f64, scale: Option<f64>) -> f64 {
    let scale = scale.unwrap_or(truth.abs());
    let e = if scale == 0.0 {
        found.abs()
    } else {
        ((found - truth) / scale).abs()
    };
    if e.is_nan() { f64::INFINITY } else { e }
}

fn same_x(a: &XValue, b: &XValue, span: f64) -> bool {
    match (x_number(a), x_number(b)) {
        (Some(a), Some(b)) => (a - b).abs() <= BAR * span.max(1.0),
        _ => a == b,
    }
}

/// RMS-F1 and the largest matched error.
fn rms_f1(truth: &[Entry], found: &[Entry], scale: Option<f64>) -> (f64, f64) {
    let span = truth
        .iter()
        .filter_map(|(_, x, _)| x_number(x))
        .fold(0.0_f64, |m, v| m.max(v.abs()));
    let mut used = vec![false; found.len()];
    let mut total = 0.0;
    let mut worst = 0.0_f64;
    for (series, x, y) in truth {
        let hit = found
            .iter()
            .enumerate()
            .find(|(i, (s, fx, _))| !used[*i] && s == series && same_x(fx, x, span));
        match hit {
            Some((i, (_, _, fy))) => {
                used[i] = true;
                let e = error(*fy, *y, scale);
                worst = worst.max(e);
                total += 1.0 - e.min(1.0);
            }
            None => worst = f64::INFINITY,
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "entry counts are small")]
    let (precision, recall) = (
        total / found.len().max(1) as f64,
        total / truth.len().max(1) as f64,
    );
    let f1 = if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    };
    (f1, worst)
}

/// Linear interpolation of sorted `(x, y)` points at `x`.
fn interpolate(points: &[(f64, f64)], x: f64) -> f64 {
    let Some(i) = points.iter().position(|p| p.0 >= x) else {
        return points.last().map_or(f64::NAN, |p| p.1);
    };
    if i == 0 || points[i].0 <= x {
        return points[i].1;
    }
    let (a, b) = (points[i - 1], points[i]);
    a.1 + (b.1 - a.1) * (x - a.0) / (b.0 - a.0)
}

/// The resampled curve error, for lines and areas on a numeric or time axis.
fn curve_error(truth: &[Entry], found: &[Entry], y_scale: f64) -> Option<f64> {
    let mut worst = 0.0_f64;
    let names: Vec<&String> = {
        let mut n: Vec<&String> = truth.iter().map(|e| &e.0).collect();
        n.dedup();
        n
    };
    for name in names {
        let curve = |entries: &[Entry]| {
            let mut c: Vec<(f64, f64)> = entries
                .iter()
                .filter(|e| &e.0 == name && e.2.is_finite())
                .filter_map(|e| Some((x_number(&e.1)?, e.2)))
                .collect();
            c.sort_by(|a, b| a.0.total_cmp(&b.0));
            c
        };
        let (t, f) = (curve(truth), curve(found));
        let (first, last) = (t.first()?.0, t.last()?.0);
        if f.is_empty() {
            return Some(f64::INFINITY);
        }
        for k in 0..=100 {
            let x = first + (last - first) * f64::from(k) / 100.0;
            let e = error(interpolate(&f, x), interpolate(&t, x), Some(y_scale));
            worst = worst.max(e);
        }
    }
    Some(worst)
}

fn score(tier: &'static str, name: &str, chart_path: &PathBuf, truth: &Value) -> Score {
    let bytes = std::fs::read(chart_path).unwrap_or_default();
    let extraction = extract(&bytes, name, &ExtractOptions::new())
        .unwrap_or_else(|d| unreachable!("{name}: {d}"));
    let chart = &extraction.chart;
    let class = truth["provenance"].as_str().unwrap_or_default().to_owned();
    let class_ok = chart
        .series
        .iter()
        .all(|s| sonarithm_data::provenance::class_name(s.provenance.class) == class);
    let (t, f) = (truth_entries(truth), found_entries(chart));
    let y_scale = t.iter().fold(0.0_f64, |m, e| m.max(e.2.abs()));
    let scale = (class != "exact").then_some(y_scale);
    let (f1, max_error) = rms_f1(&t, &f, scale);
    let curve = if matches!(truth["chart_type"].as_str(), Some("line" | "area"))
        && matches!(truth["x_kind"].as_str(), Some("number" | "date"))
    {
        curve_error(&t, &f, y_scale)
    } else {
        None
    };
    Score {
        name: name.to_owned(),
        tier,
        class,
        class_ok,
        points: t.len(),
        max_error,
        rms_f1: f1,
        curve,
    }
}

fn scores() -> Vec<Score> {
    let mut out = Vec::new();
    for (tier, folder) in [
        ("office", "office"),
        ("chart JSON", "chartjson"),
        ("SVG", "svg"),
        ("PDF", "pdf"),
    ] {
        let dir = crate::corpus::folder(folder);
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .map(|d| d.filter_map(Result::ok).map(|e| e.path()).collect())
            .unwrap_or_default();
        files.sort();
        for truth_path in files
            .iter()
            .filter(|p| p.to_string_lossy().ends_with(".truth.json"))
        {
            let file = truth_path.file_name().unwrap_or_default().to_string_lossy();
            let stem = file.trim_end_matches(".truth.json");
            let Some(chart) = files.iter().find(|p| {
                let f = p.file_name().unwrap_or_default().to_string_lossy();
                f.starts_with(&format!("{stem}.")) && !f.ends_with(".truth.json")
            }) else {
                continue;
            };
            let truth: Value = std::fs::read_to_string(truth_path)
                .ok()
                .and_then(|t| serde_json::from_str(&t).ok())
                .unwrap_or(Value::Null);
            let name = chart.file_name().unwrap_or_default().to_string_lossy();
            out.push(score(tier, &name, chart, &truth));
        }
    }
    out
}

fn percent(e: f64) -> String {
    if e.is_finite() {
        format!("{:.4} %", e * 100.0)
    } else {
        "missing".to_owned()
    }
}

fn report(scores: &[Score]) -> String {
    let mut text = String::from(
        "# C01 chart accuracy (DS-007)\n\n| Chart | Tier | Class | Points | Max error | RMS-F1 | Curve error |\n|---|---|---|---|---|---|---|\n",
    );
    for s in scores {
        let _ = writeln!(
            text,
            "| {} | {} | {}{} | {} | {} | {:.4} | {} |",
            s.name,
            s.tier,
            s.class,
            if s.class_ok { "" } else { " (wrong)" },
            s.points,
            percent(s.max_error),
            s.rms_f1,
            s.curve.map_or_else(|| "—".to_owned(), percent),
        );
    }
    text.push_str("\n| Tier | Charts | Points | Worst error | Lowest RMS-F1 | Worst curve error |\n|---|---|---|---|---|---|\n");
    for tier in ["office", "chart JSON", "SVG", "PDF"] {
        let of: Vec<&Score> = scores.iter().filter(|s| s.tier == tier).collect();
        let worst = of.iter().map(|s| s.max_error).fold(0.0_f64, f64::max);
        let f1 = of.iter().map(|s| s.rms_f1).fold(1.0_f64, f64::min);
        let curve = of.iter().filter_map(|s| s.curve).fold(0.0_f64, f64::max);
        let points: usize = of.iter().map(|s| s.points).sum();
        let _ = writeln!(
            text,
            "| {tier} | {} | {points} | {} | {f1:.4} | {} |",
            of.len(),
            percent(worst),
            percent(curve)
        );
    }
    text
}

/// C01-T11 acceptance (C01 exit): the corpus has at least 50 charts across the exact and
/// vector tiers, and every one extracts with the right provenance class, within 0.5 %
/// relative error and with an RMS-F1 of at least 0.99.
#[test]
fn the_corpus_meets_the_c01_accuracy_bar() {
    let scores = scores();
    let text = report(&scores);
    if let Ok(path) = std::env::var("SONARITHM_ACCURACY_REPORT") {
        let _ = std::fs::write(path, &text);
    }
    println!("{text}");
    assert!(scores.len() >= 50, "only {} charts", scores.len());
    for s in &scores {
        assert!(s.class_ok, "{}: provenance class", s.name);
        assert!(s.max_error <= BAR, "{}: error {}", s.name, s.max_error);
        assert!(s.rms_f1 >= 0.99, "{}: RMS-F1 {}", s.name, s.rms_f1);
        assert!(
            s.curve.is_none_or(|c| c <= BAR),
            "{}: curve {:?}",
            s.name,
            s.curve
        );
    }
    insta::assert_snapshot!(text);
}

#[test]
fn the_metrics_score_known_tables() {
    let truth = vec![
        ("a".to_owned(), XValue::Number(0.0), 10.0),
        ("a".to_owned(), XValue::Number(1.0), 20.0),
    ];
    let mut found = truth.clone();
    assert_eq!(rms_f1(&truth, &found, None), (1.0, 0.0));
    found[1].2 = 22.0;
    let (f1, worst) = rms_f1(&truth, &found, None);
    assert!((worst - 0.1).abs() < 1e-12);
    assert!((f1 - 0.95).abs() < 1e-12);
    found.pop();
    let (f1, worst) = rms_f1(&truth, &found, None);
    assert!(worst.is_infinite());
    assert!((f1 - 2.0 * 1.0 * 0.5 / 1.5).abs() < 1e-12, "{f1}");
    let curve = curve_error(&truth, &truth, 20.0);
    assert_eq!(curve, Some(0.0));
}

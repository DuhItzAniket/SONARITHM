// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Test helper: checks a folder of DS-007 corpus charts against their ground truth.
//!
//! Each chart `name.ext` has a `name.truth.json` beside it with the chart type, title,
//! provenance class, the kind of x (`category`, `number` or `date`) and the series with
//! their x and y values. A chart passes when every series, name, x and class matches and
//! every y is within `tolerance` relative error (0 for exact tiers). For calibrated tiers
//! the error of a value is taken relative to the largest absolute value of the chart's
//! ground truth, as digitizing errors are measured against the axis rather than each value.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::calibrate::ticks::parse_date;
use crate::model::XValue;
use crate::{ExtractOptions, extract};

/// The corpus folder `name` of this crate.
pub(crate) fn folder(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus")
        .join(name)
}

/// The largest relative error found per chart, after checking structure; panics with the
/// chart's name on any mismatch. Returns the number of charts checked.
#[expect(
    clippy::too_many_lines,
    reason = "one structural and numeric check per chart"
)]
pub(crate) fn check(dir: &Path, tolerance: f64) -> Vec<(String, f64)> {
    let mut truths: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("corpus folder")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.to_string_lossy().ends_with(".truth.json"))
        .collect();
    truths.sort();
    let mut results = Vec::new();
    for truth_path in truths {
        let file = truth_path.file_name().expect("name").to_string_lossy();
        let name = file.trim_end_matches(".truth.json").to_owned();
        let chart_path = std::fs::read_dir(dir)
            .expect("corpus folder")
            .map(|e| e.expect("entry").path())
            .find(|p| {
                let f = p
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_default();
                f.starts_with(&format!("{name}.")) && !f.ends_with(".truth.json")
            })
            .unwrap_or_else(|| unreachable!("{name}: no chart file beside its truth"));
        let truth: Value =
            serde_json::from_str(&std::fs::read_to_string(&truth_path).expect("truth"))
                .expect("json");
        let bytes = std::fs::read(&chart_path).expect("chart");
        let extraction = extract(&bytes, &name, &ExtractOptions::new());
        assert!(extraction.is_ok(), "{name}: {:?}", extraction.err());
        let extraction = extraction.expect("checked");
        let chart = &extraction.chart;
        let found_type = serde_json::to_value(chart.chart_type).expect("type");
        assert_eq!(
            found_type.as_str(),
            truth["chart_type"].as_str(),
            "{name}: chart type"
        );
        assert_eq!(
            chart.title.as_deref(),
            truth["title"].as_str(),
            "{name}: title"
        );
        let class = truth["provenance"].as_str().expect("class");
        let series = truth["series"].as_array().expect("series");
        assert_eq!(
            chart
                .series
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            series
                .iter()
                .map(|s| s["name"].as_str().unwrap_or_default())
                .collect::<Vec<_>>(),
            "{name}: series"
        );
        let mut worst = 0.0_f64;
        let y_scale = series
            .iter()
            .flat_map(|s| s["y"].as_array().into_iter().flatten())
            .filter_map(Value::as_f64)
            .fold(0.0_f64, |m, v| m.max(v.abs()));
        for (found, expected) in chart.series.iter().zip(series) {
            assert_eq!(
                sonarithm_data::provenance::class_name(found.provenance.class),
                class,
                "{name}: provenance"
            );
            let xs = expected["x"].as_array().expect("x");
            let ys = expected["y"].as_array().expect("y");
            assert_eq!(
                found.points.len(),
                ys.len(),
                "{name}/{}: point count",
                found.name
            );
            for ((point, x), y) in found.points.iter().zip(xs).zip(ys) {
                let expected_x = match truth["x_kind"].as_str() {
                    Some("number") => XValue::Number(x.as_f64().expect("number")),
                    Some("date") => {
                        XValue::Time(parse_date(x.as_str().expect("date")).expect("date"))
                    }
                    _ => XValue::Category(x.as_str().map_or_else(|| x.to_string(), str::to_owned)),
                };
                match (&point.x, &expected_x) {
                    (XValue::Number(a), XValue::Number(b)) if tolerance > 0.0 => {
                        let span = xs
                            .iter()
                            .filter_map(Value::as_f64)
                            .fold(0.0_f64, |m, v| m.max(v.abs()));
                        assert!(
                            (a - b).abs() <= tolerance * span.max(1.0),
                            "{name}: x {a} vs {b}"
                        );
                    }
                    _ => assert_eq!(point.x, expected_x, "{name}: x"),
                }
                let y = y.as_f64().expect("y");
                let found_y = point.y.unwrap_or(f64::NAN);
                let scale = if tolerance > 0.0 { y_scale } else { y.abs() };
                let error = if scale == 0.0 {
                    found_y.abs()
                } else {
                    ((found_y - y) / scale).abs()
                };
                worst = worst.max(if error.is_nan() { f64::INFINITY } else { error });
            }
        }
        assert!(
            worst <= tolerance,
            "{name}: relative error {worst} above {tolerance}"
        );
        let warnings: Vec<_> = extraction
            .report
            .diagnostics
            .iter()
            .filter(|d| d.severity >= sonarithm_core::Severity::Warning)
            .collect();
        assert!(warnings.is_empty(), "{name}: {warnings:?}");
        results.push((name, worst));
    }
    results
}

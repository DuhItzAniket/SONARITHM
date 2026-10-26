// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The extraction report: what was read, by which tier, with which provenance (10 §5).
//!
//! The report is what the CLI's `inspect` prints and what the app announces ("146 points:
//! 0 exact, 146 estimated, median confidence 0.82", C01-T12). It serializes to JSON for
//! `--json` and the Python package.

use std::collections::BTreeMap;

use serde::Serialize;
use sonarithm_core::{Diagnostic, ProvenanceClass};

use crate::detect::{Container, Tier};
use crate::model::{ChartData, ChartType};

/// One series in the report.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SeriesReport {
    /// The series name.
    pub name: String,
    /// The number of points, gaps included.
    pub points: usize,
    /// The number of gaps (points without a value).
    pub missing: usize,
    /// The provenance class of the series' values.
    pub class: ProvenanceClass,
    /// The confidence of the series' values; absent for exact values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    /// For calibrated series, the axis-calibration residual relative to the axis range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residual: Option<f64>,
}

/// A report's points by how they were obtained (10 §5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct ClassCounts {
    /// Every point.
    pub points: usize,
    /// Points read exactly (`exact`, or `user_edited`).
    pub exact: usize,
    /// Points calibrated from a vector drawing (`vector`).
    pub calibrated: usize,
    /// Estimated points (`cv_estimate`, `ml_estimate`, `cloud_estimate` and other
    /// non-exact classes).
    pub estimated: usize,
    /// The median confidence over the points that have one; absent when every point is
    /// exact.
    pub median_confidence: Option<f64>,
}

/// The report of one extraction.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExtractionReport {
    /// The file name, without directories (P8).
    pub locator: String,
    /// The sniffed container.
    pub container: Container,
    /// The tier that read it.
    pub tier: Tier,
    /// The extractor, such as `sonarithm-plotsona/office`.
    pub extractor: String,
    /// The chart type, when the file states it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_type: Option<ChartType>,
    /// The series, in the file's order.
    pub series: Vec<SeriesReport>,
    /// The number of points over all series.
    pub points: usize,
    /// Points per provenance class, by class name (03 §4).
    pub provenance: BTreeMap<String, usize>,
    /// Warnings and notes from extraction (errors stop it instead).
    pub diagnostics: Vec<Diagnostic>,
}

impl ExtractionReport {
    /// The report of `chart`, read from `container` by `extractor`.
    #[must_use]
    pub fn new(
        locator: &str,
        container: Container,
        extractor: &str,
        chart: &ChartData,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        let mut provenance = BTreeMap::new();
        let series: Vec<SeriesReport> = chart
            .series
            .iter()
            .map(|series| {
                *provenance
                    .entry(
                        sonarithm_data::provenance::class_name(series.provenance.class).to_owned(),
                    )
                    .or_insert(0) += series.points.len();
                SeriesReport {
                    name: series.name.clone(),
                    points: series.points.len(),
                    missing: series.points.iter().filter(|p| p.y.is_none()).count(),
                    class: series.provenance.class,
                    confidence: series.provenance.confidence.map(f32::from),
                    residual: series.residual,
                }
            })
            .collect();
        Self {
            locator: sonarithm_data::bundle::redact_locator(locator),
            container,
            tier: container.tier(),
            extractor: extractor.to_owned(),
            chart_type: chart.chart_type,
            points: chart.points(),
            series,
            provenance,
            diagnostics,
        }
    }

    /// The points by how they were obtained, and the median confidence of the points that
    /// have one, for the summary sentence of 10 §5 ("146 points: 0 exact, 146 estimated,
    /// median confidence 0.82").
    #[must_use]
    pub fn counts(&self) -> ClassCounts {
        let mut counts = ClassCounts {
            points: self.points,
            ..ClassCounts::default()
        };
        let mut confidences: Vec<(f32, usize)> = Vec::new();
        for series in &self.series {
            match series.class {
                ProvenanceClass::Exact | ProvenanceClass::UserEdited => {
                    counts.exact += series.points;
                }
                ProvenanceClass::Vector => counts.calibrated += series.points,
                _ => counts.estimated += series.points,
            }
            if let Some(confidence) = series.confidence {
                confidences.push((confidence, series.points));
            }
        }
        confidences.sort_by(|a, b| a.0.total_cmp(&b.0));
        let weighted: usize = confidences.iter().map(|c| c.1).sum();
        let mut seen = 0;
        for (confidence, points) in confidences {
            seen += points;
            if seen * 2 >= weighted && weighted > 0 {
                counts.median_confidence = Some(f64::from(confidence));
                break;
            }
        }
        counts
    }

    /// The number of points whose class is an estimate.
    #[must_use]
    pub fn estimated(&self) -> usize {
        self.series
            .iter()
            .filter(|s| s.class.is_estimate())
            .map(|s| s.points)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use sonarithm_core::{Confidence, Provenance};

    use super::*;
    use crate::model::{Point, Series};

    #[test]
    fn the_report_counts_points_per_class_and_redacts_the_path() {
        let mut estimated = Series::exact("b", vec![Point::xy(0.0, 1.0); 3]);
        estimated.provenance = Provenance {
            class: ProvenanceClass::CvEstimate,
            confidence: Confidence::new(0.5),
        };
        let chart = ChartData {
            chart_type: Some(ChartType::Line),
            series: vec![
                Series::exact(
                    "a",
                    vec![
                        Point::xy(0.0, 1.0),
                        Point {
                            x: crate::model::XValue::Number(1.0),
                            y: None,
                        },
                    ],
                ),
                estimated,
            ],
            ..ChartData::default()
        };
        let report = ExtractionReport::new(
            "C:\\Users\\someone\\charts\\sales.xlsx",
            Container::Xlsx,
            "sonarithm-plotsona/office",
            &chart,
            Vec::new(),
        );
        assert_eq!(report.locator, "sales.xlsx");
        assert_eq!(report.points, 5);
        assert_eq!(report.estimated(), 3);
        assert_eq!(report.series.first().map(|s| s.missing), Some(1));
        assert_eq!(report.provenance.get("exact"), Some(&2));
        assert_eq!(report.provenance.get("cv_estimate"), Some(&3));
        // 10 §5: the summary counts, with the median confidence of the estimated points.
        assert_eq!(
            report.counts(),
            ClassCounts {
                points: 5,
                exact: 2,
                calibrated: 0,
                estimated: 3,
                median_confidence: Some(0.5),
            }
        );
        insta::assert_json_snapshot!(report);
    }
}

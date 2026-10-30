// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! What every extractor produces: a chart's series with their points and provenance, and
//! the long table the rest of SONARITHM reads (03 §2, §4).
//!
//! The table has one row per point: `series` (the series name), `x` (numbers, times in
//! milliseconds or category labels) and `y` (a number, null where the chart has a gap),
//! then the provenance columns `__snr_prov` and `__snr_conf`. The provenance columns are
//! always written, even when every row is `exact`, so a chart's extraction class is never
//! implied (AGENTS.md rule 7).

use std::sync::Arc;

use arrow_array::{ArrayRef, Float64Array, RecordBatch, StringArray, TimestampMillisecondArray};
use arrow_schema::{DataType, Field, Schema, TimeUnit};
use serde::Serialize;
use sonarithm_core::{Confidence, Provenance, TableId};
use sonarithm_data::provenance::with_provenance;
use sonarithm_data::{DataError, FieldMeta, FieldType, Role, RowProvenance, Table};

/// The kind of chart a file draws, as far as the file says.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ChartType {
    /// Lines through points.
    Line,
    /// Filled areas under lines.
    Area,
    /// Bars or columns, side by side when there are several series.
    Bar,
    /// Bars stacked on each other.
    StackedBar,
    /// Points at (x, y).
    Scatter,
    /// Slices of a whole.
    Pie,
    /// Counts in bins.
    Histogram,
    /// Five-number summaries.
    BoxPlot,
    /// A grid of coloured cells.
    Heatmap,
    /// Anything else (radar, bubble, stock, …).
    Other,
}

/// One x position: a number, a time or a category label.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum XValue {
    /// A number on a continuous axis.
    Number(f64),
    /// A time, in milliseconds since 1970-01-01T00:00:00 UTC.
    Time(i64),
    /// A category label.
    Category(String),
}

/// One data point. `y` is `None` where the chart has a gap.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Point {
    /// The x position.
    pub x: XValue,
    /// The value.
    pub y: Option<f64>,
}

impl Point {
    /// A point at a numeric x.
    #[must_use]
    pub const fn xy(x: f64, y: f64) -> Self {
        Self {
            x: XValue::Number(x),
            y: Some(y),
        }
    }

    /// A point at a category.
    #[must_use]
    pub fn category(label: impl Into<String>, y: Option<f64>) -> Self {
        Self {
            x: XValue::Category(label.into()),
            y,
        }
    }
}

/// One series of a chart and how its values were obtained.
#[derive(Clone, Debug, PartialEq)]
pub struct Series {
    /// The series name (the legend entry), or a generated `Series 1` when the file has none.
    pub name: String,
    /// The series' own chart type, when a combined chart mixes types.
    pub chart_type: Option<ChartType>,
    /// The points in the file's order.
    pub points: Vec<Point>,
    /// The provenance of every point in the series.
    pub provenance: Provenance,
    /// For calibrated (`vector`) series, the axis-calibration residual relative to the axis
    /// range (10 §3.1).
    pub residual: Option<f64>,
    /// Per-point confidences, one per point, when they differ within the series (raster
    /// estimates, C02-T03); empty when every point has the series' confidence.
    pub confidences: Vec<Confidence>,
}

impl Series {
    /// A series of exact values (tier 1).
    #[must_use]
    pub fn exact(name: impl Into<String>, points: Vec<Point>) -> Self {
        Self {
            name: name.into(),
            chart_type: None,
            points,
            provenance: Provenance::EXACT,
            residual: None,
            confidences: Vec::new(),
        }
    }
}

/// A chart's data as extracted.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartData {
    /// The chart type, when the file states it.
    pub chart_type: Option<ChartType>,
    /// The chart title.
    pub title: Option<String>,
    /// The x-axis title.
    pub x_title: Option<String>,
    /// The y-axis title.
    pub y_title: Option<String>,
    /// The series in the file's order.
    pub series: Vec<Series>,
}

impl ChartData {
    /// The number of points over all series.
    #[must_use]
    pub fn points(&self) -> usize {
        self.series.iter().map(|series| series.points.len()).sum()
    }

    /// The long table of the chart (see the module documentation).
    ///
    /// `x` is numeric when every x is a number, a millisecond timestamp when every x is a
    /// time, and text otherwise (numbers and times written out).
    ///
    /// # Errors
    ///
    /// The data layer's error if Arrow rejects the columns, which their construction rules
    /// out.
    pub fn to_table(&self, id: TableId, name: &str) -> Result<Table, DataError> {
        let rows = self.points();
        let mut series_names = Vec::with_capacity(rows);
        let mut ys = Vec::with_capacity(rows);
        let mut provenance = Vec::with_capacity(rows);
        for series in &self.series {
            for (index, point) in series.points.iter().enumerate() {
                series_names.push(series.name.as_str());
                ys.push(point.y);
                let mut row = series.provenance;
                if let Some(confidence) = series.confidences.get(index) {
                    row.confidence = Some(*confidence);
                }
                provenance.push(row);
            }
        }
        let points = || self.series.iter().flat_map(|series| series.points.iter());
        let (x_type, x_column, x_meta): (DataType, ArrayRef, FieldMeta) =
            if points().all(|p| matches!(p.x, XValue::Number(_))) {
                let values: Float64Array = points()
                    .map(|p| match p.x {
                        XValue::Number(x) => Some(x),
                        _ => None,
                    })
                    .collect();
                (
                    DataType::Float64,
                    Arc::new(values),
                    FieldMeta::new().with_type(FieldType::Quantitative),
                )
            } else if points().all(|p| matches!(p.x, XValue::Time(_))) {
                let values: TimestampMillisecondArray = points()
                    .map(|p| match p.x {
                        XValue::Time(t) => Some(t),
                        _ => None,
                    })
                    .collect();
                (
                    DataType::Timestamp(TimeUnit::Millisecond, None),
                    Arc::new(values),
                    FieldMeta::new().with_type(FieldType::Temporal),
                )
            } else {
                let values: StringArray = points().map(|p| Some(x_label(&p.x))).collect();
                (
                    DataType::Utf8,
                    Arc::new(values),
                    FieldMeta::new().with_type(FieldType::Ordinal),
                )
            };
        let mut x_meta = x_meta.with_role(Role::X);
        if let Some(title) = &self.x_title {
            x_meta = x_meta.with_label(title.clone());
        }
        let mut y_meta = FieldMeta::new()
            .with_type(FieldType::Quantitative)
            .with_role(Role::Y);
        if let Some(title) = &self.y_title {
            y_meta = y_meta.with_label(title.clone());
        }
        let series_meta = FieldMeta::new()
            .with_type(FieldType::Nominal)
            .with_role(Role::Series);
        let schema = Arc::new(Schema::new(vec![
            series_meta.apply(Field::new("series", DataType::Utf8, false)),
            x_meta.apply(Field::new("x", x_type, false)),
            y_meta.apply(Field::new("y", DataType::Float64, true)),
        ]));
        let columns: Vec<ArrayRef> = vec![
            Arc::new(StringArray::from(series_names)),
            x_column,
            Arc::new(Float64Array::from(ys)),
        ];
        let batch = RecordBatch::try_new(Arc::clone(&schema), columns).map_err(|source| {
            DataError::MalformedTable {
                table: name.to_owned(),
                source,
            }
        })?;
        let table = Table::new(id, name, schema, vec![batch])?;
        with_provenance(&table, &RowProvenance::from_rows(provenance), true)
    }
}

/// The text form of an x position, for a mixed x column.
fn x_label(x: &XValue) -> String {
    match x {
        XValue::Number(x) => x.to_string(),
        XValue::Time(t) => t.to_string(),
        XValue::Category(label) => label.clone(),
    }
}

#[cfg(test)]
mod tests {
    use sonarithm_core::ProvenanceClass;

    use super::*;

    #[test]
    fn exact_rows_keep_their_provenance_columns() {
        let chart = ChartData {
            series: vec![Series::exact(
                "Sales",
                vec![
                    Point::category("Q1", Some(1.5)),
                    Point::category("Q2", None),
                ],
            )],
            ..ChartData::default()
        };
        let table = chart.to_table(TableId(0), "chart").expect("table");
        let names: Vec<_> = table
            .schema()
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect();
        assert_eq!(names, ["series", "x", "y", "__snr_prov", "__snr_conf"]);
        let rows = RowProvenance::read(&table).expect("provenance");
        assert!(rows.is_all_exact());
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn vector_series_carry_class_and_confidence_per_row() {
        let mut series = Series::exact("a", vec![Point::xy(0.0, 1.0), Point::xy(1.0, 2.0)]);
        series.provenance = Provenance {
            class: ProvenanceClass::Vector,
            confidence: Confidence::new(0.99),
        };
        let chart = ChartData {
            series: vec![series, Series::exact("b", vec![Point::xy(0.0, 3.0)])],
            ..ChartData::default()
        };
        let table = chart.to_table(TableId(0), "chart").expect("table");
        let rows = RowProvenance::read(&table).expect("provenance");
        assert_eq!(rows.get(0).class, ProvenanceClass::Vector);
        assert_eq!(rows.get(1).confidence, Confidence::new(0.99));
        assert_eq!(rows.get(2).class, ProvenanceClass::Exact);
        assert_eq!(
            table.schema().field(1).data_type(),
            &DataType::Float64,
            "numeric x stays numeric"
        );
    }

    #[test]
    fn times_become_a_timestamp_column_and_mixed_x_becomes_text() {
        let times = ChartData {
            series: vec![Series::exact(
                "t",
                vec![Point {
                    x: XValue::Time(86_400_000),
                    y: Some(1.0),
                }],
            )],
            ..ChartData::default()
        };
        let table = times.to_table(TableId(0), "chart").expect("table");
        assert_eq!(
            table.schema().field(1).data_type(),
            &DataType::Timestamp(TimeUnit::Millisecond, None)
        );
        let mixed = ChartData {
            series: vec![Series::exact(
                "m",
                vec![Point::xy(1.0, 1.0), Point::category("two", Some(2.0))],
            )],
            ..ChartData::default()
        };
        let table = mixed.to_table(TableId(0), "chart").expect("table");
        assert_eq!(table.schema().field(1).data_type(), &DataType::Utf8);
    }
}

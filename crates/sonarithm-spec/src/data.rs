// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `data` and `transform` (RFC-0001 §4, 04 §3–4).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::json::{JsonObject, JsonValue};
use crate::values::DurationValue;

/// `data`: where the data comes from and what its fields mean.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct DataSpec {
    /// The data source.
    pub source: Source,
    /// Field metadata that overrides inference (03 §3).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: BTreeMap<String, FieldSpec>,
    /// What missing values sound like; default `silence`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing: Option<Missing>,
}

/// A data source, tagged by `kind`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum Source {
    /// A data file.
    File {
        /// Path relative to the spec.
        path: String,
        /// File format; inferred from the extension when absent.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        format: Option<FileFormat>,
        /// Worksheet name for spreadsheets.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sheet: Option<String>,
        /// Format-specific options.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        options: Option<JsonObject>,
    },
    /// Records written in the spec.
    Inline {
        /// The records.
        values: Vec<BTreeMap<String, JsonValue>>,
    },
    /// A chart (PLOTSONA).
    Chart {
        /// Path relative to the spec.
        path: String,
        /// The extractor; default `auto`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        extractor: Option<Extractor>,
        /// Axis calibration (typed in v0.2, RFC-0004 §5; read with [`crate::v02::read`]).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(with = "Option<crate::v02::Calibration>")]
        calibration: Option<JsonValue>,
        /// Corrections a person made to the extracted chart (v0.2, RFC-0004 §6).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(with = "Option<crate::v02::Corrections>")]
        corrections: Option<JsonValue>,
    },
    /// A sampled function (CURVASONA). Produces fields `x`, `y` and `dy`.
    Function {
        /// The expression in `x`.
        expr: String,
        /// `[min, max]` of `x`.
        domain: [f64; 2],
        /// Number of samples (at least 2); default 1000. Exclusive with `step`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        samples: Option<u32>,
        /// Sampling step. Exclusive with `samples`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
        /// Named constants.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        variables: Option<BTreeMap<String, f64>>,
    },
    /// A live stream (SONOTRACE).
    Stream {
        /// The transport.
        transport: Transport,
        /// Address for `websocket` and `mqtt`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// Port or device for `serial` and `osc`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        port: Option<String>,
        /// MQTT topic.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        topic: Option<String>,
        /// File to follow for `tail`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        /// Message schema.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        schema: Option<JsonObject>,
        /// How much history to keep.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<DurationValue>,
    },
    /// A dataset loaded in a project.
    Dataset {
        /// The dataset ID.
        id: String,
    },
}

/// Data file formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
#[expect(missing_docs, reason = "the variant names are the format names")]
pub enum FileFormat {
    Csv,
    Tsv,
    Json,
    Ndjson,
    Parquet,
    Arrow,
    Xlsx,
    Ods,
}

/// Chart extractors (PLOTSONA, 10 §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
#[expect(missing_docs, reason = "the variant names are the extractor names")]
pub enum Extractor {
    Auto,
    Office,
    Svg,
    Vega,
    Plotly,
    PdfVector,
    Raster,
    Ml,
}

/// Stream transports (SONOTRACE).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
#[expect(missing_docs, reason = "the variant names are the transport names")]
pub enum Transport {
    Websocket,
    Mqtt,
    Serial,
    Osc,
    Tail,
}

/// Metadata for one field (03 §3).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct FieldSpec {
    /// The field type.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub field_type: Option<FieldType>,
    /// Unit as free text, such as `°C`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// The concept, such as `temperature`, which sets default polarity (SDR-12..14).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concept: Option<String>,
    /// The role, such as `x` or `series` (03 §3.4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    /// A spoken label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// A longer description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Field types (03 §3.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum FieldType {
    /// Numbers on an interval or ratio scale.
    Quantitative,
    /// Dates, times and years.
    Temporal,
    /// Ordered categories.
    Ordinal,
    /// Unordered categories.
    Nominal,
}

/// Field roles (03 §3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
#[allow(
    missing_docs,
    reason = "the variant names are the role names of 03 §3.4"
)]
pub enum Role {
    X,
    Y,
    Series,
    Category,
    Weight,
    Id,
    Label,
    Time,
    Lower,
    Upper,
}

/// Treatment of missing values (03 §5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Missing {
    /// Silence for the slot (default).
    Silence,
    /// A gap earcon.
    Earcon,
    /// Interpolate across the gap.
    Interpolate,
}

/// One transform, tagged by `op` (RFC-0001 §4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum Transform {
    /// Keep rows where `expr` holds.
    Filter {
        /// The predicate.
        expr: String,
    },
    /// Stable sort.
    Sort {
        /// The sort key.
        field: String,
        /// Direction; default ascending.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        order: Option<SortOrder>,
    },
    /// Group and aggregate.
    Aggregate {
        /// Grouping fields.
        groupby: Vec<String>,
        /// Aggregations; at least one.
        ops: Vec<AggregateOp>,
    },
    /// Histogram bins.
    Bin {
        /// The field.
        field: String,
        /// Binning method; default `equal-width`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        method: Option<BinMethod>,
        /// Number of bins.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        count: Option<u32>,
        /// Explicit edges for `edges`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        edges: Option<Vec<f64>>,
        /// Output field.
        #[serde(rename = "as", default, skip_serializing_if = "Option::is_none")]
        output: Option<String>,
    },
    /// Smoothing (SDR-21).
    Smooth {
        /// The field.
        field: String,
        /// `ema` or `sma`.
        method: SmoothMethod,
        /// EMA weight in (0, 1].
        #[serde(default, skip_serializing_if = "Option::is_none")]
        alpha: Option<f64>,
        /// SMA window in rows.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<u32>,
        /// Output field.
        #[serde(rename = "as", default, skip_serializing_if = "Option::is_none")]
        output: Option<String>,
    },
    /// Regular resampling of temporal data.
    Resample {
        /// The time field.
        field: String,
        /// The interval.
        every: DurationValue,
        /// How values in an interval combine.
        method: ResampleMethod,
    },
    /// A new field from an expression.
    Derive {
        /// The expression.
        expr: String,
        /// Output field.
        #[serde(rename = "as")]
        output: String,
    },
    /// Normalization.
    Normalize {
        /// The field.
        field: String,
        /// The method.
        method: NormalizeMethod,
        /// Output field.
        #[serde(rename = "as", default, skip_serializing_if = "Option::is_none")]
        output: Option<String>,
    },
}

/// One aggregation in an `aggregate` transform.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct AggregateOp {
    /// The aggregation function.
    pub op: AggregateFn,
    /// The input field.
    pub field: String,
    /// The output field.
    #[serde(rename = "as")]
    pub output: String,
}

/// Aggregation functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
#[expect(missing_docs, reason = "the variant names are the function names")]
pub enum AggregateFn {
    Sum,
    Mean,
    Median,
    Min,
    Max,
    Count,
}

/// Sort direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[expect(missing_docs, reason = "the variant names are the directions")]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Binning methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
#[expect(missing_docs, reason = "the variant names are the methods")]
pub enum BinMethod {
    EqualWidth,
    Quantile,
    Edges,
}

/// Smoothing methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum SmoothMethod {
    /// Exponential moving average (`alpha`).
    Ema,
    /// Simple moving average (`window`).
    Sma,
}

/// Resampling methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
#[expect(missing_docs, reason = "the variant names are the methods")]
pub enum ResampleMethod {
    Mean,
    Last,
    Interpolate,
}

/// Normalization methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
#[expect(missing_docs, reason = "the variant names are the methods")]
pub enum NormalizeMethod {
    ZScore,
    MinMax,
    Rank,
}

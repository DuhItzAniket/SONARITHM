// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ECHOMETRIC as a compile engine (C05-T01): its detectors registered as analyzers
//! (07 ECHOMETRIC). Each analyzes the fields the spec's tracks play (RFC-0009 §1) and records
//! its parameters in every annotation, with the field it is about.

use std::collections::BTreeMap;
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::{
    Float32Type, Float64Type, Int8Type, Int16Type, Int32Type, Int64Type, UInt8Type, UInt16Type,
    UInt32Type, UInt64Type,
};
use arrow_array::{Array, ArrowPrimitiveType, PrimitiveArray};
use arrow_schema::DataType;
use sonarithm_core::diagnostic::Diagnostic;
use sonarithm_core::engine::{
    AnalysisRequest, Analyzer, Annotation, AnnotationParam, CompileContext, CompileError, Engine,
    EngineManifest, Registrar, RegistryError,
};
use sonarithm_core::ids::{AnalyzerId, EngineId, RowRange, RuleId};
use sonarithm_core::score::EngineVersion;
use sonarithm_data::{DataBundle, Table};

use crate::codes;
use crate::detect::{self, Finding, Param, Params, Series};

/// The engine ID.
pub const ENGINE: EngineId = EngineId::from_static("echometric");

/// The `extrema` analyzer.
pub const EXTREMA: AnalyzerId = AnalyzerId::from_static("extrema");

/// The `outliers` analyzer.
pub const OUTLIERS: AnalyzerId = AnalyzerId::from_static("outliers");

/// The `changepoints` analyzer.
pub const CHANGEPOINTS: AnalyzerId = AnalyzerId::from_static("changepoints");

/// The `periodicity` analyzer.
pub const PERIODICITY: AnalyzerId = AnalyzerId::from_static("periodicity");

/// The `trend-segments` analyzer.
pub const TREND_SEGMENTS: AnalyzerId = AnalyzerId::from_static("trend-segments");

/// ECHOMETRIC's detectors, as `analysis.detect` names them.
pub const DETECTORS: [&str; 5] = [
    "extrema",
    "outliers",
    "changepoints",
    "periodicity",
    "trend-segments",
];

/// Whether `spec` (as JSON) names one of ECHOMETRIC's detectors in `analysis.detect`. The
/// facade adds ECHOMETRIC only to such compiles, as it does CURVASONA (F-197).
#[must_use]
pub fn is_used_by(spec: &serde_json::Value) -> bool {
    spec.pointer("/analysis/detect")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|detect| {
            detect
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|name| DETECTORS.contains(&name))
        })
}

/// ECHOMETRIC's analytics: its five detectors (C05-T02, T03).
#[derive(Clone, Copy, Debug, Default)]
pub struct Echometric;

impl Echometric {
    /// The engine's manifest.
    #[must_use]
    pub fn manifest() -> EngineManifest {
        let mut manifest = EngineManifest::new(
            ENGINE,
            "ECHOMETRIC",
            EngineVersion(env!("CARGO_PKG_VERSION").to_owned()),
        );
        manifest.spec_keys = vec!["analysis.detect".to_owned()];
        manifest
    }
}

impl Engine<DataBundle> for Echometric {
    fn manifest(&self) -> EngineManifest {
        Self::manifest()
    }

    fn register(&self, registrar: &mut Registrar<'_, DataBundle>) -> Result<(), RegistryError> {
        registrar.analyzer(EXTREMA, Arc::new(Detector::Extrema))?;
        registrar.analyzer(OUTLIERS, Arc::new(Detector::Outliers))?;
        registrar.analyzer(CHANGEPOINTS, Arc::new(Detector::Changepoints))?;
        registrar.analyzer(PERIODICITY, Arc::new(Detector::Periodicity))?;
        registrar.analyzer(TREND_SEGMENTS, Arc::new(Detector::TrendSegments))?;
        Ok(())
    }
}

/// One of ECHOMETRIC's detectors, as an analyzer.
#[derive(Clone, Copy, Debug)]
enum Detector {
    Extrema,
    Outliers,
    Changepoints,
    Periodicity,
    TrendSegments,
}

impl Detector {
    const fn id(self) -> AnalyzerId {
        match self {
            Self::Extrema => EXTREMA,
            Self::Outliers => OUTLIERS,
            Self::Changepoints => CHANGEPOINTS,
            Self::Periodicity => PERIODICITY,
            Self::TrendSegments => TREND_SEGMENTS,
        }
    }

    fn params(self) -> Params {
        match self {
            Self::Extrema => detect::extrema::params(),
            Self::Outliers => detect::outliers::params(),
            Self::Changepoints => detect::changepoints::params(),
            Self::Periodicity => detect::periodicity::params(),
            Self::TrendSegments => detect::trends::params(),
        }
    }

    /// The name a finding's score is recorded under in its annotation's parameters.
    const fn score_name(self) -> &'static str {
        match self {
            Self::Outliers => "robustZ",
            Self::Changepoints => "shift",
            Self::Periodicity => "correlation",
            Self::Extrema | Self::TrendSegments => "score",
        }
    }

    fn findings(self, series: &Series) -> Vec<Finding> {
        match self {
            Self::Extrema => detect::extrema::detect(series),
            Self::Outliers => detect::outliers::detect(series),
            Self::Changepoints => detect::changepoints::detect(series),
            Self::Periodicity => detect::periodicity::detect(series),
            Self::TrendSegments => detect::trends::detect(series),
        }
    }
}

impl Analyzer<DataBundle> for Detector {
    fn analyze(
        &self,
        data: &DataBundle,
        request: &AnalysisRequest<'_>,
        cx: &mut CompileContext,
    ) -> Result<Vec<Annotation>, CompileError> {
        let Some(table) = data.primary() else {
            return Ok(Vec::new());
        };
        let mut annotations = Vec::new();
        for analyzed in request.fields {
            let Some(series) = series_of(table, &analyzed.field, analyzed.rows.as_deref()) else {
                cx.report(Diagnostic::new(
                    &codes::FIELD_NOT_ANALYZED,
                    format!(
                        "The `{}` detector did not analyze `{}`: it is not a numeric field of the data.",
                        self.id(),
                        analyzed.field
                    ),
                ));
                continue;
            };
            cx.check_cancelled()?;
            for finding in self.findings(&series) {
                let mut found = annotation(*self, table, &analyzed.field, &finding);
                found.track = Some(analyzed.track.clone());
                annotations.push(found);
            }
        }
        Ok(annotations)
    }
}

/// A finding as an annotation of `field`, with the detector's parameters (RFC-0009 §1).
///
/// A track's series may skip rows (its filter) and run out of table order (its time field),
/// so a finding's first and last rows bound its range in table order, whichever comes first;
/// the annotation's `track` says which of those rows belong to it (RFC-0009 §1).
fn annotation(detector: Detector, table: &Table, field: &str, finding: &Finding) -> Annotation {
    let range = RowRange {
        table: table.id(),
        generation: table.generation(),
        start: finding.first.min(finding.last),
        end: finding.first.max(finding.last).saturating_add(1),
    };
    let mut annotation = Annotation::new(detector.id(), finding.label, range);
    annotation.field = Some(field.to_owned());
    annotation.value = Some(finding.value);
    annotation.rules = vec![RuleId::from_static("A11Y-23")];
    let mut params: BTreeMap<String, AnnotationParam> = detector
        .params()
        .into_iter()
        .map(|(name, value)| {
            let value = match value {
                Param::Number(n) => AnnotationParam::Number(n),
                Param::Text(t) => AnnotationParam::Text(t.to_owned()),
            };
            (name.to_owned(), value)
        })
        .collect();
    if let Some(score) = finding.score {
        params.insert(
            detector.score_name().to_owned(),
            AnnotationParam::Number(score),
        );
    }
    annotation.params = params;
    annotation
}

/// The finite numbers of `field` at `rows` of `table`, in that order (every row in table
/// order when `rows` is `None`), or `None` when the field is absent or not numeric.
fn series_of(table: &Table, field: &str, rows: Option<&[u64]>) -> Option<Series> {
    let column = table.column(field).ok()?;
    let values = numbers(column.as_ref())?;
    Some(match rows {
        // The track's rows, in the order it plays them (RFC-0009 §1).
        Some(rows) => Series::new(rows.iter().map(|&row| {
            let value = usize::try_from(row)
                .ok()
                .and_then(|at| values.get(at).copied().flatten());
            (row, value)
        })),
        None => Series::new((0_u64..).zip(values)),
    })
}

/// The values of a numeric array, `None` for each null.
fn numbers(array: &dyn Array) -> Option<Vec<Option<f64>>> {
    fn convert<T: ArrowPrimitiveType>(
        array: &dyn Array,
        to: impl Fn(T::Native) -> f64,
    ) -> Option<Vec<Option<f64>>> {
        let typed: &PrimitiveArray<T> = array.as_primitive_opt::<T>()?;
        Some(typed.iter().map(|v| v.map(&to)).collect())
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "data values become f64; integers above 2^53 lose their last bits, as in any f64 data path"
    )]
    match array.data_type() {
        DataType::Float64 => convert::<Float64Type>(array, |v| v),
        DataType::Float32 => convert::<Float32Type>(array, f64::from),
        DataType::Int8 => convert::<Int8Type>(array, f64::from),
        DataType::Int16 => convert::<Int16Type>(array, f64::from),
        DataType::Int32 => convert::<Int32Type>(array, f64::from),
        DataType::Int64 => convert::<Int64Type>(array, |v| v as f64),
        DataType::UInt8 => convert::<UInt8Type>(array, f64::from),
        DataType::UInt16 => convert::<UInt16Type>(array, f64::from),
        DataType::UInt32 => convert::<UInt32Type>(array, f64::from),
        DataType::UInt64 => convert::<UInt64Type>(array, |v| v as f64),
        _ => None,
    }
}

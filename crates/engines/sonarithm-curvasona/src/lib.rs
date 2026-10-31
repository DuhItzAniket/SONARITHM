// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! CURVASONA, SONARITHM's function engine (07 CURVASONA; C03).
//!
//! A spec's `{"kind": "function", "expr": …, "domain": [min, max]}` source becomes a table
//! with the fields `x`, `y` and `dy` (the first derivative, RFC-0001 Q17), every row of
//! provenance `synthetic` (03 §4). The expression is untrusted input (12 §6): it is read by
//! the [`expr`] facade under [`Budgets`] with a closed set of names, and evaluated and
//! differentiated by exmex behind that facade. evalexpr is never used (AGPL, banned in
//! `deny.toml`).
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`expr`] | The expression language: parsing, limits, the sandboxed variable set, symbolic derivatives; exmex evaluation |
//! | [`sample`] | The sample grid of a domain (`samples` or `step`) under the sample budget |
//! | [`scan`] | Poles, holes, jumps, undefined spans and off-screen spans, found from the expression's guards |
//! | [`critical`] | Zeros, extrema, inflections and horizontal asymptotes; the `critical-points` analyzer's annotations |
//! | [`audify`] | The `audify` mark: the SDR-31 speed-up calculator and the audification encoder (RFC-0007 §3) |
//! | [`cues`] | The `curvasona.cues` context track: critical points as earcons or speech (RFC-0007) |
//! | [`suite`] | The 40-curve suite with analytically known features |
//! | [`codes`] | Diagnostic codes `SNR-ENG-400` to `499` |
//! | engine | The `function` source |
//!
//! Like every engine it depends only on core, spec and data (02 §3), and it builds for
//! `wasm32-unknown-unknown`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod audify;
pub mod codes;
pub mod critical;
pub mod cues;
pub mod expr;
pub mod sample;
pub mod scan;
pub mod suite;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::sync::Arc;

use arrow_array::{ArrayRef, Float64Array, RecordBatch};
use arrow_schema::{DataType, Field, Schema};
use sonarithm_core::engine::{
    Analyzer, Annotation, CompileContext, CompileError, Engine, EngineManifest, Registrar,
    RegistryError, Source, SourceKind, SpecFragment,
};
use sonarithm_core::{
    AnalyzerId, Diagnostic, EngineId, EngineVersion, Provenance, ProvenanceClass, RowRange, RuleId,
    TableId,
};
use sonarithm_data::bundle::SourceKind as DataSourceKind;
use sonarithm_data::provenance::with_provenance;
use sonarithm_data::{
    BundleId, DataBundle, DataError, FieldMeta, FieldType, Role, RowProvenance, SourceDescriptor,
    Table,
};

pub use critical::{Calculus, Feature, FeatureKind};
pub use expr::{Expr, ExprError, Limits, NoDerivative};
pub use sample::Grid;
pub use scan::{Approach, Scan, ScanBudget, Singularity, SingularityKind, Span};

/// The engine ID.
pub const ENGINE: EngineId = EngineId::from_static("curvasona");

/// The table ID of a sampled function in its bundle.
pub const FUNCTION_TABLE: TableId = TableId(0);

/// The extractor name recorded in the bundle's source descriptor.
pub const EXTRACTOR: &str = "sonarithm-curvasona/function";

/// The `critical-points` analyzer (A11Y-28).
pub const CRITICAL_POINTS: AnalyzerId = AnalyzerId::from_static("critical-points");

/// Table metadata: the function's expression in canonical form, so the analyzer can
/// recompute its features from the table (C03-T03).
pub const META_EXPR: &str = "snr.function.expr";

/// Table metadata: the function's `variables`, as a JSON object.
pub const META_VARIABLES: &str = "snr.function.variables";

/// Limits for one function source. Engineering safety limits, not research defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budgets {
    /// The expression's limits.
    pub expr: Limits,
    /// The most samples a domain may be split into.
    pub max_samples: usize,
    /// The most evaluations of the expression and its derivative in one source.
    pub max_evaluations: u64,
}

impl Budgets {
    /// [`Limits::DEFAULT`], 100 000 samples and 1 000 000 evaluations.
    pub const DEFAULT: Self = Self {
        expr: Limits::DEFAULT,
        max_samples: 100_000,
        max_evaluations: 1_000_000,
    };
}

impl Default for Budgets {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// How `dy` was computed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Derivative {
    /// From the symbolic derivative.
    Symbolic,
    /// From central differences, and why.
    Numeric(NoDerivative),
}

/// A sampled function: its table and how it was made.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Sampled {
    /// The bundle holding the table `x`, `y`, `dy` (provenance `synthetic`).
    pub bundle: DataBundle,
    /// The parsed expression.
    pub expr: Expr,
    /// How `dy` was computed.
    pub derivative: Derivative,
    /// The number of rows whose `y` is undefined.
    pub undefined: usize,
    /// The poles, holes, jumps and undefined spans (C03-T02).
    pub scan: Scan,
    /// Every feature, critical points included (C03-T03).
    pub features: Vec<Feature>,
    /// Non-error diagnostics, such as a numerical derivative or undefined samples.
    pub diagnostics: Vec<Diagnostic>,
}

/// A function source as the spec writes it.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct FunctionSpec {
    /// The expression in `x`.
    pub expr: String,
    /// `[min, max]` of `x`.
    pub domain: [f64; 2],
    /// The number of samples; default 1000 (RFC-0001 §4).
    pub samples: Option<u32>,
    /// The sampling step, instead of `samples`.
    pub step: Option<f64>,
    /// Named constants.
    pub variables: BTreeMap<String, f64>,
}

impl FunctionSpec {
    /// A function over `domain` with the default 1000 samples and no variables.
    #[must_use]
    pub fn new(expr: impl Into<String>, domain: [f64; 2]) -> Self {
        Self {
            expr: expr.into(),
            domain,
            samples: None,
            step: None,
            variables: BTreeMap::new(),
        }
    }

    /// The same function with `samples` samples.
    #[must_use]
    pub const fn with_samples(mut self, samples: u32) -> Self {
        self.samples = Some(samples);
        self
    }

    /// The same function with the named constant `name`.
    #[must_use]
    pub fn with_variable(mut self, name: impl Into<String>, value: f64) -> Self {
        self.variables.insert(name.into(), value);
        self
    }
}

/// A diagnostic and the spec key it belongs to (`expr`, `variables`, `samples`).
#[derive(Clone, Debug, PartialEq)]
pub struct SampleError {
    /// The diagnostic.
    pub diagnostic: Box<Diagnostic>,
    /// The key of the function source it is about.
    pub key: &'static str,
}

/// Samples `function` under `budgets` into a bundle with `ids`.
///
/// # Errors
///
/// A [`SampleError`] when the expression or a variable cannot be used (`SNR-ENG-400` to
/// `403`) or the grid exceeds the budget (`SNR-ENG-404`).
pub fn sample_function(
    function: &FunctionSpec,
    budgets: &Budgets,
    ids: (BundleId, TableId),
) -> Result<Sampled, SampleError> {
    let expr = Expr::parse(&function.expr, &function.variables, &budgets.expr).map_err(|err| {
        let key = if matches!(err, ExprError::Variable { .. }) {
            "variables"
        } else {
            "expr"
        };
        SampleError {
            diagnostic: Box::new(err.to_diagnostic()),
            key,
        }
    })?;
    let grid = Grid::new(function.domain, function.samples, function.step, budgets).map_err(
        |diagnostic| SampleError {
            diagnostic,
            key: if function.step.is_some() {
                "step"
            } else {
                "samples"
            },
        },
    )?;
    let (dy_expr, derivative) = match expr.derivative() {
        Ok(dy) => (Some(dy), Derivative::Symbolic),
        Err(reason) => (None, Derivative::Numeric(reason)),
    };
    let per_sample: u64 = if dy_expr.is_some() { 2 } else { 4 };
    let evaluations = u64::try_from(grid.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(per_sample);
    if evaluations > budgets.max_evaluations {
        return Err(SampleError {
            diagnostic: Box::new(Diagnostic::new(
                &codes::TOO_MANY_SAMPLES,
                format!(
                    "The function needs {evaluations} evaluations, more than the limit of {}.",
                    budgets.max_evaluations
                ),
            )),
            key: if function.step.is_some() {
                "step"
            } else {
                "samples"
            },
        });
    }
    let grid_points: Vec<f64> = grid.points().collect();
    let scan_budget = ScanBudget {
        max_evaluations: budgets.max_evaluations.saturating_sub(evaluations),
        max_features: ScanBudget::DEFAULT.max_features,
    };
    let scan = scan::scan(&expr, &grid_points, None, &scan_budget);
    let calculus = Calculus::new(expr.clone());
    let features = critical::features(&calculus, &grid_points, &scan, scan_budget.max_features);
    let xs = with_feature_rows(&grid_points, &scan, &features);
    let (ys, dys) = evaluate(&calculus, &xs, &scan);
    let singular = |x: f64| is_singular(&scan, x);
    let undefined = ys.iter().filter(|y| y.is_none()).count();
    // The rows added at poles and holes are empty by design; only the grid's count.
    let undefined_samples = grid_points
        .iter()
        .filter(|&&x| !singular(x) && !expr.eval(x).is_finite())
        .count();
    let diagnostics = report(derivative, undefined_samples, grid_points.len(), &scan);
    let table =
        function_table(ids.1, &expr, &function.variables, xs, ys, dys).map_err(|diagnostic| {
            SampleError {
                diagnostic,
                key: "expr",
            }
        })?;
    let mut source = SourceDescriptor::new(DataSourceKind::Function, "function", "function");
    EXTRACTOR.clone_into(&mut source.extractor);
    env!("CARGO_PKG_VERSION").clone_into(&mut source.extractor_version);
    Ok(Sampled {
        bundle: DataBundle::single(ids.0, source, table),
        expr,
        derivative,
        undefined,
        scan,
        features,
        diagnostics,
    })
}

/// Whether `x` is a pole or a hole of the scan, whose row stays empty.
fn is_singular(scan: &Scan, x: f64) -> bool {
    scan.singularities.iter().any(|s| {
        s.x.total_cmp(&x).is_eq() && matches!(s.kind, SingularityKind::Pole | SingularityKind::Hole)
    })
}

/// The `y` and `dy` columns at `xs`: empty where the function is undefined, or at a pole or
/// a hole.
fn evaluate(calculus: &Calculus, xs: &[f64], scan: &Scan) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let finite = |v: f64| v.is_finite().then_some(v);
    let ys: Vec<Option<f64>> = xs
        .iter()
        .map(|&x| {
            if is_singular(scan, x) {
                None
            } else {
                finite(calculus.f().eval(x))
            }
        })
        .collect();
    let dys = xs
        .iter()
        .zip(&ys)
        .map(|(&x, y)| y.and_then(|_| finite(calculus.d1(x))))
        .collect();
    (ys, dys)
}

/// The non-error diagnostics of a sampled function.
fn report(
    derivative: Derivative,
    undefined_samples: usize,
    samples: usize,
    scan: &Scan,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if let Derivative::Numeric(reason) = derivative {
        let why = match reason {
            NoDerivative::TooLarge => "its symbolic derivative is larger than the size limit",
            _ => "it uses a function without a symbolic derivative",
        };
        diagnostics.push(Diagnostic::new(
            &codes::NUMERIC_DERIVATIVE,
            format!("`dy` is computed numerically, because {why}."),
        ));
    }
    if undefined_samples > 0 {
        diagnostics.push(Diagnostic::new(
            &codes::UNDEFINED_SAMPLES,
            format!(
                "The function is undefined at {undefined_samples} of {samples} samples; those \
                 rows have no `y`."
            ),
        ));
    }
    if scan.truncated {
        diagnostics.push(Diagnostic::new(
            &codes::SCAN_TRUNCATED,
            format!(
                "The search for poles, holes, jumps and undefined spans stopped at its limit \
                 after {} evaluations; some may be missing.",
                scan.evaluations
            ),
        ));
    }
    diagnostics
}

/// The grid plus one row at each pole or hole between grid points, so a glide breaks there
/// instead of sweeping across an asymptote (C03-T02), and one at each zero, extremum and
/// inflection, so the glide passes through it and its insight lands on it (C03-T03).
fn with_feature_rows(grid: &[f64], scan: &Scan, features: &[Feature]) -> Vec<f64> {
    let mut xs = grid.to_vec();
    for singularity in &scan.singularities {
        if matches!(
            singularity.kind,
            SingularityKind::Pole | SingularityKind::Hole
        ) {
            xs.push(singularity.x);
        }
    }
    for feature in features {
        if matches!(
            feature.kind,
            FeatureKind::Zero
                | FeatureKind::Maximum
                | FeatureKind::Minimum
                | FeatureKind::Inflection
        ) {
            xs.push(feature.x);
        }
    }
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    xs
}

/// The table `x`, `y`, `dy` with every row `synthetic`, and the expression in its metadata.
fn function_table(
    id: TableId,
    expr: &Expr,
    variables: &BTreeMap<String, f64>,
    xs: Vec<f64>,
    ys: Vec<Option<f64>>,
    dys: Vec<Option<f64>>,
) -> Result<Table, Box<Diagnostic>> {
    let rows = xs.len();
    let quantitative = || FieldMeta::new().with_type(FieldType::Quantitative);
    let schema = Arc::new(
        Schema::new(vec![
            quantitative()
                .with_role(Role::X)
                .apply(Field::new("x", DataType::Float64, false)),
            quantitative()
                .with_role(Role::Y)
                .apply(Field::new("y", DataType::Float64, true)),
            quantitative().apply(Field::new("dy", DataType::Float64, true)),
        ])
        .with_metadata(std::collections::HashMap::from([
            (META_EXPR.to_owned(), expr.canonical().to_owned()),
            (
                META_VARIABLES.to_owned(),
                serde_json::to_string(variables).unwrap_or_else(|_| "{}".to_owned()),
            ),
        ])),
    );
    let columns: Vec<ArrayRef> = vec![
        Arc::new(Float64Array::from(xs)),
        Arc::new(Float64Array::from(ys)),
        Arc::new(Float64Array::from(dys)),
    ];
    let batch = RecordBatch::try_new(Arc::clone(&schema), columns).map_err(|source| {
        DataError::MalformedTable {
            table: "function".to_owned(),
            source,
        }
    });
    let synthetic = Provenance {
        class: ProvenanceClass::Synthetic,
        confidence: None,
    };
    let provenance = RowProvenance::from_rows(vec![synthetic; rows]);
    batch
        .and_then(|batch| Table::new(id, "function", schema, vec![batch]))
        .and_then(|table| with_provenance(&table, &provenance, true))
        .map_err(|err| Box::new(err.to_diagnostic()))
}

/// Whether `spec` (as JSON) uses CURVASONA: a `function` source, a track with the `audify`
/// mark, or `critical-points` in `analysis.detect` (the manifest's spec keys). Hosts
/// register CURVASONA only for such specs, because a registered context provider is
/// recorded in every Score's header (F-197).
#[must_use]
pub fn is_used_by(spec: &serde_json::Value) -> bool {
    use serde_json::Value;
    let function = spec.pointer("/data/source/kind").and_then(Value::as_str) == Some("function");
    let audify = spec
        .get("tracks")
        .and_then(Value::as_array)
        .is_some_and(|tracks| {
            tracks
                .iter()
                .any(|t| t.get("mark").and_then(Value::as_str) == Some("audify"))
        });
    let detects = spec
        .pointer("/analysis/detect")
        .and_then(Value::as_array)
        .is_some_and(|detect| {
            detect
                .iter()
                .any(|d| d.as_str() == Some(CRITICAL_POINTS.as_str()))
        });
    function || audify || detects
}

/// The CURVASONA engine. It registers the `function` source.
#[derive(Clone, Debug, Default)]
pub struct Curvasona {
    budgets: Budgets,
}

impl Curvasona {
    /// The engine with the default budgets.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The same engine with other budgets.
    #[must_use]
    pub const fn with_budgets(mut self, budgets: Budgets) -> Self {
        self.budgets = budgets;
        self
    }

    /// The engine's manifest.
    #[must_use]
    pub fn manifest() -> EngineManifest {
        let mut manifest = EngineManifest::new(
            ENGINE,
            "CURVASONA",
            EngineVersion(env!("CARGO_PKG_VERSION").to_owned()),
        );
        manifest.spec_keys = vec![
            "data.source.kind=function".to_owned(),
            "tracks[].audify".to_owned(),
        ];
        manifest
    }
}

impl Engine<DataBundle> for Curvasona {
    fn manifest(&self) -> EngineManifest {
        Self::manifest()
    }

    fn register(&self, registrar: &mut Registrar<'_, DataBundle>) -> Result<(), RegistryError> {
        registrar.source(
            SourceKind::Function,
            Arc::new(FunctionSource {
                budgets: self.budgets,
            }),
        )?;
        registrar.analyzer(
            CRITICAL_POINTS,
            Arc::new(CriticalPoints {
                budgets: self.budgets,
            }),
        )?;
        registrar.context(Arc::new(cues::FunctionCues));
        let audify: Arc<dyn sonarithm_core::engine::Encoder<DataBundle>> =
            Arc::new(audify::AudifyEncoder);
        registrar.encoder(&[sonarithm_core::engine::MarkKind::Audify], &audify)?;
        Ok(())
    }
}

/// The `function` source.
struct FunctionSource {
    budgets: Budgets,
}

impl Source<DataBundle> for FunctionSource {
    fn ingest(
        &self,
        source: &SpecFragment,
        cx: &mut CompileContext,
    ) -> Result<DataBundle, CompileError> {
        let parsed: sonarithm_spec::Source = source.parse()?;
        let sonarithm_spec::Source::Function {
            expr,
            domain,
            samples,
            step,
            variables,
        } = parsed
        else {
            return Err(CompileError::Engine {
                engine: ENGINE,
                message: "the function source was given another kind of source".to_owned(),
            });
        };
        let function = FunctionSpec {
            expr,
            domain,
            samples,
            step,
            variables: variables.unwrap_or_default(),
        };
        let sampled = sample_function(&function, &self.budgets, (BundleId(0), FUNCTION_TABLE))
            .map_err(|err| CompileError::from(err.diagnostic.at(source.pointer().key(err.key))))?;
        cx.check_cancelled()?;
        let at = source.pointer().key("expr");
        for diagnostic in sampled.diagnostics {
            cx.report(diagnostic.at(at.clone()));
        }
        Ok(sampled.bundle)
    }
}

/// The `critical-points` analyzer: the features of a function table as annotations, which
/// AUDAXIS turns into the navigation tree's insight nodes (08 §2, A11Y-23, A11Y-28).
struct CriticalPoints {
    budgets: Budgets,
}

impl Analyzer<DataBundle> for CriticalPoints {
    fn analyze(
        &self,
        data: &DataBundle,
        _params: &SpecFragment,
        cx: &mut CompileContext,
    ) -> Result<Vec<Annotation>, CompileError> {
        match function_features(data, &self.budgets) {
            Ok(annotations) => Ok(annotations),
            Err(reason) => {
                cx.report(Diagnostic::new(
                    &codes::NOT_A_FUNCTION,
                    format!("No critical points were looked for: {reason}."),
                ));
                Ok(Vec::new())
            }
        }
    }
}

/// A function table read back: its expression (from the metadata) and its rows by `x`.
pub(crate) struct FunctionTable<'a> {
    pub(crate) table: &'a Table,
    pub(crate) expr: Expr,
    /// `(x, row)`, sorted by `x`.
    pub(crate) rows: Vec<(f64, u64)>,
    /// The distinct `x` values, increasing.
    pub(crate) xs: Vec<f64>,
}

impl<'a> FunctionTable<'a> {
    /// Reads the function table of `data`.
    ///
    /// # Errors
    ///
    /// Why it is not one, as a clause.
    pub(crate) fn read(data: &'a DataBundle, budgets: &Budgets) -> Result<Self, String> {
        let table = data
            .primary()
            .ok_or_else(|| "the data has no table".to_owned())?;
        let metadata = table.schema().metadata();
        let text = metadata
            .get(META_EXPR)
            .ok_or_else(|| "the data does not come from a function source".to_owned())?;
        let variables: BTreeMap<String, f64> = metadata
            .get(META_VARIABLES)
            .map(|json| serde_json::from_str(json))
            .transpose()
            .map_err(|err| format!("the function's variables cannot be read ({err})"))?
            .unwrap_or_default();
        let expr = Expr::parse(text, &variables, &budgets.expr)
            .map_err(|err| format!("the function's expression cannot be read ({err})"))?;
        if table.num_rows() > u64::try_from(budgets.max_samples * 2).unwrap_or(u64::MAX) {
            return Err("the table has more rows than the sample limit".to_owned());
        }
        let column = table.column("x").map_err(|err| err.to_string())?;
        let column = column
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| "the `x` column is not numeric".to_owned())?;
        let mut rows: Vec<(f64, u64)> = column
            .iter()
            .zip(0_u64..)
            .filter_map(|(x, row)| x.filter(|x| x.is_finite()).map(|x| (x, row)))
            .collect();
        rows.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut xs: Vec<f64> = rows.iter().map(|(x, _)| *x).collect();
        xs.dedup();
        Ok(Self {
            table,
            expr,
            rows,
            xs,
        })
    }

    /// The spans where the function leaves `window`, as `(start x, above, first row)`
    /// (RFC-0007 Q5).
    pub(crate) fn offscreen(&self, window: [f64; 2], budgets: &Budgets) -> Vec<(f64, bool, u64)> {
        let budget = ScanBudget {
            max_evaluations: budgets.max_evaluations,
            max_features: ScanBudget::DEFAULT.max_features,
        };
        let found = scan::scan(&self.expr, &self.xs, Some(window), &budget);
        found
            .offscreen
            .iter()
            .filter_map(|span| {
                let row = self
                    .rows
                    .iter()
                    .find(|(x, _)| *x >= span.span.start)
                    .map(|&(_, row)| row)?;
                Some((span.span.start, span.above, row))
            })
            .collect()
    }
}

/// The spans where the function table in `data` leaves the y-`window`, as
/// [`FeatureKind::Offscreen`] features in increasing `x` (RFC-0007 Q5): the spans the
/// off-screen cues mark, for a text list of the cues (C03-T07). Empty when `data` is not a
/// function table.
#[must_use]
pub fn offscreen_features(data: &DataBundle, window: [f64; 2], budgets: &Budgets) -> Vec<Feature> {
    let Ok(function) = FunctionTable::read(data, budgets) else {
        return Vec::new();
    };
    let budget = ScanBudget {
        max_evaluations: budgets.max_evaluations,
        max_features: ScanBudget::DEFAULT.max_features,
    };
    scan::scan(&function.expr, &function.xs, Some(window), &budget)
        .offscreen
        .iter()
        .map(|span| Feature {
            kind: FeatureKind::Offscreen { above: span.above },
            x: span.span.start,
            end: span.span.end,
            y: None,
        })
        .collect()
}

/// The annotations of the function table in `data`, recomputed from the expression its
/// metadata records, at the table's own `x` values.
///
/// # Errors
///
/// Why there are none, as a clause: the data is not a function table, or its expression
/// cannot be read.
pub fn function_features(data: &DataBundle, budgets: &Budgets) -> Result<Vec<Annotation>, String> {
    let function = FunctionTable::read(data, budgets)?;
    let (table, expr, rows, xs) = (function.table, function.expr, function.rows, function.xs);
    let scan_budget = ScanBudget {
        max_evaluations: budgets.max_evaluations,
        max_features: ScanBudget::DEFAULT.max_features,
    };
    let scan = scan::scan(&expr, &xs, None, &scan_budget);
    let features = critical::features(&Calculus::new(expr), &xs, &scan, scan_budget.max_features);
    let range = |start: u64, end: u64| RowRange {
        table: table.id(),
        generation: table.generation(),
        start,
        end,
    };
    let nearest = |x: f64| {
        rows.iter()
            .min_by(|a, b| (a.0 - x).abs().total_cmp(&(b.0 - x).abs()))
            .map(|&(_, row)| row)
    };
    let mut annotations = Vec::with_capacity(features.len());
    for feature in &features {
        let covered = if feature.is_point() {
            nearest(feature.x).map(|row| range(row, row + 1))
        } else {
            let inside: Vec<u64> = rows
                .iter()
                .filter(|(x, _)| *x >= feature.x && *x <= feature.end)
                .map(|&(_, row)| row)
                .collect();
            match (inside.iter().min(), inside.iter().max()) {
                (Some(&first), Some(&last)) => Some(range(first, last + 1)),
                _ => nearest(feature.x).map(|row| range(row, row + 1)),
            }
        };
        let Some(covered) = covered else { continue };
        let mut annotation = Annotation::new(CRITICAL_POINTS, feature.kind.label(), covered);
        annotation.value = feature.y;
        annotation.rules = vec![RuleId::from_static("A11Y-28")];
        annotations.push(annotation);
    }
    Ok(annotations)
}

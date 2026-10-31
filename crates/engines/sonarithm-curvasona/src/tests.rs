// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use arrow_array::Array;
use arrow_array::cast::AsArray;
use arrow_array::types::Float64Type;
use serde_json::{Value, json};
use sonarithm_compiler::{Compiled, Compiler, Failure, Platform};
use sonarithm_core::SpecPointer;
use sonarithm_core::engine::{Budget, CancelToken, CompileContext, Registry};
use sonarithm_core::score::review::ReviewOptions;
use sonarithm_core::score::{EngineVersion, ScoreEvent};
use sonarithm_numisona::Numisona;

use super::*;

fn context() -> CompileContext {
    CompileContext::new(7, Budget::default(), CancelToken::new())
}

fn function_source(source: &Value) -> SpecFragment {
    SpecFragment::new(SpecPointer::root().key("data").key("source"), source).unwrap()
}

fn ingest(source: &Value) -> (Result<DataBundle, CompileError>, CompileContext) {
    let mut registry = Registry::<DataBundle>::new();
    registry.add_engine(&Curvasona::new()).unwrap();
    let (owner, function) = registry.source(SourceKind::Function).unwrap();
    assert_eq!(owner, &ENGINE);
    let mut cx = context();
    let result = function.ingest(&function_source(source), &mut cx);
    (result, cx)
}

fn ingest_error(source: &Value) -> (String, String) {
    let (Err(CompileError::Diagnostic(diagnostic)), _) = ingest(source) else {
        unreachable!("expected a diagnostic");
    };
    let at = diagnostic
        .location
        .as_ref()
        .map(|p| p.as_str().to_owned())
        .unwrap_or_default();
    (diagnostic.code.as_str().to_owned(), at)
}

fn column(bundle: &DataBundle, name: &str) -> Vec<Option<f64>> {
    let table = bundle.primary().unwrap();
    let column = table.column(name).unwrap();
    column.as_primitive::<Float64Type>().iter().collect()
}

#[test]
fn the_function_source_gives_x_y_and_dy_with_synthetic_provenance() {
    let source =
        json!({"kind": "function", "expr": "sin(x) / x", "domain": [-20, 20], "samples": 2000});
    let (bundle, cx) = ingest(&source);
    let bundle = bundle.unwrap();
    assert!(cx.diagnostics().is_empty(), "{:?}", cx.diagnostics());
    let table = bundle.primary().unwrap();
    assert_eq!(table.num_rows(), 2000_u64);
    let names: Vec<&str> = table
        .schema()
        .fields()
        .iter()
        .map(|f| f.name().as_str())
        .collect();
    assert_eq!(&names[..3], ["x", "y", "dy"]);
    let provenance = RowProvenance::read(table).unwrap();
    assert!(
        provenance
            .rows()
            .iter()
            .all(|p| p.class == ProvenanceClass::Synthetic)
    );
    assert_eq!(bundle.source().extractor, EXTRACTOR);
    assert_eq!(bundle.source().kind, DataSourceKind::Function);
    let xs = column(&bundle, "x");
    let ys = column(&bundle, "y");
    let dys = column(&bundle, "dy");
    for ((x, y), dy) in xs.iter().zip(&ys).zip(&dys) {
        let (x, y, dy) = (x.unwrap(), y.unwrap(), dy.unwrap());
        assert!((y - x.sin() / x).abs() < 1e-12);
        let exact = (x * x.cos() - x.sin()) / (x * x);
        assert!((dy - exact).abs() < 1e-9, "dy at {x}: {dy} vs {exact}");
    }
}

#[test]
fn snr_eng_406_undefined_samples_have_no_y_and_are_reported() {
    let source = json!({"kind": "function", "expr": "ln(x)", "domain": [-1, 1], "samples": 5});
    let (bundle, cx) = ingest(&source);
    let bundle = bundle.unwrap();
    assert_eq!(
        column(&bundle, "y"),
        [None, None, None, Some(0.5_f64.ln()), Some(0.0)]
    );
    assert_eq!(column(&bundle, "dy")[..3], [None, None, None]);
    let codes: Vec<(&str, Option<&str>)> = cx
        .diagnostics()
        .iter()
        .map(|d| {
            (
                d.code.as_str(),
                d.location.as_ref().map(SpecPointer::as_str),
            )
        })
        .collect();
    assert_eq!(codes, [("SNR-ENG-406", Some("/data/source/expr"))]);
}

#[test]
fn snr_eng_405_functions_without_a_symbolic_derivative_get_a_numerical_one() {
    let source = json!({"kind": "function", "expr": "abs(x - 1)", "domain": [-2, 4], "samples": 7});
    let (bundle, cx) = ingest(&source);
    let dys = column(&bundle.unwrap(), "dy");
    assert!((dys[0].unwrap() + 1.0).abs() < 1e-6);
    assert!((dys[6].unwrap() - 1.0).abs() < 1e-6);
    assert_eq!(
        cx.diagnostics().iter().next().unwrap().code.as_str(),
        "SNR-ENG-405"
    );
}

#[test]
fn variables_are_read_from_the_source() {
    let source = json!({"kind": "function", "expr": "a * x^2", "domain": [0, 2], "samples": 3,
        "variables": {"a": 3}});
    let (bundle, _) = ingest(&source);
    let bundle = bundle.unwrap();
    assert_eq!(column(&bundle, "y"), [Some(0.0), Some(3.0), Some(12.0)]);
    assert_eq!(column(&bundle, "dy"), [Some(0.0), Some(6.0), Some(12.0)]);
}

#[test]
fn errors_point_at_the_key_they_are_about() {
    for (source, code, at) in [
        (
            json!({"kind": "function", "expr": "2x", "domain": [0, 1]}),
            "SNR-ENG-400",
            "/data/source/expr",
        ),
        (
            json!({"kind": "function", "expr": "log(x)", "domain": [0, 1]}),
            "SNR-ENG-401",
            "/data/source/expr",
        ),
        (
            json!({"kind": "function", "expr": "x", "domain": [0, 1], "variables": {"pi": 3}}),
            "SNR-ENG-403",
            "/data/source/variables",
        ),
        (
            json!({"kind": "function", "expr": "x", "domain": [0, 1], "samples": 200_000}),
            "SNR-ENG-404",
            "/data/source/samples",
        ),
        (
            json!({"kind": "function", "expr": "x", "domain": [0, 1], "step": 1e-7}),
            "SNR-ENG-404",
            "/data/source/step",
        ),
    ] {
        assert_eq!(
            ingest_error(&source),
            (code.to_owned(), at.to_owned()),
            "{source}"
        );
    }
}

#[test]
fn snr_eng_404_the_evaluation_budget_holds() {
    let budgets = Budgets {
        max_evaluations: 100,
        ..Budgets::DEFAULT
    };
    let error = sample_function(
        &FunctionSpec::new("x^2", [0.0, 1.0]).with_samples(60),
        &budgets,
        (BundleId(0), FUNCTION_TABLE),
    )
    .unwrap_err();
    assert_eq!(error.diagnostic.code.as_str(), "SNR-ENG-404");
}

#[test]
fn sampling_is_deterministic() {
    let function = FunctionSpec::new("exp(-x^2) * cos(3*x)", [-3.0, 3.0]).with_samples(500);
    let a = sample_function(&function, &Budgets::DEFAULT, (BundleId(0), FUNCTION_TABLE)).unwrap();
    let b = sample_function(&function, &Budgets::DEFAULT, (BundleId(0), FUNCTION_TABLE)).unwrap();
    assert_eq!(
        sonarithm_data::bundle_fingerprint(&a.bundle).unwrap(),
        sonarithm_data::bundle_fingerprint(&b.bundle).unwrap()
    );
    assert_eq!(a.derivative, Derivative::Symbolic);
}

fn compile(spec: &Value) -> Result<Compiled, Failure> {
    let platform = Platform::new();
    let curvasona = Curvasona::new();
    let numisona = Numisona::new();
    let compiler: Compiler<DataBundle> =
        Compiler::new(&[&platform, &curvasona, &numisona]).unwrap();
    compiler.compile(&spec.to_string(), &CancelToken::new())
}

/// 04 §6.3's example compiles: the function is a glide with pitch from `y` and brightness
/// from `dy`. Its `analysis` block (critical-point cues) arrives with C03-T03.
#[test]
fn the_04_6_3_function_example_compiles() {
    let spec = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "function", "expr": "sin(x) / x", "domain": [-20, 20], "samples": 2000 } },
        "substrate": { "time": { "pace": { "duration": { "s": 8 } } } },
        "tracks": [{ "id": "f", "mark": "glide", "channels": { "pitch": { "field": "y" }, "brightness": { "field": "dy" } } }]
    });
    let compiled = compile(&spec).unwrap();
    let track = &compiled.score.tracks[0];
    assert_eq!(track.id.as_str(), "f");
    assert!(
        track
            .events
            .iter()
            .all(|e| matches!(e, ScoreEvent::Glide(_)))
    );
    assert_ne!(track.events.len(), 0);
    assert!(compiled.score.header.engines.contains_key(&ENGINE));
    assert!((compiled.score.header.duration.0 - 8.0).abs() < 1e-6);
    assert_eq!(compiled.provenance.get("synthetic"), Some(&1.0));
}

/// The Score of a small function, for review (testing-strategy §4).
#[test]
fn a_parabola_score_snapshot() {
    let spec = json!({
        "specVersion": "0.1", "seed": 3,
        "data": { "source": { "kind": "function", "expr": "x^2 - 1", "domain": [-2, 2], "samples": 9 } },
        "substrate": { "time": { "pace": { "duration": { "s": 4 } } } },
        "tracks": [{ "id": "f", "mark": "glide", "channels": { "pitch": { "field": "y" } } }]
    });
    let mut score = compile(&spec).unwrap().score;
    for version in score.header.engines.values_mut() {
        *version = EngineVersion("*".to_owned());
    }
    insta::assert_snapshot!(score.review_json(&ReviewOptions::default()));
}

/// Without CURVASONA a function source is reported as unavailable, not silently empty.
#[test]
fn snr_comp_function_sources_need_curvasona() {
    let platform = Platform::new();
    let numisona = Numisona::new();
    let compiler: Compiler<DataBundle> = Compiler::new(&[&platform, &numisona]).unwrap();
    let spec = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "function", "expr": "x", "domain": [0, 1] } },
        "tracks": [{ "id": "f", "mark": "glide", "channels": { "pitch": { "field": "y" } } }]
    });
    let Err(Failure::Invalid(diagnostics)) =
        compiler.compile(&spec.to_string(), &CancelToken::new())
    else {
        unreachable!("expected a failure");
    };
    assert!(
        diagnostics.iter().any(|d| d.message.contains("curvasona")),
        "{diagnostics:?}"
    );
}

#[test]
fn the_manifest_owns_the_function_source() {
    let manifest = Curvasona::manifest();
    assert_eq!(manifest.id, ENGINE);
    assert_eq!(manifest.spec_keys, ["data.source.kind=function"]);
}

#[test]
fn every_column_has_the_same_length() {
    let sampled = sample_function(
        &FunctionSpec::new("1 / x", [-1.0, 1.0]).with_samples(11),
        &Budgets::DEFAULT,
        (BundleId(0), FUNCTION_TABLE),
    )
    .unwrap();
    let table = sampled.bundle.primary().unwrap();
    assert_eq!(table.column("y").unwrap().len(), 11);
    assert_eq!(table.column("y").unwrap().null_count(), 1);
    assert_eq!(sampled.undefined, 1);
}

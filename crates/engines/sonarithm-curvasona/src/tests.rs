// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use arrow_array::Array;
use arrow_array::cast::AsArray;
use arrow_array::types::Float64Type;
use serde_json::{Value, json};
use sonarithm_audaxis::Audaxis;
use sonarithm_compiler::{Compiled, Compiler, Failure, Platform};
use sonarithm_core::SpecPointer;
use sonarithm_core::engine::{Budget, CancelToken, CompileContext, Registry};
use sonarithm_core::nav::NavKind;
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
    // The 2000 samples, one empty row at the hole at 0 (C03-T02) and one row at each zero,
    // extremum and inflection (C03-T03).
    let rows = table.num_rows();
    assert!(rows > 2001 && rows < 2100, "{rows}");
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
    assert_eq!(ys.iter().filter(|y| y.is_none()).count(), 1);
    let hole = xs
        .iter()
        .zip(&ys)
        .find(|(_, y)| y.is_none())
        .unwrap()
        .0
        .unwrap();
    assert!(hole.abs() < 1e-12, "{hole}");
    for ((x, y), dy) in xs.iter().zip(&ys).zip(&dys) {
        let (Some(x), Some(y), Some(dy)) = (*x, *y, *dy) else {
            continue;
        };
        assert!((y - libm::sin(x) / x).abs() < 1e-12);
        let exact = (x * libm::cos(x) - libm::sin(x)) / (x * x);
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
        [None, None, None, Some(libm::log(0.5)), Some(0.0)]
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
    let source =
        json!({"kind": "function", "expr": "max(x - 1, 1 - x)", "domain": [-2, 4], "samples": 7});
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

fn example_6_3() -> Value {
    json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "function", "expr": "sin(x) / x", "domain": [-20, 20], "samples": 2000 } },
        "substrate": { "time": { "pace": { "duration": { "s": 8 } } } },
        "tracks": [{ "id": "f", "mark": "glide", "channels": { "pitch": { "field": "y" }, "brightness": { "field": "dy" } } }],
        "analysis": { "detect": ["critical-points"], "cue": "earcon" }
    })
}

/// 04 §6.3's example compiles: the function is a glide with pitch from `y` and brightness
/// from `dy`, and the `critical-points` detector runs (C03-T03).
#[test]
fn the_04_6_3_function_example_compiles() {
    let spec = example_6_3();
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

/// C03-T03: the critical points reach AUDAXIS's navigation tree as insight nodes, through
/// core annotations only (no engine depends on another, `check-deps`).
#[test]
fn a11y_23_critical_points_are_insight_nodes() {
    let platform = Platform::new();
    let curvasona = Curvasona::new();
    let numisona = Numisona::new();
    let audaxis = Audaxis::new();
    let compiler: Compiler<DataBundle> =
        Compiler::new(&[&platform, &curvasona, &numisona, &audaxis]).unwrap();
    let output = compiler
        .compile(&example_6_3().to_string(), &CancelToken::new())
        .unwrap();
    let tree = output.nav.unwrap();
    let labels: Vec<&str> = tree
        .iter()
        .filter(|(_, node)| matches!(node.kind, NavKind::Insight(_)))
        .filter_map(|(_, node)| node.label.as_deref())
        .collect();
    let count = |label: &str| labels.iter().filter(|&&l| l == label).count();
    assert_eq!(count("zero"), 12, "{labels:?}");
    assert_eq!(count("local maximum"), 4);
    assert_eq!(count("local minimum"), 6);
    assert_eq!(count("hole"), 1);
    assert_eq!(count("horizontal asymptote to the left"), 1);
    assert_eq!(count("horizontal asymptote to the right"), 1);
    let groups: Vec<String> = tree
        .iter()
        .filter_map(|(_, node)| match &node.kind {
            NavKind::InsightGroup(group) => Some(group.analyzer.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(groups, ["critical-points"]);
    // Every insight lands on a point.
    for (_, node) in tree.iter() {
        if let NavKind::Insight(insight) = &node.kind {
            assert!(insight.target.is_some(), "{:?}", node.label);
        }
    }
}

/// The table has a row at each zero, extremum and inflection, so the glide passes
/// through it and its insight lands on it (C03-T03).
#[test]
fn critical_points_get_their_own_rows() {
    let sampled = sample_function(
        &FunctionSpec::new("x^3 - 3*x", [-3.0, 3.0]).with_samples(7),
        &Budgets::DEFAULT,
        (BundleId(0), FUNCTION_TABLE),
    )
    .unwrap();
    let xs: Vec<f64> = column(&sampled.bundle, "x").into_iter().flatten().collect();
    let root3 = 3.0_f64.sqrt();
    for want in [-root3, -1.0, 0.0, 1.0, root3] {
        assert!(
            xs.iter().any(|&x| (x - want).abs() < 1e-12),
            "{want} in {xs:?}"
        );
    }
    let annotations = function_features(&sampled.bundle, &Budgets::DEFAULT).unwrap();
    let labels: Vec<&str> = annotations.iter().map(|a| a.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "zero",
            "local maximum",
            "zero",
            "inflection point",
            "local minimum",
            "zero"
        ]
    );
    assert!(annotations.iter().all(|a| a.range.len() == 1));
}

/// Critical points on data that is not a function are refused with a warning.
#[test]
fn snr_eng_420_critical_points_need_a_function() {
    let platform = Platform::new();
    let curvasona = Curvasona::new();
    let numisona = Numisona::new();
    let compiler: Compiler<DataBundle> =
        Compiler::new(&[&platform, &curvasona, &numisona]).unwrap();
    let spec = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "inline", "values": [{"x": 1, "y": 2}, {"x": 2, "y": 3}] } },
        "tracks": [{ "id": "f", "mark": "tone", "channels": { "pitch": { "field": "y" } } }],
        "analysis": { "detect": ["critical-points"] }
    });
    let output = compiler
        .compile(&spec.to_string(), &CancelToken::new())
        .unwrap();
    assert!(
        output
            .diagnostics
            .iter()
            .any(|d| d.code.as_str() == "SNR-ENG-420")
    );
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
    assert_eq!(
        manifest.spec_keys,
        ["data.source.kind=function", "tracks[].audify"]
    );
}

/// A glide breaks at each pole: the table has an empty row there (C03-T02).
#[test]
fn poles_between_samples_get_an_empty_row() {
    let sampled = sample_function(
        &FunctionSpec::new("tan(x)", [-2.0, 2.0]).with_samples(10),
        &Budgets::DEFAULT,
        (BundleId(0), FUNCTION_TABLE),
    )
    .unwrap();
    let xs = column(&sampled.bundle, "x");
    let ys = column(&sampled.bundle, "y");
    // Ten samples, the two poles and the zero (and inflection) at 0.
    assert_eq!(xs.len(), 13);
    let empty: Vec<f64> = xs
        .iter()
        .zip(&ys)
        .filter(|(_, y)| y.is_none())
        .map(|(x, _)| x.unwrap())
        .collect();
    let half_pi = std::f64::consts::FRAC_PI_2;
    assert_eq!(empty.len(), 2);
    assert!((empty[0] + half_pi).abs() < 1e-12 && (empty[1] - half_pi).abs() < 1e-12);
    assert!(sampled.diagnostics.is_empty(), "{:?}", sampled.diagnostics);
    assert_eq!(sampled.scan.singularities.len(), 2);
}

#[test]
fn snr_eng_407_a_scan_that_runs_out_says_so() {
    let budgets = Budgets {
        max_evaluations: 4100,
        ..Budgets::DEFAULT
    };
    let sampled = sample_function(
        &FunctionSpec::new("tan(x) + 1/(x - 0.3) + floor(3*x)", [-5.0, 5.0]).with_samples(1000),
        &budgets,
        (BundleId(0), FUNCTION_TABLE),
    )
    .unwrap();
    assert!(sampled.scan.truncated);
    let codes: Vec<&str> = sampled
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert!(codes.contains(&"SNR-ENG-407"), "{codes:?}");
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

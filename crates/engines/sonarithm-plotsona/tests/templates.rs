// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! C01-T09: chart-type templates through the `template` key (RFC-0004 §2). Every corpus
//! chart compiles through the template its type calls for, the designs keep NUMISONA's
//! conformance rules, and the templates are data-agnostic (SDR-03).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use arrow_array::Float64Array;
use arrow_array::cast::AsArray;
use serde_json::{Value, json};
use sonarithm_compiler::{Compiled, Compiler, Failure, Platform};
use sonarithm_core::Severity;
use sonarithm_core::engine::CancelToken;
use sonarithm_core::score::review::ReviewOptions;
use sonarithm_core::score::{EngineVersion, Note, Score, ScoreEvent};
use sonarithm_data::DataBundle;
use sonarithm_numisona::Numisona;
use sonarithm_plotsona::templates::{TEMPLATES, template, template_for};
use sonarithm_plotsona::{ExtractOptions, Plotsona, extract};

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus")
}

/// Every corpus chart, as (folder, file name), in order.
fn corpus() -> Vec<(PathBuf, String)> {
    let mut charts = Vec::new();
    for folder in ["office", "chartjson", "svg", "pdf"] {
        let dir = corpus_dir().join(folder);
        let mut names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| !n.ends_with(".truth.json"))
            .collect();
        names.sort();
        charts.extend(names.into_iter().map(|n| (dir.clone(), n)));
    }
    charts
}

fn compile(dir: &Path, spec: &Value) -> Result<Compiled, Failure> {
    let platform = Platform::new();
    let plotsona = Plotsona::with_base_dir(dir);
    let numisona = Numisona::new();
    let compiler: Compiler<DataBundle> = Compiler::new(&[&platform, &plotsona, &numisona]).unwrap();
    compiler.compile(&spec.to_string(), &CancelToken::new())
}

fn keyed(name: &str, id: &str) -> Value {
    json!({
        "specVersion": "0.2",
        "data": { "source": { "kind": "chart", "path": name } },
        "template": { "id": id },
    })
}

fn notes(track: &sonarithm_core::score::ScoreTrack) -> Vec<&Note> {
    track
        .events
        .iter()
        .filter_map(|e| match e {
            ScoreEvent::Note(note) => Some(note),
            _ => None,
        })
        .collect()
}

fn failed(failure: Failure) -> String {
    match failure {
        Failure::Invalid(diagnostics) => diagnostics
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n"),
        other => format!("{other:?}"),
    }
}

/// One corpus chart compiled through its template.
struct Case {
    name: String,
    template: &'static str,
    chart: sonarithm_plotsona::Extraction,
    compiled: Compiled,
}

fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for (dir, name) in corpus() {
        let bytes = std::fs::read(dir.join(&name)).unwrap();
        let chart = extract(&bytes, &name, &ExtractOptions::new()).unwrap();
        let Some(id) = template_for(chart.chart.chart_type) else {
            continue;
        };
        let compiled = compile(&dir, &keyed(&name, id))
            .unwrap_or_else(|f| panic!("{name} through {id}:\n{}", failed(f)));
        out.push(Case {
            name,
            template: id,
            chart,
            compiled,
        });
    }
    out
}

/// C01-T09 acceptance: each template compiles every compatible corpus chart. The
/// snapshot is the matrix of chart, template, binding and note count; each PLOTSONA
/// template is used by at least two charts.
#[test]
fn every_template_compiles_every_compatible_corpus_chart() {
    let cases = cases();
    let mut matrix = Vec::new();
    let mut used: BTreeMap<&str, usize> = BTreeMap::new();
    for case in &cases {
        let events: usize = case
            .compiled
            .score
            .tracks
            .iter()
            .map(|t| notes(t).len())
            .sum();
        assert!(events > 0, "{}", case.name);
        let binding = case
            .compiled
            .diagnostics
            .iter()
            .find(|d| d.code.as_str() == "SNR-COMP-033")
            .map_or_else(String::new, |d| d.message.clone());
        matrix.push(format!(
            "{} → {}: {binding} {events} notes",
            case.name, case.template
        ));
        *used.entry(case.template).or_default() += 1;
    }
    for template in &TEMPLATES {
        assert!(
            used.get(template.id).copied().unwrap_or(0) >= 2,
            "{} compiles fewer than two corpus charts",
            template.id
        );
    }
    assert!(cases.len() >= 60, "only {} charts", cases.len());
    insta::assert_snapshot!(matrix.join("\n"));
}

/// The rules NUMISONA's conformance tests check, on every chart compiled through its
/// template: no NUMISONA warning (SDR-19 pace, SDR-27 pan, SDR-23 loudness, …); every
/// pitch within the default 400–1400 Hz range (SDR-08); for PLOTSONA's templates, pitch
/// rises with the value (SDR-07), bars and slices last longer as they grow (A11Y-13), and
/// each series has its own timbre (SDR-25). Experimental templates say so (`SNR-ENG-380`).
#[test]
fn chart_templates_keep_the_numisona_conformance_rules() {
    for case in cases() {
        let name = &case.name;
        let warnings: Vec<String> = case
            .compiled
            .diagnostics
            .iter()
            .filter(|d| {
                // Engine warnings; AUDAXIS (context cues) is not in this test build.
                d.severity == Severity::Warning
                    && d.code.as_str().starts_with("SNR-ENG-")
                    && d.code.as_str() != "SNR-ENG-380"
            })
            .map(ToString::to_string)
            .collect();
        assert!(warnings.is_empty(), "{name}: {warnings:#?}");
        let experimental = case
            .compiled
            .diagnostics
            .iter()
            .any(|d| d.code.as_str() == "SNR-ENG-380");
        assert_eq!(
            experimental,
            template(case.template).is_some_and(|t| t.experimental),
            "{name}"
        );
        let track = &case.compiled.score.tracks[0];
        let notes = notes(track);
        for note in &notes {
            let hz = note.pitch.to_hz().0;
            assert!((399.9..=1400.1).contains(&hz), "{name}: {hz} Hz");
        }
        if template(case.template).is_none() {
            continue;
        }
        // The chart table's y column, by row.
        let table = case.chart.bundle.primary().unwrap();
        let column = table.column("y").unwrap();
        let y: &Float64Array = column.as_primitive();
        let mut by_value: Vec<(f64, &Note)> = notes
            .iter()
            .map(|n| {
                (
                    y.value(usize::try_from(n.src.as_ref().unwrap().row).unwrap()),
                    *n,
                )
            })
            .collect();
        by_value.sort_by(|a, b| a.0.total_cmp(&b.0));
        for pair in by_value.windows(2) {
            assert!(
                pair[0].1.pitch.0 <= pair[1].1.pitch.0 + 1e-9,
                "{name}: SDR-07"
            );
            if case.template != "chart-line" {
                assert!(pair[0].1.dur.0 <= pair[1].1.dur.0 + 1e-9, "{name}: A11Y-13");
            }
        }
        let series = case.chart.chart.series.len();
        if series > 1 && case.template != "chart-pie" {
            let timbres: std::collections::BTreeSet<String> =
                notes.iter().map(|n| format!("{:?}", n.timbre)).collect();
            assert!(timbres.len() >= series, "{name}: SDR-25, {timbres:?}");
        }
    }
}

fn release_independent(score: &Score) -> Score {
    let mut score = score.clone();
    for version in score.header.engines.values_mut() {
        *version = EngineVersion("*".to_owned());
    }
    score
}

/// Score snapshots: one chart per PLOTSONA template, in the review form.
#[test]
fn chart_template_scores_are_reviewed() {
    let picks = [
        ("chart-line", "office", "xlsx-line.xlsx"),
        ("chart-bar", "office", "xlsx-grouped-bar.xlsx"),
        ("chart-stacked-bar", "office", "xlsx-stacked-bar.xlsx"),
        ("chart-pie", "office", "xlsx-pie.xlsx"),
    ];
    for (id, folder, name) in picks {
        let dir = corpus_dir().join(folder);
        let compiled = compile(&dir, &keyed(name, id)).unwrap_or_else(|f| panic!("{}", failed(f)));
        let review = release_independent(&compiled.score).review_json(&ReviewOptions::default());
        insta::assert_snapshot!(id, review);
    }
}

/// SDR-03 (MUST): the chart templates bind any compatible dataset, not just charts, and a
/// spec naming the template compiles to the same tracks as the spec with the template's
/// sections written out (the template round-trips through the spec).
#[test]
fn sdr_03_chart_templates_reapply_and_round_trip() {
    let values = json!([
        { "quarter": "Q1", "region": "North", "sales": 12.5 },
        { "quarter": "Q2", "region": "North", "sales": 14.0 },
        { "quarter": "Q1", "region": "South", "sales": 9.0 },
        { "quarter": "Q2", "region": "South", "sales": 11.5 },
    ]);
    let source = json!({ "kind": "inline", "values": values });
    let bind = json!({ "category": "quarter", "value": "sales", "series": "region" });
    let spec = json!({
        "specVersion": "0.2",
        "data": { "source": source },
        "template": { "id": "chart-bar", "bind": bind },
    });
    let keyed = compile(&corpus_dir(), &spec).unwrap_or_else(|f| panic!("{}", failed(f)));
    let bundle = sonarithm_data::read_bytes(
        values.to_string().as_bytes(),
        sonarithm_data::Format::Json,
        "inline.json",
        &sonarithm_data::ReadOptions::new(),
    )
    .unwrap()
    .bundle;
    let table = bundle.primary().unwrap();
    let bar = template("chart-bar").unwrap();
    let given = bind
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_owned()))
        .collect();
    let bindings = bar.bind(table, &given).unwrap();
    let mut written = bar.sections(table, &bindings);
    written["specVersion"] = json!("0.2");
    written["data"] = json!({ "source": source });
    let plain = compile(&corpus_dir(), &written).unwrap_or_else(|f| panic!("{}", failed(f)));
    assert_eq!(keyed.score.tracks, plain.score.tracks);
    let timbres: std::collections::BTreeSet<String> = notes(&keyed.score.tracks[0])
        .iter()
        .map(|n| format!("{:?}", n.timbre))
        .collect();
    assert_eq!(timbres.len(), 2, "two regions, two timbres (SDR-25)");
}

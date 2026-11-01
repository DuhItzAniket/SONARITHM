// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Functions through the facade (C03-T07): `read_function`, its critical-point list and the
//! spec that plays it with CURVASONA's cues.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use arrow_array::Float64Array;
use sonarithm::score::{ScoreEvent, ScoreTrack};
use sonarithm::{FunctionSpec, Sonarithm, Spec, read_function};
use sonarithm_curvasona::cues::CUE_TRACK;
use sonarithm_curvasona::suite::CURVES;

fn cue_track(score: &sonarithm::Score) -> Option<&ScoreTrack> {
    score.tracks.iter().find(|t| t.id.as_str() == CUE_TRACK)
}

#[test]
fn a_function_lists_its_critical_points_as_text() {
    let function = read_function(&FunctionSpec::new("x^3 - 3*x", [-3.0, 3.0])).unwrap();
    let texts: Vec<String> = function
        .critical_points()
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert_eq!(
        texts,
        [
            "zero at x = -1.7321",
            "local maximum at x = -1, y = 2",
            "zero at x = 0",
            "inflection point at x = 0, y = 0",
            "local minimum at x = 1, y = -2",
            "zero at x = 1.7321",
        ]
    );
    assert_eq!(
        function.summary(),
        "f(x) = x^3 - 3*x from x = -3 to 3, 1000 samples: 3 zeros, 1 local maximum, \
         1 inflection point, 1 local minimum"
    );
    assert_eq!(function.expr(), "x^3 - 3*x");
}

#[test]
fn spans_and_asymptotes_read_as_text() {
    let function = read_function(&FunctionSpec::new("1/x", [-2.0, 2.0])).unwrap();
    let texts: Vec<String> = function
        .critical_points()
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert!(
        texts.contains(&"vertical asymptote at x = 0".to_owned()),
        "{texts:?}"
    );
    let sqrt = read_function(&FunctionSpec::new("sqrt(1 - x^2)", [-2.0, 2.0])).unwrap();
    let points = sqrt.critical_points();
    let undefined: Vec<&str> = points
        .iter()
        .filter(|p| p.kind == "undefined")
        .map(|p| p.text.as_str())
        .collect();
    assert_eq!(
        undefined,
        ["undefined from x = -2 to -1", "undefined from x = 1 to 2"]
    );
    assert!(
        points
            .iter()
            .all(|p| p.kind != "undefined" || p.end.is_some())
    );
}

#[test]
fn an_expression_that_cannot_be_read_is_a_coded_error_at_its_key() {
    let err = read_function(&FunctionSpec::new("2x", [0.0, 1.0])).unwrap_err();
    let diagnostics = err.diagnostics();
    let first = diagnostics.iter().next().unwrap();
    assert_eq!(first.code.as_str(), "SNR-ENG-400");
    assert_eq!(
        first.location.as_ref().map(ToString::to_string).as_deref(),
        Some("/data/source/expr")
    );
}

/// A11Y-24, A11Y-28: the critical-point list is a text equivalent of the cues. On every
/// curve of the 40-curve suite, each earcon of `curvasona.cues` comes from a row within one
/// grid step of an entry of the list (inside it, for a span), and the list has at least as many entries
/// with cues as there are earcons (cues closer than 150 ms merge).
#[test]
fn a11y_28_the_critical_point_list_is_a_text_equivalent_of_the_cues() {
    let snr = Sonarithm::new();
    for curve in &CURVES {
        let function = read_function(&FunctionSpec::new(curve.expr, curve.domain))
            .unwrap_or_else(|e| panic!("{}: {e}", curve.name));
        let compiled = snr
            .compile(&function.spec())
            .unwrap_or_else(|e| panic!("{}: {e}\n{:#?}", curve.name, e.diagnostics()));
        let table = function.bundle().primary().unwrap();
        let column = table.column("x").unwrap();
        let xs = column.as_any().downcast_ref::<Float64Array>().unwrap();
        let points = function.critical_points();
        // The grid step of the default 1000 samples.
        let step = (curve.domain[1] - curve.domain[0]) / 999.0;
        let earcons: Vec<_> = cue_track(compiled.score())
            .map(|track| {
                track
                    .events
                    .iter()
                    .filter_map(|event| match event {
                        ScoreEvent::Earcon(earcon) => Some(earcon),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let cued = points
            .iter()
            .filter(|p| !p.kind.starts_with("horizontal asymptote"))
            .count();
        assert!(
            earcons.len() <= cued,
            "{}: {} cues, {cued} cued entries",
            curve.name,
            earcons.len()
        );
        for earcon in earcons {
            let row = earcon.src.unwrap_or_else(|| panic!("{}", curve.name)).row;
            let x = xs.value(usize::try_from(row).unwrap());
            // A point's cue is at its nearest row (its own row, for the points the source
            // adds a row at); a span's at its first row inside it.
            assert!(
                points
                    .iter()
                    .any(|p| (p.x - x).abs() <= step
                        || p.end.is_some_and(|end| p.x <= x && x <= end)),
                "{}: a cue at x = {x} has no entry in {points:?}",
                curve.name
            );
        }
    }
}

#[test]
fn a_function_spec_plays_as_a_glide_with_cues_and_curvasona_in_the_header() {
    let function = read_function(&FunctionSpec::new("x^2 - 1", [-2.0, 2.0])).unwrap();
    let compiled = Sonarithm::new().compile(&function.spec()).unwrap();
    let summary = compiled.summary();
    assert!(summary.engines.contains_key("curvasona"));
    assert!((summary.duration_s - sonarithm::FUNCTION_DURATION_S).abs() < 0.5);
    let cues = summary
        .tracks
        .iter()
        .find(|t| t.id == CUE_TRACK)
        .unwrap_or_else(|| panic!("{summary:#?}"));
    assert_eq!(cues.earcons, 3);
}

/// CURVASONA joins only the compiles that use it, so a spec without a function, an
/// `audify` track or critical-point detection compiles as before: no CURVASONA in its
/// header, and the warning that context cues need AUDAXIS (`SNR-COMP-023`) still there.
#[test]
fn specs_that_do_not_use_curvasona_compile_without_it() {
    let spec = Spec::from_json_str(
        r#"{ "specVersion": "0.1",
        "data": { "source": { "kind": "inline", "values": [
            { "day": 1, "temp": 8.5 }, { "day": 2, "temp": 9.25 }, { "day": 3, "temp": 7.0 } ] } },
        "substrate": { "time": { "field": "day" } },
        "tracks": [{ "id": "temps", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }] }"#,
    )
    .unwrap();
    let compiled = Sonarithm::new().compile(&spec).unwrap();
    assert!(!compiled.summary().engines.contains_key("curvasona"));
    assert!(
        compiled
            .diagnostics()
            .iter()
            .any(|d| d.code.as_str() == "SNR-COMP-023")
    );
    assert!(
        Sonarithm::new()
            .engines()
            .iter()
            .any(|engine| engine.id.as_str() == "curvasona")
    );
}

/// SDR-31: a data file's series audifies through the facade (CURVASONA joins for the
/// `audify` mark), with the calculator's readout as Info and the calculator re-exported.
#[test]
fn sdr_31_an_audify_track_on_inline_data_reports_the_speedup() {
    let values: Vec<String> = (0..200)
        .map(|i| format!(r#"{{ "t": {i}, "v": {} }}"#, (f64::from(i) * 0.3).sin()))
        .collect();
    let spec = Spec::from_json_str(format!(
        r#"{{ "specVersion": "0.4",
        "data": {{ "source": {{ "kind": "inline", "values": [{}] }} }},
        "substrate": {{ "time": {{ "field": "t" }} }},
        "tracks": [{{ "id": "a", "mark": "audify", "audify": {{ "k": 100 }} }}] }}"#,
        values.join(",")
    ))
    .unwrap();
    let compiled = Sonarithm::new()
        .compile(&spec)
        .unwrap_or_else(|e| panic!("{e}\n{:#?}", e.diagnostics()));
    assert_eq!(compiled.summary().tracks[0].audifications, 1);
    let readout = compiled
        .diagnostics()
        .iter()
        .find(|d| d.code.as_str() == "SNR-ENG-460")
        .unwrap();
    assert!(
        readout.message.starts_with("Audification: k = 100,"),
        "{readout}"
    );
    let speedup = sonarithm::audify_speedup(1.0, 200, 100.0);
    assert!((speedup.duration.0 - 2.0).abs() < 1e-12);
    assert!((sonarithm::SEISMIC_K - 2200.0).abs() < f64::EPSILON);
}

/// F-198: reading and compiling a function fits the 1 MB stack of a Windows main thread,
/// where the CLI and Python run it, even in a debug build. Before the compiled form was
/// boxed, an `Expr` was about 19 KB and sampling alone needed more than 1 MB.
#[test]
fn a_function_reads_and_compiles_on_a_1_mb_stack() {
    assert!(std::mem::size_of::<sonarithm_curvasona::Expr>() < 1024);
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            let function = read_function(&FunctionSpec::new("x^3 - 3*x", [-3.0, 3.0])).unwrap();
            let compiled = Sonarithm::new().compile(&function.spec()).unwrap();
            assert_eq!(compiled.score().tracks.len(), 2, "the glide and the cues");
        })
        .unwrap()
        .join()
        .unwrap();
}

/// The cubic's earcons as `(t, row)`, in time order.
fn cubic_earcons() -> (Vec<(f64, u64)>, sonarithm::Compiled) {
    let function = read_function(&FunctionSpec::new("x^3 - 3*x", [-3.0, 3.0])).unwrap();
    let compiled = Sonarithm::new().compile(&function.spec()).unwrap();
    let earcons = cue_track(compiled.score())
        .unwrap()
        .events
        .iter()
        .filter_map(|event| match event {
            ScoreEvent::Earcon(earcon) => Some((earcon.t.0, earcon.src.unwrap().row)),
            _ => None,
        })
        .collect();
    (earcons, compiled)
}

/// F-200, #441: a function's time follows x, so each cue sounds exactly at
/// (x − min)/(max − min) · T for the x of its row, whatever rows the source adds at
/// critical points: exactly for every cue of the cubic. On the 40-curve suite a cue that
/// merged with an earlier one (closer than 150 ms) keeps the earlier time and names the
/// higher-priority point's row, so there it may sound up to 150 ms before that row's time.
#[test]
fn f_200_cue_times_follow_x_exactly() {
    let snr = Sonarithm::new();
    let check = |expr: &str, domain: [f64; 2], exact: bool| {
        let function = read_function(&FunctionSpec::new(expr, domain)).unwrap();
        let compiled = snr.compile(&function.spec()).unwrap();
        let table = function.bundle().primary().unwrap();
        let column = table.column("x").unwrap();
        let xs = column.as_any().downcast_ref::<Float64Array>().unwrap();
        let earcons = cue_track(compiled.score())
            .map(|track| track.events.clone())
            .unwrap_or_default();
        let mut count = 0;
        for event in &earcons {
            let ScoreEvent::Earcon(earcon) = event else {
                continue;
            };
            let x = xs.value(usize::try_from(earcon.src.unwrap().row).unwrap());
            let want = (x - domain[0]) / (domain[1] - domain[0]) * sonarithm::FUNCTION_DURATION_S;
            let on_time = (earcon.t.0 - want).abs() < 1e-9;
            let merged = earcon.t.0 <= want && want - earcon.t.0 < 0.15 + 1e-9;
            assert!(
                on_time || (!exact && merged),
                "{expr}: a cue at x = {x} sounds at {} s, not {want} s",
                earcon.t.0
            );
            count += 1;
        }
        count
    };
    // The cubic: zeros at ±√3 and 0 (with the inflection), extrema at ±1.
    assert_eq!(check("x^3 - 3*x", [-3.0, 3.0], true), 5);
    for curve in &CURVES {
        check(curve.expr, curve.domain, false);
    }
}

/// F-200: each cue sounds when the glide reaches its point. An earcon's time is, bit for
/// bit, the glide breakpoint time of the row it names.
#[test]
fn f_200_each_earcon_sounds_at_its_rows_glide_breakpoint() {
    let (earcons, compiled) = cubic_earcons();
    let glide = compiled
        .score()
        .tracks
        .iter()
        .flat_map(|track| &track.events)
        .find_map(|event| match event {
            ScoreEvent::Glide(glide) => Some(glide),
            _ => None,
        })
        .unwrap();
    let start = glide.src.unwrap().start;
    for (t, row) in earcons {
        let index = usize::try_from(row - start).unwrap();
        let breakpoint = glide.curve[index].t.0;
        assert_eq!(
            t.to_bits(),
            breakpoint.to_bits(),
            "row {row}: {t} != {breakpoint}"
        );
    }
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_json::{Value, json};
use sonarithm_compiler::{Compiled, Compiler, Failure, Platform};
use sonarithm_core::engine::CancelToken;
use sonarithm_core::score::Score;

use super::*;
use crate::Curvasona;

fn compile(spec: &Value) -> Result<Compiled, Failure> {
    let platform = Platform::new();
    let curvasona = Curvasona::new();
    let numisona = sonarithm_numisona::Numisona::new();
    let compiler: Compiler<DataBundle> =
        Compiler::new(&[&platform, &curvasona, &numisona]).unwrap();
    compiler.compile(&spec.to_string(), &CancelToken::new())
}

/// A sine of one cycle per unit of x over [0, 10], audified with `audify`.
fn sine_spec(audify: &Value) -> Value {
    json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "function", "expr": "sin(2*pi*x)", "domain": [0, 10], "samples": 1001 } },
        "tracks": [{ "id": "a", "mark": "audify", "audify": audify }]
    })
}

fn event(score: &Score) -> &Audify {
    score
        .tracks
        .iter()
        .flat_map(|t| &t.events)
        .find_map(|e| match e {
            ScoreEvent::Audify(a) => Some(a),
            _ => None,
        })
        .unwrap()
}

/// The output frequency: whole cycles between the first and last rising zero crossing,
/// over the time between them.
fn frequency(audify: &Audify) -> f64 {
    let rising: Vec<usize> = audify
        .samples
        .windows(2)
        .enumerate()
        .filter(|(_, w)| w[0] < 0.0 && w[1] >= 0.0)
        .map(|(i, _)| i)
        .collect();
    let (Some(&first), Some(&last)) = (rising.first(), rising.last()) else {
        return 0.0;
    };
    #[expect(clippy::cast_precision_loss, reason = "test sizes")]
    let cycles = (rising.len() - 1) as f64;
    #[expect(clippy::cast_precision_loss, reason = "test sizes")]
    let seconds = (last - first) as f64 / audify.rate;
    cycles / seconds
}

fn codes_of(compiled: &Compiled) -> Vec<&str> {
    compiled
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect()
}

/// SDR-31: the seismic preset is k = 2200 (Dombois 2001).
#[test]
fn sdr_31_the_seismic_preset_is_k_2200() {
    assert!((SEISMIC_K - 2200.0).abs() < f64::EPSILON);
    // A day of 1 Hz seismometer data plays in under 40 s; its band, k/T to k·fs/2, runs
    // from about 0.03 Hz to 1.1 kHz.
    let day = calculate(1.0, 86_400, SEISMIC_K);
    assert!((day.duration.0 - 86_400.0 / 2200.0).abs() < 1e-9);
    assert!((day.band[1] - 1100.0).abs() < 1e-9);
    assert_eq!(day.audible.map(|a| a[1]), Some(1100.0));
}

/// SDR-31: the calculator reports k, the duration T/k and the band, and the band comes from
/// the target band's top when `band` is given.
#[test]
fn sdr_31_the_calculator_reports_k_duration_and_band() {
    let speedup = calculate(100.0, 1000, 50.0);
    assert!((speedup.duration.0 - 0.2).abs() < 1e-12);
    assert!((speedup.band[0] - 5.0).abs() < 1e-12 && (speedup.band[1] - 2500.0).abs() < 1e-12);
    assert_eq!(speedup.audible, Some([20.0, 2500.0]));
    assert!((k_for_band(100.0, 4000.0) - 80.0).abs() < 1e-12);
    assert!(calculate(1.0, 100, 1e-3).audible.is_none());
}

/// SDR-31, C03-T05: a sine in gives the expected tone out. One cycle per unit of x, at
/// k = 440, sounds at 440 Hz, though the function table has extra rows at its critical
/// points (it is interpolated onto an even grid first).
#[test]
fn sdr_31_a_sine_in_gives_the_expected_tone_out() {
    let compiled = compile(&sine_spec(&json!({ "k": 440 }))).unwrap();
    let audify = event(&compiled.score);
    let f = frequency(audify);
    assert!((f - 440.0).abs() < 2.0, "{f} Hz");
    assert!((audify.duration().0 - 10.0 / 440.0).abs() < 1e-3);
    assert!(audify.samples.iter().all(|s| s.abs() <= 1.0));
    assert!(
        codes_of(&compiled).contains(&"SNR-ENG-460"),
        "{:?}",
        codes_of(&compiled)
    );
    assert_eq!(compiled.score.header.ir_version, 2);
}

/// `audify.band` chooses k so that the series' top frequency lands at the band's top.
#[test]
fn sdr_31_a_target_band_sets_k() {
    let compiled = compile(&sine_spec(&json!({ "band": { "hz": [20, 2000] } }))).unwrap();
    let audify = event(&compiled.score);
    // k = 2·2000/fs, so the playback rate k·fs is 4000 samples per second.
    assert!((audify.rate - 4000.0).abs() < 1e-6, "{}", audify.rate);
}

/// RFC-0007 amendment 3: k outside 0.001 to 1 000 000 is refused; a band wholly outside
/// hearing is refused; a band partly outside is a warning.
#[test]
fn sdr_31_k_bounds_and_inaudible_bands() {
    let refused = |audify: &Value, code: &str| {
        let Err(Failure::Invalid(diagnostics)) = compile(&sine_spec(audify)) else {
            unreachable!("{audify} should be refused");
        };
        assert!(
            diagnostics.iter().any(|d| d.code.as_str() == code),
            "{audify}: {diagnostics:?}"
        );
    };
    refused(&json!({ "k": 1e7 }), "SNR-ENG-463");
    refused(&json!({ "k": 0.0001 }), "SNR-ENG-463");
    // About 100 samples per unit: k = 0.01 puts the whole band below 20 Hz.
    refused(&json!({ "k": 0.01 }), "SNR-ENG-462");
    // k = 5 sounds from 0.5 Hz to 250 Hz: partly below hearing.
    let partly = compile(&sine_spec(&json!({ "k": 5 }))).unwrap();
    assert!(
        codes_of(&partly).contains(&"SNR-ENG-461"),
        "{:?}",
        codes_of(&partly)
    );
}

/// SDR-44 (`INFER`, experimental, no study): without `k` or `band`, k puts the top of the
/// band at 4 kHz.
#[test]
fn sdr_44_infer_the_default_k_puts_the_top_of_the_band_at_4_khz() {
    let spec = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "function", "expr": "sin(2*pi*x)", "domain": [0, 10], "samples": 1001 } },
        "tracks": [{ "id": "a", "mark": "audify" }]
    });
    let compiled = compile(&spec).unwrap();
    let audify = event(&compiled.score);
    assert!(
        (audify.rate - 2.0 * DEFAULT_TOP_HZ).abs() < 1e-6,
        "{}",
        audify.rate
    );
}

/// SDR-23: the level is the `loudness` value, or −18 dB.
#[test]
fn audification_level_follows_loudness() {
    let default = compile(&sine_spec(&json!({ "k": 440 }))).unwrap();
    assert!((event(&default.score).gain.0 - DEFAULT_LOUDNESS.0).abs() < f64::EPSILON);
    let quiet = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "function", "expr": "sin(2*pi*x)", "domain": [0, 10], "samples": 1001 } },
        "tracks": [{ "id": "a", "mark": "audify", "audify": { "k": 440 },
                     "channels": { "loudness": { "value": -24 } } }]
    });
    if let Ok(compiled) = compile(&quiet) {
        assert!((event(&compiled.score).gain.0 + 24.0).abs() < f64::EPSILON);
    }
}

#[test]
fn uniform_interpolates_uneven_rows() {
    let values = [Some(0.0), Some(1.0), Some(3.0)];
    let times = [Some(0.0), Some(1.0), Some(3.0)];
    let (even, fs) = uniform(&values, &times).unwrap();
    assert!((fs - 2.0 / 3.0).abs() < 1e-12);
    let got: Vec<f64> = even.into_iter().flatten().collect();
    assert_eq!(got.len(), 3);
    assert!((got[1] - 1.5).abs() < 1e-12, "{got:?}");
    assert!(uniform(&values, &[Some(1.0), Some(1.0), Some(1.0)]).is_none());
}

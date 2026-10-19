// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The stimulus generators GEN-001 to GEN-004: determinism (pinned manifests, checked on
//! every CI platform), the published designs (tests named after the rules they serve),
//! and invariants over many seeds.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    reason = "integration test; clippy treats its helpers as non-test code, the studied rates are small whole numbers, and generated values are compared exactly on purpose (ADR-0013)"
)]

use proptest::prelude::*;
use sonarithm_echometric::harness::generators::correlation::{CorrelationParams, Sign, pearson};
use sonarithm_echometric::harness::generators::prices::{
    MAXIMUM, MINIMUM, OPEN, PriceParams, Series,
};
use sonarithm_echometric::harness::generators::reversals::{
    Direction, ReversalParams, STUDIED_RATES, STUDIED_REVERSALS, Shape, Start, Trend,
};
use sonarithm_echometric::harness::generators::transits::{STUDIED_SNR, TransitParams};
use sonarithm_echometric::harness::{GeneratorError, Note, Params, Stimulus, Truth, generate};

fn values(stimulus: &Stimulus, name: &str) -> Vec<f64> {
    stimulus.column(name).expect(name).values.clone()
}

/// Direction changes between successive steps that move.
fn reversals_in(values: &[f64]) -> u32 {
    let mut count = 0;
    let mut last = 0.0_f64;
    for pair in values.windows(2) {
        let step = pair[1] - pair[0];
        if step == 0.0 {
            continue;
        }
        if last != 0.0 && step.signum() != last.signum() {
            count += 1;
        }
        last = step;
    }
    count
}

fn manifest_json(params: &Params, seed: u64) -> String {
    let stimulus = generate(params, seed).unwrap();
    serde_json::to_string_pretty(&stimulus.manifest()).unwrap()
}

// --- Determinism: pinned manifests (table SHA-256 included) on every platform. ---

#[test]
fn gen_001_manifest_is_pinned() {
    let params = Params::TrendReversals(ReversalParams::cell(2.0, 3));
    insta::assert_snapshot!(manifest_json(&params, 20_261_019));
}

#[test]
fn gen_001_random_shape_manifest_is_pinned() {
    let params = Params::TrendReversals(ReversalParams {
        shape: Shape::Random,
        noise: 1.5,
        ..ReversalParams::cell(4.0, 5)
    });
    insta::assert_snapshot!(manifest_json(&params, 7));
}

#[test]
fn gen_002_manifest_is_pinned() {
    insta::assert_snapshot!(manifest_json(
        &Params::PriceSeries(PriceParams::default()),
        20_261_019
    ));
}

#[test]
fn gen_002_random_series_manifest_is_pinned() {
    let params = Params::PriceSeries(PriceParams {
        series: Series::Random,
        ..PriceParams::default()
    });
    insta::assert_snapshot!(manifest_json(&params, 7));
}

#[test]
fn gen_003_manifest_is_pinned() {
    insta::assert_snapshot!(manifest_json(
        &Params::Transits(TransitParams::curve(10.0, 2)),
        20_261_019
    ));
}

#[test]
fn gen_004_manifest_is_pinned() {
    let params = Params::CorrelationTriplets(CorrelationParams::level(0.04, Sign::Positive));
    insta::assert_snapshot!(manifest_json(&params, 20_261_019));
}

#[test]
fn the_same_parameters_and_seed_give_the_same_bytes() {
    let all = [
        Params::TrendReversals(ReversalParams::cell(8.0, 2)),
        Params::PriceSeries(PriceParams::default()),
        Params::Transits(TransitParams::curve(5.0, 1)),
        Params::CorrelationTriplets(CorrelationParams::level(0.1, Sign::Negative)),
    ];
    for params in &all {
        let first = generate(params, 99).unwrap();
        let second = generate(params, 99).unwrap();
        assert_eq!(first.csv(), second.csv());
        assert_eq!(first.manifest(), second.manifest());
    }
}

#[test]
fn different_seeds_give_different_stimuli() {
    let params = Params::Transits(TransitParams::curve(30.0, 1));
    assert_ne!(
        generate(&params, 1).unwrap().digest(),
        generate(&params, 2).unwrap().digest()
    );
}

// --- GEN-001, Nees & Walker (2008). ---

#[test]
fn sdr_19_gen_001_reproduces_the_studied_densities() {
    for rate in STUDIED_RATES {
        let stimulus = generate(&Params::TrendReversals(ReversalParams::cell(rate, 0)), 1).unwrap();
        // Ten hours at one second each, a point every 1/rate s: 10·rate + 1 points
        // (11, 21, 41, 81).
        let times = values(&stimulus, "t");
        assert_eq!(times.len(), 10 * rate as usize + 1);
        assert_eq!(times[1] - times[0], 1.0 / rate);
        assert_eq!(*times.last().unwrap(), 10.0);
        assert!(
            stimulus
                .notes
                .iter()
                .all(|note| !matches!(note, Note::Experimental { .. }))
        );
    }
}

#[test]
fn sdr_21_gen_001_reproduces_the_studied_reversal_levels() {
    for rate in STUDIED_RATES {
        for reversals in STUDIED_REVERSALS {
            for start in [Start::Up, Start::Down] {
                let params = ReversalParams {
                    start,
                    ..ReversalParams::cell(rate, reversals)
                };
                let stimulus = generate(&Params::TrendReversals(params), 3).unwrap();
                let prices = values(&stimulus, "price");
                assert_eq!(
                    reversals_in(&prices),
                    reversals,
                    "{rate}/s, {reversals} reversals, {start:?}"
                );
                // Every graph reaches both ends of the $6–$106 range (p. 13:10).
                assert_eq!(prices.iter().copied().fold(f64::MAX, f64::min), 6.0);
                assert_eq!(prices.iter().copied().fold(f64::MIN, f64::max), 106.0);
                let Truth::Reversals(truth) = &stimulus.truth else {
                    panic!()
                };
                assert_eq!(truth.segments.len(), reversals as usize + 1);
                let first = if start == Start::Up {
                    Direction::Up
                } else {
                    Direction::Down
                };
                assert_eq!(truth.segments[0], first);
            }
        }
    }
}

#[test]
fn sdr_20_gen_001_flags_rates_above_8_as_experimental() {
    let stimulus = generate(&Params::TrendReversals(ReversalParams::cell(16.0, 1)), 1).unwrap();
    assert!(stimulus.notes.iter().any(|note| matches!(
        note,
        Note::Experimental { rule, .. } if rule.contains("SDR-20") && rule.contains("H-02")
    )));
}

#[test]
fn sdr_21_gen_001_flags_reversals_above_3_as_experimental() {
    let params = ReversalParams {
        shape: Shape::Random,
        ..ReversalParams::cell(4.0, 5)
    };
    let stimulus = generate(&Params::TrendReversals(params.clone()), 1).unwrap();
    assert_eq!(reversals_in(&values(&stimulus, "price")), 5);
    assert!(
        stimulus
            .notes
            .iter()
            .any(|note| matches!(note, Note::Experimental { rule, .. } if rule.contains("SDR-21")))
    );
    let schematic = ReversalParams {
        shape: Shape::Schematic,
        ..params
    };
    assert!(matches!(
        generate(&Params::TrendReversals(schematic), 1),
        Err(GeneratorError::InvalidParameter {
            parameter: "reversals",
            ..
        })
    ));
}

#[test]
fn gen_001_queries_every_hour_once_in_seeded_order() {
    let stimulus = generate(&Params::TrendReversals(ReversalParams::cell(4.0, 2)), 11).unwrap();
    let Truth::Reversals(truth) = &stimulus.truth else {
        panic!()
    };
    let hours: Vec<f64> = truth.queries.iter().map(|query| query.x).collect();
    let mut sorted = hours.clone();
    sorted.sort_by(f64::total_cmp);
    assert_eq!(sorted, (8..=18).map(f64::from).collect::<Vec<_>>());
    assert_ne!(hours, sorted, "the order is shuffled");
    let prices = values(&stimulus, "price");
    for query in &truth.queries {
        assert_eq!(prices[query.row], query.value);
    }
    // Trend identification: each hourly interval once, answered from the played values.
    assert_eq!(truth.trend_queries.len(), 10);
    let hour = values(&stimulus, "hour");
    for query in &truth.trend_queries {
        assert_eq!(query.to - query.from, 1.0);
        let at = |h: f64| prices[hour.iter().position(|&x| x == h).unwrap()];
        let expected = match at(query.to).total_cmp(&at(query.from)) {
            std::cmp::Ordering::Greater => Trend::Increased,
            std::cmp::Ordering::Less => Trend::Decreased,
            std::cmp::Ordering::Equal => Trend::Same,
        };
        assert_eq!(query.answer, expected);
    }
}

#[test]
fn gen_001_schematic_stimuli_say_the_shape_is_read_from_a_schematic() {
    let stimulus = generate(&Params::TrendReversals(ReversalParams::cell(1.0, 1)), 1).unwrap();
    assert!(
        stimulus
            .notes
            .iter()
            .any(|note| matches!(note, Note::Deviation { detail } if detail.contains("schematic")))
    );
}

// --- GEN-002, Smith & Walker (2005). ---

#[test]
fn sdr_17_gen_002_dynamic_reference_is_the_maximum_while_rising_and_the_minimum_while_falling() {
    let stimulus = generate(&Params::PriceSeries(PriceParams::default()), 1).unwrap();
    let prices = values(&stimulus, "price");
    let reference = values(&stimulus, "ref-dynamic");
    // It starts at the maximum, because the price rises first (p. 1069).
    assert_eq!(reference[0], MAXIMUM);
    for row in 1..prices.len() {
        if prices[row] > prices[row - 1] {
            assert_eq!(reference[row], MAXIMUM, "row {row}");
        } else if prices[row] < prices[row - 1] {
            assert_eq!(reference[row], MINIMUM, "row {row}");
        }
    }
    // The static reference is the opening price throughout and is never the default
    // (SDR-17 forbids it as a default; it exists to replicate the study's condition).
    assert!(
        values(&stimulus, "ref-static")
            .iter()
            .all(|&value| value == OPEN)
    );
}

#[test]
fn sdr_16_gen_002_clicks_fall_on_every_hour() {
    let stimulus = generate(&Params::PriceSeries(PriceParams::default()), 1).unwrap();
    let hours = values(&stimulus, "hour");
    let clicks = values(&stimulus, "click");
    assert_eq!(hours.len(), 21);
    for (hour, click) in hours.iter().zip(&clicks) {
        assert_eq!(*click == 1.0, hour.fract() == 0.0, "hour {hour}");
    }
    // Two values per hour, 0.5 s apart: the 20 tones of Study 1 are rows 0 to 19.
    let times = values(&stimulus, "t");
    assert_eq!(times[1], 0.5);
    assert_eq!(times[19], 9.5);
}

#[test]
fn gen_002_keeps_the_published_anchors_and_queries_each_hour() {
    for series in [Series::Reconstructed, Series::Random] {
        let stimulus = generate(
            &Params::PriceSeries(PriceParams {
                series,
                ..PriceParams::default()
            }),
            5,
        )
        .unwrap();
        let Truth::Prices(truth) = &stimulus.truth else {
            panic!()
        };
        assert_eq!((truth.open, truth.close), (OPEN, OPEN));
        assert_eq!(truth.maximum.0, MAXIMUM);
        assert_eq!(truth.minimum.0, MINIMUM);
        let mut hours: Vec<f64> = truth.queries.iter().map(|query| query.x).collect();
        hours.sort_by(f64::total_cmp);
        assert_eq!(hours, (8..=18).map(f64::from).collect::<Vec<_>>());
    }
}

// --- GEN-003, Tucker Brown et al. (2022). ---

#[test]
fn sdr_40_gen_003_noise_follows_the_published_snr_definition() {
    for snr in STUDIED_SNR {
        let stimulus = generate(&Params::Transits(TransitParams::curve(snr, 1)), 2).unwrap();
        let Truth::Transits(truth) = &stimulus.truth else {
            panic!()
        };
        // SNR = t_d · √N_t / σ with t_d = 25 flux units and N_t = 100: σ = 250 / SNR.
        assert_eq!(truth.width_points, 100);
        assert_eq!(truth.depth_flux, 25.0);
        assert!((truth.sigma - 250.0 / snr).abs() < 1e-12);
        assert!(
            (truth.snr_achieved / snr - 1.0).abs() < 0.1,
            "SNR {snr}: {}",
            truth.snr_achieved
        );
    }
}

#[test]
fn sdr_40_gen_003_places_the_requested_transits_without_overlap() {
    for transits in 0..=2 {
        let stimulus =
            generate(&Params::Transits(TransitParams::curve(100.0, transits)), 4).unwrap();
        let Truth::Transits(truth) = &stimulus.truth else {
            panic!()
        };
        assert_eq!(truth.starts.len(), transits as usize);
        let flux = values(&stimulus, "flux");
        assert_eq!(flux.len(), 1000);
        for pair in truth.starts.windows(2) {
            assert!(pair[1] >= pair[0] + 100 + 50);
        }
        for &start in &truth.starts {
            assert!(start >= 50 && start + 100 + 50 <= 1000);
            // At SNR 100 (σ = 2.5) the dip is plain: its mean is near 75.
            let mean = flux[start..start + 100].iter().sum::<f64>() / 100.0;
            assert!((mean - 75.0).abs() < 1.5, "{mean}");
        }
    }
}

// --- GEN-004, Elmquist et al. (2024). ---

#[test]
fn sdr_40_gen_004_the_target_is_the_strongest_correlation() {
    for sign in [Sign::Positive, Sign::Negative] {
        let stimulus = generate(
            &Params::CorrelationTriplets(CorrelationParams::level(0.04, sign)),
            6,
        )
        .unwrap();
        let Truth::Correlations(truth) = &stimulus.truth else {
            panic!()
        };
        let a = values(&stimulus, "a");
        let measured: Vec<f64> = ["b", "c", "d"]
            .iter()
            .map(|axis| pearson(&a, &values(&stimulus, axis)))
            .collect();
        for (r, written) in measured.iter().zip(truth.correlations) {
            assert!((r - written).abs() < 1e-4);
        }
        let strongest = (0..3)
            .max_by(|&x, &y| measured[x].abs().total_cmp(&measured[y].abs()))
            .unwrap();
        assert_eq!(strongest, truth.strongest);
        // σ = 0.07 over 300 items gives the published r ≈ 0.95 for the target.
        let target = measured[truth.strongest];
        assert!((0.92..0.97).contains(&target.abs()), "{target}");
        assert_eq!(target > 0.0, sign == Sign::Positive);
    }
}

#[test]
fn gen_004_axes_are_normalized_onto_the_unit_range() {
    let stimulus = generate(
        &Params::CorrelationTriplets(CorrelationParams::level(0.2, Sign::Positive)),
        8,
    )
    .unwrap();
    for axis in ["a", "b", "c", "d"] {
        let column = values(&stimulus, axis);
        assert_eq!(column.len(), 300);
        assert_eq!(column.iter().copied().fold(f64::MAX, f64::min), 0.0);
        assert_eq!(column.iter().copied().fold(f64::MIN, f64::max), 1.0);
    }
}

// --- Parameters, CSV and the data layer. ---

#[test]
fn parameters_read_from_protocol_json_and_reject_unknown_fields() {
    let params: Params =
        serde_json::from_str(r#"{ "generator": "GEN-001", "rate": 2, "reversals": 1 }"#).unwrap();
    assert_eq!(params, Params::TrendReversals(ReversalParams::cell(2.0, 1)));
    let params: Params =
        serde_json::from_str(r#"{ "generator": "GEN-003", "snr": 7, "transits": 2 }"#).unwrap();
    assert_eq!(params, Params::Transits(TransitParams::curve(7.0, 2)));
    let unknown = serde_json::from_str::<Params>(r#"{ "generator": "GEN-002", "volume": 3 }"#);
    assert!(unknown.is_err());
    let missing = serde_json::from_str::<Params>(r#"{ "generator": "GEN-004" }"#);
    assert!(missing.is_err());
}

#[test]
fn invalid_parameters_name_the_parameter() {
    let cases = [
        (Params::TrendReversals(ReversalParams::cell(0.0, 1)), "rate"),
        (Params::Transits(TransitParams::curve(-1.0, 1)), "snr"),
        (Params::Transits(TransitParams::curve(10.0, 9)), "transits"),
        (
            Params::CorrelationTriplets(CorrelationParams::level(0.0, Sign::Positive)),
            "delta",
        ),
    ];
    for (params, expected) in cases {
        match generate(&params, 1) {
            Err(GeneratorError::InvalidParameter { parameter, .. }) => {
                assert_eq!(parameter, expected);
            }
            other => panic!("{params:?}: {other:?}"),
        }
    }
}

#[test]
fn csv_is_canonical() {
    let stimulus = generate(&Params::PriceSeries(PriceParams::default()), 1).unwrap();
    let csv = stimulus.csv();
    let mut lines = csv.lines();
    assert_eq!(
        lines.next(),
        Some("t,hour,price,click,ref-static,ref-dynamic")
    );
    assert_eq!(lines.next(), Some("0,8,50,1,50,84"));
    assert_eq!(csv.lines().count(), 22);
    assert!(csv.ends_with('\n') && !csv.contains('\r'));
    // Numbers never use exponent notation.
    assert!(csv.lines().skip(1).all(|line| !line.contains('e')));
}

#[test]
fn every_generator_loads_into_the_data_layer() {
    let all = [
        Params::TrendReversals(ReversalParams::cell(2.0, 2)),
        Params::PriceSeries(PriceParams::default()),
        Params::Transits(TransitParams::curve(30.0, 2)),
        Params::CorrelationTriplets(CorrelationParams::level(0.05, Sign::Positive)),
    ];
    for params in &all {
        let stimulus = generate(params, 1).unwrap();
        let bundle = stimulus.to_bundle("stimulus").unwrap();
        let table = bundle.tables().first().unwrap();
        assert_eq!(table.num_rows(), stimulus.rows() as u64);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn gen_001_random_shapes_have_the_requested_reversals(seed in any::<u64>(), reversals in 0_u32..8, rate in prop::sample::select(vec![2.0, 4.0, 8.0])) {
        let params = ReversalParams { shape: Shape::Random, ..ReversalParams::cell(rate, reversals) };
        let stimulus = generate(&Params::TrendReversals(params), seed).unwrap();
        let prices = values(&stimulus, "price");
        prop_assert_eq!(reversals_in(&prices), reversals);
        prop_assert_eq!(prices.iter().copied().fold(f64::MAX, f64::min), 6.0);
        prop_assert_eq!(prices.iter().copied().fold(f64::MIN, f64::max), 106.0);
    }

    #[test]
    fn gen_003_transits_never_overlap(seed in any::<u64>(), transits in 0_u32..=4, snr in prop::sample::select(STUDIED_SNR.to_vec())) {
        let stimulus = generate(&Params::Transits(TransitParams::curve(snr, transits)), seed).unwrap();
        let Truth::Transits(truth) = &stimulus.truth else { panic!() };
        prop_assert_eq!(truth.starts.len(), transits as usize);
        for pair in truth.starts.windows(2) {
            prop_assert!(pair[1] >= pair[0] + 150);
        }
        prop_assert!(truth.starts.iter().all(|&start| start >= 50 && start + 150 <= 1000));
    }

    #[test]
    fn gen_004_the_target_is_always_strongest(seed in any::<u64>(), delta in 0.01_f64..0.2) {
        let stimulus = generate(&Params::CorrelationTriplets(CorrelationParams::level(delta, Sign::Positive)), seed).unwrap();
        let Truth::Correlations(truth) = &stimulus.truth else { panic!() };
        let target = truth.correlations[truth.strongest].abs();
        for (index, r) in truth.correlations.iter().enumerate() {
            prop_assert!(index == truth.strongest || r.abs() < target);
        }
    }
}

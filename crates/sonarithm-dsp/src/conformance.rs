// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Design-rule conformance tests for the rules this crate owns (testing-strategy §6).

#![allow(
    clippy::cast_possible_truncation,
    reason = "test signals are computed in f64 and stored as f32 samples"
)]

use sonarithm_core::{SampleRate, Seconds};

use crate::env::{Adsr, AdsrParams, MIN_RAMP, REFERENCE_RAMP, Stage, note_ramp};
use crate::osc::sine_turns;
use crate::testing::{magnitude_db, windowed_spectrum};

const RATE: SampleRate = SampleRate::HZ_48000;

/// Renders a note envelope of `frames`, releasing early enough to end inside the note.
fn note_envelope(duration: Seconds) -> Vec<f32> {
    let mut env = Adsr::new(&AdsrParams::note(duration), RATE);
    let frames = duration.to_frames(RATE).unwrap().0 as usize;
    let release_at = frames.saturating_sub(env.release_frames() as usize);
    env.trigger();
    (0..frames.max(2 * env.release_frames() as usize))
        .map(|i| {
            if i == release_at {
                env.release();
            }
            env.next_level()
        })
        .collect()
}

/// SDR-22 (MUST): no note has a ramp shorter than 5 ms, whatever its duration.
#[test]
fn sdr_22_note_ramps_are_never_shorter_than_5_ms() {
    for ms in [
        0.0,
        0.1,
        1.0,
        5.0,
        19.9,
        20.0,
        100.0,
        1e6,
        f64::NAN,
        f64::INFINITY,
    ] {
        let ramp = note_ramp(Seconds::from_millis(ms));
        assert!(ramp.0 >= MIN_RAMP.0, "{ms} ms note: {ramp}");
        assert!(ramp.0 <= REFERENCE_RAMP.0, "{ms} ms note: {ramp}");
    }
}

/// SDR-22: the ramp is the 40 ms reference for long notes and 25 % of short notes.
#[test]
fn sdr_22_note_ramps_follow_the_40_ms_reference() {
    assert_eq!(note_ramp(Seconds(1.0)), REFERENCE_RAMP);
    assert_eq!(note_ramp(Seconds(0.160)), REFERENCE_RAMP);
    assert_eq!(note_ramp(Seconds(0.100)), Seconds(0.025));
    assert_eq!(note_ramp(Seconds(0.010)), MIN_RAMP);
}

/// SDR-22: every note envelope starts near zero, ends exactly at zero and never steps.
#[test]
fn sdr_22_every_note_envelope_starts_and_ends_at_zero_without_steps() {
    for ms in [1.0, 10.0, 25.0, 120.0, 500.0, 2_000.0] {
        let levels = note_envelope(Seconds::from_millis(ms));
        let ramp = note_ramp(Seconds::from_millis(ms));
        let ramp_frames = ramp.0 * f64::from(RATE.get());
        // A raised cosine over n frames changes by at most π/(2n) per frame.
        let max_step = core::f64::consts::FRAC_PI_2 / ramp_frames + 1e-6;
        assert!(
            f64::from(levels[0]) <= max_step,
            "{ms} ms: first level {}",
            levels[0]
        );
        assert_eq!(*levels.last().unwrap(), 0.0, "{ms} ms");
        let worst = levels
            .windows(2)
            .map(|w| f64::from((w[1] - w[0]).abs()))
            .fold(0.0, f64::max);
        assert!(worst <= max_step, "{ms} ms: step {worst} > {max_step}");
    }
}

/// SDR-22: retriggering or releasing at any point of any segment never steps.
#[test]
fn sdr_22_retrigger_and_release_never_step() {
    let params = AdsrParams {
        attack: MIN_RAMP,
        decay: Seconds(0.02),
        sustain: 0.6,
        release: MIN_RAMP,
    };
    let max_step = core::f64::consts::FRAC_PI_2 / 240.0 + 1e-6;
    for switch_at in [1, 7, 120, 239, 240, 241, 600, 5_000] {
        let mut env = Adsr::new(&params, RATE);
        env.trigger();
        let mut previous = 0.0_f32;
        for i in 0..6_000 {
            if i == switch_at {
                env.release();
            }
            if i == switch_at + 100 {
                env.trigger();
            }
            let level = env.next_level();
            assert!(
                f64::from((level - previous).abs()) <= max_step,
                "switch at {switch_at}, frame {i}"
            );
            previous = level;
        }
        assert_ne!(env.stage(), Stage::Idle);
    }
}

/// SDR-22: a ramped onset does not click. A tone that starts at its peak (a cosine) has a
/// step at the onset, which spreads energy across the spectrum; with the minimum 5 ms ramp,
/// the energy above 6 kHz is at least 40 dB lower.
#[test]
fn sdr_22_ramped_onsets_do_not_click() {
    let frames = 8_192;
    let onset = 2_048;
    let hz = 1_000.0;
    let tone = |i: usize| sine_turns(0.25 + hz * (i - onset) as f64 / 48_000.0) as f32;
    let gated: Vec<f32> = (0..frames)
        .map(|i| if i < onset { 0.0 } else { tone(i) })
        .collect();
    let mut env = Adsr::new(
        &AdsrParams {
            attack: MIN_RAMP,
            ..AdsrParams::note(Seconds(1.0))
        },
        RATE,
    );
    let ramped: Vec<f32> = (0..frames)
        .map(|i| {
            if i == onset {
                env.trigger();
            }
            if i < onset {
                0.0
            } else {
                tone(i) * env.next_level()
            }
        })
        .collect();
    let high_band = |signal: &[f32]| {
        let mags = windowed_spectrum(signal);
        let first = 6_000 * frames / 48_000;
        magnitude_db(mags[first..].iter().copied().fold(0.0, f64::max))
    };
    let (click, smooth) = (high_band(&gated), high_band(&ramped));
    assert!(
        click - smooth > 40.0,
        "gated {click:.1} dB, ramped {smooth:.1} dB"
    );
}

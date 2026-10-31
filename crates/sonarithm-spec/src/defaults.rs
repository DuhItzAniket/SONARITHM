// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Data-independent defaults (RFC-0001 §2–§11), applied before canonicalization and
//! reported as `Info` diagnostics with the rule each implements (05 §7: "explain what you
//! assumed").
//!
//! Choices that need the data stay symbolic: the time field, `domain: "auto"`,
//! `polarity: "auto"`, `ticks.enabled: "auto"`, the tick interval and the audification
//! factor. Channels a track does not write are not added; the encoder's channel defaults
//! are pinned by its engine version in the Score header.

use std::collections::BTreeMap;

use sonarithm_core::diagnostic::{Diagnostic, Diagnostics, SpecPointer};
use sonarithm_core::ids::RuleId;

use crate::codes;
use crate::data::Missing;
use crate::sections::{
    Analysis, Axis, Context, Cue, Describe, DescribeLevel, Earcons, Legend, LegendPart,
    LegendSpeak, Output, OutputLoudness, Pace, Playback, Reference, ReferenceKind, Spatial,
    SpatialMode, Substrate, Ticks, TimeSubstrate,
};
use crate::spec::{Mode, Spec};
use crate::track::{ChannelName, Domain, DomainKeyword, PitchSpace, Polarity, Scale, ScaleType};
use crate::values::{AutoKeyword, ChannelRange, Toggle, UnitRange};

/// Default pace: 3 points per second (SDR-19: 2–4 for overviews).
pub const DEFAULT_POINTS_PER_SECOND: f64 = 3.0;
/// Default pitch range: 400–1400 Hz (SDR-08).
pub const DEFAULT_PITCH_RANGE_HZ: [f64; 2] = [400.0, 1400.0];
/// Default output sample rate in Hz.
pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;
/// Default output block size in frames.
pub const DEFAULT_BLOCK_SIZE: u32 = 256;
/// Proposed export loudness target in LUFS (sound-design §6.3, not yet evidence-backed).
pub const DEFAULT_TARGET_LUFS: f64 = -16.0;
/// Proposed true-peak limit in dBTP (sound-design §6.3).
pub const DEFAULT_TRUE_PEAK_DBTP: f64 = -1.0;

/// Records applied defaults as `Info` diagnostics.
struct Applied {
    out: Diagnostics,
}

impl Applied {
    /// Fills `slot` with `value` when unset and reports "Assumed `what`." at `at`.
    fn fill<T>(
        &mut self,
        slot: &mut Option<T>,
        value: T,
        at: SpecPointer,
        what: &str,
        rules: &[&'static str],
    ) {
        if slot.is_none() {
            *slot = Some(value);
            let mut diagnostic =
                Diagnostic::new(&codes::DEFAULT_APPLIED, format!("Assumed {what}."));
            diagnostic.rules = rules.iter().map(|rule| RuleId::from_static(rule)).collect();
            self.out.push(diagnostic.at(at));
        }
    }
}

/// The spec with every data-independent default written in, and an `Info` diagnostic per
/// default.
#[must_use]
pub fn apply_defaults(spec: &Spec) -> (Spec, Diagnostics) {
    let mut spec = spec.clone();
    let mut a = Applied {
        out: Diagnostics::new(),
    };
    let root = SpecPointer::root();
    a.fill(
        &mut spec.mode,
        Mode::Analysis,
        root.key("mode"),
        "Analysis mode",
        &["SDR-36", "SDR-10"],
    );
    a.fill(&mut spec.seed, 0, root.key("seed"), "seed 0", &["SDR-02"]);
    let missing = root.key("data").key("missing");
    a.fill(
        &mut spec.data.missing,
        Missing::Silence,
        missing,
        "silence for missing values",
        &[],
    );
    let time = spec
        .substrate
        .get_or_insert_with(Substrate::default)
        .time
        .get_or_insert_with(TimeSubstrate::default);
    let pace = Pace::PointsPerSecond(DEFAULT_POINTS_PER_SECOND);
    let at = root.key("substrate").key("time").key("pace");
    a.fill(
        &mut time.pace,
        pace,
        at,
        "a pace of 3 points per second",
        &["SDR-19"],
    );
    tracks(&mut spec, &mut a);
    context(&mut spec, &mut a);
    rest(&mut spec, &mut a);
    if spec.spec_version >= sonarithm_core::score::SpecVersion::V0_2 {
        v0_2(&mut spec, &mut a);
    }
    // Not a default: switches that are off leave the canonical form, so `{}`, all-`false`
    // and no block share one spec hash (RFC-0006 §2).
    spec.experimental = spec
        .experimental
        .as_ref()
        .and_then(crate::experimental::Experimental::canonical);
    (spec, a.out)
}

/// The data-independent defaults of the sections v0.2 adds, filled only where a section is
/// present (RFC-0004 §9). A v0.1 document gets none, so its canonical form and spec hash
/// stay as they were.
fn v0_2(spec: &mut Spec, a: &mut Applied) {
    use crate::json::JsonValue;
    use crate::v02::Instrument;

    let root = SpecPointer::root();
    for (name, instrument) in &mut spec.instruments {
        let at = root.key("instruments").key(name);
        match instrument {
            Instrument::Soundfont { bank, program, .. } => {
                a.fill(bank, 0, at.key("bank"), "bank 0", &[]);
                a.fill(program, 0, at.key("program"), "program 0", &[]);
            }
        }
    }
    if let Some(lint) = &mut spec.lint {
        let at = root.key("lint");
        a.fill(
            &mut lint.rules,
            BTreeMap::new(),
            at.key("rules"),
            "every lint rule at its own level",
            &[],
        );
        a.fill(
            &mut lint.suppress,
            Vec::new(),
            at.key("suppress"),
            "no lint suppressions",
            &[],
        );
        a.fill(
            &mut lint.experimental,
            false,
            at.key("experimental"),
            "experimental lint rules off",
            &[],
        );
    }
    if let Some(quantize) = spec.music.as_mut().and_then(|m| m.quantize.as_mut())
        && matches!(quantize.scale, JsonValue::String(_))
    {
        let at = root.key("music").key("quantize").key("tuning");
        a.fill(
            &mut quantize.tuning,
            JsonValue::String("12-TET".to_owned()),
            at,
            "12-TET tuning",
            &["SDR-10"],
        );
    }
    if let crate::data::Source::Chart {
        calibration: Some(JsonValue::Object(calibration)),
        ..
    } = &mut spec.data.source
    {
        let at = root.key("data").key("source").key("calibration");
        for (key, value, what) in [
            (
                "system",
                JsonValue::String("cartesian".to_owned()),
                "a cartesian calibration",
            ),
            ("page", JsonValue::Number(1.0), "page 1"),
        ] {
            let mut slot = calibration.get(key).cloned();
            a.fill(&mut slot, value, at.key(key), what, &["SDR-01"]);
            if let Some(value) = slot {
                calibration.insert(key.to_owned(), value);
            }
        }
    }
}

/// Scale defaults for every mapped channel; range, space and polarity for pitch.
fn tracks(spec: &mut Spec, a: &mut Applied) {
    for (index, track) in spec.tracks.iter_mut().enumerate() {
        let at = SpecPointer::root()
            .key("tracks")
            .index(index)
            .key("channels");
        let channels = &mut track.channels;
        let slots = [
            (ChannelName::Pitch, &mut channels.pitch),
            (ChannelName::Duration, &mut channels.duration),
            (ChannelName::Taps, &mut channels.taps),
            (ChannelName::Loudness, &mut channels.loudness),
            (ChannelName::Pan, &mut channels.pan),
            (ChannelName::Timbre, &mut channels.timbre),
            (ChannelName::Brightness, &mut channels.brightness),
            (ChannelName::Rate, &mut channels.rate),
            (ChannelName::Density, &mut channels.density),
        ];
        for (name, slot) in slots {
            let Some(channel) = slot.as_mut().filter(|channel| channel.field.is_some()) else {
                continue;
            };
            let at = at.key(name.key());
            let scale = channel.scale.get_or_insert(Scale {
                scale_type: None,
                domain: None,
                nice: None,
                exponent: None,
                reference: None,
            });
            let s = at.key("scale");
            a.fill(
                &mut scale.scale_type,
                ScaleType::Linear,
                s.key("type"),
                "a linear scale",
                &[],
            );
            let auto = Domain::Keyword(DomainKeyword::Auto);
            a.fill(
                &mut scale.domain,
                auto,
                s.key("domain"),
                "the data extent as the domain",
                &[],
            );
            a.fill(
                &mut scale.nice,
                false,
                s.key("nice"),
                "no domain rounding",
                &[],
            );
            if name == ChannelName::Pitch {
                let range = ChannelRange::Unit(UnitRange::Hz(DEFAULT_PITCH_RANGE_HZ));
                let what = "a pitch range of 400 to 1400 Hz";
                a.fill(
                    &mut channel.range,
                    range,
                    at.key("range"),
                    what,
                    &["SDR-08"],
                );
                let log = PitchSpace::LogFrequency;
                a.fill(
                    &mut channel.space,
                    log,
                    at.key("space"),
                    "log-frequency pitch",
                    &["SDR-09"],
                );
                let rules = &["SDR-12", "SDR-13", "SDR-14"];
                let what = "polarity from the field concept";
                a.fill(
                    &mut channel.polarity,
                    Polarity::Auto,
                    at.key("polarity"),
                    what,
                    rules,
                );
            }
        }
    }
}

/// AUDAXIS context defaults (SDR-16..18, A11Y-08).
fn context(spec: &mut Spec, a: &mut Applied) {
    let first_track = spec.tracks.first().map(|track| track.id.clone());
    let context = spec.context.get_or_insert_with(Context::default);
    let at = SpecPointer::root().key("context");
    let ticks = context.ticks.get_or_insert_with(Ticks::default);
    let t = at.key("ticks");
    a.fill(
        &mut ticks.axis,
        Axis::X,
        t.key("axis"),
        "ticks on the x axis",
        &["SDR-16"],
    );
    a.fill(
        &mut ticks.minor,
        false,
        t.key("minor"),
        "no minor ticks",
        &["SDR-16"],
    );
    let auto = Toggle::Auto(AutoKeyword::Auto);
    let what = "ticks for discrete marks only";
    a.fill(
        &mut ticks.enabled,
        auto,
        t.key("enabled"),
        what,
        &["SDR-16"],
    );
    let reference = Reference {
        kind: ReferenceKind::DynamicExtreme,
        track: first_track,
        timbre: None,
        value: None,
    };
    let what = "a dynamic-extreme reference tone on the first track";
    a.fill(
        &mut context.reference,
        reference,
        at.key("reference"),
        what,
        &["SDR-17"],
    );
    let legend = context.legend.get_or_insert_with(Legend::default);
    let l = at.key("legend");
    let what = "a spoken legend before playback";
    a.fill(
        &mut legend.speak,
        LegendSpeak::BeforePlay,
        l.key("speak"),
        what,
        &["SDR-18"],
    );
    let parts = vec![
        LegendPart::Title,
        LegendPart::XRange,
        LegendPart::YRange,
        LegendPart::Units,
        LegendPart::Polarity,
    ];
    let what = "a legend with title, ranges, units and polarity";
    a.fill(&mut legend.parts, parts, l.key("parts"), what, &["SDR-18"]);
    let earcons = context.earcons.get_or_insert_with(Earcons::default);
    let e = at.key("earcons");
    a.fill(
        &mut earcons.boundary,
        true,
        e.key("boundary"),
        "boundary earcons",
        &["A11Y-08"],
    );
    a.fill(
        &mut earcons.landmarks,
        true,
        e.key("landmarks"),
        "landmark earcons",
        &["A11Y-08"],
    );
    let stereo = Spatial {
        mode: SpatialMode::Stereo,
    };
    a.fill(
        &mut context.spatial,
        stereo,
        at.key("spatial"),
        "stereo output",
        &[],
    );
}

/// Analysis, playback and output defaults.
fn rest(spec: &mut Spec, a: &mut Applied) {
    let root = SpecPointer::root();
    let analysis = spec.analysis.get_or_insert_with(Analysis::default);
    let at = root.key("analysis");
    a.fill(
        &mut analysis.cue,
        Cue::Earcon,
        at.key("cue"),
        "earcon cues",
        &["A11Y-08"],
    );
    let describe = analysis.describe.get_or_insert_with(Describe::default);
    let levels = vec![DescribeLevel::L1, DescribeLevel::L2];
    let at = at.key("describe").key("levels");
    a.fill(
        &mut describe.levels,
        levels,
        at,
        "L1 and L2 descriptions",
        &["A11Y-02"],
    );

    let playback = spec.playback.get_or_insert_with(Playback::default);
    let at = root.key("playback");
    a.fill(
        &mut playback.speed,
        1.0,
        at.key("speed"),
        "playback speed 1",
        &[],
    );
    a.fill(
        &mut playback.looping,
        false,
        at.key("loop"),
        "no looping",
        &[],
    );

    let output = spec.output.get_or_insert_with(Output::default);
    let at = root.key("output");
    let rate = DEFAULT_SAMPLE_RATE;
    a.fill(
        &mut output.sample_rate,
        rate,
        at.key("sampleRate"),
        "48000 Hz output",
        &[],
    );
    a.fill(
        &mut output.channels,
        2,
        at.key("channels"),
        "2 output channels",
        &[],
    );
    let block = DEFAULT_BLOCK_SIZE;
    a.fill(
        &mut output.block_size,
        block,
        at.key("blockSize"),
        "256-frame blocks",
        &[],
    );
    let loudness = output.loudness.get_or_insert_with(OutputLoudness::default);
    let l = at.key("loudness");
    let what = "an export target of -16 LUFS (proposed)";
    a.fill(
        &mut loudness.target_lufs,
        DEFAULT_TARGET_LUFS,
        l.key("targetLufs"),
        what,
        &[],
    );
    let what = "a true-peak limit of -1 dBTP (proposed)";
    let peak = DEFAULT_TRUE_PEAK_DBTP;
    a.fill(
        &mut loudness.true_peak_dbtp,
        peak,
        l.key("truePeakDbtp"),
        what,
        &[],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    const MINIMAL: &str = r#"{ "specVersion": "0.1", "data": { "source": { "kind": "file", "path": "temps.csv" } },
      "tracks": [{ "id": "y", "mark": "tone", "channels": { "pitch": { "field": "anomaly" } } }] }"#;

    /// SDR-08 and SDR-09: the default pitch range is 400–1400 Hz in log-frequency.
    #[test]
    fn sdr_08_sdr_09_default_pitch_range_is_400_to_1400_hz_log_frequency() {
        let (spec, _) = apply_defaults(&parse(MINIMAL).unwrap());
        let pitch = spec.tracks[0].channels.pitch.clone().unwrap();
        assert_eq!(
            pitch.range,
            Some(ChannelRange::Unit(UnitRange::Hz([400.0, 1400.0])))
        );
        assert_eq!(pitch.space, Some(PitchSpace::LogFrequency));
        assert_eq!(pitch.polarity, Some(Polarity::Auto));
    }

    /// SDR-19: the default pace is 3 points per second.
    #[test]
    fn sdr_19_default_pace_is_3_points_per_second() {
        let (spec, diagnostics) = apply_defaults(&parse(MINIMAL).unwrap());
        let pace = spec.substrate.and_then(|s| s.time).and_then(|t| t.pace);
        assert_eq!(pace, Some(Pace::PointsPerSecond(3.0)));
        let info = diagnostics
            .iter()
            .find(|d| d.location.as_ref().map(SpecPointer::as_str) == Some("/substrate/time/pace"))
            .unwrap();
        assert_eq!(info.code.as_str(), "SNR-SPEC-080");
        assert_eq!(info.rules, [RuleId::from_static("SDR-19")]);
    }

    /// SDR-36 and SDR-17: Analysis mode and a dynamic-extreme reference by default.
    #[test]
    fn sdr_36_sdr_17_mode_and_reference_defaults() {
        let (spec, _) = apply_defaults(&parse(MINIMAL).unwrap());
        assert_eq!(spec.mode, Some(Mode::Analysis));
        let reference = spec.context.and_then(|c| c.reference).unwrap();
        assert_eq!(reference.kind, ReferenceKind::DynamicExtreme);
        assert_eq!(reference.track.as_deref(), Some("y"));
    }

    #[test]
    fn written_values_are_kept_and_defaults_are_idempotent() {
        let written = MINIMAL.replace(
            r#""field": "anomaly""#,
            r#""field": "anomaly", "range": { "hz": [65, 1480] }"#,
        );
        let (spec, _) = apply_defaults(&parse(&written).unwrap());
        let range = spec.tracks[0].channels.pitch.as_ref().and_then(|p| p.range);
        assert_eq!(
            range,
            Some(ChannelRange::Unit(UnitRange::Hz([65.0, 1480.0])))
        );
        let (again, more) = apply_defaults(&spec);
        assert_eq!(again, spec);
        assert!(more.is_empty());
    }
}

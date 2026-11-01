// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The Score IR: SONARITHM's canonical artifact (05 §4, ADR-0005).
//!
//! The compiler produces a [`Score`]; the renderer, sinks and navigation consume it, and
//! tests snapshot it. Time is in [`Seconds`] and pitch in [`Cents`] relative to A4 (05 §4.1).
//! Every collection is a sorted `Vec` or a `BTreeMap`, so serialization order is stable.
//!
//! Event kinds for audification, grains and model excitation are added when their engines
//! land (C03, D01, D02); new kinds and fields are additive and raise [`IR_VERSION`]. A Score
//! records the lowest version that can read it ([`Score::required_ir_version`], RFC-0007
//! §4), so a Score that uses none of the additions keeps version 1 and its hash.
//! [`Score::check`] verifies the invariants of 05 §4.2, and the [`canonical`] and
//! [`review`] modules write its two serialized forms (05 §4.3).

pub mod canonical;
pub mod invariants;
pub mod review;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub use crate::digest::Sha256Digest;
use crate::ids::{EngineId, RowRange, RowRef, TrackId};
use crate::provenance::Confidence;
use crate::units::{Cents, Db, Pan, Seconds};

pub use invariants::InvariantViolation;

/// The newest Score IR version this crate reads and writes. A change to the serialized
/// form (02 §6) raises it. Since RFC-0007 a Score's header records the lowest version that
/// can read it ([`Score::required_ir_version`]), so additive changes leave existing Scores,
/// snapshots and golden hashes as they were; a change to existing fields still needs a PR
/// that regenerates every snapshot and golden hash and contains nothing else
/// (testing-strategy §4).
///
/// Version 2 (RFC-0007): the earcon kinds `zero`, `asymptote`, `undefined` and `offscreen`,
/// and the [`Audify`] event.
pub const IR_VERSION: u32 = 2;

/// The first Score IR version (A03), which every Score without later additions records.
pub const IR_VERSION_BASE: u32 = 1;

/// A spec language version, `MAJOR.MINOR` (04 §7), serialized as a string such as `"0.1"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpecVersion {
    /// Major version: renames, removals and semantic changes.
    pub major: u16,
    /// Minor version: additions.
    pub minor: u16,
}

impl SpecVersion {
    /// `specVersion: "0.1"`, frozen by RFC-0001.
    pub const V0_1: Self = Self { major: 0, minor: 1 };
    /// `specVersion: "0.2"`: `template`, `instruments`, `lint`, chart calibration and
    /// corrections, and the quantize vocabularies (RFC-0004).
    pub const V0_2: Self = Self { major: 0, minor: 2 };
    /// `specVersion: "0.3"`: `data.source.corrections.path`, a chart correction file
    /// (RFC-0005, ADR-0026).
    pub const V0_3: Self = Self { major: 0, minor: 3 };
    /// `specVersion: "0.4"`: the `experimental` block of opt-in sound experiments
    /// (RFC-0006, ADR-0027).
    pub const V0_4: Self = Self { major: 0, minor: 4 };
}

impl fmt::Display for SpecVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// The error for text that is not `MAJOR.MINOR` with decimal numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("a spec version is written MAJOR.MINOR, for example \"0.1\"")]
pub struct InvalidSpecVersion;

impl FromStr for SpecVersion {
    type Err = InvalidSpecVersion;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (major, minor) = text.split_once('.').ok_or(InvalidSpecVersion)?;
        let number = |part: &str| {
            let canonical = part == "0" || (!part.starts_with('0') && !part.is_empty());
            if canonical && part.bytes().all(|b| b.is_ascii_digit()) {
                part.parse::<u16>().map_err(|_| InvalidSpecVersion)
            } else {
                Err(InvalidSpecVersion)
            }
        };
        Ok(Self {
            major: number(major)?,
            minor: number(minor)?,
        })
    }
}

impl Serialize for SpecVersion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for SpecVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

/// The sonification mode, explicit and recorded (SDR-36).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Analytic listening: continuous pitch, no quantization (SDR-10).
    #[default]
    Analysis,
    /// Presentation: musical quantization allowed (AXONOTE).
    Presentation,
    /// Peripheral monitoring of live data (SONOTRACE, SDR-35).
    Monitoring,
}

/// The fixed shaper slots, in the order they run (05 §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShaperStage {
    /// Musical quantization (AXONOTE); Presentation mode only (SDR-10).
    Quantize,
    /// Equal-loudness compensation (SDR-24).
    Loudness,
    /// Spatial placement (AUDAXIS).
    Spatial,
    /// Perceptual lint (ECHOMETRIC); never mutates the Score.
    Lint,
}

/// An engine crate version, such as `0.1.0`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EngineVersion(pub String);

/// The compiled Score.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Score {
    /// Provenance and global facts.
    pub header: ScoreHeader,
    /// Tracks in the order the compiler emitted them.
    pub tracks: Vec<ScoreTrack>,
    /// Insights, landmarks, sections and legend boundaries, sorted by time.
    pub markers: Vec<Marker>,
}

impl Score {
    /// The lowest Score IR version that can read this Score (RFC-0007 §4): the highest
    /// version any of its events needs, at least [`IR_VERSION_BASE`]. The compiler writes
    /// it into the header.
    #[must_use]
    pub fn required_ir_version(&self) -> u32 {
        self.tracks
            .iter()
            .flat_map(|track| &track.events)
            .map(|event| match event {
                ScoreEvent::Earcon(earcon) => earcon.kind.ir_version(),
                ScoreEvent::Audify(_) => 2,
                _ => IR_VERSION_BASE,
            })
            .max()
            .unwrap_or(IR_VERSION_BASE)
    }

    /// An empty Score with `header`.
    #[must_use]
    pub fn new(header: ScoreHeader) -> Self {
        Self {
            header,
            tracks: Vec::new(),
            markers: Vec::new(),
        }
    }

    /// Moves every event and marker `by` later and lengthens the Score by as much: the
    /// Score of a live window placed on its stream's timeline, so each new window plays on
    /// from the same position (B06-T07). The result is for playback; its header still
    /// names the window's compile. A `by` that is not a positive number changes nothing.
    pub fn offset(&mut self, by: Seconds) {
        let by = by.0;
        if !(by > 0.0 && by.is_finite()) {
            return;
        }
        let shift = |t: &mut Seconds| t.0 += by;
        for track in &mut self.tracks {
            for event in &mut track.events {
                match event {
                    ScoreEvent::Note(note) => shift(&mut note.t),
                    ScoreEvent::Glide(glide) => {
                        shift(&mut glide.t0);
                        shift(&mut glide.t1);
                        for point in &mut glide.curve {
                            shift(&mut point.t);
                        }
                    }
                    ScoreEvent::Earcon(earcon) => shift(&mut earcon.t),
                    ScoreEvent::Speech(speech) => shift(&mut speech.t),
                    ScoreEvent::Audify(audify) => shift(&mut audify.t),
                }
            }
        }
        for marker in &mut self.markers {
            shift(&mut marker.t);
        }
        shift(&mut self.header.duration);
    }
}

/// The Score header: everything needed to reproduce and attribute the Score (ADR-0013).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ScoreHeader {
    /// The lowest Score IR version that can read the Score (RFC-0007 §4).
    pub ir_version: u32,
    /// The spec language version.
    pub spec_version: SpecVersion,
    /// SHA-256 of the RFC 8785 canonical spec with defaults applied.
    pub spec_hash: Sha256Digest,
    /// SHA-256 fingerprint of the input data (03 §2).
    pub data_fingerprint: Sha256Digest,
    /// The seed every random stream derives from (SDR-02).
    pub seed: u64,
    /// The mode (SDR-36).
    pub mode: Mode,
    /// Total length; every note ends by then.
    pub duration: Seconds,
    /// The engines that contributed, with their versions.
    pub engines: BTreeMap<EngineId, EngineVersion>,
    /// The shapers that ran, in order (05 §2).
    pub shapers: Vec<ShaperStage>,
}

impl ScoreHeader {
    /// A header at [`IR_VERSION_BASE`] (the compiler raises it to
    /// [`Score::required_ir_version`]), with zero hashes, zero duration and no engines or
    /// shapers yet.
    #[must_use]
    pub fn new(spec_version: SpecVersion, mode: Mode, seed: u64) -> Self {
        Self {
            ir_version: IR_VERSION_BASE,
            spec_version,
            spec_hash: Sha256Digest::ZERO,
            data_fingerprint: Sha256Digest::ZERO,
            seed,
            mode,
            duration: Seconds::ZERO,
            engines: BTreeMap::new(),
            shapers: Vec::new(),
        }
    }
}

/// What a track carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrackRole {
    /// Data encoded by an encoder.
    Data,
    /// Context cues: ticks, reference tone (AUDAXIS).
    Context,
    /// Earcons (A11Y-08).
    Earcon,
    /// Speech, routed to the announcer or a sidecar, never rendered by default (05 §4.1).
    Speech,
    /// Monitoring soundscape (SONOTRACE).
    Monitor,
}

/// A reference to an instrument preset, such as `mallet`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstrumentRef(pub String);

/// The resolved pitch range of a track, low to high, after polarity is applied.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PitchRange {
    /// The lowest pitch.
    pub low: Cents,
    /// The highest pitch.
    pub high: Cents,
}

impl PitchRange {
    /// Whether `pitch` lies within the range, bounds included.
    #[must_use]
    pub fn contains(&self, pitch: Cents) -> bool {
        self.low.0 <= pitch.0 && pitch.0 <= self.high.0
    }

    /// The pitch mirrored within the range, `low + high − pitch`, clamped to the range so
    /// rounding cannot leave it (05 §4.2 invariant 4, SDR-15).
    #[must_use]
    pub fn mirror(&self, pitch: Cents) -> Cents {
        Cents((self.low.0 + self.high.0 - pitch.0).clamp(self.low.0, self.high.0))
    }
}

/// One stream of events.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ScoreTrack {
    /// The track ID from the spec.
    pub id: TrackId,
    /// What the track carries.
    pub role: TrackRole,
    /// The engine that produced it.
    pub engine: EngineId,
    /// The instrument that plays it.
    pub instrument: InstrumentRef,
    /// Track gain.
    pub gain: Db,
    /// Track position in the stereo field.
    pub pan: Pan,
    /// The resolved pitch range, for tracks that map data to pitch.
    pub pitch_range: Option<PitchRange>,
    /// Events sorted by start time; ties keep insertion order.
    pub events: Vec<ScoreEvent>,
}

impl ScoreTrack {
    /// An empty track at 0 dB, centred, with no pitch range.
    #[must_use]
    pub fn new(id: TrackId, role: TrackRole, engine: EngineId, instrument: InstrumentRef) -> Self {
        Self {
            id,
            role,
            engine,
            instrument,
            gain: Db::ZERO,
            pan: Pan::CENTRE,
            pitch_range: None,
            events: Vec::new(),
        }
    }
}

/// One Score event. Serialized with a `type` tag: `{"type": "note", "t": 0.0, …}`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum ScoreEvent {
    /// A discrete note.
    Note(Note),
    /// A continuous contour.
    Glide(Glide),
    /// An earcon.
    Earcon(Earcon),
    /// Speech text.
    Speech(Speech),
    /// An audified series (CURVASONA, SDR-31; IR version 2).
    Audify(Audify),
}

impl ScoreEvent {
    /// When the event starts.
    #[must_use]
    pub fn start(&self) -> Seconds {
        match self {
            Self::Note(note) => note.t,
            Self::Glide(glide) => glide.t0,
            Self::Earcon(earcon) => earcon.t,
            Self::Speech(speech) => speech.t,
            Self::Audify(audify) => audify.t,
        }
    }
}

/// An audified series (RFC-0007 §3, SDR-31): the samples of a series, played at `rate`
/// samples per second, which the plan builder resamples to the output rate (never a
/// pitch shift). The samples are inline, so the Score stays self-contained and its hash
/// covers everything audible (ADR-0013).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Audify {
    /// Onset.
    pub t: Seconds,
    /// Playback samples per second: k times the series' own rate.
    pub rate: f64,
    /// The series, normalized to [−1, 1]; undefined values are 0.
    pub samples: Vec<f32>,
    /// Gain relative to the track.
    pub gain: Db,
    /// The rows the series comes from.
    pub src: Option<RowRange>,
}

impl Audify {
    /// The most samples one event may hold (RFC-0007 Q4: 2^22, 16 MB as f32).
    pub const MAX_SAMPLES: usize = 1 << 22;

    /// An event at 0 dB with no source rows.
    #[must_use]
    pub fn new(t: Seconds, rate: f64, samples: Vec<f32>) -> Self {
        Self {
            t,
            rate,
            samples,
            gain: Db(0.0),
            src: None,
        }
    }

    /// How long it plays: the sample count over the rate.
    #[must_use]
    pub fn duration(&self) -> Seconds {
        #[expect(
            clippy::cast_precision_loss,
            reason = "at most 2^22 samples, exact in f64"
        )]
        let count = self.samples.len() as f64;
        if self.rate > 0.0 {
            Seconds(count / self.rate)
        } else {
            Seconds::ZERO
        }
    }
}

/// Per-note timbre parameters.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Timbre {
    /// A preset that replaces the track instrument for this note (nominal timbre, SDR-25).
    pub preset: Option<InstrumentRef>,
    /// Brightness in [0, 1]: filter cutoff and spectral tilt.
    pub brightness: f64,
}

impl Timbre {
    /// The track instrument at medium brightness (0.5, 04 §5.3).
    #[must_use]
    pub fn new() -> Self {
        Self {
            preset: None,
            brightness: 0.5,
        }
    }
}

/// A discrete note.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Note {
    /// Onset.
    pub t: Seconds,
    /// Duration.
    pub dur: Seconds,
    /// Pitch relative to A4.
    pub pitch: Cents,
    /// Gain relative to the track.
    pub gain: Db,
    /// Stereo position.
    pub pan: Pan,
    /// Timbre.
    pub timbre: Timbre,
    /// Tap count for redundant encoding (A11Y-13); 1 is a single onset.
    pub taps: u8,
    /// The row that produced the note.
    pub src: Option<RowRef>,
    /// Confidence for estimated data (03 §4).
    pub conf: Option<Confidence>,
}

impl Note {
    /// A note at 0 dB, centred, one tap, with no source row.
    #[must_use]
    pub fn new(t: Seconds, dur: Seconds, pitch: Cents) -> Self {
        Self {
            t,
            dur,
            pitch,
            gain: Db::ZERO,
            pan: Pan::CENTRE,
            timbre: Timbre::new(),
            taps: 1,
            src: None,
            conf: None,
        }
    }
}

/// One breakpoint of a glide contour.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Breakpoint {
    /// Time.
    pub t: Seconds,
    /// Pitch relative to A4.
    pub pitch: Cents,
    /// Gain relative to the track.
    pub gain: Db,
    /// Stereo position.
    pub pan: Pan,
    /// Brightness in [0, 1].
    pub brightness: f64,
}

impl Breakpoint {
    /// A breakpoint at 0 dB, centred, medium brightness.
    #[must_use]
    pub fn new(t: Seconds, pitch: Cents) -> Self {
        Self {
            t,
            pitch,
            gain: Db::ZERO,
            pan: Pan::CENTRE,
            brightness: 0.5,
        }
    }
}

/// A continuous contour through breakpoints, interpolated linearly in cents and dB.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Glide {
    /// Start.
    pub t0: Seconds,
    /// End.
    pub t1: Seconds,
    /// Breakpoints in time order, inside `[t0, t1]`.
    pub curve: Vec<Breakpoint>,
    /// The rows the contour passes through.
    pub src: Option<RowRange>,
}

impl Glide {
    /// A glide from `t0` to `t1` through `curve`.
    #[must_use]
    pub fn new(t0: Seconds, t1: Seconds, curve: Vec<Breakpoint>) -> Self {
        Self {
            t0,
            t1,
            curve,
            src: None,
        }
    }
}

/// The earcons a compiled Score can contain (keyboard-and-navigation §11). Interactive
/// earcons (invalid action, mode and level changes) are produced at run time, not compiled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum EarconKind {
    /// `E-BOUNDARY`: start or end of the structure.
    Boundary,
    /// `E-LANDMARK`: an insight, landmark or annotation.
    Landmark,
    /// `E-THRESHOLD`: a threshold crossing.
    Threshold,
    /// `E-STREAM`: a stream event (Monitoring).
    Stream,
    /// `E-ALARM`: a user-configured alarm (SDR-35).
    Alarm,
    /// `E-ZERO`: a function crosses or touches zero (CURVASONA, A11Y-28; IR version 2).
    Zero,
    /// `E-ASYMPTOTE`: a vertical asymptote, where the glide breaks (IR version 2).
    Asymptote,
    /// `E-UNDEFINED`: an undefined span of a function starts (IR version 2).
    Undefined,
    /// `E-OFFSCREEN`: a function leaves the y-window (IR version 2).
    Offscreen,
}

impl EarconKind {
    /// The lowest Score IR version that has this kind.
    #[must_use]
    pub const fn ir_version(self) -> u32 {
        match self {
            Self::Zero | Self::Asymptote | Self::Undefined | Self::Offscreen => 2,
            _ => IR_VERSION_BASE,
        }
    }
}

/// An earcon.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Earcon {
    /// Onset.
    pub t: Seconds,
    /// Which earcon.
    pub kind: EarconKind,
    /// Gain relative to the track.
    pub gain: Db,
    /// The row that triggered it.
    pub src: Option<RowRef>,
}

impl Earcon {
    /// An earcon at 0 dB with no source row.
    #[must_use]
    pub fn new(t: Seconds, kind: EarconKind) -> Self {
        Self {
            t,
            kind,
            gain: Db::ZERO,
            src: None,
        }
    }
}

/// How urgently speech should be delivered.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    /// May be dropped under rate limiting.
    Low,
    /// Normal announcements.
    #[default]
    Normal,
    /// Supersedes queued announcements (A11Y-17).
    High,
}

/// Speech text. Not rendered to audio by default; exports write it to a `WebVTT` sidecar
/// (05 §4.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Speech {
    /// When to speak.
    pub t: Seconds,
    /// The text.
    pub text: String,
    /// Delivery priority.
    pub priority: Priority,
    /// The row it describes.
    pub src: Option<RowRef>,
}

impl Speech {
    /// Normal-priority speech with no source row.
    #[must_use]
    pub fn new(t: Seconds, text: impl Into<String>) -> Self {
        Self {
            t,
            text: text.into(),
            priority: Priority::Normal,
            src: None,
        }
    }
}

/// What a marker marks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MarkerKind {
    /// An analyzer insight (ECHOMETRIC).
    Insight,
    /// A navigation landmark.
    Landmark,
    /// A section boundary.
    Section,
    /// The end of the spoken legend (SDR-18).
    Legend,
}

/// A labelled point in time, used by navigation and descriptions (A11Y-23).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Marker {
    /// Time.
    pub t: Seconds,
    /// What it marks.
    pub kind: MarkerKind,
    /// A short label, also its text equivalent (A11Y-24).
    pub label: String,
    /// The rows it refers to.
    pub src: Option<RowRange>,
}

impl Marker {
    /// A marker with no source rows.
    #[must_use]
    pub fn new(t: Seconds, kind: MarkerKind, label: impl Into<String>) -> Self {
        Self {
            t,
            kind,
            label: label.into(),
            src: None,
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn spec_versions_parse_only_canonical_major_minor() {
        assert_eq!("0.1".parse(), Ok(SpecVersion::V0_1));
        assert_eq!(
            "1.10".parse(),
            Ok(SpecVersion {
                major: 1,
                minor: 10
            })
        );
        for bad in [
            "0", "0.1.0", "00.1", "0.01", "a.1", "0.", ".1", "+1.0", "0.1 ",
        ] {
            assert_eq!(bad.parse::<SpecVersion>(), Err(InvalidSpecVersion), "{bad}");
        }
        assert_eq!(
            serde_json::to_string(&SpecVersion::V0_1).unwrap(),
            r#""0.1""#
        );
    }

    #[test]
    fn events_serialize_with_a_type_tag() {
        let event = ScoreEvent::Earcon(Earcon::new(Seconds(1.5), EarconKind::Boundary));
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(
            json,
            r#"{"type":"earcon","t":1.5,"kind":"boundary","gain":0.0,"src":null}"#
        );
        assert_eq!(serde_json::from_str::<ScoreEvent>(&json).unwrap(), event);
    }

    use crate::units::Hz;

    #[test]
    fn mirroring_stays_in_range_and_is_an_involution_at_the_bounds() {
        let range = PitchRange {
            low: Hz(400.0).to_cents(),
            high: Hz(1400.0).to_cents(),
        };
        assert_eq!(range.mirror(range.low), range.high);
        assert!(range.contains(range.mirror(range.high)));
    }
}

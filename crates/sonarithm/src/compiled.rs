// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A compiled spec: the Score, its diagnostics, a summary for `inspect`, and a
//! deterministic text description.

use std::collections::BTreeMap;
use std::fmt::{self, Write as _};

use serde::Serialize;
use sonarithm_core::score::review::ReviewOptions;
use sonarithm_core::score::{Mode, ScoreEvent, ScoreTrack, TrackRole};
use sonarithm_core::{Cents, Diagnostics, Score, Severity};
use sonarithm_spec::track::ChannelName;

use crate::error::Error;

/// A successful compile: the Score IR (05 §4), the canonical spec it came from, and every
/// warning and applied default as diagnostics.
#[derive(Clone, Debug)]
pub struct Compiled {
    inner: sonarithm_compiler::Compiled,
    spec_name: String,
}

impl Compiled {
    pub(crate) fn new(inner: sonarithm_compiler::Compiled, spec_name: &str) -> Self {
        Self {
            inner,
            spec_name: spec_name.to_owned(),
        }
    }

    /// The Score.
    #[must_use]
    pub fn score(&self) -> &Score {
        &self.inner.score
    }

    /// The Score, consumed.
    #[must_use]
    pub fn into_score(self) -> Score {
        self.inner.score
    }

    /// Warnings and `Info` diagnostics (applied defaults with their rule IDs), in pipeline
    /// order. A successful compile has no errors.
    #[must_use]
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.inner.diagnostics
    }

    /// The RFC 8785 canonical spec with defaults applied, as recorded in render manifests.
    #[must_use]
    pub fn canonical_spec(&self) -> &str {
        &self.inner.canonical_spec
    }

    /// The share of data rows in each provenance class (03 §4), as the render manifest
    /// records it, such as `{"exact": 1.0}`.
    #[must_use]
    pub fn provenance(&self) -> &BTreeMap<String, f64> {
        &self.inner.provenance
    }

    /// The spec's name in messages, such as `temps.spec.json`.
    #[must_use]
    pub fn spec_name(&self) -> &str {
        &self.spec_name
    }

    /// The canonical Score hash (ADR-0013), as lowercase hex.
    ///
    /// # Errors
    ///
    /// [`Error::Canonical`] if the Score holds a non-finite number, which the compiler's
    /// invariants rule out.
    pub fn score_hash(&self) -> Result<String, Error> {
        self.inner
            .score
            .canonical_hash()
            .map(|hash| hash.to_string())
            .map_err(|err| Error::Canonical(err.to_string()))
    }

    /// The Score's review form: rounded, sorted JSON for snapshots and diffs (05 §4.3). It
    /// never replaces [`Compiled::score_hash`].
    #[must_use]
    pub fn review_json(&self) -> String {
        self.inner.score.review_json(&ReviewOptions::default())
    }

    /// What `sonarithm inspect` prints: the header, one line of facts per track, and the
    /// diagnostic counts.
    #[must_use]
    pub fn summary(&self) -> Summary {
        let score = &self.inner.score;
        let header = &score.header;
        let mut counts = DiagnosticCounts::default();
        for diagnostic in &self.inner.diagnostics {
            match diagnostic.severity {
                Severity::Error => counts.errors += 1,
                Severity::Warning => counts.warnings += 1,
                Severity::Info => counts.info += 1,
            }
        }
        Summary {
            spec: self.spec_name.clone(),
            spec_version: header.spec_version.to_string(),
            ir_version: header.ir_version,
            mode: header.mode,
            seed: header.seed,
            duration_s: micro(header.duration.0),
            spec_hash: header.spec_hash.to_string(),
            data_fingerprint: header.data_fingerprint.to_string(),
            score_hash: self.score_hash().ok(),
            engines: header
                .engines
                .iter()
                .map(|(id, version)| (id.to_string(), version.0.clone()))
                .collect(),
            tracks: score.tracks.iter().map(TrackSummary::of).collect(),
            markers: score.markers.len(),
            diagnostics: counts,
        }
    }

    /// A deterministic text description of the sonification: L1 encoding facts (what maps
    /// to what) and L2 facts about the Score (extremes and estimates), in the sense of
    /// Lundgard & Satyanarayan (A11Y-02). The same Score always gives the same text.
    ///
    /// Statistics of the data itself, and L3 trends, come from ECHOMETRIC's descriptions
    /// (C05); this description stays at what the Score shows.
    #[must_use]
    pub fn describe(&self) -> Description {
        let spec = sonarithm_spec::parse(&self.inner.canonical_spec).ok();
        let score = &self.inner.score;
        let data: Vec<&ScoreTrack> = score
            .tracks
            .iter()
            .filter(|track| track.role == TrackRole::Data)
            .collect();
        let title = spec
            .as_ref()
            .and_then(|spec| spec.name.clone())
            .unwrap_or_else(|| self.spec_name.clone());
        let mut l1 = vec![format!(
            "{title}: {} data {}, {} seconds, {} mode.",
            data.len(),
            plural(data.len(), "track", "tracks"),
            seconds(score.header.duration.0),
            mode_name(score.header.mode),
        )];
        if let Some(text) = spec.as_ref().and_then(|spec| spec.description.clone()) {
            l1.push(text);
        }
        let time_field = spec
            .as_ref()
            .and_then(|spec| spec.substrate.as_ref())
            .and_then(|substrate| substrate.time.as_ref())
            .and_then(|time| time.field.clone());
        let mut l2 = Vec::new();
        for track in &data {
            let track_spec = spec
                .as_ref()
                .and_then(|spec| spec.tracks.iter().find(|t| t.id == track.id.as_str()));
            l1.push(encoding_line(track, track_spec, time_field.as_deref()));
            l2.extend(extremes_line(track));
            l2.extend(estimates_line(track));
        }
        Description { l1, l2 }
    }
}

/// Microsecond resolution, as in the review form.
fn micro(seconds: f64) -> f64 {
    (seconds * 1e6).round() / 1e6
}

/// Hundredths of a hertz.
fn centi(hz: f64) -> f64 {
    (hz * 100.0).round() / 100.0
}

/// Seconds to the hundredth, as the summary rounds them.
fn seconds(value: f64) -> String {
    format!("{:.2}", micro(value))
}

fn hz(pitch: Cents) -> String {
    format!("{:.0}", pitch.to_hz().0)
}

fn plural<'a>(count: usize, one: &'a str, many: &'a str) -> &'a str {
    if count == 1 { one } else { many }
}

fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Analysis => "analysis",
        Mode::Presentation => "presentation",
        Mode::Monitoring => "monitoring",
    }
}

/// "Track temps: 120 tones from 0.0 to 40.0 seconds. Pitch follows temp, 400 to 1400 Hz;
/// time follows day."
fn encoding_line(
    track: &ScoreTrack,
    spec: Option<&sonarithm_spec::TrackSpec>,
    time_field: Option<&str>,
) -> String {
    let counts = EventCounts::of(track);
    let mut sounds = Vec::new();
    if counts.notes > 0 {
        sounds.push(format!(
            "{} {}",
            counts.notes,
            plural(counts.notes, "tone", "tones")
        ));
    }
    if counts.glides > 0 {
        sounds.push(format!(
            "{} {}",
            counts.glides,
            plural(counts.glides, "glide", "glides")
        ));
    }
    if sounds.is_empty() {
        sounds.push("no sounds".to_owned());
    }
    let mut line = format!("Track {}: {}", track.id, sounds.join(" and "));
    if let Some((start, end)) = span(track) {
        let _ = write!(line, " from {} to {} seconds", seconds(start), seconds(end));
    }
    line.push('.');
    let mut mapped = Vec::new();
    if let Some(spec) = spec {
        for (name, channel) in spec.channels.present() {
            let Some(field) = &channel.field else {
                continue;
            };
            let mut fact = format!("{} follows {field}", name.key());
            if name == ChannelName::Pitch
                && let Some(range) = track.pitch_range
            {
                let _ = write!(fact, ", {} to {} Hz", hz(range.low), hz(range.high));
            }
            mapped.push(fact);
        }
    }
    if let Some(field) = time_field {
        mapped.push(format!("time follows {field}"));
    }
    if !mapped.is_empty() {
        line.push(' ');
        // Every fact starts with a channel name ("pitch", "time"), never a field name, so
        // capitalizing the first letter never changes a name from the data.
        let facts = mapped.join("; ");
        if let Some(first) = facts.get(..1) {
            line.push_str(&first.to_uppercase());
            line.push_str(facts.get(1..).unwrap_or_default());
        }
        line.push('.');
    }
    line
}

/// "In temps, the highest pitch, 1400 Hz, is at 12.0 seconds (row 36); the lowest, 400 Hz,
/// at 1.0 seconds (row 3)."
fn extremes_line(track: &ScoreTrack) -> Option<String> {
    let mut high: Option<(f64, f64, Option<u64>)> = None;
    let mut low: Option<(f64, f64, Option<u64>)> = None;
    for event in &track.events {
        let points: Vec<(f64, f64, Option<u64>)> = match event {
            ScoreEvent::Note(note) => vec![(note.pitch.0, note.t.0, note.src.map(|r| r.row))],
            ScoreEvent::Glide(glide) => glide
                .curve
                .iter()
                .map(|point| (point.pitch.0, point.t.0, None))
                .collect(),
            _ => Vec::new(),
        };
        for point in points {
            // Strict comparisons keep the earliest of equal extremes.
            if high.is_none_or(|h| point.0 > h.0) {
                high = Some(point);
            }
            if low.is_none_or(|l| point.0 < l.0) {
                low = Some(point);
            }
        }
    }
    let (high, low) = (high?, low?);
    let at = |(_, t, row): (f64, f64, Option<u64>)| match row {
        Some(row) => format!("{} seconds (row {row})", seconds(t)),
        None => format!("{} seconds", seconds(t)),
    };
    Some(format!(
        "In {}, the highest pitch, {} Hz, is at {}; the lowest, {} Hz, at {}.",
        track.id,
        hz(Cents(high.0)),
        at(high),
        hz(Cents(low.0)),
        at(low),
    ))
}

/// "In temps, 3 of 10 tones are estimates; the lowest confidence is 0.75." (03 §4)
fn estimates_line(track: &ScoreTrack) -> Option<String> {
    let mut notes = 0_usize;
    let mut estimated = 0_usize;
    let mut lowest = f32::INFINITY;
    for event in &track.events {
        if let ScoreEvent::Note(note) = event {
            notes += 1;
            if let Some(conf) = note.conf {
                let value = f32::from(conf);
                if value < 1.0 {
                    estimated += 1;
                    lowest = lowest.min(value);
                }
            }
        }
    }
    (estimated > 0).then(|| {
        format!(
            "In {}, {estimated} of {notes} tones are estimates; the lowest confidence is {lowest:.2}.",
            track.id
        )
    })
}

/// When a track's first event starts and its last ends.
fn span(track: &ScoreTrack) -> Option<(f64, f64)> {
    let mut start = f64::INFINITY;
    let mut end = f64::NEG_INFINITY;
    for event in &track.events {
        let (from, to) = match event {
            ScoreEvent::Note(note) => (note.t.0, note.t.0 + note.dur.0),
            ScoreEvent::Glide(glide) => (glide.t0.0, glide.t1.0),
            other => (other.start().0, other.start().0),
        };
        start = start.min(from);
        end = end.max(to);
    }
    (start <= end).then_some((start, end))
}

#[derive(Default)]
struct EventCounts {
    notes: usize,
    glides: usize,
    earcons: usize,
    speech: usize,
    estimated: usize,
}

impl EventCounts {
    fn of(track: &ScoreTrack) -> Self {
        let mut counts = Self::default();
        for event in &track.events {
            match event {
                ScoreEvent::Note(note) => {
                    counts.notes += 1;
                    if note.conf.is_some_and(|conf| f32::from(conf) < 1.0) {
                        counts.estimated += 1;
                    }
                }
                ScoreEvent::Glide(_) => counts.glides += 1,
                ScoreEvent::Earcon(_) => counts.earcons += 1,
                ScoreEvent::Speech(_) => counts.speech += 1,
                _ => {}
            }
        }
        counts
    }
}

/// The facts `sonarithm inspect` prints. Field names are stable in `--json` output.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[non_exhaustive]
pub struct Summary {
    /// The spec's name, such as `temps.spec.json`.
    pub spec: String,
    /// The spec language version.
    pub spec_version: String,
    /// The Score IR version.
    pub ir_version: u32,
    /// The mode (SDR-36).
    pub mode: Mode,
    /// The seed of every random stream (SDR-02).
    pub seed: u64,
    /// Total length in seconds, to the microsecond.
    pub duration_s: f64,
    /// SHA-256 of the canonical spec.
    pub spec_hash: String,
    /// SHA-256 fingerprint of the data.
    pub data_fingerprint: String,
    /// The canonical Score hash; `None` only for a Score with a non-finite number.
    pub score_hash: Option<String>,
    /// The engines that contributed, with their versions.
    pub engines: BTreeMap<String, String>,
    /// One entry per track, in Score order.
    pub tracks: Vec<TrackSummary>,
    /// The number of markers (insights, landmarks, sections).
    pub markers: usize,
    /// How many diagnostics of each severity the compile reported.
    pub diagnostics: DiagnosticCounts,
}

/// One track in a [`Summary`].
#[derive(Clone, Debug, PartialEq, Serialize)]
#[non_exhaustive]
pub struct TrackSummary {
    /// The track ID.
    pub id: String,
    /// What the track carries.
    pub role: TrackRole,
    /// The engine that produced it.
    pub engine: String,
    /// The instrument that plays it.
    pub instrument: String,
    /// All events.
    pub events: usize,
    /// Notes.
    pub notes: usize,
    /// Glides.
    pub glides: usize,
    /// Earcons.
    pub earcons: usize,
    /// Speech events.
    pub speech: usize,
    /// Notes from estimated data (confidence below 1, 03 §4).
    pub estimated: usize,
    /// When the first event starts, in seconds.
    pub start_s: Option<f64>,
    /// When the last event ends, in seconds.
    pub end_s: Option<f64>,
    /// The resolved pitch range in hertz, low to high, to 0.01 Hz.
    pub pitch_range_hz: Option<[f64; 2]>,
    /// Track gain in dB.
    pub gain_db: f64,
    /// Stereo position, −1 (left) to 1 (right).
    pub pan: f64,
}

impl TrackSummary {
    fn of(track: &ScoreTrack) -> Self {
        let counts = EventCounts::of(track);
        let span = span(track);
        Self {
            id: track.id.to_string(),
            role: track.role,
            engine: track.engine.to_string(),
            instrument: track.instrument.0.clone(),
            events: track.events.len(),
            notes: counts.notes,
            glides: counts.glides,
            earcons: counts.earcons,
            speech: counts.speech,
            estimated: counts.estimated,
            start_s: span.map(|(start, _)| micro(start)),
            end_s: span.map(|(_, end)| micro(end)),
            pitch_range_hz: track
                .pitch_range
                .map(|range| [centi(range.low.to_hz().0), centi(range.high.to_hz().0)]),
            gain_db: track.gain.0,
            pan: track.pan.0,
        }
    }
}

/// Diagnostic counts by severity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct DiagnosticCounts {
    /// Errors (none after a successful compile).
    pub errors: usize,
    /// Warnings.
    pub warnings: usize,
    /// `Info` diagnostics: applied defaults and automatic choices.
    pub info: usize,
}

/// A text description of a sonification ([`Compiled::describe`]): one sentence per line,
/// readable by a screen reader in order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Description {
    /// L1: what is encoded and how.
    pub l1: Vec<String>,
    /// L2: facts about the sound, such as where the extremes are.
    pub l2: Vec<String>,
}

impl fmt::Display for Description {
    /// The L1 lines, then the L2 lines, one per line.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.l1.iter().chain(&self.l2) {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

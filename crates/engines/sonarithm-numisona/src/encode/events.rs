// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Building `Note` and `Glide` events from the resolved channels.

use sonarithm_core::diagnostic::Diagnostic;
use sonarithm_core::engine::{CompileContext, CompileError};
use sonarithm_core::ids::RowRange;
use sonarithm_core::score::{Breakpoint, Glide, InstrumentRef, Note, ScoreEvent, Timbre};
use sonarithm_core::units::{Cents, Db, Pan, Seconds};

use super::{CANCEL_EVERY, MIN_TAP_S, NoteParts, PitchChannel, Run, Sources, Timeline, Values};
use crate::codes;

impl Run<'_> {
    /// One note per row; a row with a missing value in any mapped channel is silent.
    pub(super) fn notes(
        &mut self,
        timeline: &Timeline,
        parts: &NoteParts<'_>,
        sources: &Sources<'_>,
        cx: &mut CompileContext,
    ) -> Result<Vec<ScoreEvent>, CompileError> {
        let instrument = self
            .track
            .instrument
            .clone()
            .unwrap_or_else(|| super::DEFAULT_INSTRUMENT.to_owned());
        let mut events = Vec::with_capacity(timeline.rows.len());
        let mut fused = 0_usize;
        // A constant timbre is resolved once, not once per note.
        let constant = match parts.timbre {
            Values::Constant(name) => Some(preset_of(name, &instrument)),
            Values::PerRow(_) => None,
        };
        for (i, (&row, &onset)) in timeline.rows.iter().zip(&timeline.onsets).enumerate() {
            if i % CANCEL_EVERY == 0 {
                cx.check_cancelled()?;
            }
            let (Some(pitch), Some(duration), Some(taps), Some(gain), Some(pan), Some(brightness)) = (
                parts.pitch.get(i),
                parts.duration.get(i),
                parts.taps.get(i),
                parts.loudness.get(i),
                parts.pan.get(i),
                parts.brightness.get(i),
            ) else {
                continue;
            };
            let preset = match (&constant, parts.timbre) {
                (Some(preset), _) => preset.clone(),
                (None, Values::PerRow(names)) => match names.get(i) {
                    Some(Some(name)) => preset_of(name, &instrument),
                    _ => continue,
                },
                (None, Values::Constant(name)) => preset_of(name, &instrument),
            };
            if taps > 1.0 && duration / taps < MIN_TAP_S {
                fused += 1;
            }
            let mut note = Note::new(Seconds(onset), Seconds(duration), pitch);
            note.gain = Db(gain);
            note.pan = Pan::clamped(pan);
            note.timbre = timbre(preset, sources.brightness(row, brightness));
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "tap counts are whole numbers from 1 to 8"
            )]
            {
                note.taps = taps as u8;
            }
            note.src = sources.table.row_ref(row as u64);
            note.conf = sources.confidence(row);
            events.push(ScoreEvent::Note(note));
        }
        if fused > 0 {
            let message = format!(
                "NUMISONA: track `{}`: {fused} notes play their taps faster than about 10 per second, where taps may fuse and stop being countable (SDR-20, experimental). Lengthen the notes or slow the pace.",
                self.request.id
            );
            self.diagnostics
                .push(Diagnostic::new(&codes::TAPS_TOO_FAST, message).at(self.at.key("channels")));
        }
        Ok(events)
    }
}

/// One contour per run of rows without missing values. Each contour passes through its
/// rows' values at their onsets and holds the last one for one step.
pub(super) fn glides(
    timeline: &Timeline,
    pitch: &PitchChannel,
    loudness: &Values<f64>,
    pan: &Values<f64>,
    brightness: &Values<f64>,
    sources: &Sources<'_>,
    cx: &mut CompileContext,
) -> Result<Vec<ScoreEvent>, CompileError> {
    let mut events = Vec::new();
    let mut run: Vec<(usize, Breakpoint)> = Vec::new();
    let finish = |run: &mut Vec<(usize, Breakpoint)>, events: &mut Vec<ScoreEvent>| {
        let (Some(&(_, first)), Some(&(_, last))) = (run.first(), run.last()) else {
            return;
        };
        let mut hold = last;
        hold.t = Seconds(last.t.0 + timeline.step);
        let mut curve: Vec<Breakpoint> = run.iter().map(|(_, point)| *point).collect();
        curve.push(hold);
        let rows = run.iter().map(|(row, _)| *row as u64);
        let (start, end) = rows.fold((u64::MAX, 0), |(lo, hi), row| {
            (lo.min(row), hi.max(row + 1))
        });
        let mut glide = Glide::new(first.t, hold.t, curve);
        let all = sources.table.rows();
        glide.src = Some(RowRange { start, end, ..all });
        events.push(ScoreEvent::Glide(glide));
        run.clear();
    };
    for (i, &onset) in timeline.onsets.iter().enumerate() {
        if i % CANCEL_EVERY == 0 {
            cx.check_cancelled()?;
        }
        let values = (
            pitch.values.get(i),
            loudness.get(i),
            pan.get(i),
            brightness.get(i),
        );
        let (Some(cents), Some(gain), Some(pan), Some(brightness)) = values else {
            finish(&mut run, &mut events);
            continue;
        };
        let row = timeline.rows[i];
        run.push((
            row,
            breakpoint(onset, cents, gain, pan, sources.brightness(row, brightness)),
        ));
    }
    finish(&mut run, &mut events);
    Ok(events)
}

fn breakpoint(t: f64, pitch: Cents, gain: f64, pan: f64, brightness: f64) -> Breakpoint {
    let mut point = Breakpoint::new(Seconds(t), pitch);
    point.gain = Db(gain);
    point.pan = Pan::clamped(pan);
    point.brightness = brightness.clamp(0.0, 1.0);
    point
}

/// The per-note preset: `None` for the track instrument itself.
fn preset_of(name: &str, instrument: &str) -> Option<InstrumentRef> {
    (name != instrument).then(|| InstrumentRef(name.to_owned()))
}

fn timbre(preset: Option<InstrumentRef>, brightness: f64) -> Timbre {
    let mut timbre = Timbre::new();
    timbre.preset = preset;
    timbre.brightness = brightness.clamp(0.0, 1.0);
    timbre
}

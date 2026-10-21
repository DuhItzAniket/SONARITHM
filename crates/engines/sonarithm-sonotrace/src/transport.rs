// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Transport helpers (06 §5; B06-T03): scrub grains and long-series windows.
//!
//! **Scrub grains.** Scrubbing (by keyboard or pointer) is a stream of seeks plus short
//! grains that preview the local value (06 §5), so the listener hears the data under the
//! cursor while moving it (overview, then detail: A11Y-09). [`scrub_grain`] gives the grain
//! of one track: a note that starts exactly at the cursor, with the pitch, level, pan and
//! timbre the track has there.
//!
//! **Long-series windows.** [`window`] cuts the part of a track inside a time range and
//! moves it to start at 0, so a long series can be played a section at a time; [`windows`]
//! splits a duration into consecutive ranges.

use sonarithm_core::score::{Breakpoint, Glide, Note, ScoreEvent, ScoreTrack, Timbre};
use sonarithm_core::{Cents, Db, Pan, Seconds};

/// The length of a scrub grain: 80 ms.
///
/// Provisional and experimental: there is no scrub-specific evidence yet. 80 ms is the
/// lower end of SDR-32's grain range (80–300 ms), short enough to follow a moving cursor.
/// TODO(verify): the HP-11 listening check and co-design round 1 (B07-T04).
pub const GRAIN: Seconds = Seconds(0.08);

/// The preview of `track` at `cursor`: a note of length [`GRAIN`] starting at `cursor`,
/// with the value the track has there.
///
/// The value is the glide contour at `cursor` (interpolated linearly in cents and dB, as
/// the renderer does), else the note sounding at `cursor`, else the last note before it,
/// else the track's first note. `None` for a track without notes or glides.
#[must_use]
pub fn scrub_grain(track: &ScoreTrack, cursor: Seconds) -> Option<Note> {
    let t = cursor.0;
    let mut sounding: Option<&Note> = None;
    let mut before: Option<&Note> = None;
    let mut first: Option<&Note> = None;
    for event in &track.events {
        match event {
            ScoreEvent::Glide(glide) if glide.t0.0 <= t && t <= glide.t1.0 => {
                if let Some(point) = contour_at(glide, t) {
                    let mut grain = Note::new(cursor, GRAIN, point.pitch);
                    grain.gain = point.gain;
                    grain.pan = point.pan;
                    grain.timbre = Timbre::new();
                    grain.timbre.brightness = point.brightness;
                    return Some(grain);
                }
            }
            ScoreEvent::Note(note) => {
                first = first.or(Some(note));
                if note.t.0 <= t {
                    before = Some(note);
                    if t < note.t.0 + note.dur.0 {
                        sounding = Some(note);
                    }
                }
            }
            _ => {}
        }
    }
    let source = sounding.or(before).or(first)?;
    let mut grain = source.clone();
    grain.t = cursor;
    grain.dur = GRAIN;
    grain.taps = 1;
    Some(grain)
}

/// The glide's contour at time `t`, inside `[t0, t1]`.
fn contour_at(glide: &Glide, t: f64) -> Option<Breakpoint> {
    let points = &glide.curve;
    let first = points.first()?;
    if t <= first.t.0 {
        return Some(*first);
    }
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if t <= b.t.0 {
            let span = b.t.0 - a.t.0;
            let f = if span > 0.0 { (t - a.t.0) / span } else { 1.0 };
            let mut point =
                Breakpoint::new(Seconds(t), Cents(a.pitch.0 + (b.pitch.0 - a.pitch.0) * f));
            point.gain = Db(a.gain.0 + (b.gain.0 - a.gain.0) * f);
            point.pan = Pan::clamped(a.pan.0 + (b.pan.0 - a.pan.0) * f);
            point.brightness = a.brightness + (b.brightness - a.brightness) * f;
            return Some(point);
        }
    }
    points.last().copied()
}

/// The part of `track` inside `[start, end)`, moved to start at 0.
///
/// Notes keep their length and are included when they start inside the range. Glides that
/// overlap the range are cut at its edges, with breakpoints interpolated there. Earcons and
/// speech are included when they start inside the range.
#[must_use]
pub fn window(track: &ScoreTrack, start: Seconds, end: Seconds) -> ScoreTrack {
    let (s, e) = (start.0, end.0);
    let mut out = track.clone();
    out.events = Vec::new();
    let shift = |t: Seconds| Seconds(t.0 - s);
    for event in &track.events {
        match event {
            ScoreEvent::Glide(glide) => {
                if glide.t1.0 < s || glide.t0.0 >= e {
                    continue;
                }
                let (a, b) = (glide.t0.0.max(s), glide.t1.0.min(e));
                let mut curve = Vec::new();
                if let Some(point) = contour_at(glide, a) {
                    curve.push(point);
                }
                curve.extend(
                    glide
                        .curve
                        .iter()
                        .filter(|p| p.t.0 > a && p.t.0 < b)
                        .copied(),
                );
                if b > a
                    && let Some(point) = contour_at(glide, b)
                {
                    curve.push(point);
                }
                for point in &mut curve {
                    point.t = shift(point.t);
                }
                let mut cut = Glide::new(Seconds(a - s), Seconds(b - s), curve);
                cut.src = glide.src;
                out.events.push(ScoreEvent::Glide(cut));
            }
            other => {
                let t = other.start().0;
                if t >= s && t < e {
                    let mut moved = other.clone();
                    match &mut moved {
                        ScoreEvent::Note(note) => note.t = shift(note.t),
                        ScoreEvent::Earcon(earcon) => earcon.t = shift(earcon.t),
                        ScoreEvent::Speech(speech) => speech.t = shift(speech.t),
                        _ => continue,
                    }
                    out.events.push(moved);
                }
            }
        }
    }
    out
}

/// Consecutive ranges of `length` that cover `[0, duration)`; the last may be shorter.
/// Empty for a non-positive or non-finite duration or length.
#[must_use]
pub fn windows(duration: Seconds, length: Seconds) -> Vec<(Seconds, Seconds)> {
    let (d, l) = (duration.0, length.0);
    if !(d.is_finite() && l.is_finite() && d > 0.0 && l > 0.0) {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    let mut i = 0_u32;
    loop {
        let start = f64::from(i) * l;
        if start >= d {
            break;
        }
        ranges.push((Seconds(start), Seconds((start + l).min(d))));
        i += 1;
    }
    ranges
}

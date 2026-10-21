// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Context cues (07 AUDAXIS, `context`; sound-design §7): the tick track (SDR-16), the
//! dynamic reference tone (SDR-17) and the spoken legend (SDR-18).
//!
//! - **Ticks.** About 10 ms broadband clicks (the render engine's `click`) at major x
//!   ticks, centred, on the context bus (−10 dB). `enabled: auto` puts them on when every
//!   data track plays discrete notes and off when any plays glides, where Smith & Walker
//!   (2005) found they did not help. Without `every`, the interval is a "nice" step for
//!   about [`TICK_TARGET`] intervals across the x range (provisional, OD-19, to match the
//!   visual axis of B04).
//! - **Reference tone.** In a timbre distinct from the data, it sounds the series maximum
//!   while values rise and the minimum while they fall. The direction comes from a 3-point
//!   moving average with hysteresis of [`HYSTERESIS`] of the value range, so noise does not
//!   make it flap, and switches overlap by [`CROSSFADE`]. A static first-value reference is
//!   never built; an explicit `static-value` reference is a warning (SDR-17).
//! - **Legend.** Spoken before playback by default: title, x range, value ranges with
//!   units, the pitch mapping with its polarity, the mode and the cues that are on. The
//!   Score keeps it as speech in the source locale (`en-US`) for exports; the app speaks
//!   it in the listener's locale with [`Composer::legend`].

use sonarithm_core::diagnostic::{Diagnostic, SpecPointer};
use sonarithm_core::engine::{Annotation, CompileContext, CompileError};
use sonarithm_core::ids::TrackId;
use sonarithm_core::math;
use sonarithm_core::nav::XType;
use sonarithm_core::score::{
    InstrumentRef, Mode, Note, Score, ScoreEvent, ScoreTrack, Speech, TrackRole,
};
use sonarithm_core::units::{Cents, Seconds};
use sonarithm_data::DataBundle;
use sonarithm_i18n::{Localizer, SOURCE_LOCALE};
use sonarithm_spec::sections::{
    Context, LegendPart, LegendSpeak, ReferenceKind, SpatialMode, Ticks,
};
use sonarithm_spec::spec::Spec;
use sonarithm_spec::values::Toggle;

use crate::ENGINE;
use crate::announce::{Composer, LegendCues, Settings};
use crate::codes;
use crate::nav::tree::{self, Point, Series};

/// The tick track's ID; a dot cannot appear in a spec track ID, so it never collides.
pub const TICK_TRACK: &str = "audaxis.ticks";
/// The reference tone's track ID.
pub const REFERENCE_TRACK: &str = "audaxis.reference";
/// The spoken legend's track ID.
pub const LEGEND_TRACK: &str = "audaxis.legend";

/// SDR-16: a tick is a click of about 10 ms.
pub const TICK_LENGTH: Seconds = Seconds(0.010);
/// About this many major tick intervals across the x range when `every` is not set.
/// Provisional (OD-19), not a research default.
pub const TICK_TARGET: f64 = 5.0;
/// SDR-17 (sound-design §7.2): reference switches overlap by at least 40 ms.
pub const CROSSFADE: Seconds = Seconds(0.040);
/// The share of the value range a smoothed step must exceed to turn the reference tone.
/// Provisional, as sound-design §7.2's "smoothed slope with hysteresis (proposal)".
pub const HYSTERESIS: f64 = 0.05;

/// The context tracks for a compiled Score.
///
/// # Errors
///
/// [`CompileError::Cancelled`], [`CompileError::Engine`] when a track filter cannot be
/// applied or the legend cannot be composed.
pub(crate) fn tracks(
    spec: &Spec,
    score: &Score,
    data: &DataBundle,
    annotations: &[Annotation],
    cx: &mut CompileContext,
) -> Result<Vec<ScoreTrack>, CompileError> {
    let context = spec.context.clone().unwrap_or_default();
    spatial_mode(&context, cx);
    let series = tree::read(spec, score, data, cx)?;
    let ticks = ticks(&context.ticks.clone().unwrap_or_default(), &series, cx);
    let reference = reference(&context, &series, cx);
    let cues = LegendCues {
        ticks: ticks.is_some(),
        reference: reference.is_some(),
    };
    let mut tracks: Vec<ScoreTrack> = ticks.into_iter().chain(reference).collect();
    tracks.extend(legend(spec, &context, score, &series, annotations, cues)?);
    Ok(tracks)
}

/// SDR-30: binaural and multichannel arrive in D03; this build plays stereo.
fn spatial_mode(context: &Context, cx: &mut CompileContext) {
    if let Some(spatial) = context.spatial
        && spatial.mode != SpatialMode::Stereo
    {
        let at = SpecPointer::root()
            .key("context")
            .key("spatial")
            .key("mode");
        cx.report(
            Diagnostic::new(
                &codes::SPATIAL_MODE_UNAVAILABLE,
                "Binaural and multichannel output arrive in D03; this Score plays in stereo.",
            )
            .at(at),
        );
    }
}

/// From x positions to onsets: the played points of the first series that has two at
/// different positions, interpolated linearly.
struct TimeMap {
    pairs: Vec<(f64, f64)>,
    x_type: XType,
}

impl TimeMap {
    fn of(series: &[Series]) -> Option<Self> {
        series.iter().find_map(|s| {
            let mut pairs: Vec<(f64, f64)> = Vec::new();
            for point in &s.points {
                if let Some(t) = point.t
                    && pairs.last().is_none_or(|(x, _)| point.position > *x)
                {
                    pairs.push((point.position, t.0));
                }
            }
            (pairs.len() >= 2).then_some(Self {
                pairs,
                x_type: s.x_type,
            })
        })
    }

    fn span(&self) -> (f64, f64) {
        let first = self.pairs.first().map_or(0.0, |p| p.0);
        let last = self.pairs.last().map_or(first, |p| p.0);
        (first, last)
    }

    /// The onset at x position `x`, inside the span.
    fn time(&self, x: f64) -> f64 {
        let index = self.pairs.partition_point(|(px, _)| *px < x);
        match (
            index.checked_sub(1).and_then(|i| self.pairs.get(i)),
            self.pairs.get(index),
        ) {
            (Some(&(x0, t0)), Some(&(x1, t1))) if x1 > x0 => t0 + (t1 - t0) * (x - x0) / (x1 - x0),
            (_, Some(&(_, t))) | (Some(&(_, t)), None) => t,
            (None, None) => 0.0,
        }
    }
}

/// A "nice" step (1, 2 or 5 times a power of ten) for about `target` intervals over
/// `range`. Positions that count rows or categories step by whole numbers.
fn nice_step(range: f64, target: f64, whole: bool) -> f64 {
    let raw = (range / target).max(f64::MIN_POSITIVE);
    let power = math::pow(10.0, math::log10(raw).floor());
    let error = raw / power;
    let step = power
        * if error >= 7.07 {
            10.0
        } else if error >= 3.16 {
            5.0
        } else if error >= 1.41 {
            2.0
        } else {
            1.0
        };
    if whole { step.round().max(1.0) } else { step }
}

/// SDR-16: the tick track, or `None` when ticks are off or cannot be placed.
fn ticks(settings: &Ticks, series: &[Series], cx: &mut CompileContext) -> Option<ScoreTrack> {
    let at = SpecPointer::root().key("context").key("ticks");
    let on = if let Some(Toggle::Set(on)) = settings.enabled {
        on
    } else {
        let on = !series.is_empty() && series.iter().all(|s| !s.glides);
        let message = if on {
            "Ticks are on at the major x ticks, because every data track plays discrete notes (SDR-16)."
        } else {
            "Ticks are off, because a data track plays glides; ticks did not help with continuous tones (SDR-16)."
        };
        cx.report(Diagnostic::new(&codes::TICKS_DECIDED, message).at(at.key("enabled")));
        on
    };
    if !on {
        return None;
    }
    let left_out = |cx: &mut CompileContext, why: &str| {
        cx.report(Diagnostic::new(&codes::TICKS_UNAVAILABLE, why).at(at.clone()));
        None
    };
    let Some(map) = TimeMap::of(series) else {
        return left_out(
            cx,
            "Ticks are left out: no data track has two points at different x positions.",
        );
    };
    let (low, high) = map.span();
    let whole = matches!(map.x_type, XType::Category | XType::RowOrder);
    let step = match settings.every {
        Some(every) if every.is_finite() && every > 0.0 => every,
        Some(_) => {
            return left_out(
                cx,
                "Ticks are left out: `context.ticks.every` must be positive.",
            );
        }
        None => nice_step(high - low, TICK_TARGET, whole),
    };
    let first = (low / step).ceil();
    let last = (high / step).floor();
    if last - first > 10_000.0 {
        return left_out(
            cx,
            "Ticks are left out: `context.ticks.every` gives more than 10 000 ticks.",
        );
    }
    let mut track = ScoreTrack::new(
        TrackId::new(TICK_TRACK),
        TrackRole::Context,
        ENGINE,
        InstrumentRef("click".to_owned()),
    );
    let mut k = first;
    while k <= last {
        let t = map.time(k * step);
        track.events.push(ScoreEvent::Note(Note::new(
            Seconds(t),
            TICK_LENGTH,
            Cents(0.0),
        )));
        k += 1.0;
    }
    Some(track)
}

/// A reference timbre distinct from the data instrument (sound-design §7.2): a triangle
/// for mallet, square and saw data, a filtered pulse (the comfort-filtered square) for
/// sine data, a sine for triangle data.
fn distinct(data: &str) -> &'static str {
    match data {
        "sine" => "square",
        "triangle" => "sine",
        _ => "triangle",
    }
}

/// SDR-17: the reference tone, or `None` when it is off or cannot be built.
fn reference(context: &Context, series: &[Series], cx: &mut CompileContext) -> Option<ScoreTrack> {
    let reference = context.reference.clone()?;
    let at = SpecPointer::root().key("context").key("reference");
    if reference.kind == ReferenceKind::None {
        return None;
    }
    let left_out = |cx: &mut CompileContext, why: String| {
        cx.report(Diagnostic::new(&codes::REFERENCE_UNAVAILABLE, why).at(at.clone()));
        None
    };
    let wanted = reference
        .track
        .clone()
        .or_else(|| series.first().map(|s| s.track.as_str().to_owned()));
    let Some(data) = series
        .iter()
        .find(|s| Some(s.track.as_str()) == wanted.as_deref())
    else {
        return left_out(
            cx,
            format!(
                "The reference tone is left out: there is no data track `{}`.",
                wanted.unwrap_or_default()
            ),
        );
    };
    let mut played: Vec<&Point> = data
        .points
        .iter()
        .filter(|p| p.t.is_some() && p.pitch.is_some() && p.value.is_some())
        .collect();
    played.sort_by(|a, b| {
        a.t.map(|t| t.0)
            .unwrap_or_default()
            .total_cmp(&b.t.map(|t| t.0).unwrap_or_default())
    });
    let extremes = data.extremes().filter(|(low, high)| high.value > low.value);
    let (Some((low, high)), true) = (extremes, played.len() >= 2) else {
        return left_out(
            cx,
            format!(
                "The reference tone is left out: track `{}` has fewer than two different values.",
                data.track
            ),
        );
    };
    let events = match reference.kind {
        ReferenceKind::DynamicExtreme => dynamic(&played, low, high),
        ReferenceKind::StaticValue => {
            let Some(value) = reference.value else {
                return left_out(
                    cx,
                    "The reference tone is left out: a `static-value` reference needs `value`."
                        .to_owned(),
                );
            };
            cx.report(
                Diagnostic::new(
                    &codes::STATIC_REFERENCE,
                    format!("A static reference tone at {value} was asked for; the dynamic-extreme reference helped listeners and a static one did not (SDR-17)."),
                )
                .at(at.key("kind")),
            );
            constant(&played, value)
        }
        ReferenceKind::ZeroLine => {
            let (Some(lo), Some(hi)) = (low.value, high.value) else {
                return None;
            };
            if !(lo..=hi).contains(&0.0) {
                return left_out(
                    cx,
                    format!(
                        "The zero-line reference is left out: zero is outside track `{}`'s values ({lo} to {hi}).",
                        data.track
                    ),
                );
            }
            constant(&played, 0.0)
        }
        _ => return None,
    };
    let instrument = reference
        .timbre
        .clone()
        .unwrap_or_else(|| distinct(&data.instrument).to_owned());
    let mut track = ScoreTrack::new(
        TrackId::new(REFERENCE_TRACK),
        TrackRole::Context,
        ENGINE,
        InstrumentRef(instrument),
    );
    track.events = events;
    Some(track)
}

fn onset(point: &Point) -> f64 {
    point.t.map_or(0.0, |t| t.0)
}

fn end(point: &Point) -> f64 {
    onset(point) + point.dur.map_or(0.0, |d| d.0)
}

/// The maximum's pitch while values rise, the minimum's while they fall.
fn dynamic(played: &[&Point], low: &Point, high: &Point) -> Vec<ScoreEvent> {
    let values: Vec<f64> = played.iter().filter_map(|p| p.value).collect();
    let n = values.len();
    #[expect(
        clippy::cast_precision_loss,
        reason = "a window of at most three values"
    )]
    let smoothed: Vec<f64> = (0..n)
        .map(|k| {
            let window = &values[k.saturating_sub(1)..(k + 2).min(n)];
            window.iter().sum::<f64>() / window.len() as f64
        })
        .collect();
    let threshold = HYSTERESIS * (high.value.unwrap_or(0.0) - low.value.unwrap_or(0.0));
    let mut rising = smoothed
        .windows(2)
        .map(|w| w[1] - w[0])
        .find(|step| step.abs() > threshold)
        .map_or_else(|| values[n - 1] >= values[0], |step| step > 0.0);
    let mut directions = Vec::with_capacity(n);
    for k in 0..n {
        if k > 0 {
            let step = smoothed[k] - smoothed[k - 1];
            if step > threshold {
                rising = true;
            } else if step < -threshold {
                rising = false;
            }
        }
        directions.push(rising);
    }
    let pitch = |rising: bool| if rising { high.pitch } else { low.pitch }.unwrap_or(Cents(0.0));
    let mut events = Vec::new();
    let mut start = 0;
    while start < n {
        let mut stop = start;
        while stop + 1 < n && directions[stop + 1] == directions[start] {
            stop += 1;
        }
        let from = onset(played[start]);
        let to = match played.get(stop + 1) {
            Some(next) => onset(next) + CROSSFADE.0,
            None => end(played[stop]),
        };
        events.push(ScoreEvent::Note(Note::new(
            Seconds(from),
            Seconds((to - from).max(CROSSFADE.0)),
            pitch(directions[start]),
        )));
        start = stop + 1;
    }
    events
}

/// One note at the pitch of `value`, from the first onset to the last end. The pitch is
/// interpolated between the played points, so it follows the track's own mapping.
fn constant(played: &[&Point], value: f64) -> Vec<ScoreEvent> {
    let mut by_value: Vec<(f64, f64)> = played
        .iter()
        .filter_map(|p| Some((p.value?, p.pitch?.0)))
        .collect();
    by_value.sort_by(|a, b| a.0.total_cmp(&b.0));
    let index = by_value.partition_point(|(v, _)| *v < value);
    let cents = match (
        index.checked_sub(1).and_then(|i| by_value.get(i)),
        by_value.get(index),
    ) {
        (Some(&(v0, c0)), Some(&(v1, c1))) if v1 > v0 => c0 + (c1 - c0) * (value - v0) / (v1 - v0),
        (_, Some(&(_, c))) | (Some(&(_, c)), None) => c,
        (None, None) => 0.0,
    };
    let from = played.first().map_or(0.0, |p| onset(p));
    let to = played.iter().map(|p| end(p)).fold(from, f64::max);
    vec![ScoreEvent::Note(Note::new(
        Seconds(from),
        Seconds((to - from).max(CROSSFADE.0)),
        Cents(cents),
    ))]
}

/// SDR-18: the spoken legend as a speech track, when it is spoken before playback.
fn legend(
    spec: &Spec,
    context: &Context,
    score: &Score,
    series: &[Series],
    annotations: &[Annotation],
    cues: LegendCues,
) -> Result<Option<ScoreTrack>, CompileError> {
    let legend = context.legend.clone().unwrap_or_default();
    if !matches!(legend.speak, None | Some(LegendSpeak::BeforePlay)) {
        return Ok(None);
    }
    let parts = legend.parts.unwrap_or_else(|| {
        vec![
            LegendPart::Title,
            LegendPart::XRange,
            LegendPart::YRange,
            LegendPart::Units,
            LegendPart::Polarity,
        ]
    });
    let failed = |e: sonarithm_i18n::I18nError| CompileError::Engine {
        engine: ENGINE,
        message: format!("the spoken legend cannot be composed: {e}"),
    };
    let tree = tree::assemble(spec, series, annotations);
    let localizer = Localizer::new(SOURCE_LOCALE).map_err(failed)?;
    let composer = Composer::new(&localizer, Settings::default());
    let mode = if score.header.mode == Mode::Presentation {
        "presentation"
    } else {
        "analysis"
    };
    let text = composer
        .legend(&tree, &parts, mode, cues)
        .map_err(failed)?
        .speech;
    let mut track = ScoreTrack::new(
        TrackId::new(LEGEND_TRACK),
        TrackRole::Speech,
        ENGINE,
        InstrumentRef("speech".to_owned()),
    );
    track
        .events
        .push(ScoreEvent::Speech(Speech::new(Seconds::ZERO, text)));
    Ok(Some(track))
}

#[cfg(test)]
mod tests {
    use super::nice_step;

    #[test]
    fn nice_steps_are_one_two_or_five_times_a_power_of_ten() {
        assert!((nice_step(145.0, 5.0, false) - 20.0).abs() < 1e-9);
        assert!((nice_step(19.0, 5.0, false) - 5.0).abs() < 1e-9);
        assert!((nice_step(1.0, 5.0, false) - 0.2).abs() < 1e-9);
        assert!((nice_step(4.0, 5.0, true) - 1.0).abs() < 1e-9);
    }
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Audification (C03-T05; RFC-0007 §3; SDR-31): the `audify` mark plays a series as a
//! waveform, sped up by a factor k, by resampling (the render engine resamples to the
//! output rate; nothing is pitch-shifted).
//!
//! **The speed-up calculator (SDR-31).** A series of `n` samples has its own rate `fs`:
//! samples per second of its time field when the field is temporal, else per unit of the
//! time field, else per row. Rows that are not evenly spaced (a function table's extra rows
//! at critical points) are first interpolated onto an even grid. Its length is `T = n / fs` and its band runs from `1/T` to
//! `fs/2`. Played `k` times faster, it lasts `T/k` and sounds in `[k/T, k·fs/2]`. The
//! compile reports k, the duration and the audible part of that band as Info (`SNR-ENG-460`).
//! `tracks[].audify.k` sets k; `tracks[].audify.band` sets the target band `[lo, hi]` in Hz
//! and k maps `fs/2` to `hi` (`k = 2·hi/fs`).
//!
//! **Defaults and evidence.** The seismic preset k = 2200 is SDR-31's (Dombois 2001). With
//! neither `k` nor `band`, k puts the top of the band at 4 kHz (SDR-44, `INFER`,
//! experimental, F-196): an extrapolation of ED-06, so the energy stays where ageing ears
//! keep it, until a listening study. No EEG preset: none has a documented
//! source yet (TODO(verify), C03-T05).

use arrow_array::{Float64Array, Int32Array, Int64Array, TimestampMillisecondArray};
use serde_json::Value;
use sonarithm_core::engine::{
    Annotation, CompileContext, CompileError, Encoder, SpecFragment, TrackRequest,
};
use sonarithm_core::score::{Audify, InstrumentRef, ScoreEvent, ScoreTrack, TrackRole};
use sonarithm_core::{Db, Diagnostic, RowRange, Seconds, TrackId};
use sonarithm_data::{DataBundle, Table};

use crate::{ENGINE, codes};

/// SDR-31's seismic preset (Dombois 2001).
pub const SEISMIC_K: f64 = 2200.0;

/// The smallest allowed k (RFC-0007 amendment 3).
pub const K_MIN: f64 = 1e-3;

/// The largest allowed k.
pub const K_MAX: f64 = 1e6;

/// The audible band.
pub const AUDIBLE: [f64; 2] = [20.0, 20_000.0];

/// Where the top of the band goes when the spec gives neither `k` nor `band` (SDR-44,
/// `INFER`, experimental).
pub const DEFAULT_TOP_HZ: f64 = 4_000.0;

/// The level of an audification without a `loudness` value (SDR-23's default).
pub const DEFAULT_LOUDNESS: Db = Db(-18.0);

/// The result of the speed-up calculator (SDR-31).
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Speedup {
    /// The speed-up factor.
    pub k: f64,
    /// The series' own rate, samples per second or per unit.
    pub fs: f64,
    /// The output duration, T/k.
    pub duration: Seconds,
    /// The band it sounds in, `[k/T, k·fs/2]`, in Hz.
    pub band: [f64; 2],
    /// The part of the band inside 20 Hz–20 kHz, if any.
    pub audible: Option<[f64; 2]>,
}

/// The speed-up of `count` samples at `fs` played `k` times faster.
#[must_use]
pub fn calculate(fs: f64, count: usize, k: f64) -> Speedup {
    #[expect(
        clippy::cast_precision_loss,
        reason = "at most 2^22 samples, exact in f64"
    )]
    let length = count as f64 / fs;
    let band = [k / length, k * fs / 2.0];
    let low = band[0].max(AUDIBLE[0]);
    let high = band[1].min(AUDIBLE[1]);
    Speedup {
        k,
        fs,
        duration: Seconds(length / k),
        band,
        audible: (low < high).then_some([low, high]),
    }
}

/// The k that maps `fs/2` to `hi` Hz.
#[must_use]
pub fn k_for_band(fs: f64, hi: f64) -> f64 {
    2.0 * hi / fs
}

/// Why a track cannot be audified.
fn refuse(code: &'static sonarithm_core::codes::CodeInfo, message: String) -> CompileError {
    CompileError::from(Diagnostic::new(code, message))
}

/// The numeric values of a column, `None` where missing or not finite.
fn numbers(table: &Table, name: &str) -> Option<Vec<Option<f64>>> {
    let column = table.column(name).ok()?;
    if let Some(a) = column.as_any().downcast_ref::<Float64Array>() {
        return Some(a.iter().map(|v| v.filter(|v| v.is_finite())).collect());
    }
    if let Some(a) = column.as_any().downcast_ref::<Int64Array>() {
        #[expect(clippy::cast_precision_loss, reason = "data values, rounding accepted")]
        return Some(a.iter().map(|v| v.map(|v| v as f64)).collect());
    }
    if let Some(a) = column.as_any().downcast_ref::<Int32Array>() {
        return Some(a.iter().map(|v| v.map(f64::from)).collect());
    }
    None
}

/// The time field's values per row, in seconds when it is temporal, else in its units.
fn times(table: &Table, name: &str) -> Option<Vec<Option<f64>>> {
    let column = table.column(name).ok()?;
    if let Some(a) = column.as_any().downcast_ref::<TimestampMillisecondArray>() {
        #[expect(
            clippy::cast_precision_loss,
            reason = "millisecond times, rounding accepted"
        )]
        return Some(a.iter().map(|v| v.map(|ms| ms as f64 / 1000.0)).collect());
    }
    numbers(table, name)
}

/// The series on a uniform grid over its time values: as many points as timed rows, from
/// the first time to the last, linearly interpolated (a missing value makes a gap of 0), and
/// the grid's rate. Rows need not be evenly spaced: a function table has extra rows at its
/// critical points (C03-T03). `None` without two distinct times.
fn uniform(values: &[Option<f64>], times: &[Option<f64>]) -> Option<(Vec<Option<f64>>, f64)> {
    let mut points: Vec<(f64, Option<f64>)> = times
        .iter()
        .zip(values)
        .filter_map(|(t, v)| t.map(|t| (t, *v)))
        .collect();
    points.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (first, last) = (points.first()?.0, points.last()?.0);
    let span = last - first;
    if !(span > 0.0 && span.is_finite()) || points.len() < 2 {
        return None;
    }
    let count = points.len();
    #[expect(clippy::cast_precision_loss, reason = "at most 2^22 samples")]
    let steps = (count - 1) as f64;
    let mut out = Vec::with_capacity(count);
    let mut j = 0;
    for i in 0..count {
        #[expect(clippy::cast_precision_loss, reason = "at most 2^22 samples")]
        let t = first + span * (i as f64 / steps);
        while j + 2 < points.len() && points.get(j + 1).is_some_and(|p| p.0 <= t) {
            j += 1;
        }
        let (Some(&(t0, v0)), Some(&(t1, v1))) = (points.get(j), points.get(j + 1)) else {
            out.push(None);
            continue;
        };
        out.push(match (v0, v1) {
            (Some(a), Some(b)) if t1 > t0 => {
                Some(a + (b - a) * ((t - t0) / (t1 - t0)).clamp(0.0, 1.0))
            }
            (Some(a), _) if (t - t0).abs() <= f64::EPSILON * t0.abs().max(1.0) => Some(a),
            _ => None,
        });
    }
    Some((out, steps / span))
}

/// The field with `role`, from the schema's field metadata (`snr.role`).
fn field_with_role(table: &Table, role: &str) -> Option<String> {
    table
        .schema()
        .fields()
        .iter()
        .find(|f| f.metadata().get("snr.role").map(String::as_str) == Some(role))
        .map(|f| f.name().clone())
}

/// The audifier encoder.
pub(crate) struct AudifyEncoder;

impl Encoder<DataBundle> for AudifyEncoder {
    fn encode(
        &self,
        track: &TrackRequest,
        data: &DataBundle,
        _annotations: &[Annotation],
        cx: &mut CompileContext,
    ) -> Result<ScoreTrack, CompileError> {
        let at = track.spec.pointer().clone();
        let spec: Value = track.spec.parse()?;
        let substrate: Option<Value> = track
            .substrate
            .as_ref()
            .map(SpecFragment::parse)
            .transpose()?;
        let table = data.primary().ok_or_else(|| {
            refuse(
                &codes::AUDIFY_NO_SERIES,
                "There is no data to audify.".to_owned(),
            )
        })?;
        let (time, series) = fields(table, substrate.as_ref()).ok_or_else(|| {
            refuse(
                &codes::AUDIFY_NO_SERIES,
                "The data has no numeric field to audify.".to_owned(),
            )
            .at_spec(at.clone())
        })?;
        let (values, fs) = series_grid(table, time.as_deref(), &series);
        let count = values.len();
        if count > Audify::MAX_SAMPLES {
            return Err(refuse(
                &codes::AUDIFY_TOO_MANY_SAMPLES,
                format!(
                    "The series has {count} samples, more than the {} an audification can hold.",
                    Audify::MAX_SAMPLES
                ),
            )
            .at_spec(at.clone()));
        }
        let k = speedup_of(&spec, fs, count, &at, cx)?;
        let peak = values.iter().flatten().fold(0.0_f64, |m, v| m.max(v.abs()));
        let scale = if peak > 0.0 { 1.0 / peak } else { 0.0 };
        #[expect(clippy::cast_possible_truncation, reason = "samples are f32")]
        let samples: Vec<f32> = values
            .iter()
            .map(|v| v.map_or(0.0, |v| (v * scale).clamp(-1.0, 1.0) as f32))
            .collect();
        let mut event = Audify::new(Seconds::ZERO, k * fs, samples);
        event.gain = loudness(&spec);
        event.src = Some(RowRange {
            table: table.id(),
            generation: table.generation(),
            start: 0,
            end: u64::try_from(count).unwrap_or(u64::MAX),
        });
        cx.charge_events(1)?;
        let mut out = ScoreTrack::new(
            TrackId::new(track.id.as_str()),
            TrackRole::Data,
            ENGINE,
            InstrumentRef("sine".to_owned()),
        );
        out.events.push(ScoreEvent::Audify(event));
        Ok(out)
    }
}

/// The time field (the substrate's, else the field with the x role) and the series to
/// play (the field with the y role, else the first numeric field that is not the time
/// field); `None` without a numeric series.
fn fields(table: &Table, substrate: Option<&Value>) -> Option<(Option<String>, String)> {
    let time = substrate
        .and_then(|s| s.pointer("/time/field"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| field_with_role(table, "x"));
    let series = field_with_role(table, "y")
        .filter(|name| numbers(table, name).is_some())
        .or_else(|| {
            table
                .schema()
                .fields()
                .iter()
                .map(|f| f.name().clone())
                .find(|name| {
                    Some(name) != time.as_ref()
                        && !name.starts_with("__snr_")
                        && numbers(table, name).is_some()
                })
        })?;
    Some((time, series))
}

/// The speed-up k of the track `spec` for `count` samples at `fs` (SDR-31): checked against
/// its bounds and the audible band, and reported as Info (`SNR-ENG-460`) with a warning when
/// part of the band is inaudible (`461`).
fn speedup_of(
    spec: &Value,
    fs: f64,
    count: usize,
    at: &sonarithm_core::SpecPointer,
    cx: &mut CompileContext,
) -> Result<f64, CompileError> {
    let audify = spec.get("audify");
    let k = choose_k(
        audify.and_then(|a| a.get("k")).and_then(Value::as_f64),
        audify
            .and_then(|a| a.pointer("/band/hz/1"))
            .and_then(Value::as_f64),
        fs,
    );
    if !(k.is_finite() && (K_MIN..=K_MAX).contains(&k)) {
        return Err(refuse(
            &codes::AUDIFY_K_OUT_OF_RANGE,
            format!("The speed-up k = {k} is outside {K_MIN} to {K_MAX}."),
        )
        .at_spec(at.key("audify")));
    }
    let speedup = calculate(fs, count.max(1), k);
    let band = speedup.band;
    let Some(audible) = speedup.audible else {
        return Err(refuse(
            &codes::AUDIFY_INAUDIBLE,
            format!(
                "At k = {k} the series sounds from {:.3} Hz to {:.3} Hz, wholly outside 20 Hz to 20 kHz.",
                band[0], band[1]
            ),
        )
        .at_spec(at.key("audify")));
    };
    cx.report(
        Diagnostic::new(
            &codes::AUDIFY_SPEEDUP,
            format!(
                "Audification: k = {k}, {:.3} s long (T/k), audible from {:.1} Hz to {:.1} Hz.",
                speedup.duration.0, audible[0], audible[1]
            ),
        )
        .at(at.clone()),
    );
    if band[0] < AUDIBLE[0] || band[1] > AUDIBLE[1] {
        cx.report(
            Diagnostic::new(
                &codes::AUDIFY_PART_INAUDIBLE,
                format!(
                    "Part of the band is inaudible: the series sounds from {:.3} Hz to {:.1} Hz, and only {:.1} Hz to {:.1} Hz can be heard.",
                    band[0], band[1], audible[0], audible[1]
                ),
            )
            .at(at.key("audify")),
        );
    }
    Ok(k)
}

/// The series on a uniform grid over the time field, whose rate `fs` is the series' own;
/// one sample per row (`fs` = 1) without a usable time field. The encoder and
/// [`readout`] share it, so the calculator and a compile always agree.
fn series_grid(table: &Table, time: Option<&str>, series: &str) -> (Vec<Option<f64>>, f64) {
    let raw = numbers(table, series).unwrap_or_default();
    time.and_then(|name| times(table, name))
        .and_then(|t| uniform(&raw, &t))
        .unwrap_or((raw, 1.0))
}

/// k from `audify.k`, else from the top of `audify.band` (`k = 2·hi/fs`), else the SDR-44
/// default top of 4 kHz (`INFER`).
fn choose_k(k: Option<f64>, top_hz: Option<f64>, fs: f64) -> f64 {
    match (k, top_hz) {
        (Some(k), _) => k,
        (None, Some(hi)) => k_for_band(fs, hi),
        (None, None) => k_for_band(fs, DEFAULT_TOP_HZ),
    }
}

/// What the audification calculator shows for one series before it compiles (SDR-31,
/// C03-T08): the speed-up with its duration and bands, the series it plays, and the codes
/// a compile would report.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Readout {
    /// k, the series' rate, the duration T/k, the band and its audible part.
    pub speedup: Speedup,
    /// The samples on the uniform grid.
    pub samples: usize,
    /// The field it plays.
    pub series: String,
    /// `SNR-ENG-461` when part of the band is inaudible, `462` when all of it is.
    pub codes: Vec<&'static str>,
}

/// The calculator for the series a track with the `audify` mark would play from `table`,
/// with the time field `time` (else the field with the x role), and `k` or the band's top
/// `top_hz` (else the SDR-44 default). The same grid and the same rule for k as the
/// encoder, so it gives the compile's numbers.
///
/// # Errors
///
/// The diagnostic a compile would refuse with: no numeric series (`SNR-ENG-465`), too many
/// samples (`464`) or k out of bounds (`463`).
pub fn readout(
    table: &Table,
    time: Option<&str>,
    k: Option<f64>,
    top_hz: Option<f64>,
) -> Result<Readout, Box<Diagnostic>> {
    let substrate = time.map(|t| serde_json::json!({ "time": { "field": t } }));
    let (time, series) = fields(table, substrate.as_ref()).ok_or_else(|| {
        Box::new(Diagnostic::new(
            &codes::AUDIFY_NO_SERIES,
            "The data has no numeric field to audify.",
        ))
    })?;
    let (values, fs) = series_grid(table, time.as_deref(), &series);
    let count = values.len();
    if count > Audify::MAX_SAMPLES {
        return Err(Box::new(Diagnostic::new(
            &codes::AUDIFY_TOO_MANY_SAMPLES,
            format!(
                "The series has {count} samples, more than the {} an audification can hold.",
                Audify::MAX_SAMPLES
            ),
        )));
    }
    let k = choose_k(k, top_hz, fs);
    if !(k.is_finite() && (K_MIN..=K_MAX).contains(&k)) {
        return Err(Box::new(Diagnostic::new(
            &codes::AUDIFY_K_OUT_OF_RANGE,
            format!("The speed-up k = {k} is outside {K_MIN} to {K_MAX}."),
        )));
    }
    let speedup = calculate(fs, count.max(1), k);
    let codes = match speedup.audible {
        None => vec![codes::AUDIFY_INAUDIBLE.code],
        Some(_) if speedup.band[0] < AUDIBLE[0] || speedup.band[1] > AUDIBLE[1] => {
            vec![codes::AUDIFY_PART_INAUDIBLE.code]
        }
        Some(_) => Vec::new(),
    };
    Ok(Readout {
        speedup,
        samples: count,
        series,
        codes,
    })
}

/// The `loudness` channel's constant value in dB, or SDR-23's default.
fn loudness(spec: &Value) -> Db {
    let value = spec.pointer("/channels/loudness/value");
    value
        .and_then(Value::as_f64)
        .or_else(|| value.and_then(|v| v.get("db")).and_then(Value::as_f64))
        .filter(|db| db.is_finite() && *db <= 0.0)
        .map_or(DEFAULT_LOUDNESS, Db)
}

/// Locates a refusal in the spec.
trait AtSpec {
    fn at_spec(self, at: sonarithm_core::SpecPointer) -> Self;
}

impl AtSpec for CompileError {
    fn at_spec(self, at: sonarithm_core::SpecPointer) -> Self {
        match self {
            Self::Diagnostic(diagnostic) => Self::Diagnostic(Box::new(diagnostic.at(at))),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests;

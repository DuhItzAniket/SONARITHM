// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Function cues (C03-T04, RFC-0007 §2; A11Y-28): the `curvasona.cues` context track that
//! plays a function's critical points as earcons, or speaks them, while the function plays.
//!
//! The track is added when the spec's `analysis.detect` contains `critical-points` and its
//! data is a function table; `analysis.cue` chooses earcons (the default), speech or none.
//! Each cue's time is the time of its feature's row in the first data track (CURVASONA adds
//! a row at every feature, so a glide has a breakpoint there), read back from the encoded
//! Score. Every cue has a text equivalent: its earcon's catalogue message (`earcon-zero`,
//! …), and the feature labels in [`cue_log`], which names every feature a merged cue
//! carries (A11Y-08, A11Y-24, ED-05).
//!
//! **Defaults and evidence.** That a zero, an asymptote, an undefined span and an off-screen
//! span get earcons of their own follows A11Y-28 (SHOULD). The sounds (render engine,
//! RFC-0007 §1), the 150 ms merge window and the priority order are proposals without
//! study evidence (INFER), experimental until the listening check (HP-21).

use std::collections::BTreeMap;

use arrow_array::{Array, Float64Array};
use serde_json::Value;
use sonarithm_core::engine::{
    Annotation, CompileContext, CompileError, ContextProvider, SpecFragment,
};
use sonarithm_core::score::{
    Earcon, EarconKind, InstrumentRef, Score, ScoreEvent, ScoreTrack, Speech, TrackRole,
};
use sonarithm_core::{RowRange, RowRef, Seconds, TrackId};
use sonarithm_data::DataBundle;

use crate::critical::FeatureKind;
use crate::{Budgets, CRITICAL_POINTS, ENGINE, FunctionTable};

/// The cue track's ID.
pub const CUE_TRACK: &str = "curvasona.cues";

/// Earcons closer than this merge into one (RFC-0007 Q3: the ED-02 earcon length; a
/// proposal, INFER).
pub const MERGE_WINDOW: Seconds = Seconds(0.150);

/// How cues are delivered (`analysis.cue`, RFC-0001 §8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Delivery {
    /// Earcons (the default).
    Earcon,
    /// Speech.
    Speech,
}

/// One cue: when, which earcon, and every feature it stands for.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Cue {
    /// When it plays.
    pub t: Seconds,
    /// The earcon: the highest-priority kind among its features (RFC-0007 amendment 1).
    pub kind: EarconKind,
    /// The text equivalent of each feature, in time order (its insight label).
    pub labels: Vec<String>,
    /// The row of each feature.
    pub rows: Vec<RowRef>,
}

/// The earcon a feature plays, if any: zeros, vertical asymptotes, undefined and off-screen
/// spans have their own (A11Y-28); extrema, inflections, holes and jumps play `E-LANDMARK`;
/// a horizontal asymptote is a limit, not a point on the glide, and plays nothing.
#[must_use]
pub const fn earcon_for(kind: FeatureKind) -> Option<EarconKind> {
    match kind {
        FeatureKind::Zero => Some(EarconKind::Zero),
        FeatureKind::VerticalAsymptote => Some(EarconKind::Asymptote),
        FeatureKind::Undefined => Some(EarconKind::Undefined),
        FeatureKind::Offscreen { .. } => Some(EarconKind::Offscreen),
        FeatureKind::HorizontalAsymptote { .. } => None,
        _ => Some(EarconKind::Landmark),
    }
}

/// The merge priority of an earcon (RFC-0007 amendment 1): asymptote, undefined, off-screen,
/// zero, then landmark. Higher wins.
#[must_use]
pub const fn priority(kind: EarconKind) -> u8 {
    match kind {
        EarconKind::Asymptote => 5,
        EarconKind::Undefined => 4,
        EarconKind::Offscreen => 3,
        EarconKind::Zero => 2,
        _ => 1,
    }
}

/// Every feature kind with a label, so annotations can be read back.
const KINDS: [FeatureKind; 12] = [
    FeatureKind::Zero,
    FeatureKind::Maximum,
    FeatureKind::Minimum,
    FeatureKind::Inflection,
    FeatureKind::VerticalAsymptote,
    FeatureKind::HorizontalAsymptote { right: false },
    FeatureKind::HorizontalAsymptote { right: true },
    FeatureKind::Hole,
    FeatureKind::Jump,
    FeatureKind::Undefined,
    FeatureKind::Offscreen { above: true },
    FeatureKind::Offscreen { above: false },
];

fn kind_of(label: &str) -> Option<FeatureKind> {
    KINDS.into_iter().find(|kind| kind.label() == label)
}

/// What the spec asks for: `None` when it asks for no cues.
fn delivery(spec: &Value) -> Option<Delivery> {
    let analysis = spec.get("analysis")?;
    let detects = analysis
        .get("detect")?
        .as_array()?
        .iter()
        .any(|d| d.as_str() == Some(CRITICAL_POINTS.as_str()));
    if !detects {
        return None;
    }
    match analysis.get("cue").and_then(Value::as_str) {
        None | Some("earcon") => Some(Delivery::Earcon),
        Some("speech") => Some(Delivery::Speech),
        _ => None,
    }
}

/// The y-window of off-screen cues (RFC-0007 Q5): the explicit `scale.domain` of the first
/// track's pitch channel, if the spec writes one.
fn window(spec: &Value) -> Option<[f64; 2]> {
    let domain = spec
        .get("tracks")?
        .get(0)?
        .get("channels")?
        .get("pitch")?
        .get("scale")?
        .get("domain")?
        .as_array()?;
    match domain.as_slice() {
        [low, high] => {
            let (low, high) = (low.as_f64()?, high.as_f64()?);
            (low < high).then_some([low, high])
        }
        _ => None,
    }
}

/// Off-screen spans as annotations of the `critical-points` analyzer, so they cue like the
/// others.
fn offscreen(data: &DataBundle, window: [f64; 2]) -> Vec<Annotation> {
    let Ok(function) = FunctionTable::read(data, &Budgets::DEFAULT) else {
        return Vec::new();
    };
    function
        .offscreen(window, &Budgets::DEFAULT)
        .into_iter()
        .map(|(_, above, row)| {
            Annotation::new(
                CRITICAL_POINTS,
                FeatureKind::Offscreen { above }.label(),
                RowRange {
                    table: function.table.id(),
                    generation: function.table.generation(),
                    start: row,
                    end: row + 1,
                },
            )
        })
        .collect()
}

/// Row → time, from the first data track's events that name their rows: a glide has one
/// breakpoint per row of its `src` range, a note names its row.
fn row_times(score: &Score) -> BTreeMap<u64, f64> {
    let mut times = BTreeMap::new();
    let Some(track) = score
        .tracks
        .iter()
        .find(|t| t.role == TrackRole::Data && t.events.iter().any(has_src))
    else {
        return times;
    };
    for event in &track.events {
        match event {
            ScoreEvent::Glide(glide) => {
                let Some(src) = glide.src else { continue };
                let rows = src.start..src.end;
                if usize::try_from(src.len()).ok() != Some(glide.curve.len()) {
                    continue;
                }
                for (row, point) in rows.zip(&glide.curve) {
                    times.insert(row, point.t.0);
                }
            }
            ScoreEvent::Note(note) => {
                if let Some(src) = note.src {
                    times.insert(src.row, note.t.0);
                }
            }
            _ => {}
        }
    }
    times
}

const fn has_src(event: &ScoreEvent) -> bool {
    match event {
        ScoreEvent::Glide(glide) => glide.src.is_some(),
        ScoreEvent::Note(note) => note.src.is_some(),
        _ => false,
    }
}

/// The time at `x`, interpolated between the timed rows around it (`(x, t)` sorted by x).
fn time_at(timed: &[(f64, f64)], x: f64) -> Option<f64> {
    let after = timed.partition_point(|&(px, _)| px < x);
    match (after.checked_sub(1).and_then(|i| timed.get(i)), timed.get(after)) {
        (_, Some(&(px, t))) if px.total_cmp(&x).is_eq() => Some(t),
        (Some(&(x0, t0)), Some(&(x1, t1))) if x1 > x0 => {
            Some(t0 + (t1 - t0) * (x - x0) / (x1 - x0))
        }
        (Some(&(_, t)), None) | (None, Some(&(_, t))) => Some(t),
        _ => None,
    }
}

/// The cues of a compiled function: one per feature annotation that has an earcon, timed
/// on the first data track, with features closer than [`MERGE_WINDOW`] merged.
#[must_use]
pub fn cues(score: &Score, data: &DataBundle, annotations: &[Annotation]) -> Vec<Cue> {
    let Some(table) = data.primary() else {
        return Vec::new();
    };
    let Ok(column) = table.column("x") else {
        return Vec::new();
    };
    let Some(xs) = column.as_any().downcast_ref::<Float64Array>() else {
        return Vec::new();
    };
    let row_x = |row: u64| {
        usize::try_from(row)
            .ok()
            .filter(|&r| r < xs.len() && xs.is_valid(r))
            .map(|r| xs.value(r))
    };
    let mut timed: Vec<(f64, f64)> = row_times(score)
        .into_iter()
        .filter_map(|(row, t)| row_x(row).map(|x| (x, t)))
        .collect();
    timed.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut singles: Vec<Cue> = Vec::new();
    for annotation in annotations.iter().filter(|a| a.source == CRITICAL_POINTS) {
        let Some(kind) = kind_of(&annotation.label).and_then(earcon_for) else {
            continue;
        };
        let range = annotation.range;
        let Some(t) = row_x(range.start).and_then(|x| time_at(&timed, x)) else {
            continue;
        };
        singles.push(Cue {
            t: Seconds(t),
            kind,
            labels: vec![annotation.label.clone()],
            rows: vec![RowRef {
                table: range.table,
                generation: range.generation,
                row: range.start,
            }],
        });
    }
    singles.sort_by(|a, b| a.t.0.total_cmp(&b.t.0));
    let mut merged: Vec<Cue> = Vec::new();
    for cue in singles {
        match merged.last_mut() {
            Some(last) if cue.t.0 - last.t.0 < MERGE_WINDOW.0 => {
                if priority(cue.kind) > priority(last.kind) {
                    last.kind = cue.kind;
                }
                last.labels.extend(cue.labels);
                last.rows.extend(cue.rows);
            }
            _ => merged.push(cue),
        }
    }
    merged
}

/// The text log of the cues (A11Y-24, RFC-0007 amendment 1): one line per cue, with its time
/// in seconds and every feature it stands for, such as `2.000 s: vertical asymptote,
/// local maximum`.
#[must_use]
pub fn cue_log(cues: &[Cue]) -> Vec<String> {
    cues.iter()
        .map(|cue| format!("{:.3} s: {}", cue.t.0, cue.labels.join(", ")))
        .collect()
}

/// The `curvasona.cues` context provider.
pub(crate) struct FunctionCues;

impl ContextProvider<DataBundle> for FunctionCues {
    fn context_tracks(
        &self,
        spec: &SpecFragment,
        score: &Score,
        data: &DataBundle,
        annotations: &[Annotation],
        _cx: &mut CompileContext,
    ) -> Result<Vec<ScoreTrack>, CompileError> {
        let spec: Value = spec.parse()?;
        let Some(delivery) = delivery(&spec) else {
            return Ok(Vec::new());
        };
        let mut all = annotations.to_vec();
        if let Some(window) = window(&spec) {
            all.extend(offscreen(data, window));
        }
        let found = cues(score, data, &all);
        if found.is_empty() {
            return Ok(Vec::new());
        }
        let (role, instrument) = match delivery {
            Delivery::Earcon => (TrackRole::Earcon, "sine"),
            Delivery::Speech => (TrackRole::Speech, "speech"),
        };
        let mut track = ScoreTrack::new(
            TrackId::new(CUE_TRACK),
            role,
            ENGINE,
            InstrumentRef(instrument.to_owned()),
        );
        for cue in found {
            let event = match delivery {
                Delivery::Earcon => {
                    let mut earcon = Earcon::new(cue.t, cue.kind);
                    // The highest-priority feature's row; `cue_log` names them all.
                    earcon.src = cue
                        .kind
                        .eq(&EarconKind::Landmark)
                        .then(|| cue.rows.first().copied())
                        .unwrap_or_else(|| first_of_kind(&cue));
                    ScoreEvent::Earcon(earcon)
                }
                Delivery::Speech => {
                    let mut speech = Speech::new(cue.t, cue.labels.join(", "));
                    speech.src = cue.rows.first().copied();
                    ScoreEvent::Speech(speech)
                }
            };
            track.events.push(event);
        }
        Ok(vec![track])
    }
}

/// The row of the first feature in `cue` whose own earcon is the cue's kind.
fn first_of_kind(cue: &Cue) -> Option<RowRef> {
    cue.labels
        .iter()
        .zip(&cue.rows)
        .find(|(label, _)| kind_of(label).and_then(earcon_for) == Some(cue.kind))
        .map(|(_, row)| *row)
        .or_else(|| cue.rows.first().copied())
}

#[cfg(test)]
mod tests;

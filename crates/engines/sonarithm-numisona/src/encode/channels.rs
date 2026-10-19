// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The channels besides pitch: duration, taps, loudness, pan, brightness and timbre, and the
//! redundant encodings of the pitch field (A11Y-13).

use sonarithm_core::diagnostic::{Diagnostic, SpecPointer};
use sonarithm_core::engine::CompileError;
use sonarithm_spec::track::{Channel, ChannelName, Channels, Polarity as SpecPolarity, Redundant};
use sonarithm_spec::values::{ChannelRange, ChannelValue, UnitRange, UnitValue};

use super::columns::Column;
use super::{
    DEFAULT_DURATION_S, DEFAULT_LOUDNESS_DB, PROPOSED_DURATION_RANGE_S, PROPOSED_LOUDNESS_RANGE_DB,
    PitchChannel, Run, TAPS_RANGE, TIMBRE_PALETTE, Timeline, Values,
};
use crate::codes;
use crate::scale::{Category, categories_in_order_of_appearance};

/// Where a channel's default range comes from, for the diagnostic that reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RangeOrigin {
    /// The channel's bounds in RFC-0001 §6.
    Bounds,
    /// A sound-design proposal recorded in F-082, not yet evidence-backed.
    Proposal,
}

impl Run<'_> {
    /// A numeric channel: a constant, or a field through its scale into its range.
    pub(super) fn scalar(
        &mut self,
        name: ChannelName,
        channel: Option<&Channel>,
        timeline: &Timeline,
        default: f64,
        range: [f64; 2],
        origin: Option<RangeOrigin>,
    ) -> Result<Values<f64>, CompileError> {
        let Some(channel) = channel else {
            return Ok(Values::Constant(default));
        };
        if let Some(value) = channel.value.as_ref().and_then(base_value) {
            return Ok(Values::Constant(value));
        }
        let Some(field) = channel.field.clone() else {
            return Ok(Values::Constant(default));
        };
        let [low, high] = if let Some(written) = channel.range.and_then(base_range) {
            written
        } else {
            {
                let why = match origin.unwrap_or(RangeOrigin::Bounds) {
                    RangeOrigin::Bounds => "the channel's full range (RFC-0001 §6)",
                    RangeOrigin::Proposal => {
                        "a sound-design proposal, not yet evidence-backed (F-082)"
                    }
                };
                let message = format!(
                    "`{}` has no range, so field `{field}` maps onto {} to {}, {why}.",
                    name.key(),
                    range[0],
                    range[1]
                );
                self.report(
                    &codes::CHANNEL_RANGE,
                    &message,
                    self.channel_at(name).key("range"),
                );
                range
            }
        };
        let flip = channel.polarity == Some(SpecPolarity::Negative);
        let positions = self.positions(name, channel, &field, timeline)?;
        Ok(Values::PerRow(
            positions
                .iter()
                .map(|p| {
                    p.map(|p| {
                        let u = if flip { 1.0 - p.u } else { p.u };
                        low + u * (high - low)
                    })
                })
                .collect(),
        ))
    }

    /// Loudness: constant −18 dB unless mapped (SDR-23); a mapping warns.
    pub(super) fn loudness(
        &mut self,
        channel: Option<&Channel>,
        timeline: &Timeline,
    ) -> Result<Values<f64>, CompileError> {
        let at = self.channel_at(ChannelName::Loudness);
        match channel {
            None => {
                let message = format!(
                    "loudness is constant at {DEFAULT_LOUDNESS_DB} dB, so pitch alone carries the data (SDR-23)."
                );
                self.report(&codes::DEFAULT_LOUDNESS, &message, at);
            }
            Some(channel) if channel.field.is_some() => {
                let message = format!(
                    "NUMISONA: track `{}`: loudness carries field `{}`. Loudness distorts pitch judgments and must never be the primary channel for precise values (SDR-23).",
                    self.request.id,
                    channel.field.as_deref().unwrap_or_default()
                );
                self.diagnostics
                    .push(Diagnostic::new(&codes::LOUDNESS_CARRIES_DATA, message).at(at));
            }
            Some(_) => {}
        }
        self.scalar(
            ChannelName::Loudness,
            channel,
            timeline,
            DEFAULT_LOUDNESS_DB,
            PROPOSED_LOUDNESS_RANGE_DB,
            Some(RangeOrigin::Proposal),
        )
    }

    /// Note duration: the channel, else the redundant encoding of the pitch field
    /// (A11Y-13), else 0.25 s.
    pub(super) fn duration(
        &mut self,
        channel: Option<&Channel>,
        timeline: &Timeline,
        pitch: &PitchChannel,
    ) -> Result<Values<f64>, CompileError> {
        let redundant = self.track.redundant.contains(&Redundant::Duration);
        if channel.is_some() {
            if redundant {
                self.redundant_overridden(Redundant::Duration);
            }
            return self.scalar(
                ChannelName::Duration,
                channel,
                timeline,
                DEFAULT_DURATION_S,
                PROPOSED_DURATION_RANGE_S,
                Some(RangeOrigin::Proposal),
            );
        }
        if redundant && let Some(positions) = &pitch.positions {
            let [low, high] = PROPOSED_DURATION_RANGE_S;
            let message = format!(
                "duration also encodes the pitch field, from {low} s to {high} s (A11Y-13; the range is a sound-design proposal, F-082)."
            );
            self.report(
                &codes::REDUNDANT_ENCODING,
                &message,
                self.at.key("redundant"),
            );
            return Ok(Values::PerRow(
                positions
                    .iter()
                    .map(|u| u.map(|u| low + u * (high - low)))
                    .collect(),
            ));
        }
        if redundant {
            self.redundant_without_pitch(Redundant::Duration);
        }
        let message =
            format!("notes last {DEFAULT_DURATION_S} s, the default duration (RFC-0001 §6).");
        self.report(
            &codes::DEFAULT_DURATION,
            &message,
            self.channel_at(ChannelName::Duration),
        );
        Ok(Values::Constant(DEFAULT_DURATION_S))
    }

    /// Tap count: the channel, else the redundant encoding of the pitch field (A11Y-13),
    /// else one tap; whole numbers from 1 to 8.
    pub(super) fn taps(
        &mut self,
        channel: Option<&Channel>,
        timeline: &Timeline,
        pitch: &PitchChannel,
    ) -> Result<Values<f64>, CompileError> {
        let redundant = self.track.redundant.contains(&Redundant::Taps);
        let [low, high] = TAPS_RANGE;
        let values = if channel.is_some() {
            if redundant {
                self.redundant_overridden(Redundant::Taps);
            }
            self.scalar(
                ChannelName::Taps,
                channel,
                timeline,
                1.0,
                TAPS_RANGE,
                Some(RangeOrigin::Bounds),
            )?
        } else if redundant && let Some(positions) = &pitch.positions {
            let message = format!(
                "the tap count also encodes the pitch field, from {low} to {high} taps (A11Y-13)."
            );
            self.report(
                &codes::REDUNDANT_ENCODING,
                &message,
                self.at.key("redundant"),
            );
            Values::PerRow(
                positions
                    .iter()
                    .map(|u| u.map(|u| low + u * (high - low)))
                    .collect(),
            )
        } else {
            if redundant {
                self.redundant_without_pitch(Redundant::Taps);
            }
            Values::Constant(1.0)
        };
        let whole = |taps: f64| taps.round().clamp(low, high);
        Ok(match values {
            Values::Constant(taps) => Values::Constant(whole(taps)),
            Values::PerRow(taps) => {
                Values::PerRow(taps.into_iter().map(|t| t.map(whole)).collect())
            }
        })
    }

    fn redundant_overridden(&mut self, kind: Redundant) {
        let name = redundant_name(kind);
        let message = format!(
            "`redundant` asks for {name}, but the track maps `{name}` itself, which wins (RFC-0001 Q21)."
        );
        self.report(
            &codes::REDUNDANT_IGNORED,
            &message,
            self.at.key("redundant"),
        );
    }

    fn redundant_without_pitch(&mut self, kind: Redundant) {
        let name = redundant_name(kind);
        let message = format!(
            "`redundant` asks for {name}, which encodes the pitch field, but the track maps no field to pitch (A11Y-13)."
        );
        self.report(
            &codes::REDUNDANT_IGNORED,
            &message,
            self.at.key("redundant"),
        );
    }

    /// Timbre: a preset, or one preset per category of a field (SDR-25). Categories take
    /// the track instrument first, then [`TIMBRE_PALETTE`] in order.
    pub(super) fn timbre(
        &mut self,
        channel: Option<&Channel>,
        timeline: &Timeline,
        instrument: &str,
    ) -> Result<Values<String>, CompileError> {
        let Some(channel) = channel else {
            return Ok(Values::Constant(instrument.to_owned()));
        };
        if let Some(ChannelValue::Name(preset)) = &channel.value {
            return Ok(Values::Constant(preset.clone()));
        }
        let Some(field) = channel.field.clone() else {
            return Ok(Values::Constant(instrument.to_owned()));
        };
        let at = self.channel_at(ChannelName::Timbre);
        let labels: Vec<Option<Category>> = match self.column(&field, &at.key("field"))? {
            Column::Categories { values, .. } => values,
            Column::Numbers(values) => values
                .into_iter()
                .map(|v| (!v.is_nan()).then_some(Category::Number(v)))
                .collect(),
        };
        let order = categories_in_order_of_appearance(
            timeline.rows.iter().filter_map(|&row| labels[row].clone()),
        );
        let mut palette = vec![instrument.to_owned()];
        palette.extend(
            TIMBRE_PALETTE
                .iter()
                .filter(|preset| **preset != instrument)
                .map(|preset| (*preset).to_owned()),
        );
        let preset_of = |category: &Category| {
            order
                .iter()
                .position(|c| c == category)
                .map(|i| palette[i % palette.len()].clone())
        };
        let assignment: Vec<String> = order
            .iter()
            .filter_map(|c| preset_of(c).map(|p| format!("{} → {p}", category_label(c))))
            .collect();
        let message = format!(
            "field `{field}` is categorical, so each category gets its own timbre (SDR-25): {}.",
            assignment.join(", ")
        );
        self.report(&codes::TIMBRE_CATEGORIES, &message, at.clone());
        if order.len() > palette.len() {
            let message = format!(
                "NUMISONA: track `{}`: field `{field}` has {} categories but only {} timbres exist, so timbres repeat. Timbre must not be the only carrier of a category (A11Y-14); split the track or speak the categories.",
                self.request.id,
                order.len(),
                palette.len()
            );
            self.diagnostics
                .push(Diagnostic::new(&codes::TIMBRE_REPEATS, message).at(at));
        }
        Ok(Values::PerRow(
            timeline
                .rows
                .iter()
                .map(|&row| labels[row].as_ref().and_then(preset_of))
                .collect(),
        ))
    }

    /// A11Y-14 and SDR-27: pan must not be the only carrier of a field, and in Analysis mode
    /// it should not repeat the pitch field.
    pub(super) fn spatial_checks(&mut self, channels: &Channels) {
        let Some(pan_field) = channels.pan.as_ref().and_then(|c| c.field.clone()) else {
            return;
        };
        let at = self.channel_at(ChannelName::Pan);
        let others = [
            &channels.pitch,
            &channels.duration,
            &channels.taps,
            &channels.loudness,
            &channels.timbre,
            &channels.brightness,
        ];
        let carried = others
            .iter()
            .any(|c| c.as_ref().and_then(|c| c.field.as_deref()) == Some(pan_field.as_str()));
        if !carried {
            let message = format!(
                "NUMISONA: track `{}`: field `{pan_field}` is carried by pan alone, which mono output, NVDA sound split and one-sided hearing lose (A11Y-14). Map it to another channel as well.",
                self.request.id
            );
            self.diagnostics
                .push(Diagnostic::new(&codes::PAN_SOLE_CARRIER, message).at(at.clone()));
        }
        let pitch_field = channels.pitch.as_ref().and_then(|c| c.field.as_deref());
        if self.request.mode == sonarithm_core::score::Mode::Analysis
            && pitch_field == Some(pan_field.as_str())
        {
            let message = format!(
                "NUMISONA: track `{}`: field `{pan_field}` drives both pitch and pan. Analysis mode avoids redundant spatial encodings, which hurt the reproduction of simple functions (SDR-27).",
                self.request.id
            );
            self.diagnostics
                .push(Diagnostic::new(&codes::REDUNDANT_SPATIAL, message).at(at));
        }
    }

    /// Glides have no notes, so tone-only channels and redundant encodings do not apply.
    pub(super) fn ignore_tone_only(&mut self, channels: &Channels) {
        let mut ignored = Vec::new();
        if channels.duration.is_some() {
            ignored.push("`duration`");
        }
        if channels.taps.is_some() {
            ignored.push("`taps`");
        }
        if !self.track.redundant.is_empty() {
            ignored.push("`redundant`");
        }
        if channels.timbre.as_ref().is_some_and(|c| c.field.is_some()) {
            ignored.push("a timbre field (a glide has one timbre)");
        }
        if ignored.is_empty() {
            return;
        }
        let message = format!(
            "NUMISONA: track `{}`: a glide is one continuous contour, so {} do not apply. Use `mark: \"tone\"` for discrete notes (A11Y-13).",
            self.request.id,
            ignored.join(", ")
        );
        self.diagnostics
            .push(Diagnostic::new(&codes::CHANNEL_NOT_RENDERED, message).at(self.at.key("mark")));
    }

    /// Reports a diagnostic about this track.
    pub(super) fn report(
        &mut self,
        code: &sonarithm_core::codes::CodeInfo,
        message: &str,
        at: SpecPointer,
    ) {
        self.diagnostics.push(
            Diagnostic::new(
                code,
                format!("NUMISONA: track `{}`: {message}", self.request.id),
            )
            .at(at),
        );
    }
}

fn redundant_name(kind: Redundant) -> &'static str {
    match kind {
        Redundant::Taps => "taps",
        _ => "duration",
    }
}

fn category_label(category: &Category) -> String {
    match category {
        Category::Number(n) => n.to_string(),
        Category::Text(text) => text.clone(),
    }
}

/// A constant in the channel's base unit: seconds, dB, or a plain number.
fn base_value(value: &ChannelValue) -> Option<f64> {
    match value {
        ChannelValue::Number(x) => Some(*x),
        ChannelValue::Unit(UnitValue::S(s)) => Some(*s),
        ChannelValue::Unit(UnitValue::Ms(ms)) => Some(ms / 1000.0),
        ChannelValue::Unit(UnitValue::Db(db)) => Some(*db),
        _ => None,
    }
}

/// A range in the channel's base unit.
fn base_range(range: ChannelRange) -> Option<[f64; 2]> {
    match range {
        ChannelRange::Plain(r) | ChannelRange::Unit(UnitRange::S(r) | UnitRange::Db(r)) => Some(r),
        ChannelRange::Unit(UnitRange::Ms([a, b])) => Some([a / 1000.0, b / 1000.0]),
        ChannelRange::Unit(_) => None,
    }
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Checks of tracks and channels (RFC-0001 §6): 020–026, 030–032, 042, 070, 071.

use sonarithm_core::diagnostic::{Diagnostic, Diagnostics, SpecPointer};
use sonarithm_core::units::{Cents, Midi};

use super::{FLAGGED_CONCEPTS, FieldCatalog, positive};
use crate::codes;
use crate::data::FieldType;
use crate::spec::Spec;
use crate::track::{
    Audify, Channel, ChannelName, Domain, Mark, Polarity, Scale, ScaleType, TrackSpec,
};
use crate::values::{ChannelValue, Unit};

/// The channels each mark accepts (04 §5.3; `loudness` replaces `gain`, RFC-0001 Q2).
fn accepts(mark: Mark, channel: ChannelName) -> bool {
    use ChannelName::{Brightness, Density, Duration, Loudness, Pan, Pitch, Rate, Taps, Timbre};
    let allowed: &[ChannelName] = match mark {
        Mark::Tone => &[Pitch, Duration, Loudness, Pan, Timbre, Brightness, Taps],
        Mark::Glide => &[Pitch, Loudness, Pan, Timbre, Brightness, Rate],
        Mark::Grain => &[Density, Pitch, Duration, Pan, Brightness],
        Mark::Audify => &[Loudness],
        Mark::Sample => &[Loudness, Pan, Rate],
        Mark::Auto => &[
            Pitch, Duration, Loudness, Pan, Timbre, Brightness, Taps, Rate,
        ],
        Mark::Model => &[],
    };
    allowed.contains(&channel)
}

/// What a channel's `value` and `range` may be.
#[expect(
    clippy::struct_excessive_bools,
    reason = "a flat table of independent flags reads best here"
)]
struct Rule {
    /// Unit keys accepted.
    units: &'static [Unit],
    /// Whether a plain number is accepted.
    plain: bool,
    /// Whether a name is accepted.
    name: bool,
    /// Inclusive bounds, in the channel's base unit (Hz, s, dB or plain).
    bounds: (f64, f64),
    /// Whether the lower bound is exclusive.
    open_low: bool,
    /// Integer values only.
    integer: bool,
}

const fn rule(units: &'static [Unit], plain: bool, low: f64, high: f64) -> Rule {
    Rule {
        units,
        plain,
        name: false,
        bounds: (low, high),
        open_low: false,
        integer: false,
    }
}

fn channel_rule(channel: ChannelName) -> Rule {
    match channel {
        ChannelName::Pitch => rule(&[Unit::Hz, Unit::Midi, Unit::Cents], false, 20.0, 20_000.0),
        ChannelName::Duration => Rule {
            open_low: true,
            ..rule(&[Unit::S, Unit::Ms], false, 0.0, f64::MAX)
        },
        ChannelName::Taps => Rule {
            integer: true,
            ..rule(&[], true, 1.0, 8.0)
        },
        ChannelName::Loudness => rule(&[Unit::Db], false, f64::MIN, 0.0),
        ChannelName::Pan => rule(&[], true, -1.0, 1.0),
        ChannelName::Brightness => rule(&[], true, 0.0, 1.0),
        ChannelName::Rate => Rule {
            open_low: true,
            ..rule(&[Unit::Hz], true, 0.0, f64::MAX)
        },
        ChannelName::Density => Rule {
            open_low: true,
            ..rule(&[], true, 0.0, f64::MAX)
        },
        ChannelName::Timbre => Rule {
            name: true,
            ..rule(&[], false, 0.0, 0.0)
        },
    }
}

/// A value in the channel's base unit: Hz for pitch, seconds for durations.
fn base(unit: Option<Unit>, x: f64) -> f64 {
    match unit {
        Some(Unit::Midi) => Midi(x).to_hz().0,
        Some(Unit::Cents) => Cents(x).to_hz().0,
        Some(Unit::Ms) => x / 1000.0,
        _ => x,
    }
}

/// The unit as people write it in prose.
fn unit_name(unit: Option<Unit>) -> &'static str {
    match unit {
        None => "",
        Some(Unit::Hz) => "Hz",
        Some(Unit::Midi) => "MIDI",
        Some(Unit::Cents) => "cents",
        Some(Unit::S) => "s",
        Some(Unit::Ms) => "ms",
        Some(Unit::Db) => "dB",
    }
}

/// Checks one track.
pub(super) fn track(spec: &Spec, track: &TrackSpec, at: &SpecPointer, out: &mut Diagnostics) {
    for (name, channel) in track.channels.present() {
        let at = at.key("channels").key(name.key());
        if !accepts(track.mark, name) {
            out.push(
                Diagnostic::new(
                    &codes::CHANNEL_NOT_FOR_MARK,
                    format!(
                        "The `{}` mark does not take a `{}` channel (04 §5.3).",
                        mark_key(track.mark),
                        name.key()
                    ),
                )
                .at(at.clone()),
            );
            continue;
        }
        channel_checks(spec, name, channel, &at, out);
    }
    if let Some(audify) = &track.audify {
        let at = at.key("audify");
        if track.mark != Mark::Audify {
            out.push(
                Diagnostic::new(
                    &codes::CHANNEL_NOT_FOR_MARK,
                    "`audify` settings apply only to tracks with the `audify` mark.",
                )
                .at(at),
            );
        } else if let Audify::K(k) = audify
            && !positive(*k)
        {
            out.push(
                Diagnostic::new(
                    &codes::OUT_OF_BOUNDS,
                    "The audification factor `k` must be positive.",
                )
                .at(at.key("k")),
            );
        }
    }
}

fn channel_checks(
    spec: &Spec,
    name: ChannelName,
    channel: &Channel,
    at: &SpecPointer,
    out: &mut Diagnostics,
) {
    let rule = channel_rule(name);
    match (&channel.field, &channel.value) {
        (Some(_), Some(_)) => out.push(
            Diagnostic::new(
                &codes::FIELD_AND_VALUE,
                "A channel has either `field` (to map data) or `value` (a constant), not both.",
            )
            .at(at.clone()),
        ),
        (None, None) => out.push(
            Diagnostic::new(
                &codes::NO_FIELD_OR_VALUE,
                "Add `field` to map data to this channel, or `value` for a constant.",
            )
            .at(at.clone()),
        ),
        (None, Some(value)) => {
            value_checks(name, &rule, value, &at.key("value"), out);
            let options = [
                ("scale", channel.scale.is_some()),
                ("range", channel.range.is_some()),
                ("space", channel.space.is_some()),
                ("polarity", channel.polarity.is_some()),
            ];
            for (key, present) in options {
                if present {
                    out.push(
                        Diagnostic::new(
                            &codes::OPTION_ON_CONSTANT,
                            format!("`{key}` applies only to a channel mapped from a `field`."),
                        )
                        .at(at.key(key)),
                    );
                }
            }
        }
        (Some(field), None) => {
            if let Some(scale) = &channel.scale {
                scale_checks(name, scale, &at.key("scale"), out);
            }
            if name == ChannelName::Pitch
                && matches!(channel.polarity, None | Some(Polarity::Auto))
                && let Some(concept) = spec
                    .data
                    .fields
                    .get(field)
                    .and_then(|f| f.concept.as_deref())
                && (FLAGGED_CONCEPTS.contains(&concept) || concept.starts_with("graph."))
            {
                out.push(
                    Diagnostic::new(
                        &codes::FLAGGED_CONCEPT,
                        format!(
                            "Listeners disagree on the pitch direction for `{concept}`. Set \"polarity\" to \"positive\" or \"negative\"."
                        ),
                    )
                    .at(at.clone()),
                );
            }
        }
    }
    if channel.space.is_some() && name != ChannelName::Pitch {
        out.push(
            Diagnostic::new(
                &codes::SPACE_NOT_PITCH,
                "`space` applies only to the pitch channel.",
            )
            .at(at.key("space")),
        );
    }
    if let Some(range) = channel.range {
        let (unit, [low, high]) = range.parts();
        let at = at.key("range");
        if !unit_accepted(&rule, unit) {
            out.push(unit_diagnostic(name, &rule, at));
        } else if !in_bounds(&rule, unit, low) || !in_bounds(&rule, unit, high) {
            out.push(bounds_diagnostic(name, at));
        } else if low >= high {
            out.push(
                Diagnostic::new(
                    &codes::REVERSED_RANGE,
                    format!(
                        "{} range [{low}, {high}] {} is reversed. Use \"polarity\": \"negative\" to invert the mapping instead.",
                        capitalize(name.key()),
                        unit_name(unit)
                    ),
                )
                .at(at),
            );
        }
    }
}

fn unit_accepted(rule: &Rule, unit: Option<Unit>) -> bool {
    match unit {
        Some(unit) => rule.units.contains(&unit),
        None => rule.plain,
    }
}

fn in_bounds(rule: &Rule, unit: Option<Unit>, x: f64) -> bool {
    let value = base(unit, x);
    let (low, high) = rule.bounds;
    let low_ok = if rule.open_low {
        value > low
    } else {
        value >= low
    };
    value.is_finite() && low_ok && value <= high && (!rule.integer || value.fract() == 0.0)
}

fn value_checks(
    name: ChannelName,
    rule: &Rule,
    value: &ChannelValue,
    at: &SpecPointer,
    out: &mut Diagnostics,
) {
    let (unit, x) = match value {
        ChannelValue::Name(_) if rule.name => return,
        ChannelValue::Name(_) => {
            out.push(unit_diagnostic(name, rule, at.clone()));
            return;
        }
        ChannelValue::Number(x) => (None, *x),
        ChannelValue::Unit(unit_value) => {
            let (unit, x) = unit_value.parts();
            (Some(unit), x)
        }
    };
    if !unit_accepted(rule, unit) {
        out.push(unit_diagnostic(name, rule, at.clone()));
    } else if !in_bounds(rule, unit, x) {
        out.push(bounds_diagnostic(name, at.clone()));
    }
}

fn unit_diagnostic(name: ChannelName, rule: &Rule, at: SpecPointer) -> Diagnostic {
    let mut accepted: Vec<String> = rule
        .units
        .iter()
        .map(|u| format!("`{}`", u.key()))
        .collect();
    if rule.plain {
        accepted.push("a plain number".to_owned());
    }
    if rule.name {
        accepted.push("a preset name".to_owned());
    }
    Diagnostic::new(
        &codes::UNIT_NOT_ACCEPTED,
        format!(
            "The `{}` channel takes {}.",
            name.key(),
            accepted.join(" or ")
        ),
    )
    .at(at)
}

fn bounds_diagnostic(name: ChannelName, at: SpecPointer) -> Diagnostic {
    let bounds = match name {
        ChannelName::Pitch => "from 20 to 20000 Hz",
        ChannelName::Duration => "greater than 0 s",
        ChannelName::Taps => "a whole number from 1 to 8",
        ChannelName::Loudness => "at most 0 dB",
        ChannelName::Pan => "from -1 to 1",
        ChannelName::Brightness => "from 0 to 1",
        _ => "greater than 0",
    };
    Diagnostic::new(
        &codes::OUT_OF_BOUNDS,
        format!("The `{}` channel takes values {bounds}.", name.key()),
    )
    .at(at)
}

fn scale_checks(name: ChannelName, scale: &Scale, at: &SpecPointer, out: &mut Diagnostics) {
    let scale_type = scale.scale_type.unwrap_or(ScaleType::Linear);
    let needs_exponent = matches!(scale_type, ScaleType::Pow | ScaleType::Psychophysical);
    let problem = if needs_exponent && !scale.exponent.is_some_and(positive) {
        Some("This scale needs a positive `exponent`.")
    } else if !needs_exponent && scale.exponent.is_some() {
        Some("`exponent` applies only to `pow` and `psychophysical` scales.")
    } else if scale_type == ScaleType::Psychophysical && name != ChannelName::Pitch {
        Some("The `psychophysical` scale applies only to pitch (SDR-11).")
    } else if scale_type == ScaleType::Psychophysical
        && !scale.reference.is_some_and(|r| {
            r.value.is_finite() && r.value != 0.0 && (20.0..=20_000.0).contains(&r.hz)
        })
    {
        Some(
            "A `psychophysical` scale needs a `reference` with a non-zero `value` and `hz` from 20 to 20000.",
        )
    } else if scale_type != ScaleType::Psychophysical && scale.reference.is_some() {
        Some("`reference` applies only to `psychophysical` scales.")
    } else {
        None
    };
    if let Some(problem) = problem {
        out.push(Diagnostic::new(&codes::INVALID_SCALE, problem).at(at.clone()));
    }
    if let Some(Domain::Range([min, max])) = scale.domain {
        let valid = min.is_finite() && max.is_finite() && min < max;
        let positive_for_log = scale_type != ScaleType::Log || min > 0.0;
        if !valid || !positive_for_log {
            out.push(
                Diagnostic::new(
                    &codes::INVALID_DOMAIN,
                    "Write the domain as [min, max] with min below max; a `log` scale needs a positive domain.",
                )
                .at(at.key("domain")),
            );
        }
    }
}

/// Field names and types (070, 071) against the catalog.
pub(super) fn fields(spec: &Spec, catalog: &FieldCatalog, out: &mut Diagnostics) {
    let mut catalog = catalog.clone();
    for transform in &spec.transform {
        for output in transform_outputs(transform) {
            catalog.insert(output, None);
        }
    }
    let root = SpecPointer::root();
    let unknown = |name: &str, at: SpecPointer, out: &mut Diagnostics| {
        if catalog.get(name).is_none() {
            out.push(
                Diagnostic::new(
                    &codes::UNKNOWN_FIELD,
                    format!("The data has no field \"{name}\". Name a field of the data, or declare it in `data.fields`."),
                )
                .at(at),
            );
            None
        } else {
            catalog.get(name).flatten()
        }
    };
    if let Some(field) = spec
        .substrate
        .as_ref()
        .and_then(|s| s.time.as_ref())
        .and_then(|t| t.field.as_ref())
    {
        unknown(field, root.key("substrate").key("time").key("field"), out);
    }
    for (index, track) in spec.tracks.iter().enumerate() {
        for (name, channel) in track.channels.present() {
            let Some(field) = &channel.field else {
                continue;
            };
            let at = root
                .key("tracks")
                .index(index)
                .key("channels")
                .key(name.key())
                .key("field");
            let field_type = unknown(field, at.clone(), out);
            let compatible = match (name, field_type) {
                (_, None) => true,
                (ChannelName::Timbre, Some(t)) => {
                    matches!(t, FieldType::Nominal | FieldType::Ordinal)
                }
                (_, Some(t)) => t != FieldType::Nominal,
            };
            if !compatible {
                out.push(
                    Diagnostic::new(
                        &codes::INCOMPATIBLE_FIELD,
                        format!(
                            "Field \"{field}\" does not fit the `{}` channel: map nominal fields to timbre and quantitative fields to pitch, duration or rate (SDR-25).",
                            name.key()
                        ),
                    )
                    .at(at),
                );
            }
        }
    }
}

fn transform_outputs(transform: &crate::data::Transform) -> Vec<String> {
    use crate::data::Transform;
    match transform {
        Transform::Aggregate { ops, .. } => ops.iter().map(|op| op.output.clone()).collect(),
        Transform::Derive { output, .. } => vec![output.clone()],
        Transform::Bin { output, .. }
        | Transform::Smooth { output, .. }
        | Transform::Normalize { output, .. } => output.iter().cloned().collect(),
        _ => Vec::new(),
    }
}

fn mark_key(mark: Mark) -> &'static str {
    match mark {
        Mark::Tone => "tone",
        Mark::Glide => "glide",
        Mark::Grain => "grain",
        Mark::Audify => "audify",
        Mark::Sample => "sample",
        Mark::Model => "model",
        Mark::Auto => "auto",
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

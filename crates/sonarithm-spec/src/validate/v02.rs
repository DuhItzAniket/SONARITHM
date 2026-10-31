// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Checks of the sections spec v0.2 adds (RFC-0004): the version gate (a v0.2 key in a
//! document that declares `"0.1"` is unknown, `004`), `template`, `instruments`, `lint`
//! (`045`, `046`), chart `calibration` (`054`) and `corrections` (`055`, `056`), the
//! `music.quantize` vocabularies (`043`, `044`), and the v0.4 `experimental` block (`047`,
//! RFC-0006).

use std::collections::BTreeSet;

use serde::de::DeserializeOwned;
use sonarithm_core::diagnostic::{Diagnostic, Diagnostics, SpecPointer};
use sonarithm_core::score::SpecVersion;

use super::{path_check, valid_id};
use crate::codes;
use crate::data::Source;
use crate::json::JsonValue;
use crate::spec::Spec;
use crate::v02::{
    AnchorValue, AxisKind, Calibration, CalibrationAxis, Corrections, CustomScale, CustomTuning,
    Edit, Instrument, LINT_RULES, PointValue, TUNINGS, parse_scale_name, tonic_class, valid_cents,
};
use sonarithm_core::codes::CodeInfo;

fn root() -> SpecPointer {
    SpecPointer::root()
}

/// Whether `path` is relative and stays under the spec's folder (RFC-0005 §7): not empty,
/// no drive letter, URL scheme or leading separator, no `..` segment.
fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(':')
        && !path.starts_with(['/', '\\'])
        && !path.split(['/', '\\']).any(|part| part == "..")
}

/// Every v0.2 check, in document order.
pub(super) fn check(spec: &Spec, ids: &BTreeSet<String>, out: &mut Diagnostics) {
    version_gate(spec, out);
    template(spec, out);
    instruments(spec, out);
    lint(spec, ids, out);
    experimental(spec, out);
    if let Source::Chart {
        calibration,
        corrections,
        ..
    } = &spec.data.source
    {
        let at = root().key("data").key("source");
        if let Some(value) = calibration {
            self::calibration(value, &at.key("calibration"), out);
        }
        if let Some(value) = corrections {
            self::corrections(value, &at.key("corrections"), spec.spec_version, out);
        }
    }
    quantize(spec, out);
}

/// A v0.2 key in a document that declares an older version is unknown (RFC-0004 §8).
fn version_gate(spec: &Spec, out: &mut Diagnostics) {
    if spec.spec_version >= SpecVersion::V0_2 {
        return;
    }
    let mut found: Vec<(&str, SpecPointer)> = Vec::new();
    if spec.template.is_some() {
        found.push(("template", root().key("template")));
    }
    if !spec.instruments.is_empty() {
        found.push(("instruments", root().key("instruments")));
    }
    if spec.lint.is_some() {
        found.push(("lint", root().key("lint")));
    }
    if matches!(
        &spec.data.source,
        Source::Chart {
            corrections: Some(_),
            ..
        }
    ) {
        found.push((
            "corrections",
            root().key("data").key("source").key("corrections"),
        ));
    }
    for (key, at) in found {
        out.push(
            Diagnostic::new(
                &codes::UNKNOWN_KEY,
                format!(
                    "`{key}` is a spec v0.2 key, but this spec declares \"{}\". Set \"specVersion\": \"0.2\" to use it.",
                    spec.spec_version
                ),
            )
            .at(at),
        );
    }
}

/// `experimental` (RFC-0006 §2): a spec 0.4 key, so unknown (`004`) in older documents;
/// a switch this version does not know is a warning (`047`) and has no effect. A known
/// switch that is not a boolean fails to parse (`002`).
fn experimental(spec: &Spec, out: &mut Diagnostics) {
    let Some(block) = &spec.experimental else {
        return;
    };
    let at = root().key("experimental");
    if spec.spec_version < SpecVersion::V0_4 {
        out.push(
            Diagnostic::new(
                &codes::UNKNOWN_KEY,
                format!(
                    "`experimental` is a spec v0.4 key, but this spec declares \"{}\". Set \"specVersion\": \"0.4\" to use it.",
                    spec.spec_version
                ),
            )
            .at(at),
        );
        return;
    }
    for key in block.other.keys() {
        out.push(
            Diagnostic::new(
                &codes::UNKNOWN_EXPERIMENT,
                format!(
                    "Experimental switch `{key}` is not known to this SONARITHM; it is ignored."
                ),
            )
            .at(at.key(key)),
        );
    }
}

fn template(spec: &Spec, out: &mut Diagnostics) {
    let Some(template) = &spec.template else {
        return;
    };
    let at = root().key("template");
    let kebab = !template.id.is_empty()
        && template.id.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
        && template.id.starts_with(|c: char| c.is_ascii_lowercase());
    if !kebab {
        out.push(
            Diagnostic::new(
                &codes::INVALID_VALUE,
                format!(
                    "Template ID \"{}\" is invalid. Template IDs are lowercase words joined by `-`, such as \"bar-discrete\".",
                    template.id
                ),
            )
            .at(at.key("id")),
        );
    }
    for (slot, field) in &template.bind {
        if slot.trim().is_empty() || field.trim().is_empty() {
            out.push(
                Diagnostic::new(
                    &codes::INVALID_VALUE,
                    "A template binding names a slot and a field; neither may be empty.",
                )
                .at(at.key("bind").key(slot)),
            );
        }
    }
}

fn instruments(spec: &Spec, out: &mut Diagnostics) {
    let at = root().key("instruments");
    for (name, instrument) in &spec.instruments {
        let here = at.key(name);
        if !valid_id(name) {
            out.push(
                Diagnostic::new(
                    &codes::INVALID_VALUE,
                    format!(
                        "Instrument name \"{name}\" is invalid. Start with a letter and use only letters, digits, `_` and `-`, up to 64 characters."
                    ),
                )
                .at(here.clone()),
            );
        }
        match instrument {
            Instrument::Soundfont {
                path,
                bank,
                program,
            } => {
                if path.trim().is_empty() {
                    out.push(
                        Diagnostic::new(&codes::INVALID_VALUE, "The SoundFont `path` is empty.")
                            .at(here.key("path")),
                    );
                }
                path_check(path, here.key("path"), out);
                if bank.is_some_and(|b| b > 16_383) {
                    out.push(
                        Diagnostic::new(&codes::OUT_OF_BOUNDS, "`bank` must be 0 … 16383.")
                            .at(here.key("bank")),
                    );
                }
                if program.is_some_and(|p| p > 127) {
                    out.push(
                        Diagnostic::new(&codes::OUT_OF_BOUNDS, "`program` must be 0 … 127.")
                            .at(here.key("program")),
                    );
                }
            }
        }
    }
}

fn lint(spec: &Spec, ids: &BTreeSet<String>, out: &mut Diagnostics) {
    let Some(lint) = &spec.lint else {
        return;
    };
    let at = root().key("lint");
    let unknown = |rule: &str, at: SpecPointer, out: &mut Diagnostics| {
        if !LINT_RULES.contains(&rule) {
            out.push(
                Diagnostic::new(
                    &codes::UNKNOWN_LINT_RULE,
                    format!(
                        "The lint does not implement rule `{rule}`, so this setting has no effect. Implemented rules: {}.",
                        LINT_RULES.join(", ")
                    ),
                )
                .at(at),
            );
        }
    };
    for rule in lint.rules.iter().flatten().map(|(rule, _)| rule) {
        unknown(rule, at.key("rules").key(rule), out);
    }
    for (index, suppression) in lint.suppress.iter().flatten().enumerate() {
        let here = at.key("suppress").index(index);
        unknown(&suppression.rule, here.key("rule"), out);
        if suppression.reason.trim().is_empty() {
            out.push(
                Diagnostic::new(
                    &codes::SUPPRESSION_WITHOUT_REASON,
                    format!(
                        "The suppression of `{}` has no reason. Say why the rule does not apply.",
                        suppression.rule
                    ),
                )
                .at(here.key("reason")),
            );
        }
        if let Some(track) = &suppression.track
            && !ids.contains(track)
        {
            out.push(
                Diagnostic::new(
                    &codes::UNKNOWN_TRACK,
                    format!(
                        "The suppression names track \"{track}\", which the spec does not have."
                    ),
                )
                .at(here.key("track")),
            );
        }
    }
}

/// Reads `value` as `T`; a shape error becomes `code` at the failing pointer under `at`.
fn typed<T: DeserializeOwned>(
    value: &JsonValue,
    at: &SpecPointer,
    code: &CodeInfo,
    out: &mut Diagnostics,
) -> Option<T> {
    let json = crate::v02::to_json(value);
    match serde_path_to_error::deserialize::<_, T>(json) {
        Ok(typed) => Some(typed),
        Err(err) => {
            let pointer = crate::parse::extend(at.clone(), err.path().iter());
            let message = crate::parse::strip_position(&err.inner().to_string()).to_owned();
            out.push(Diagnostic::new(code, format!("{}.", capitalize(&message))).at(pointer));
            None
        }
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

fn calibration(value: &JsonValue, at: &SpecPointer, out: &mut Diagnostics) {
    let Some(calibration) = typed::<Calibration>(value, at, &codes::INVALID_CALIBRATION, out)
    else {
        return;
    };
    if let Some(area) = calibration.plot_area {
        let ok = area.min.iter().chain(&area.max).all(|v| v.is_finite())
            && area.min[0] < area.max[0]
            && area.min[1] < area.max[1];
        if !ok {
            out.push(
                Diagnostic::new(
                    &codes::INVALID_CALIBRATION,
                    "`plotArea` needs `min` below `max` on both coordinates.",
                )
                .at(at.key("plotArea")),
            );
        }
    }
    let axes = at.key("axes");
    for (name, axis) in [("x", &calibration.axes.x), ("y", &calibration.axes.y)] {
        if let Some(problem) = axis_problem(axis) {
            out.push(
                Diagnostic::new(
                    &codes::INVALID_CALIBRATION,
                    format!("The {name} axis calibration is invalid: {problem}."),
                )
                .at(axes.key(name)),
            );
        }
    }
}

/// What is wrong with an axis, if anything (RFC-0004 §5).
fn axis_problem(axis: &CalibrationAxis) -> Option<String> {
    if axis.anchors.len() < 2 {
        return Some("it needs at least two anchors".to_owned());
    }
    if axis
        .anchors
        .iter()
        .any(|a| !a.at[0].is_finite() || !a.at[1].is_finite())
    {
        return Some("an anchor's point is not two finite numbers".to_owned());
    }
    for (i, a) in axis.anchors.iter().enumerate() {
        for b in axis.anchors.iter().skip(i + 1) {
            if a.at
                .iter()
                .zip(&b.at)
                .all(|(p, q)| p.to_bits() == q.to_bits())
            {
                return Some(format!(
                    "two anchors share the point [{}, {}]",
                    a.at[0], a.at[1]
                ));
            }
            if a.value == b.value {
                return Some("two anchors have the same value".to_owned());
            }
        }
    }
    for anchor in &axis.anchors {
        let ok = match (axis.kind, &anchor.value) {
            (AxisKind::Linear, AnchorValue::Number(v)) => v.is_finite(),
            (AxisKind::Log, AnchorValue::Number(v)) => v.is_finite() && *v > 0.0,
            (AxisKind::Date, AnchorValue::Text(t)) => iso_date(t),
            (AxisKind::Category, AnchorValue::Text(t)) => !t.trim().is_empty(),
            _ => false,
        };
        if !ok {
            return Some(match axis.kind {
                AxisKind::Log => "log anchors take numbers above zero".to_owned(),
                AxisKind::Date => {
                    "date anchors take ISO 8601 dates such as \"2024-03-15\"".to_owned()
                }
                AxisKind::Category => "category anchors take labels".to_owned(),
                _ => "linear anchors take numbers".to_owned(),
            });
        }
    }
    None
}

/// `YYYY`, `YYYY-MM`, `YYYY-MM-DD`, or a date-time with seconds optional and an offset
/// (`Z` or `±hh:mm`).
fn iso_date(text: &str) -> bool {
    let digits = |s: &str, n: usize| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
    let in_range =
        |s: &str, lo: u32, hi: u32| s.parse::<u32>().is_ok_and(|v| (lo..=hi).contains(&v));
    let (date, time) = match text.split_once('T') {
        Some((d, t)) => (d, Some(t)),
        None => (text, None),
    };
    let parts: Vec<&str> = date.split('-').collect();
    let date_ok = match parts.as_slice() {
        [y] => digits(y, 4),
        [y, m] => digits(y, 4) && digits(m, 2) && in_range(m, 1, 12),
        [y, m, d] => {
            digits(y, 4) && digits(m, 2) && in_range(m, 1, 12) && digits(d, 2) && in_range(d, 1, 31)
        }
        _ => false,
    };
    let Some(time) = time else {
        return date_ok;
    };
    if parts.len() != 3 {
        return false;
    }
    let (clock, offset_ok) = if let Some(clock) = time.strip_suffix('Z') {
        (clock, true)
    } else if let Some(i) = time.rfind(['+', '-']) {
        let offset = time.get(i + 1..).unwrap_or("");
        let ok = offset.split_once(':').is_some_and(|(h, m)| {
            digits(h, 2) && digits(m, 2) && in_range(h, 0, 23) && in_range(m, 0, 59)
        });
        (time.get(..i).unwrap_or(""), ok)
    } else {
        return false;
    };
    let fields: Vec<&str> = clock.split(':').collect();
    let clock_ok = match fields.as_slice() {
        [h, m] => digits(h, 2) && digits(m, 2) && in_range(h, 0, 23) && in_range(m, 0, 59),
        [h, m, s] => {
            let whole = s.split('.').next().unwrap_or("");
            digits(h, 2)
                && digits(m, 2)
                && in_range(h, 0, 23)
                && in_range(m, 0, 59)
                && digits(whole, 2)
                && in_range(whole, 0, 60)
        }
        _ => false,
    };
    date_ok && clock_ok && offset_ok
}

fn corrections(value: &JsonValue, at: &SpecPointer, version: SpecVersion, out: &mut Diagnostics) {
    if let JsonValue::Object(object) = value
        && let Some(path) = object.get("path")
    {
        if version < SpecVersion::V0_3 {
            out.push(
                Diagnostic::new(
                    &codes::CORRECTION_FILE_RESERVED,
                    format!(
                        "`corrections.path`, a correction file (RFC-0005), is spec 0.3; this spec declares \"{version}\". Set \"specVersion\": \"0.3\" or write the edits inline."
                    ),
                )
                .at(at.key("path")),
            );
            return;
        }
        if object.len() > 1 {
            out.push(
                Diagnostic::new(
                    &codes::INVALID_CORRECTION,
                    "`corrections` is either {\"path\": …} or {\"edits\": […]}, not both.",
                )
                .at(at.clone()),
            );
        }
        let safe = match path {
            JsonValue::String(path) => safe_relative(path),
            _ => false,
        };
        if !safe {
            out.push(
                Diagnostic::new(
                    &codes::CORRECTION_PATH_UNSAFE,
                    "Name the correction file by a path relative to the spec, without `..`, a drive letter or a URL.",
                )
                .at(at.key("path")),
            );
        }
        return;
    }
    let Some(corrections) = typed::<Corrections>(value, at, &codes::INVALID_CORRECTION, out) else {
        return;
    };
    for (index, edit) in corrections.edits.iter().enumerate() {
        let here = at.key("edits").index(index);
        let problem = match edit {
            Edit::Add { series, to } => blank(series).or_else(|| {
                (to.x.is_none() || to.y.is_none()).then_some("`to` needs both `x` and `y`")
            }),
            Edit::Move {
                series, was, to, ..
            } => blank(series).or_else(|| guard(was)).or_else(|| {
                (to.x.is_none() && to.y.is_none()).then_some("`to` needs `x`, `y` or both")
            }),
            Edit::Delete { series, was, .. } => blank(series).or_else(|| guard(was)),
            Edit::Retype {
                series, was, to, ..
            } => blank(series)
                .or_else(|| guard(was))
                .or_else(|| (to.series.trim().is_empty()).then_some("`to.series` is empty"))
                .or_else(|| {
                    (to.series == *series)
                        .then_some("`to.series` is the series the point is already in")
                }),
        };
        if let Some(problem) = problem {
            out.push(
                Diagnostic::new(
                    &codes::INVALID_CORRECTION,
                    format!("The correction is invalid: {problem}."),
                )
                .at(here),
            );
        }
    }
}

fn blank(series: &str) -> Option<&'static str> {
    series.trim().is_empty().then_some("`series` is empty")
}

fn guard(was: &PointValue) -> Option<&'static str> {
    (was.x.is_none() && was.y.is_none()).then_some("`was` needs the extracted `x` or `y`")
}

fn quantize(spec: &Spec, out: &mut Diagnostics) {
    let Some(quantize) = spec.music.as_ref().and_then(|m| m.quantize.as_ref()) else {
        return;
    };
    let at = root().key("music").key("quantize");
    let unknown_scale = |message: String, out: &mut Diagnostics| {
        out.push(Diagnostic::new(&codes::UNKNOWN_SCALE_NAME, message).at(at.key("scale")));
    };
    let custom = match &quantize.scale {
        JsonValue::String(name) => {
            if parse_scale_name(name).is_none() {
                unknown_scale(
                    format!(
                        "\"{name}\" is not a scale this SONARITHM knows: write a tonic and a mode, such as \"D minor pentatonic\"."
                    ),
                    out,
                );
            }
            false
        }
        JsonValue::Object(_) => {
            if let Some(custom) = typed::<CustomScale>(
                &quantize.scale,
                &at.key("scale"),
                &codes::UNKNOWN_SCALE_NAME,
                out,
            ) && (tonic_class(&custom.tonic).is_none() || !valid_cents(&custom.cents))
            {
                unknown_scale(
                    "A custom scale needs a tonic A to G (with an optional # or b) and cents that start at 0, rise strictly and stay below 1200.".to_owned(),
                    out,
                );
            }
            true
        }
        _ => {
            unknown_scale(
                "The scale is a name such as \"C major\" or an object with `tonic` and `cents`."
                    .to_owned(),
                out,
            );
            false
        }
    };
    let Some(tuning) = &quantize.tuning else {
        return;
    };
    let invalid = |message: &str, out: &mut Diagnostics| {
        out.push(Diagnostic::new(&codes::INVALID_TUNING, message).at(at.key("tuning")));
    };
    if custom {
        invalid(
            "A custom scale carries its own cents, so it takes no tuning; remove `tuning`.",
            out,
        );
        return;
    }
    match tuning {
        JsonValue::String(name) if TUNINGS.contains(&name.as_str()) => {}
        JsonValue::String(name) if name == "just" => invalid(
            "`just` is reserved until AXONOTE documents its ratio table (RFC-0004 Q13); use \"12-TET\" or custom cents.",
            out,
        ),
        JsonValue::Object(_) => {
            if let Some(custom) =
                typed::<CustomTuning>(tuning, &at.key("tuning"), &codes::INVALID_TUNING, out)
                && (custom.cents.len() != 12 || !valid_cents(&custom.cents))
            {
                invalid(
                    "A custom tuning gives the twelve pitch classes in cents: twelve values from 0, rising strictly, below 1200.",
                    out,
                );
            }
        }
        _ => invalid(
            "The tuning is \"12-TET\", \"just\" or {\"cents\": [...]}.",
            out,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::iso_date;

    #[test]
    fn iso_dates_read_as_the_rfc_lists_them() {
        for ok in [
            "2024",
            "2024-03",
            "2024-03-15",
            "2024-03-15T10:30Z",
            "2024-03-15T10:30:15+05:30",
            "2024-03-15T10:30:15.25-08:00",
        ] {
            assert!(iso_date(ok), "{ok}");
        }
        for bad in [
            "24",
            "2024-13",
            "2024-03-32",
            "2024-03-15T10:30",
            "March 2024",
            "2024-03T10:00Z",
            "2024-03-15T25:00Z",
        ] {
            assert!(!iso_date(bad), "{bad}");
        }
    }
}

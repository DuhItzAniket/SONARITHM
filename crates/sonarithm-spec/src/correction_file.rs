// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The chart correction file, `sonarithm-corrections` 1.0 (RFC-0005, ADR-0026): types, a
//! strict reader and a canonical writer.
//!
//! The reader accepts a file whole or refuses it with diagnostics (§8): JSON with no
//! duplicate keys and at most 32 levels of nesting, the format and a version it reads,
//! known keys only (a newer minor version's unknown keys are dropped with a warning), the
//! limits of §8, and edits whose IDs and references are consistent (§4). It reads no files
//! and decodes no image; the host does both.

use std::collections::BTreeSet;
use std::fmt;

use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sonarithm_core::{Diagnostic, Diagnostics, ProvenanceClass, SpecPointer};

use crate::codes;
use crate::parse::{extend, one};
use crate::v02::{Calibration, PlotArea, PointValue, SeriesTarget};

/// The `format` of every correction file.
pub const FORMAT: &str = "sonarithm-corrections";
/// The version this crate writes, and the newest major version it reads.
pub const VERSION: &str = "1.0";
const MAJOR: u64 = 1;
const MINOR: u64 = 0;

/// The largest file read, in bytes (§8).
pub const MAX_BYTES: usize = 16 * 1024 * 1024;
/// The most edits in one file (§8).
pub const MAX_EDITS: usize = 100_000;
/// The longest string other than a note or the image, in bytes (§8).
pub const MAX_STRING: usize = 1024;
/// The longest note, in characters (§4).
pub const MAX_NOTE: usize = 1000;
/// The longest image, in base64 bytes (§6).
pub const MAX_IMAGE: usize = 4 * 1024 * 1024;
/// The deepest JSON nesting read (§8).
pub const MAX_DEPTH: usize = 32;

/// A correction file (§2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CorrectionFile {
    /// Always [`FORMAT`].
    pub format: String,
    /// `"<major>.<minor>"`; the writer writes [`VERSION`].
    pub version: String,
    /// The chart the corrections were made on (§3).
    pub chart: ChartRef,
    /// The calibration the edits were made against (RFC-0004 §5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration: Option<Calibration>,
    /// The edits, in the order they apply (§4).
    pub edits: Vec<FileEdit>,
    /// The chart's pixels, only when the person asked for them (§6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<ChartImage>,
}

impl CorrectionFile {
    /// An empty correction set for `chart`, at [`VERSION`].
    #[must_use]
    pub fn new(chart: ChartRef) -> Self {
        Self {
            format: FORMAT.to_owned(),
            version: VERSION.to_owned(),
            chart,
            calibration: None,
            edits: Vec::new(),
            image: None,
        }
    }
}

/// The chart a correction file is bound to, by content (§3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ChartRef {
    /// SHA-256 of the chart file's bytes, 64 lowercase hex digits.
    pub sha256: String,
    /// The chart's file name only, for display.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The PDF page, from 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// The extractor that produced the points the edits refer to.
    pub extractor: String,
    /// Its version.
    pub extractor_version: String,
}

impl ChartRef {
    /// A binding to the chart with this SHA-256, read by `extractor` at `extractor_version`.
    #[must_use]
    pub fn new(
        sha256: impl Into<String>,
        extractor: impl Into<String>,
        extractor_version: impl Into<String>,
    ) -> Self {
        Self {
            sha256: sha256.into(),
            name: None,
            page: None,
            extractor: extractor.into(),
            extractor_version: extractor_version.into(),
        }
    }
}

/// Which point an edit refers to: an extracted point by its index in its series, or a point
/// an earlier `add` created, as `+e<n>` (§4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PointRef {
    /// The 0-based index in the extractor's order for the series.
    Index(u32),
    /// `+e<n>`: the point the edit `e<n>` added.
    Added(String),
}

impl PointRef {
    /// The ID of the `add` this refers to, without the `+`.
    #[must_use]
    pub fn added(&self) -> Option<&str> {
        match self {
            Self::Added(text) => text.strip_prefix('+'),
            Self::Index(_) => None,
        }
    }
}

/// A point's provenance when an edit was made: the history a `user_edited` value keeps (§4).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Before {
    /// The provenance class.
    pub class: ProvenanceClass,
    /// The confidence, when the class has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

impl Before {
    /// The provenance `class` with `confidence`.
    #[must_use]
    pub const fn new(class: ProvenanceClass, confidence: Option<f64>) -> Self {
        Self { class, confidence }
    }
}

/// One edit in a correction file: RFC-0004 §6's edit with an `id`, the point's provenance
/// `before` it, and an optional `note` (§4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
#[non_exhaustive]
pub enum FileEdit {
    /// A point the extractor missed.
    #[serde(rename_all = "camelCase")]
    Add {
        /// `e<n>`.
        id: String,
        /// The series name as extracted.
        series: String,
        /// The new point, in data units.
        to: PointValue,
        /// The person's words.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    /// A point at the wrong value.
    #[serde(rename_all = "camelCase")]
    Move {
        /// `e<n>`.
        id: String,
        /// The series name as extracted.
        series: String,
        /// The point.
        point: PointRef,
        /// Its value when the edit was made, a guard.
        was: PointValue,
        /// The new value.
        to: PointValue,
        /// Its provenance when the edit was made.
        before: Before,
        /// The person's words.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    /// A point that is not data.
    #[serde(rename_all = "camelCase")]
    Delete {
        /// `e<n>`.
        id: String,
        /// The series name as extracted.
        series: String,
        /// The point.
        point: PointRef,
        /// Its value when the edit was made, a guard.
        was: PointValue,
        /// Its provenance when the edit was made.
        before: Before,
        /// The person's words.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    /// A point given to the wrong series.
    #[serde(rename_all = "camelCase")]
    Retype {
        /// `e<n>`.
        id: String,
        /// The series name as extracted.
        series: String,
        /// The point.
        point: PointRef,
        /// Its value when the edit was made, a guard.
        was: PointValue,
        /// The series it belongs to.
        to: SeriesTarget,
        /// Its provenance when the edit was made.
        before: Before,
        /// The person's words.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
}

impl FileEdit {
    /// The edit's ID.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Add { id, .. }
            | Self::Move { id, .. }
            | Self::Delete { id, .. }
            | Self::Retype { id, .. } => id,
        }
    }

    /// The point the edit refers to; `None` for an `add`.
    #[must_use]
    pub const fn point(&self) -> Option<&PointRef> {
        match self {
            Self::Add { .. } => None,
            Self::Move { point, .. } | Self::Delete { point, .. } | Self::Retype { point, .. } => {
                Some(point)
            }
        }
    }

    /// The person's note.
    #[must_use]
    pub fn note(&self) -> Option<&str> {
        match self {
            Self::Add { note, .. }
            | Self::Move { note, .. }
            | Self::Delete { note, .. }
            | Self::Retype { note, .. } => note.as_deref(),
        }
    }

    /// The series name as extracted.
    #[must_use]
    pub fn series(&self) -> &str {
        match self {
            Self::Add { series, .. }
            | Self::Move { series, .. }
            | Self::Delete { series, .. }
            | Self::Retype { series, .. } => series,
        }
    }
}

/// The chart's pixels (§6).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ChartImage {
    /// `image/png`.
    pub media: String,
    /// The PNG, base64 (RFC 4648 §4).
    pub data: String,
    /// Where the pixels sit in the chart's coordinates.
    pub region: PlotArea,
}

/// Writes `file` canonically (§1): keys in declaration order, two-space indentation, `\n`
/// line ends and a final newline. The same file gives the same bytes.
///
/// # Panics
///
/// Never: the types hold only finite numbers and strings. (A non-finite `f64` serializes
/// as `null`, which reading then refuses.)
#[must_use]
pub fn write(file: &CorrectionFile) -> String {
    let mut text = serde_json::to_string_pretty(file).unwrap_or_default();
    text.push('\n');
    text
}

/// Reads a correction file (§8). On success, also returns warnings (`SNR-SPEC-094` for a
/// newer minor version).
///
/// # Errors
///
/// `SNR-SPEC-090` for something that is not a correction file or comes from a newer major
/// version, `091` for a shape error, `092` over a limit; never a partial result.
pub fn read(bytes: &[u8]) -> Result<(CorrectionFile, Diagnostics), Diagnostics> {
    if bytes.len() > MAX_BYTES {
        return Err(one(limit(format!(
            "The file is {} bytes; a correction file holds at most {MAX_BYTES}.",
            bytes.len()
        ))));
    }
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = Strict { depth: 0 }
        .deserialize(&mut deserializer)
        .and_then(|value| deserializer.end().map(|()| value))
        .map_err(|err| {
            one(shape(
                SpecPointer::root(),
                format!("Not valid JSON: {err}."),
            ))
        })?;
    let mut warnings = Diagnostics::new();
    let strict = check_header(&value, &mut warnings)?;
    let mut value = value;
    let mut problems = Diagnostics::new();
    prune(&mut value, strict, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let file: CorrectionFile = serde_path_to_error::deserialize(value).map_err(|err| {
        let at = extend(SpecPointer::root(), err.path().iter());
        one(shape(at, format!("{}.", err.inner())))
    })?;
    check(&file, &mut problems);
    if problems.is_empty() {
        Ok((file, warnings))
    } else {
        Err(problems)
    }
}

fn shape(at: SpecPointer, message: String) -> Diagnostic {
    Diagnostic::new(&codes::CORRECTION_FILE_SHAPE, message).at(at)
}

fn limit(message: String) -> Diagnostic {
    Diagnostic::new(&codes::CORRECTION_FILE_LIMIT, message)
}

/// Checks `format` and `version`; returns whether unknown keys are errors (a version this
/// crate knows) or are dropped (a newer minor version, with a warning).
fn check_header(value: &Value, warnings: &mut Diagnostics) -> Result<bool, Diagnostics> {
    let unreadable =
        |message: String| one(Diagnostic::new(&codes::CORRECTION_FILE_UNREADABLE, message));
    let Some(object) = value.as_object() else {
        return Err(unreadable("A correction file is a JSON object.".to_owned()));
    };
    if object.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err(unreadable(format!(
            "This is not a correction file: `format` is not \"{FORMAT}\"."
        )));
    }
    let version = object
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let parsed = version
        .split_once('.')
        .and_then(|(major, minor)| Some((major.parse::<u64>().ok()?, minor.parse::<u64>().ok()?)));
    match parsed {
        Some((MAJOR, minor)) if minor > MINOR => {
            warnings.push(Diagnostic::new(
                &codes::CORRECTION_FILE_NEWER,
                format!("The file is version {version}; keys version {VERSION} does not know were ignored."),
            ));
            Ok(false)
        }
        Some((MAJOR, _)) => Ok(true),
        Some((major, _)) if major > MAJOR => Err(unreadable(format!(
            "The file is version {version}, made by a newer SONARITHM; this one reads 1.x."
        ))),
        _ => Err(unreadable(format!(
            "`version` must be \"<major>.<minor>\", such as \"{VERSION}\"."
        ))),
    }
}

/// The keys each object of the format may have, by where it is (§2–§6). Calibration is not
/// listed: RFC-0004 §5's type checks its own keys.
fn known(path: &[&str], op: Option<&str>) -> Option<&'static [&'static str]> {
    Some(match (path, op) {
        ([], _) => &[
            "format",
            "version",
            "chart",
            "calibration",
            "edits",
            "image",
        ],
        (["chart"], _) => &["sha256", "name", "page", "extractor", "extractorVersion"],
        (["edits"], Some("add")) => &["op", "id", "series", "to", "note"],
        (["edits"], Some("move" | "retype")) => {
            &["op", "id", "series", "point", "was", "to", "before", "note"]
        }
        (["edits"], Some("delete")) => &["op", "id", "series", "point", "was", "before", "note"],
        (["edits", "was" | "to"], _) => &["x", "y", "series"],
        (["edits", "before"], _) => &["class", "confidence"],
        (["image"], _) => &["media", "data", "region"],
        (["image", "region"], _) => &["min", "max"],
        _ => return None,
    })
}

/// Reports (`strict`) or drops unknown keys.
fn prune(value: &mut Value, strict: bool, problems: &mut Diagnostics) {
    fn walk(
        value: &mut Value,
        path: &mut Vec<&'static str>,
        at: &SpecPointer,
        op: Option<&str>,
        strict: bool,
        problems: &mut Diagnostics,
    ) {
        let Some(object) = value.as_object_mut() else {
            return;
        };
        let Some(keys) = known(path, op) else { return };
        let unknown: Vec<String> = object
            .keys()
            .filter(|k| !keys.contains(&k.as_str()))
            .cloned()
            .collect();
        for key in unknown {
            if strict {
                problems.push(shape(
                    at.key(&key),
                    format!("`{key}` is not a key of a correction file here."),
                ));
            } else {
                object.remove(&key);
            }
        }
        for (key, child) in object.iter_mut() {
            let Some(&name) = keys.iter().find(|k| **k == key) else {
                continue;
            };
            let at = at.key(key);
            match (path.as_slice(), name) {
                ([], "edits") => {
                    if let Some(edits) = child.as_array_mut() {
                        for (index, edit) in edits.iter_mut().enumerate() {
                            let op = edit.get("op").and_then(Value::as_str).map(str::to_owned);
                            path.push("edits");
                            walk(
                                edit,
                                path,
                                &at.index(index),
                                op.as_deref(),
                                strict,
                                problems,
                            );
                            path.pop();
                        }
                    }
                }
                ([], "chart" | "image")
                | (["edits"], "was" | "to" | "before")
                | (["image"], "region") => {
                    path.push(name);
                    walk(child, path, &at, None, strict, problems);
                    path.pop();
                }
                _ => {}
            }
        }
    }
    walk(
        value,
        &mut Vec::new(),
        &SpecPointer::root(),
        None,
        strict,
        problems,
    );
}

fn is_edit_id(id: &str) -> bool {
    id.strip_prefix('e').is_some_and(|n| {
        !n.is_empty() && !n.starts_with('0') && n.bytes().all(|b| b.is_ascii_digit())
    })
}

/// The checks the types cannot make (§3, §4, §6, §8).
fn check(file: &CorrectionFile, problems: &mut Diagnostics) {
    check_chart(&file.chart, problems);
    check_edits(&file.edits, problems);
    if let Some(image) = &file.image {
        check_image(image, problems);
    }
}

/// §3, §5: the binding.
fn check_chart(file_chart: &ChartRef, problems: &mut Diagnostics) {
    let chart = SpecPointer::root().key("chart");
    if file_chart.sha256.len() != 64
        || !file_chart
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        problems.push(shape(
            chart.key("sha256"),
            "`sha256` is 64 lowercase hexadecimal digits.".to_owned(),
        ));
    }
    if let Some(name) = &file_chart.name
        && (name.is_empty()
            || name.chars().count() > 255
            || name
                .chars()
                .any(|c| matches!(c, '/' | '\\' | ':') || c.is_control()))
    {
        problems.push(shape(
            chart.key("name"),
            "`name` is the chart's file name only, without a folder.".to_owned(),
        ));
    }
    if file_chart.page == Some(0) {
        problems.push(shape(chart.key("page"), "`page` counts from 1.".to_owned()));
    }
    for (key, text) in [
        ("extractor", &file_chart.extractor),
        ("extractorVersion", &file_chart.extractor_version),
    ] {
        if text.is_empty() || text.len() > MAX_STRING {
            problems.push(shape(
                chart.key(key),
                format!("`{key}` is a name of 1 to {MAX_STRING} bytes."),
            ));
        }
    }
}

/// §4, §8: IDs, references, notes and limits.
fn check_edits(edits: &[FileEdit], problems: &mut Diagnostics) {
    let root = SpecPointer::root();
    if edits.len() > MAX_EDITS {
        problems.push(limit(format!(
            "The file has {} edits; at most {MAX_EDITS}.",
            edits.len()
        )));
        return;
    }
    let mut ids: BTreeSet<&str> = BTreeSet::new();
    let mut added: BTreeSet<&str> = BTreeSet::new();
    for (index, edit) in edits.iter().enumerate() {
        let at = root.key("edits").index(index);
        let id = edit.id();
        if !is_edit_id(id) {
            problems.push(shape(
                at.key("id"),
                format!("`{id}` is not an edit ID; IDs are e1, e2, …"),
            ));
        } else if !ids.insert(id) {
            problems.push(shape(
                at.key("id"),
                format!("The edit ID `{id}` is used twice."),
            ));
        }
        if edit.series().is_empty() || edit.series().len() > MAX_STRING {
            problems.push(shape(
                at.key("series"),
                format!("`series` is a name of 1 to {MAX_STRING} bytes."),
            ));
        }
        if let Some(note) = edit.note()
            && note.chars().count() > MAX_NOTE
        {
            problems.push(limit(format!(
                "The note of `{id}` is longer than {MAX_NOTE} characters."
            )));
        }
        match edit.point() {
            Some(PointRef::Added(text)) => match text.strip_prefix('+') {
                Some(target) if added.contains(target) => {}
                _ => problems.push(shape(
                    at.key("point"),
                    format!("`{text}` does not name an `add` earlier in the file (`+e<n>`)."),
                )),
            },
            Some(PointRef::Index(_)) | None => {}
        }
        if let FileEdit::Move { before, .. }
        | FileEdit::Delete { before, .. }
        | FileEdit::Retype { before, .. } = edit
            && before.confidence.is_some_and(|c| !(0.0..=1.0).contains(&c))
        {
            problems.push(shape(
                at.key("before").key("confidence"),
                "A confidence lies in [0, 1].".to_owned(),
            ));
        }
        if matches!(edit, FileEdit::Add { .. }) {
            added.insert(id);
        }
    }
}

/// §6: a PNG as base64, under the limit.
fn check_image(image: &ChartImage, problems: &mut Diagnostics) {
    {
        let at = SpecPointer::root().key("image");
        if image.media != "image/png" {
            problems.push(shape(
                at.key("media"),
                "Only `image/png` images are read.".to_owned(),
            ));
        }
        if image.data.len() > MAX_IMAGE {
            problems.push(limit(format!(
                "The image is longer than {MAX_IMAGE} bytes of base64."
            )));
        } else if !image
            .data
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'='))
            || !image.data.len().is_multiple_of(4)
        {
            problems.push(shape(
                at.key("data"),
                "`data` is base64 (RFC 4648 §4).".to_owned(),
            ));
        }
    }
}

/// A JSON value read with no duplicate keys and at most [`MAX_DEPTH`] levels.
struct Strict {
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for Strict {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Strict {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
        Ok(Value::from(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
        Ok(Value::from(v))
    }

    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Value, E> {
        serde_json::Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| E::custom("a number is not finite"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let depth = self.deeper::<A::Error>()?;
        let mut items = Vec::new();
        while let Some(item) = seq.next_element_seed(Strict { depth })? {
            items.push(item);
        }
        Ok(Value::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let depth = self.deeper::<A::Error>()?;
        let mut object = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if object.contains_key(&key) {
                return Err(serde::de::Error::custom(format!(
                    "the key `{key}` appears twice"
                )));
            }
            let value = map.next_value_seed(Strict { depth })?;
            object.insert(key, value);
        }
        Ok(Value::Object(object))
    }
}

impl Strict {
    fn deeper<E: serde::de::Error>(&self) -> Result<usize, E> {
        if self.depth >= MAX_DEPTH {
            return Err(E::custom(format!("nesting deeper than {MAX_DEPTH} levels")));
        }
        Ok(self.depth + 1)
    }
}

#[cfg(test)]
mod tests;

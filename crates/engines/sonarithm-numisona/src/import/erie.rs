// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The Erie importer, A06 subset (SDR-05; 04 §9).
//!
//! [Erie](https://see-mike-out.github.io/erie-documentation) (Kim, Kim & Hullman, CHI 2024)
//! is a declarative sonification grammar. This importer turns an Erie spec into SONARITHM
//! spec v0.1 JSON. It was written from Erie's published documentation; no Erie source code
//! was used.
//!
//! **Every construct is accounted for.** A construct that maps exactly is imported. One
//! that maps only approximately is imported and reported as [`Outcome::Approximated`]; one
//! with no v0.1 equivalent is left out and reported as [`Outcome::Unmapped`], with its JSON
//! pointer in the Erie spec (04 §9: importers never drop a construct silently).
//!
//! | Erie | SONARITHM v0.1 |
//! |---|---|
//! | `title`, `description` | `name`, `description` |
//! | `data.values`, `data.url`, `data.name` + `datasets` | `data.source` `inline` or `file` (network URLs are not read, ADR-0012) |
//! | `transform`: `filter`, `calculate`, `aggregate`, `bin` | `filter`, `derive`, `aggregate`, `bin`; expressions are translated from JavaScript (approximated) |
//! | `tone.type`, `tone.continued` | `instrument`; `mark` `glide` (continued) or `tone` |
//! | `time` (absolute with `length`, relative with `band`) | `substrate.time` field and pace |
//! | `pitch`, `detune` | `pitch` (Hz ranges, note names converted; detune as cents) |
//! | `duration`, `tapCount`, `tapSpeed` | `duration`, `taps` (1–8) |
//! | `loudness` (gain) | `loudness` (dB) |
//! | `pan`, `panX` | `pan` |
//! | `timbre` | `timbre` |
//! | `overlay` with one dataset | one spec with one track per stream |
//! | `sequence`, or `overlay` with several datasets | one spec per stream, with the composition reported |
//!
//! Field `type`s become `data.fields` types and `scale.title` becomes the field `label`.

#![allow(
    clippy::float_cmp,
    reason = "a range or domain whose two ends are exactly equal is empty; the importer tests exactly that"
)]

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt;

use serde_json::{Map, Value, json};
use sonarithm_core::math;

/// How the imported specs play together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Composition {
    /// One stream: one spec with one track.
    Single,
    /// Streams at the same time. With one dataset they are tracks of one spec (tracks
    /// overlay by default, 04 §5.2); otherwise one spec each, to be played together.
    Overlay,
    /// Streams one after another: one spec each, in order. Spec v0.1 has no sequential
    /// composition.
    Sequence,
}

/// What became of one Erie construct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    /// Imported with a change of meaning the note describes.
    Approximated,
    /// Not imported: spec v0.1 has no equivalent.
    Unmapped,
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Approximated => "approximated",
            Self::Unmapped => "unmapped",
        })
    }
}

/// One report line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportEntry {
    /// JSON pointer to the construct in the Erie spec.
    pub pointer: String,
    /// The construct, such as `encoding.time2` or `transform.fold`.
    pub construct: String,
    /// What became of it.
    pub outcome: Outcome,
    /// Why, and what the user can do.
    pub note: String,
}

impl fmt::Display for ReportEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} at {}: {}",
            self.outcome, self.construct, self.pointer, self.note
        )
    }
}

/// Everything the importer could not map exactly, in the order it met it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    entries: Vec<ReportEntry>,
}

impl ImportReport {
    /// The entries, in spec order.
    #[must_use]
    pub fn entries(&self) -> &[ReportEntry] {
        &self.entries
    }

    /// The entries with `outcome`.
    pub fn with(&self, outcome: Outcome) -> impl Iterator<Item = &ReportEntry> {
        self.entries.iter().filter(move |e| e.outcome == outcome)
    }

    /// Whether everything mapped exactly.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, pointer: &Ptr, construct: &str, outcome: Outcome, note: impl Into<String>) {
        self.entries.push(ReportEntry {
            pointer: pointer.0.clone(),
            construct: construct.to_owned(),
            outcome,
            note: note.into(),
        });
    }

    fn unmapped(&mut self, pointer: &Ptr, construct: &str, note: impl Into<String>) {
        self.push(pointer, construct, Outcome::Unmapped, note);
    }

    fn approximated(&mut self, pointer: &Ptr, construct: &str, note: impl Into<String>) {
        self.push(pointer, construct, Outcome::Approximated, note);
    }
}

/// The result of an import.
#[derive(Clone, Debug, PartialEq)]
pub struct ErieImport {
    /// SONARITHM spec v0.1 documents, in play order.
    pub specs: Vec<Value>,
    /// How the specs play together.
    pub composition: Composition,
    /// What was approximated or left out.
    pub report: ImportReport,
}

/// Why nothing could be imported.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ImportError {
    /// The text is not JSON.
    #[error("the Erie spec is not valid JSON: {0}")]
    Json(String),
    /// The document is not a JSON object.
    #[error("an Erie spec is a JSON object")]
    NotAnObject,
    /// A stream has no data SONARITHM can read.
    #[error("stream at {pointer} has no data that spec v0.1 can read: {reason}")]
    NoData {
        /// The stream's JSON pointer.
        pointer: String,
        /// Why.
        reason: String,
    },
    /// A `sequence` or `overlay` is not a non-empty array of streams.
    #[error("`{0}` must be a non-empty array of streams")]
    NoStreams(&'static str),
}

/// Imports an Erie spec given as JSON text.
///
/// # Errors
///
/// An [`ImportError`] when the text is not a JSON object or a stream has no usable data.
pub fn import(text: &str) -> Result<ErieImport, ImportError> {
    let value: Value = serde_json::from_str(text).map_err(|e| ImportError::Json(e.to_string()))?;
    import_value(&value)
}

/// Imports an Erie spec.
///
/// # Errors
///
/// As for [`import`].
pub fn import_value(erie: &Value) -> Result<ErieImport, ImportError> {
    let root = erie.as_object().ok_or(ImportError::NotAnObject)?;
    let mut report = ImportReport::default();
    let at = Ptr::root();
    let datasets = datasets(root, &at, &mut report);
    for (key, _) in root {
        if !ROOT_KEYS.contains(&key.as_str()) && !STREAM_KEYS.contains(&key.as_str()) {
            report.unmapped(&at.key(key), key, "not an Erie key this importer knows");
        }
    }
    unmapped_root(root, &at, &mut report);
    let composite = ["overlay", "sequence"]
        .into_iter()
        .find(|key| root.contains_key(*key));
    let (specs, composition) = match composite {
        None => {
            let stream = Stream::read(root, None, 0, &at, &mut report);
            let spec = Group::single(root, &stream, &datasets, &at, &mut report)?;
            (vec![spec], Composition::Single)
        }
        Some(key) => {
            let items = root
                .get(key)
                .and_then(Value::as_array)
                .filter(|items| !items.is_empty())
                .ok_or(ImportError::NoStreams(if key == "overlay" {
                    "overlay"
                } else {
                    "sequence"
                }))?;
            if root.contains_key("overlay") && root.contains_key("sequence") {
                report.unmapped(
                    &at.key("sequence"),
                    "sequence",
                    "a spec with both `overlay` and `sequence` is imported as its overlay only",
                );
            }
            composite_specs(root, key, items, &datasets, &at, &mut report)?
        }
    };
    Ok(ErieImport {
        specs,
        composition,
        report,
    })
}

/// Top-level keys of a composite spec besides the stream keys.
const ROOT_KEYS: &[&str] = &["datasets", "overlay", "sequence"];

/// Keys of a stream (Erie documentation, "Stream").
const STREAM_KEYS: &[&str] = &[
    "name",
    "title",
    "description",
    "data",
    "transform",
    "tone",
    "encoding",
    "tick",
    "sampling",
    "synth",
    "wave",
    "config",
    "ordering",
];

/// A JSON pointer into the Erie spec.
#[derive(Clone, Debug)]
struct Ptr(String);

impl Ptr {
    fn root() -> Self {
        Self(String::new())
    }

    fn key(&self, key: &str) -> Self {
        Self(format!(
            "{}/{}",
            self.0,
            key.replace('~', "~0").replace('/', "~1")
        ))
    }

    fn index(&self, index: usize) -> Self {
        Self(format!("{}/{index}", self.0))
    }
}

/// Reports the top-level keys that configure players or instruments.
fn unmapped_root(root: &Map<String, Value>, at: &Ptr, report: &mut ImportReport) {
    for (key, note) in [
        (
            "sampling",
            "sampled instruments arrive with AXONOTE (C04); the default instrument plays",
        ),
        (
            "synth",
            "synth definitions arrive with AXONOTE (C04); the default instrument plays",
        ),
        (
            "wave",
            "periodic-wave definitions arrive with AXONOTE (C04); the default instrument plays",
        ),
        (
            "ordering",
            "stream ordering has no v0.1 equivalent; streams keep their spec order",
        ),
        (
            "tick",
            "named tick definitions are not imported; ticks follow the context defaults (SDR-16)",
        ),
    ] {
        if root.contains_key(key) {
            report.unmapped(&at.key(key), key, note);
        }
    }
    if let Some(config) = root.get("config").and_then(Value::as_object) {
        for key in config.keys() {
            report.unmapped(
                &at.key("config").key(key),
                &format!("config.{key}"),
                "player configuration (speech, scale announcements, time units) has no v0.1 equivalent; the spoken legend follows SDR-18",
            );
        }
    }
}

/// Named datasets: `datasets: [{ name, url | values }]`.
fn datasets(
    root: &Map<String, Value>,
    at: &Ptr,
    report: &mut ImportReport,
) -> BTreeMap<String, Value> {
    let mut named = BTreeMap::new();
    let Some(list) = root.get("datasets") else {
        return named;
    };
    let at = at.key("datasets");
    let Some(list) = list.as_array() else {
        report.unmapped(&at, "datasets", "`datasets` is not an array");
        return named;
    };
    for (i, dataset) in list.iter().enumerate() {
        match dataset.get("name").and_then(Value::as_str) {
            Some(name) => {
                named.insert(name.to_owned(), dataset.clone());
            }
            None => report.unmapped(&at.index(i), "datasets[]", "a dataset without a `name`"),
        }
    }
    named
}

/// The streams of an `overlay` or `sequence`.
fn composite_specs(
    root: &Map<String, Value>,
    key: &str,
    items: &[Value],
    datasets: &BTreeMap<String, Value>,
    at: &Ptr,
    report: &mut ImportReport,
) -> Result<(Vec<Value>, Composition), ImportError> {
    let list_at = at.key(key);
    let mut streams = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let item_at = list_at.index(i);
        let Some(object) = item.as_object() else {
            report.unmapped(&item_at, key, "a stream that is not an object");
            continue;
        };
        for item_key in object.keys() {
            if !STREAM_KEYS.contains(&item_key.as_str()) {
                report.unmapped(
                    &item_at.key(item_key),
                    item_key,
                    "not an Erie stream key this importer knows",
                );
            }
        }
        unmapped_root(object, &item_at, report);
        let stream = Stream::read(object, Some(root), i, &item_at, report);
        streams.push((stream, object, item_at));
    }
    if streams.is_empty() {
        return Err(ImportError::NoStreams(if key == "overlay" {
            "overlay"
        } else {
            "sequence"
        }));
    }
    let shared = streams
        .iter()
        .all(|(_, object, _)| !object.contains_key("data") && !object.contains_key("transform"));
    if key == "overlay" && shared {
        let spec = Group::overlay(root, &streams, datasets, at, report)?;
        return Ok((vec![spec], Composition::Overlay));
    }
    let composition = if key == "overlay" {
        report.approximated(
            &list_at,
            "overlay",
            "the streams use different data, and a v0.1 spec has one dataset; each stream is imported as its own spec, to be played together",
        );
        Composition::Overlay
    } else {
        report.approximated(
            &list_at,
            "sequence",
            "spec v0.1 has no sequential composition; each stream is imported as its own spec, to be played in order",
        );
        Composition::Sequence
    };
    let mut specs = Vec::new();
    for (stream, object, item_at) in &streams {
        let mut context: Map<String, Value> = (*object).clone();
        for shared_key in ["data", "transform", "title", "description"] {
            if !context.contains_key(shared_key)
                && let Some(value) = root.get(shared_key)
            {
                context.insert(shared_key.to_owned(), value.clone());
            }
        }
        specs.push(Group::single(&context, stream, datasets, item_at, report)?);
    }
    Ok((specs, composition))
}

/// One Erie stream, read into a SONARITHM track and its time mapping.
struct Stream {
    track: Map<String, Value>,
    time: Option<Time>,
    /// Field name → (type, label), from channel `type`s and `scale.title`s.
    fields: BTreeMap<String, Map<String, Value>>,
}

/// An Erie `time` channel.
#[derive(Clone, Debug, PartialEq)]
struct Time {
    field: Option<String>,
    timing: Timing,
    at: String,
}

#[derive(Clone, Debug, PartialEq)]
enum Timing {
    /// Positions proportional to the field over `length` seconds.
    Absolute { length: Option<f64> },
    /// One tone after another, each `band` seconds.
    Relative { band: Option<f64> },
}

impl Stream {
    fn read(
        stream: &Map<String, Value>,
        root: Option<&Map<String, Value>>,
        index: usize,
        at: &Ptr,
        report: &mut ImportReport,
    ) -> Self {
        let mut out = Self {
            track: Map::new(),
            time: None,
            fields: BTreeMap::new(),
        };
        let name = stream
            .get("name")
            .and_then(Value::as_str)
            .or_else(|| root.is_none().then_some("y"));
        out.track
            .insert("id".to_owned(), json!(track_id(name, index)));
        let continued = out.tone(stream.get("tone"), &at.key("tone"), report);
        out.track.insert(
            "mark".to_owned(),
            json!(if continued { "glide" } else { "tone" }),
        );
        let encoding_at = at.key("encoding");
        let mut channels = Map::new();
        match stream.get("encoding").and_then(Value::as_object) {
            Some(encoding) => {
                out.encoding(encoding, continued, &encoding_at, &mut channels, report);
            }
            None => report.unmapped(
                &encoding_at,
                "encoding",
                "the stream has no encoding; its track has no channels",
            ),
        }
        out.track
            .insert("channels".to_owned(), Value::Object(channels));
        out
    }

    /// `tone`: the instrument and whether the tone is continued; returns `continued`.
    fn tone(&mut self, tone: Option<&Value>, at: &Ptr, report: &mut ImportReport) -> bool {
        let Some(tone) = tone.and_then(Value::as_object) else {
            return false;
        };
        let continued = tone.get("continued").and_then(Value::as_bool) == Some(true);
        for (key, value) in tone {
            match key.as_str() {
                "continued" => {}
                "type" => match value.as_str() {
                    Some(kind) => {
                        let (instrument, exact) = instrument(kind);
                        self.track
                            .insert("instrument".to_owned(), json!(instrument));
                        if !exact {
                            report.approximated(
                                &at.key("type"),
                                "tone.type",
                                format!(
                                    "instrument `{kind}` is passed through by name; instruments beyond the oscillators arrive with AXONOTE (C04)"
                                ),
                            );
                        }
                    }
                    None => report.unmapped(
                        &at.key("type"),
                        "tone.type",
                        "an inline instrument definition arrives with AXONOTE (C04)",
                    ),
                },
                "filter" => report.unmapped(
                    &at.key("filter"),
                    "tone.filter",
                    "audio filters on tones have no v0.1 equivalent",
                ),
                other => report.unmapped(
                    &at.key(other),
                    &format!("tone.{other}"),
                    "no v0.1 equivalent (streaming base tones arrive with SONOTRACE, B06)",
                ),
            }
        }
        continued
    }

    fn encoding(
        &mut self,
        encoding: &Map<String, Value>,
        continued: bool,
        at: &Ptr,
        channels: &mut Map<String, Value>,
        report: &mut ImportReport,
    ) {
        for (name, spec) in encoding {
            let channel_at = at.key(name);
            let Some(spec) = spec.as_object() else {
                report.unmapped(&channel_at, name, "a channel that is not an object");
                continue;
            };
            match name.as_str() {
                "time" => self.time = Some(self.time(spec, &channel_at, report)),
                "pitch" => {
                    let channel = self.pitch(spec, &channel_at, report);
                    channels.insert("pitch".to_owned(), channel);
                }
                "detune" => {
                    if encoding.contains_key("pitch") {
                        report.unmapped(
                            &channel_at,
                            "encoding.detune",
                            "the stream also maps pitch; detune is left out",
                        );
                    } else {
                        let channel = self.detune(spec, &channel_at, report);
                        channels.insert("pitch".to_owned(), channel);
                    }
                }
                "duration" if continued => report.unmapped(
                    &channel_at,
                    "encoding.duration",
                    "durations apply to discrete tones only, and this tone is continued",
                ),
                "duration" => {
                    let channel = self.duration(spec, &channel_at, report);
                    channels.insert("duration".to_owned(), channel);
                }
                "tapCount" | "tapSpeed" if continued => report.unmapped(
                    &channel_at,
                    &format!("encoding.{name}"),
                    "taps apply to discrete tones only, and this tone is continued",
                ),
                "tapCount" => {
                    let channel = self.taps(spec, Taps::Count, &channel_at, report);
                    channels.insert("taps".to_owned(), channel);
                }
                "tapSpeed" if encoding.contains_key("tapCount") => report.unmapped(
                    &channel_at,
                    "encoding.tapSpeed",
                    "the stream also maps tapCount, which sets the number of taps; tap speed is the renderer's",
                ),
                "tapSpeed" => {
                    let band = spec
                        .get("scale")
                        .and_then(|s| s.get("band"))
                        .and_then(Value::as_f64);
                    let channel = self.taps(spec, Taps::Speed { band }, &channel_at, report);
                    channels.insert("taps".to_owned(), channel);
                }
                "loudness" => {
                    let channel = self.loudness(spec, &channel_at, report);
                    channels.insert("loudness".to_owned(), channel);
                }
                "pan" | "panX" => {
                    if name == "panX" && encoding.contains_key("pan") {
                        report.unmapped(&channel_at, "encoding.panX", "`pan` is used instead");
                    } else {
                        let channel = self.pan(spec, &channel_at, report);
                        channels.insert("pan".to_owned(), channel);
                    }
                }
                "timbre" => {
                    let channel = self.timbre(spec, &channel_at, report);
                    channels.insert("timbre".to_owned(), channel);
                }
                "time2" => report.unmapped(
                    &channel_at,
                    "encoding.time2",
                    "intervals with an end field have no v0.1 equivalent; tones start at `time` and use the duration channel",
                ),
                "panY" | "panZ" | "panRadius" | "panPolar" | "panAzimuth" => report.unmapped(
                    &channel_at,
                    &format!("encoding.{name}"),
                    "3D spatial positions arrive with spatial audio (D03); v0.1 has stereo pan only",
                ),
                "speechBefore" | "speechAfter" => report.unmapped(
                    &channel_at,
                    &format!("encoding.{name}"),
                    "per-tone speech has no v0.1 channel; spoken legends come from `context.legend` (SDR-18)",
                ),
                "repeat" => report.unmapped(
                    &channel_at,
                    "encoding.repeat",
                    "data-driven repetition over categories has no v0.1 equivalent; split the data into one track per category with `filter`",
                ),
                other => report.unmapped(
                    &channel_at,
                    &format!("encoding.{other}"),
                    "no v0.1 channel (modulation, harmonicity and post-reverb arrive with AXONOTE instruments, C04)",
                ),
            }
        }
    }

    /// The channel parts every Erie channel shares: `field` and `type`, plus the keys spec
    /// v0.1 cannot express.
    fn field(
        &mut self,
        spec: &Map<String, Value>,
        at: &Ptr,
        name: &str,
        report: &mut ImportReport,
    ) -> Option<String> {
        for (key, note) in [
            (
                "aggregate",
                "inline aggregation is not imported; add an `aggregate` transform",
            ),
            (
                "bin",
                "inline binning is not imported; add a `bin` transform",
            ),
            ("condition", "conditional values have no v0.1 equivalent"),
            (
                "format",
                "number and date formats for announcements have no v0.1 equivalent",
            ),
            (
                "formatType",
                "number and date formats for announcements have no v0.1 equivalent",
            ),
            (
                "ramp",
                "per-channel ramps are not imported; glides interpolate linearly",
            ),
            ("speech", "per-channel speech has no v0.1 equivalent"),
        ] {
            if spec.contains_key(key) {
                report.unmapped(&at.key(key), &format!("encoding.{name}.{key}"), note);
            }
        }
        let field = spec.get("field")?;
        let Some(field) = field.as_str() else {
            report.unmapped(
                &at.key("field"),
                &format!("encoding.{name}.field"),
                "a list of fields is not supported",
            );
            return None;
        };
        let entry = self.fields.entry(field.to_owned()).or_default();
        if let Some(kind) = spec.get("type").and_then(Value::as_str) {
            if ["quantitative", "temporal", "ordinal", "nominal"].contains(&kind) {
                entry.entry("type").or_insert_with(|| json!(kind));
            } else {
                report.unmapped(
                    &at.key("type"),
                    &format!("encoding.{name}.type"),
                    format!("`{kind}` is not a field type"),
                );
            }
        }
        if let Some(title) = spec
            .get("scale")
            .and_then(|s| s.get("title"))
            .and_then(Value::as_str)
        {
            entry.entry("label").or_insert_with(|| json!(title));
        }
        Some(field.to_owned())
    }

    /// Reports `scale` keys other than `keep`.
    fn scale_rest(
        scale: Option<&Value>,
        keep: &[&str],
        at: &Ptr,
        name: &str,
        report: &mut ImportReport,
    ) {
        let Some(scale) = scale.and_then(Value::as_object) else {
            return;
        };
        for key in scale.keys() {
            if keep.contains(&key.as_str()) || key == "title" {
                continue;
            }
            let note = match key.as_str() {
                "description" => "scale announcements follow the spoken legend (SDR-18)",
                "maxDistinct" | "times" => "no v0.1 equivalent; the range is used as given",
                _ => "no v0.1 equivalent",
            };
            report.unmapped(
                &at.key("scale").key(key),
                &format!("encoding.{name}.scale.{key}"),
                note,
            );
        }
    }

    fn time(&mut self, spec: &Map<String, Value>, at: &Ptr, report: &mut ImportReport) -> Time {
        let field = self.field(spec, at, "time", report);
        if spec.contains_key("tick") {
            report.approximated(
                &at.key("tick"),
                "encoding.time.tick",
                "the tick track follows the context defaults (SDR-16: on for discrete tones, off for glides); its interval is not imported",
            );
        }
        let scale = spec.get("scale");
        Self::scale_rest(
            scale,
            &["timing", "length", "band", "domain", "range"],
            at,
            "time",
            report,
        );
        let get = |key: &str| scale.and_then(|s| s.get(key));
        let timing = get("timing").and_then(Value::as_str).unwrap_or("absolute");
        let timing = match timing {
            "relative" => Timing::Relative {
                band: get("band").and_then(Value::as_f64),
            },
            "simultaneous" => {
                report.unmapped(
                    &at.key("scale").key("timing"),
                    "encoding.time.scale.timing",
                    "simultaneous timing (every tone at 0 s) has no v0.1 equivalent; tones play one after another",
                );
                Timing::Relative { band: None }
            }
            _ => {
                let length = get("length").and_then(Value::as_f64).or_else(|| {
                    get("range")
                        .and_then(Value::as_array)
                        .and_then(|r| Some(r.last()?.as_f64()? - r.first()?.as_f64()?))
                });
                if get("domain").is_some() {
                    report.unmapped(
                        &at.key("scale").key("domain"),
                        "encoding.time.scale.domain",
                        "an explicit time domain is not imported; time spans the data",
                    );
                }
                Timing::Absolute { length }
            }
        };
        Time {
            field,
            timing,
            at: at.0.clone(),
        }
    }

    fn pitch(&mut self, spec: &Map<String, Value>, at: &Ptr, report: &mut ImportReport) -> Value {
        let mut channel = Map::new();
        if let Some(field) = self.field(spec, at, "pitch", report) {
            channel.insert("field".to_owned(), json!(field));
        }
        if spec.get("roundToNote").and_then(Value::as_bool) == Some(true) {
            report.unmapped(
                &at.key("roundToNote"),
                "encoding.pitch.roundToNote",
                "Analysis mode keeps pitch continuous (SDR-10); quantization arrives with AXONOTE in Presentation mode (C04)",
            );
        }
        let scale = spec.get("scale");
        Self::scale_rest(scale, MAPPING_KEYS, at, "pitch", report);
        let mut flip = false;
        if let Some(range) = scale.and_then(|s| s.get("range"))
            && let Some([low, high]) = hz_range(range, &at.key("scale").key("range"), report)
        {
            flip ^= low > high;
            let (low, high) = (low.min(high), low.max(high));
            channel.insert("range".to_owned(), json!({ "hz": [low, high] }));
        }
        let kind = spec.get("type").and_then(Value::as_str);
        flip ^= Self::domain(scale, kind, at, "pitch", &mut channel, report);
        polarity(scale, flip, &mut channel);
        if let Some(value) = spec.get("value") {
            match pitch_hz(value) {
                Some(hz) if (20.0..=20_000.0).contains(&hz) => {
                    channel.insert("value".to_owned(), json!({ "hz": hz }));
                }
                _ => report.unmapped(
                    &at.key("value"),
                    "encoding.pitch.value",
                    "not a frequency from 20 to 20000 Hz or a note name",
                ),
            }
        }
        if kind == Some("nominal") {
            report.approximated(
                at,
                "encoding.pitch.type",
                "a nominal field on pitch is imported with a band scale; SDR-25 recommends timbre for categories",
            );
        }
        Value::Object(channel)
    }

    /// `scale.domain` and the scale type for a field's `type`; returns whether the domain
    /// was written high to low (a flip of polarity).
    fn domain(
        scale: Option<&Value>,
        kind: Option<&str>,
        at: &Ptr,
        name: &str,
        channel: &mut Map<String, Value>,
        report: &mut ImportReport,
    ) -> bool {
        let mut out = Map::new();
        match kind {
            Some("nominal") => {
                out.insert("type".to_owned(), json!("band"));
            }
            Some("ordinal") => {
                out.insert("type".to_owned(), json!("ordinal"));
            }
            _ => {}
        }
        let mut flip = false;
        let domain = scale.and_then(|s| s.get("domain"));
        let numbers: Option<Vec<f64>> = domain
            .and_then(Value::as_array)
            .and_then(|d| d.iter().map(Value::as_f64).collect());
        match (domain, numbers) {
            (None, _) => {
                if scale.and_then(|s| s.get("zero")).and_then(Value::as_bool) == Some(true) {
                    out.insert("domain".to_owned(), json!("zero-based"));
                }
            }
            (Some(_), Some(numbers)) if numbers.len() >= 2 => {
                let (first, last) = (numbers[0], numbers[numbers.len() - 1]);
                if numbers.len() > 2 {
                    report.approximated(
                        &at.key("scale").key("domain"),
                        &format!("encoding.{name}.scale.domain"),
                        "a piecewise domain is imported as its two ends",
                    );
                }
                if first == last {
                    report.unmapped(
                        &at.key("scale").key("domain"),
                        &format!("encoding.{name}.scale.domain"),
                        "an empty domain; the data extent is used",
                    );
                } else {
                    flip = first > last;
                    out.insert("domain".to_owned(), json!([first.min(last), first.max(last)]));
                }
            }
            (Some(_), _) => report.unmapped(
                &at.key("scale").key("domain"),
                &format!("encoding.{name}.scale.domain"),
                "a list of categories sets their order, which v0.1 cannot express; categories keep their data order",
            ),
        }
        if !out.is_empty() {
            channel.insert("scale".to_owned(), Value::Object(out));
        }
        flip
    }

    fn detune(&mut self, spec: &Map<String, Value>, at: &Ptr, report: &mut ImportReport) -> Value {
        report.approximated(
            at,
            "encoding.detune",
            "detune applies to noise tones in Erie; it is imported as pitch in cents relative to A4 on the stream's instrument",
        );
        let mut channel = Map::new();
        if let Some(field) = self.field(spec, at, "detune", report) {
            channel.insert("field".to_owned(), json!(field));
        }
        let scale = spec.get("scale");
        Self::scale_rest(scale, MAPPING_KEYS, at, "detune", report);
        let mut flip = false;
        let range = numbers(scale, "range");
        if let Some(range) = range.filter(|r| r.len() >= 2) {
            let (first, last) = (range[0], range[range.len() - 1]);
            if first != last {
                flip ^= first > last;
                channel.insert(
                    "range".to_owned(),
                    json!({ "cents": [first.min(last), first.max(last)] }),
                );
            }
        }
        let kind = spec.get("type").and_then(Value::as_str);
        flip ^= Self::domain(scale, kind, at, "detune", &mut channel, report);
        polarity(scale, flip, &mut channel);
        if let Some(cents) = spec.get("value").and_then(Value::as_f64) {
            channel.insert("value".to_owned(), json!({ "cents": cents }));
        }
        Value::Object(channel)
    }

    fn duration(
        &mut self,
        spec: &Map<String, Value>,
        at: &Ptr,
        report: &mut ImportReport,
    ) -> Value {
        let mut channel = Map::new();
        if let Some(field) = self.field(spec, at, "duration", report) {
            channel.insert("field".to_owned(), json!(field));
        }
        let scale = spec.get("scale");
        Self::scale_rest(scale, MAPPING_KEYS, at, "duration", report);
        let range = numbers(scale, "range");
        let mut flip = false;
        if let Some(range) = range.filter(|r| r.len() >= 2) {
            let (first, last) = (range[0], range[range.len() - 1]);
            flip ^= first > last;
            let (mut low, high) = (first.min(last), first.max(last));
            if low <= 0.0 {
                low = MIN_DURATION_S.min(high / 2.0);
                report.approximated(
                    &at.key("scale").key("range"),
                    "encoding.duration.scale.range",
                    format!(
                        "durations must be positive; the range starts at {low} s instead of {}",
                        first.min(last)
                    ),
                );
            }
            if high > low {
                channel.insert("range".to_owned(), json!({ "s": [low, high] }));
            }
        }
        let kind = spec.get("type").and_then(Value::as_str);
        flip ^= Self::domain(scale, kind, at, "duration", &mut channel, report);
        polarity(scale, flip, &mut channel);
        if let Some(seconds) = spec.get("value").and_then(Value::as_f64) {
            if seconds > 0.0 {
                channel.insert("value".to_owned(), json!({ "s": seconds }));
            } else {
                report.unmapped(
                    &at.key("value"),
                    "encoding.duration.value",
                    "durations must be positive",
                );
            }
        }
        Value::Object(channel)
    }

    /// `tapCount` (counts) or `tapSpeed` (taps per second over `band` seconds).
    fn taps(
        &mut self,
        spec: &Map<String, Value>,
        kind: Taps,
        at: &Ptr,
        report: &mut ImportReport,
    ) -> Value {
        let name = match kind {
            Taps::Count => "tapCount",
            Taps::Speed { .. } => "tapSpeed",
        };
        let mut channel = Map::new();
        if let Some(field) = self.field(spec, at, name, report) {
            channel.insert("field".to_owned(), json!(field));
        }
        let scale = spec.get("scale");
        Self::scale_rest(
            scale,
            &["domain", "range", "polarity", "zero", "band"],
            at,
            name,
            report,
        );
        for key in [
            "pauseRate",
            "pauseLength",
            "singleTappingPosition",
            "maxTappingLength",
        ] {
            if scale.and_then(|s| s.get(key)).is_some() {
                report.unmapped(
                    &at.key("scale").key(key),
                    &format!("encoding.{name}.scale.{key}"),
                    "tap timing is the renderer's in v0.1",
                );
            }
        }
        let factor = match kind {
            Taps::Count => {
                if scale.and_then(|s| s.get("band")).is_some() {
                    report.unmapped(
                        &at.key("scale").key("band"),
                        "encoding.tapCount.scale.band",
                        "tap length is the renderer's in v0.1",
                    );
                }
                1.0
            }
            Taps::Speed { band: Some(band) } if band > 0.0 => {
                report.approximated(
                    at,
                    "encoding.tapSpeed",
                    format!(
                        "taps per second over {band} s are imported as tap counts (speed × {band})"
                    ),
                );
                band
            }
            Taps::Speed { .. } => {
                report.approximated(
                    at,
                    "encoding.tapSpeed",
                    "tap speed without a `band` is imported as tap counts per second",
                );
                1.0
            }
        };
        let range = numbers(scale, "range");
        let mut flip = false;
        let [min, max] = TAPS_RANGE;
        if let Some(range) = range.filter(|r| r.len() >= 2) {
            let (first, last) = (range[0] * factor, range[range.len() - 1] * factor);
            flip ^= first > last;
            let (low, high) = (first.min(last).round(), first.max(last).round());
            let clamped = (low.clamp(min, max), high.clamp(min, max));
            if clamped != (low, high) {
                report.approximated(
                    &at.key("scale").key("range"),
                    &format!("encoding.{name}.scale.range"),
                    format!(
                        "tap counts are 1 to 8 in v0.1; {low}–{high} becomes {}–{}",
                        clamped.0, clamped.1
                    ),
                );
            }
            if clamped.1 > clamped.0 {
                channel.insert("range".to_owned(), json!([clamped.0, clamped.1]));
            }
        }
        let kind = spec.get("type").and_then(Value::as_str);
        flip ^= Self::domain(scale, kind, at, name, &mut channel, report);
        polarity(scale, flip, &mut channel);
        if let Some(value) = spec.get("value").and_then(Value::as_f64) {
            channel.insert(
                "value".to_owned(),
                json!((value * factor).round().clamp(min, max)),
            );
        }
        Value::Object(channel)
    }

    fn loudness(
        &mut self,
        spec: &Map<String, Value>,
        at: &Ptr,
        report: &mut ImportReport,
    ) -> Value {
        let mut channel = Map::new();
        if let Some(field) = self.field(spec, at, "loudness", report) {
            channel.insert("field".to_owned(), json!(field));
        }
        let scale = spec.get("scale");
        Self::scale_rest(scale, MAPPING_KEYS, at, "loudness", report);
        let mut clamped = false;
        let mut db = |gain: f64| {
            let limited = gain.clamp(MIN_GAIN, 1.0);
            clamped |= limited != gain;
            (100.0 * 20.0 * math::log10(limited)).round() / 100.0
        };
        let range = numbers(scale, "range");
        let mut flip = false;
        let mut value = None;
        let mut db_range = None;
        if let Some(range) = range.filter(|r| r.len() >= 2) {
            let (first, last) = (range[0], range[range.len() - 1]);
            flip ^= first > last;
            db_range = Some([db(first.min(last)), db(first.max(last))]);
        }
        if let Some(gain) = spec.get("value").and_then(Value::as_f64) {
            value = Some(db(gain));
        }
        if clamped {
            report.approximated(
                &at.key("scale"),
                "encoding.loudness",
                "Erie gains are converted to dB and kept within −60 to 0 dB (loudness is relative and at most 0 dB in v0.1)",
            );
        }
        if let Some([low, high]) = db_range.filter(|[low, high]| high > low) {
            channel.insert("range".to_owned(), json!({ "db": [low, high] }));
        }
        let kind = spec.get("type").and_then(Value::as_str);
        flip ^= Self::domain(scale, kind, at, "loudness", &mut channel, report);
        polarity(scale, flip, &mut channel);
        if let Some(value) = value {
            channel.insert("value".to_owned(), json!({ "db": value }));
        }
        Value::Object(channel)
    }

    fn pan(&mut self, spec: &Map<String, Value>, at: &Ptr, report: &mut ImportReport) -> Value {
        let mut channel = Map::new();
        if let Some(field) = self.field(spec, at, "pan", report) {
            channel.insert("field".to_owned(), json!(field));
        }
        let scale = spec.get("scale");
        Self::scale_rest(scale, MAPPING_KEYS, at, "pan", report);
        let range = numbers(scale, "range");
        let mut flip = false;
        if let Some(range) = range.filter(|r| r.len() >= 2) {
            if range.len() > 2 {
                report.approximated(
                    &at.key("scale").key("range"),
                    "encoding.pan.scale.range",
                    "a piecewise range is imported as its two ends",
                );
            }
            let (first, last) = (
                range[0].clamp(-1.0, 1.0),
                range[range.len() - 1].clamp(-1.0, 1.0),
            );
            flip ^= first > last;
            if first != last {
                channel.insert(
                    "range".to_owned(),
                    json!([first.min(last), first.max(last)]),
                );
            }
        }
        let kind = spec.get("type").and_then(Value::as_str);
        flip ^= Self::domain(scale, kind, at, "pan", &mut channel, report);
        polarity(scale, flip, &mut channel);
        if let Some(value) = spec.get("value").and_then(Value::as_f64) {
            channel.insert("value".to_owned(), json!(value.clamp(-1.0, 1.0)));
        }
        Value::Object(channel)
    }

    fn timbre(&mut self, spec: &Map<String, Value>, at: &Ptr, report: &mut ImportReport) -> Value {
        let mut channel = Map::new();
        if let Some(field) = self.field(spec, at, "timbre", report) {
            channel.insert("field".to_owned(), json!(field));
        }
        Self::scale_rest(spec.get("scale"), &[], at, "timbre", report);
        if let Some(name) = spec.get("value").and_then(Value::as_str) {
            channel.insert("value".to_owned(), json!(instrument(name).0));
        }
        Value::Object(channel)
    }
}

/// Which Erie tap channel.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Taps {
    /// `tapCount`: a number of taps.
    Count,
    /// `tapSpeed`: taps per second over `band` seconds.
    Speed {
        /// The scale's `band`, if given.
        band: Option<f64>,
    },
}

/// Polarity from `scale.polarity`, flipped once more for each range or domain written
/// high to low (spec v0.1 writes ranges low to high and inverts with polarity).
fn polarity(scale: Option<&Value>, flip: bool, channel: &mut Map<String, Value>) {
    let written = scale
        .and_then(|s| s.get("polarity"))
        .and_then(Value::as_str);
    let negative = match written {
        Some("negative") => !flip,
        Some("positive") => flip,
        _ if flip => true,
        _ => return,
    };
    if channel.contains_key("field") {
        channel.insert(
            "polarity".to_owned(),
            json!(if negative { "negative" } else { "positive" }),
        );
    }
}

/// The `scale` keys every mapped channel imports.
const MAPPING_KEYS: &[&str] = &["domain", "range", "polarity", "zero"];

/// `scale.<key>` as a list of numbers, if it is one.
fn numbers(scale: Option<&Value>, key: &str) -> Option<Vec<f64>> {
    scale
        .and_then(|s| s.get(key))
        .and_then(Value::as_array)
        .and_then(|list| list.iter().map(Value::as_f64).collect())
}

/// Erie durations start at 0 s; spec v0.1 durations are positive, so a range starting at
/// zero starts here instead.
const MIN_DURATION_S: f64 = 0.05;

/// Erie gains below this (−60 dB) are kept at it: spec v0.1 loudness is in dB.
const MIN_GAIN: f64 = 0.001;

/// Spec v0.1 tap counts (RFC-0001 §6).
const TAPS_RANGE: [f64; 2] = [1.0, 8.0];

/// Erie's built-in instrument names to the render presets (`mallet`, `sine`, `square`,
/// `triangle`, `saw`); `true` when the preset is the same sound. Other names pass through
/// and render with the default preset until AXONOTE adds instruments (C04).
fn instrument(kind: &str) -> (String, bool) {
    match kind {
        "default" | "sine" => ("sine".to_owned(), true),
        "square" | "triangle" => (kind.to_owned(), true),
        "sawtooth" => ("saw".to_owned(), true),
        other => (other.to_owned(), false),
    }
}

/// A track ID from a stream name (`^[A-Za-z][A-Za-z0-9_-]{0,63}$`).
fn track_id(name: Option<&str>, index: usize) -> String {
    let cleaned: String = name
        .unwrap_or("")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let cleaned = cleaned.trim_matches('-');
    let mut id = if cleaned.starts_with(|c: char| c.is_ascii_alphabetic()) {
        cleaned.to_owned()
    } else if cleaned.is_empty() {
        format!("stream{}", index + 1)
    } else {
        format!("s{cleaned}")
    };
    id.truncate(64);
    id
}

/// A frequency from a number (Hz) or a note name such as `C4` or `F#3` (C4 = MIDI 60).
fn pitch_hz(value: &Value) -> Option<f64> {
    if let Some(hz) = value.as_f64() {
        return Some(hz);
    }
    let name = value.as_str()?;
    let mut chars = name.chars();
    let letter = chars.next()?.to_ascii_uppercase();
    let class = match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let rest: String = chars.collect();
    let (shift, octave) = if let Some(octave) = rest.strip_prefix('#') {
        (1, octave)
    } else if let Some(octave) = rest.strip_prefix('b') {
        (-1, octave)
    } else {
        (0, rest.as_str())
    };
    let octave: i32 = octave.parse().ok()?;
    let midi = 12 * (octave + 1) + class + shift;
    Some(sonarithm_core::units::Midi(f64::from(midi)).to_hz().0)
}

/// A pitch range in hertz from numbers or note names; reports and returns `None` when it
/// cannot be read. A piecewise range keeps its ends.
fn hz_range(range: &Value, at: &Ptr, report: &mut ImportReport) -> Option<[f64; 2]> {
    let values: Option<Vec<f64>> = range
        .as_array()
        .and_then(|r| r.iter().map(pitch_hz).collect());
    let Some(values) = values.filter(|v| v.len() >= 2) else {
        report.unmapped(
            at,
            "encoding.pitch.scale.range",
            "not a list of frequencies or note names",
        );
        return None;
    };
    if values.len() > 2 {
        report.approximated(
            at,
            "encoding.pitch.scale.range",
            "a piecewise range is imported as its two ends",
        );
    }
    let (first, last) = (values[0], values[values.len() - 1]);
    let clamp = |hz: f64| (hz * 1000.0).round() / 1000.0;
    let (first, last) = (
        clamp(first.clamp(20.0, 20_000.0)),
        clamp(last.clamp(20.0, 20_000.0)),
    );
    if first == last {
        report.unmapped(
            at,
            "encoding.pitch.scale.range",
            "an empty range; the default range applies (SDR-08)",
        );
        return None;
    }
    Some([first, last])
}

/// One SONARITHM spec built from streams that share data.
struct Group;

impl Group {
    fn single(
        context: &Map<String, Value>,
        stream: &Stream,
        datasets: &BTreeMap<String, Value>,
        at: &Ptr,
        report: &mut ImportReport,
    ) -> Result<Value, ImportError> {
        Self::build(context, &[stream], datasets, at, report)
    }

    fn overlay(
        root: &Map<String, Value>,
        streams: &[(Stream, &Map<String, Value>, Ptr)],
        datasets: &BTreeMap<String, Value>,
        at: &Ptr,
        report: &mut ImportReport,
    ) -> Result<Value, ImportError> {
        let refs: Vec<&Stream> = streams.iter().map(|(stream, _, _)| stream).collect();
        Self::build(root, &refs, datasets, at, report)
    }

    fn build(
        context: &Map<String, Value>,
        streams: &[&Stream],
        datasets: &BTreeMap<String, Value>,
        at: &Ptr,
        report: &mut ImportReport,
    ) -> Result<Value, ImportError> {
        let mut spec = Map::new();
        spec.insert("specVersion".to_owned(), json!("0.1"));
        for (from, to) in [("title", "name"), ("description", "description")] {
            if let Some(text) = context.get(from).and_then(Value::as_str) {
                spec.insert(to.to_owned(), json!(text));
            }
        }
        let (data, values) = data(context, datasets, at, report)?;
        let mut data = data;
        let mut fields = Map::new();
        for stream in streams {
            for (field, meta) in &stream.fields {
                if !fields.contains_key(field) {
                    fields.insert(field.clone(), Value::Object(meta.clone()));
                }
            }
        }
        if !fields.is_empty() {
            data.insert("fields".to_owned(), Value::Object(fields));
        }
        spec.insert("data".to_owned(), Value::Object(data));
        let transforms = transforms(context.get("transform"), &at.key("transform"), report);
        if !transforms.is_empty() {
            spec.insert("transform".to_owned(), Value::Array(transforms));
        }
        let times: Vec<&Time> = streams.iter().filter_map(|s| s.time.as_ref()).collect();
        let mut tracks = Vec::new();
        let mut ids = std::collections::BTreeSet::new();
        for (i, stream) in streams.iter().enumerate() {
            let mut track = stream.track.clone();
            let id = track
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("stream")
                .to_owned();
            let id = if ids.contains(&id) {
                format!("{id}-{}", i + 1)
            } else {
                id
            };
            ids.insert(id.clone());
            track.insert("id".to_owned(), json!(id));
            if let Some(Time {
                timing: Timing::Relative { band: Some(band) },
                ..
            }) = &stream.time
            {
                let channels = track
                    .entry("channels")
                    .or_insert_with(|| Value::Object(Map::new()));
                if let Some(channels) = channels.as_object_mut()
                    && !channels.contains_key("duration")
                    && track_mark_is_tone(&stream.track)
                {
                    channels.insert("duration".to_owned(), json!({ "value": { "s": band } }));
                }
            }
            tracks.push(Value::Object(track));
        }
        spec.insert("tracks".to_owned(), Value::Array(tracks));
        if let Some(substrate) = substrate(&times, values.as_deref(), report) {
            spec.insert("substrate".to_owned(), substrate);
        }
        Ok(Value::Object(spec))
    }
}

fn track_mark_is_tone(track: &Map<String, Value>) -> bool {
    track.get("mark").and_then(Value::as_str) == Some("tone")
}

/// The SONARITHM `data` object and the inline values, when there are any.
type ImportedData = (Map<String, Value>, Option<Vec<Value>>);

/// `data.source` from Erie's `data`, and the inline values when there are any.
fn data(
    context: &Map<String, Value>,
    datasets: &BTreeMap<String, Value>,
    at: &Ptr,
    report: &mut ImportReport,
) -> Result<ImportedData, ImportError> {
    let at = at.key("data");
    let no_data = |reason: &str| ImportError::NoData {
        pointer: at.0.clone(),
        reason: reason.to_owned(),
    };
    let mut data = context
        .get("data")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| no_data("the stream has no `data`"))?;
    if let Some(name) = data.get("name").and_then(Value::as_str).map(str::to_owned) {
        let named = datasets
            .get(&name)
            .and_then(Value::as_object)
            .ok_or_else(|| no_data(&format!("no dataset is named `{name}`")))?;
        data.clone_from(named);
        data.remove("name");
    }
    if data.get("stream").and_then(Value::as_bool) == Some(true) {
        report.unmapped(
            &at.key("stream"),
            "data.stream",
            "streaming data arrives with SONOTRACE (B06); the stream's `test` data is imported instead",
        );
        data = data
            .get("test")
            .and_then(Value::as_object)
            .cloned()
            .ok_or_else(|| no_data("streaming data without `test` data"))?;
    }
    for key in data.keys() {
        if !["values", "url", "test", "stream", "name"].contains(&key.as_str()) {
            report.unmapped(
                &at.key(key),
                &format!("data.{key}"),
                "no v0.1 equivalent; the format follows the file extension",
            );
        }
    }
    let mut out = Map::new();
    if let Some(values) = data.get("values").and_then(Value::as_array) {
        out.insert(
            "source".to_owned(),
            json!({ "kind": "inline", "values": values }),
        );
        return Ok((out, Some(values.clone())));
    }
    let url = data
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| no_data("`data` has neither `values` nor `url`"))?;
    let path = if url.contains("://") {
        let file = url.rsplit('/').next().unwrap_or(url);
        report.approximated(
            &at.key("url"),
            "data.url",
            format!(
                "SONARITHM does not download data (ADR-0012); the source is the local file `{file}`, next to the spec"
            ),
        );
        file.to_owned()
    } else {
        url.trim_start_matches("./").to_owned()
    };
    out.insert("source".to_owned(), json!({ "kind": "file", "path": path }));
    Ok((out, None))
}

/// `substrate.time` from the streams' `time` channels.
fn substrate(
    times: &[&Time],
    values: Option<&[Value]>,
    report: &mut ImportReport,
) -> Option<Value> {
    let first = times.first()?;
    for other in &times[1..] {
        if (other.field.as_ref(), &other.timing) != (first.field.as_ref(), &first.timing) {
            report.approximated(
                &Ptr(other.at.clone()),
                "encoding.time",
                "a v0.1 spec has one time substrate for all tracks; the first stream's time mapping is used",
            );
        }
    }
    let at = Ptr(first.at.clone());
    let mut time = Map::new();
    if let Some(field) = &first.field {
        time.insert("field".to_owned(), json!(field));
    }
    match &first.timing {
        Timing::Relative { band: Some(band) } if *band > 0.0 => {
            time.insert("pace".to_owned(), json!({ "pointsPerSecond": 1.0 / band }));
        }
        Timing::Relative { .. } => report.approximated(
            &at,
            "encoding.time.scale.band",
            "no tone length is given; the SONARITHM default pace applies (3 points/s, SDR-19)",
        ),
        Timing::Absolute {
            length: Some(length),
        } if *length > 0.0 => {
            let extent = first
                .field
                .as_ref()
                .and_then(|field| numeric_extent(values?, field));
            match extent {
                Some(span) if span > 0.0 => {
                    time.insert(
                        "pace".to_owned(),
                        json!({ "secondsPerUnit": length / span }),
                    );
                }
                _ => {
                    report.approximated(
                        &at,
                        "encoding.time.scale.length",
                        format!(
                            "tone positions proportional to the time field need numeric inline data; the {length} s are spread evenly over the data points instead"
                        ),
                    );
                    time.insert("pace".to_owned(), json!({ "duration": { "s": length } }));
                }
            }
        }
        Timing::Absolute { .. } => report.approximated(
            &at,
            "encoding.time.scale.length",
            "no stream length is given; the SONARITHM default pace applies (3 points/s, SDR-19)",
        ),
    }
    (!time.is_empty()).then(|| json!({ "time": time }))
}

/// `max − min` of a numeric field in inline values, if every present value is a number.
fn numeric_extent(values: &[Value], field: &str) -> Option<f64> {
    let mut extent: Option<(f64, f64)> = None;
    for row in values {
        match row.get(field) {
            None | Some(Value::Null) => {}
            Some(value) => {
                let x = value.as_f64()?;
                extent = Some(extent.map_or((x, x), |(lo, hi)| (lo.min(x), hi.max(x))));
            }
        }
    }
    extent.map(|(lo, hi)| hi - lo)
}

/// SONARITHM transforms from Erie's `transform` list.
fn transforms(list: Option<&Value>, at: &Ptr, report: &mut ImportReport) -> Vec<Value> {
    let Some(list) = list else {
        return Vec::new();
    };
    let Some(list) = list.as_array() else {
        report.unmapped(at, "transform", "`transform` is not an array");
        return Vec::new();
    };
    let mut out = Vec::new();
    for (i, transform) in list.iter().enumerate() {
        let at = at.index(i);
        let Some(transform) = transform.as_object() else {
            report.unmapped(&at, "transform", "a transform that is not an object");
            continue;
        };
        if let Some(expr) = transform.get("filter").and_then(Value::as_str) {
            let expr = expression(expr, &at.key("filter"), "transform.filter", report);
            out.push(json!({ "op": "filter", "expr": expr }));
        } else if let Some(expr) = transform.get("calculate").and_then(Value::as_str) {
            let Some(name) = transform.get("as").and_then(Value::as_str) else {
                report.unmapped(&at, "transform.calculate", "a calculation without `as`");
                continue;
            };
            if transform.contains_key("groupby") {
                report.unmapped(
                    &at.key("groupby"),
                    "transform.calculate.groupby",
                    "grouped calculations have no v0.1 equivalent",
                );
            }
            let expr = expression(expr, &at.key("calculate"), "transform.calculate", report);
            out.push(json!({ "op": "derive", "expr": expr, "as": name }));
        } else if transform.contains_key("aggregate") {
            if let Some(aggregate) = aggregate(transform, &at, report) {
                out.push(aggregate);
            }
        } else if let Some(field) = transform.get("bin").and_then(Value::as_str) {
            out.push(bin(field, transform, &at, report));
        } else {
            let kind = transform.keys().next().map_or("transform", String::as_str);
            report.unmapped(
                &at,
                &format!("transform.{kind}"),
                "no v0.1 transform (fold, density, quantile, boxplot and diffing are not in spec v0.1)",
            );
        }
    }
    out
}

fn aggregate(transform: &Map<String, Value>, at: &Ptr, report: &mut ImportReport) -> Option<Value> {
    let groupby: Vec<String> = transform
        .get("groupby")
        .and_then(Value::as_array)
        .map(|g| {
            g.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let mut ops = Vec::new();
    for (i, op) in transform
        .get("aggregate")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let at = at.key("aggregate").index(i);
        let name = op.get("op").and_then(Value::as_str).unwrap_or("");
        let function = match name {
            "mean" | "average" => "mean",
            "count" | "sum" | "median" | "min" | "max" => name,
            other => {
                report.unmapped(
                    &at,
                    &format!("transform.aggregate.{other}"),
                    "spec v0.1 aggregates are sum, mean, median, min, max and count",
                );
                continue;
            }
        };
        let field = op
            .get("field")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| {
                (function == "count")
                    .then(|| groupby.first().cloned())
                    .flatten()
            });
        let Some(field) = field else {
            report.unmapped(&at, "transform.aggregate", "an aggregation without a field");
            continue;
        };
        let output = op.get("as").and_then(Value::as_str).map_or_else(
            || {
                if function == "count" {
                    "count".to_owned()
                } else {
                    format!("{function}_{field}")
                }
            },
            str::to_owned,
        );
        ops.push(json!({ "op": function, "field": field, "as": output }));
    }
    if ops.is_empty() {
        return None;
    }
    Some(json!({ "op": "aggregate", "groupby": groupby, "ops": ops }))
}

fn bin(field: &str, transform: &Map<String, Value>, at: &Ptr, report: &mut ImportReport) -> Value {
    let mut out = Map::new();
    out.insert("op".to_owned(), json!("bin"));
    out.insert("field".to_owned(), json!(field));
    let output = transform
        .get("as")
        .and_then(Value::as_str)
        .map_or_else(|| format!("{field}__bin"), str::to_owned);
    out.insert("as".to_owned(), json!(output));
    if let Some(edges) = transform.get("exact").and_then(Value::as_array) {
        out.insert("method".to_owned(), json!("edges"));
        out.insert("edges".to_owned(), Value::Array(edges.clone()));
    } else {
        let count = transform
            .get("maxbins")
            .and_then(Value::as_u64)
            .unwrap_or(10);
        out.insert("count".to_owned(), json!(count));
        if transform.contains_key("step") {
            report.unmapped(
                &at.key("step"),
                "transform.bin.step",
                "a bin width is not in spec v0.1; `maxbins` sets the bin count",
            );
        }
        if transform.get("nice").and_then(Value::as_bool) != Some(false) {
            report.approximated(
                at,
                "transform.bin.nice",
                "Erie rounds bins to nice values by default; v0.1 equal-width bins span the data exactly",
            );
        }
    }
    if transform.contains_key("end") {
        report.unmapped(
            &at.key("end"),
            "transform.bin.end",
            "v0.1 bins give the bin start only",
        );
    }
    Value::Object(out)
}

/// Translates an Erie (JavaScript) expression to the SONARITHM expression subset:
/// `datum.f` and `datum["f"]` become field references, `===`/`!==` become `==`/`!=`,
/// `Math.f(` becomes `f(`, and single-quoted text becomes double-quoted. Always reported as
/// approximated, since the languages differ.
fn expression(expr: &str, at: &Ptr, construct: &str, report: &mut ImportReport) -> String {
    let mut out = String::with_capacity(expr.len());
    let mut rest = expr;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("datum.") {
            let end = after
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(after.len());
            out.push_str(&after[..end]);
            rest = &after[end..];
        } else if let Some(after) = rest.strip_prefix("datum[") {
            let quote = after.chars().next().filter(|c| *c == '"' || *c == '\'');
            let parsed = quote.and_then(|q| {
                let inner = &after[1..];
                let close = inner.find(q)?;
                let tail = inner[close + 1..].strip_prefix(']')?;
                Some((&inner[..close], tail))
            });
            if let Some((name, tail)) = parsed {
                out.push_str("col(\"");
                out.push_str(name);
                out.push_str("\")");
                rest = tail;
            } else {
                out.push_str("datum[");
                rest = after;
            }
        } else if let Some(after) = rest.strip_prefix("===") {
            out.push_str("==");
            rest = after;
        } else if let Some(after) = rest.strip_prefix("!==") {
            out.push_str("!=");
            rest = after;
        } else if let Some(after) = rest.strip_prefix("Math.") {
            rest = after;
        } else if let Some(after) = rest.strip_prefix('\'')
            && let Some(close) = after.find('\'')
        {
            // JavaScript text in single quotes; the expression language uses double quotes.
            out.push('"');
            out.push_str(&after[..close].replace('"', "\\\""));
            out.push('"');
            rest = &after[close + 1..];
        } else {
            let mut chars = rest.chars();
            if let Some(c) = chars.next() {
                out.push(c);
            }
            rest = chars.as_str();
        }
    }
    report.approximated(
        at,
        construct,
        format!("the JavaScript expression `{expr}` is translated to `{out}`; check it against the SONARITHM expression language"),
    );
    out
}

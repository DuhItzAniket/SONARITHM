// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `tone` and `glide` encoders (07 NUMISONA, `encode`): data rows become `Note` and
//! `Glide` events that keep the row they came from (`RowRef`) and its confidence.
//!
//! One track is encoded in four steps:
//!
//! 1. **Rows.** The track's `filter` selects rows (with the data layer's expression
//!    language); every event keeps the row's index in the original table.
//! 2. **Time.** Rows are ordered by the substrate's time field (or the field with
//!    `role: x`, or row order) and placed by the pace: `pointsPerSecond` and `duration`
//!    space the points evenly (`duration` is `n` points over the whole series), and
//!    `secondsPerUnit` places them in proportion to the time field.
//! 3. **Channels.** Each channel is a constant or a field through a scale ([`crate::scale`])
//!    and a range; pitch also goes through polarity ([`crate::polarity`]) and a pitch space
//!    ([`crate::pitch`]). `redundant` adds a tap-count or duration encoding of the pitch
//!    field (A11Y-13), and a nominal field on `timbre` picks one preset per category
//!    (SDR-25).
//! 4. **Events.** A `tone` track gets one note per row; a `glide` track one contour per run
//!    of rows without missing values, holding its last value for one step.
//!
//! Every default the encoder applies is reported as an `Info` diagnostic with its rule
//! (FR-026); the spec's own defaults (pace, pitch range and space, scale type) arrive
//! already applied and reported by `sonarithm-spec`.

mod channels;
mod columns;
mod events;

use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::Int64Type;
use arrow_array::{ArrayRef, Int64Array, RecordBatch};
use arrow_schema::{DataType, Field, Schema};
use sonarithm_core::diagnostic::{Diagnostic, Diagnostics, SpecPointer};
use sonarithm_core::engine::{
    Annotation, CompileContext, CompileError, Encoder, Engine, EngineManifest, MarkKind, Registrar,
    RegistryError, SpecFragment, TrackRequest,
};
use sonarithm_core::ids::{EngineId, RowRange};
use sonarithm_core::provenance::Confidence;
use sonarithm_core::score::{EngineVersion, InstrumentRef, PitchRange, ScoreTrack, TrackRole};
use sonarithm_core::units::Cents;
use sonarithm_data::{DataBundle, RowProvenance, Table};
use sonarithm_spec::data::{FieldSpec, Missing, Role};
use sonarithm_spec::sections::{Pace, Substrate};
use sonarithm_spec::track::{
    Channel, ChannelName, Domain as SpecDomain, DomainKeyword, Mark, PitchSpace as SpecSpace,
    Polarity as SpecPolarity, ScaleType, TrackSpec,
};
use sonarithm_spec::values::{ChannelRange, ChannelValue, UnitRange, UnitValue};

use self::channels::RangeOrigin;
use self::columns::Column;
use crate::codes;
use crate::pitch::{DEFAULT_RANGE_HZ, PitchMap, PitchSpace, PitchValue};
use crate::polarity::{
    ConceptOrigin, ConceptTable, Polarity, PolarityChoice, PolarityProfile, PolarityRequest,
    resolve,
};
use crate::scale::{
    ContinuousScale, DiscreteKind, DiscreteScale, Domain, OffScale, Position, Psychophysical,
    QuantileScale, Scale, ScaleKind, Transform, categories_in_order_of_appearance,
    sorted_categories,
};

/// NUMISONA's engine ID.
pub const ENGINE: EngineId = EngineId::from_static("numisona");

/// SDR-29 (DEFAULT): the default data instrument, a harmonic mallet-like timbre.
pub const DEFAULT_INSTRUMENT: &str = "mallet";

/// RFC-0001 §6: the default note duration, 0.25 s.
pub const DEFAULT_DURATION_S: f64 = 0.25;

/// SDR-23 (MUST) and RFC-0001 §6: loudness is constant, −18 dB, unless the spec maps it.
pub const DEFAULT_LOUDNESS_DB: f64 = -18.0;

/// RFC-0001 §6: the default brightness, 0.5.
pub const DEFAULT_BRIGHTNESS: f64 = 0.5;

/// Proposal (F-082, not evidence-backed): the duration range of a duration encoding with no
/// `range`, including the redundant one (A11Y-13). The low end is the ~100 ms event floor
/// behind SDR-20 (INFER); the high end fits inside the default inter-onset interval of
/// 1/3 s (SDR-19).
pub const PROPOSED_DURATION_RANGE_S: [f64; 2] = [0.1, 0.3];

/// RFC-0001 §6: tap counts run from 1 to 8; a tap encoding with no `range` uses all of it.
pub const TAPS_RANGE: [f64; 2] = [1.0, 8.0];

/// Proposal (F-082, not evidence-backed): ±6 dB around the default loudness, for a loudness
/// mapping with no `range` (SDR-23 warns against mapping loudness at all).
pub const PROPOSED_LOUDNESS_RANGE_DB: [f64; 2] = [-24.0, -12.0];

/// SDR-25 (DEFAULT): the presets nominal categories take, in order, after the track's own
/// instrument (the render presets of A05).
pub const TIMBRE_PALETTE: [&str; 5] = ["mallet", "square", "triangle", "saw", "sine"];

/// SDR-19 (DEFAULT): above about 8 discrete points per second the encoder warns.
pub const LINT_POINTS_PER_SECOND: f64 = 8.0;

/// SDR-20 (INFER, experimental): taps shorter than about 100 ms (more than about 10 per
/// second) may fuse and stop being countable.
pub const MIN_TAP_S: f64 = 0.1;

/// The column that carries original row numbers through a track filter.
const ROW_COLUMN: &str = "__numisona_row";

/// How often (in rows) the encoder checks for cancellation.
const CANCEL_EVERY: usize = 8192;

/// NUMISONA: registers the `tone` and `glide` encoders (07 summary matrix). The polarity
/// profile holds the listener's polarity choices (SDR-15, SDR-38); it is part of what the
/// Score depends on, and every choice it makes is reported as a diagnostic.
#[derive(Clone, Debug, Default)]
pub struct Numisona {
    profile: Arc<PolarityProfile>,
}

impl Numisona {
    /// The engine with an empty profile.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The engine with a listener's polarity profile.
    #[must_use]
    pub fn with_profile(profile: PolarityProfile) -> Self {
        Self {
            profile: Arc::new(profile),
        }
    }

    /// The engine's manifest.
    #[must_use]
    pub fn manifest() -> EngineManifest {
        let mut manifest = EngineManifest::new(
            ENGINE,
            "NUMISONA",
            EngineVersion(env!("CARGO_PKG_VERSION").to_owned()),
        );
        manifest.spec_keys = vec![
            "substrate".to_owned(),
            "tracks[].channels".to_owned(),
            "tracks[].redundant".to_owned(),
        ];
        manifest
    }
}

impl<D: Borrow<DataBundle> + 'static> Engine<D> for Numisona {
    fn manifest(&self) -> EngineManifest {
        Self::manifest()
    }

    fn register(&self, registrar: &mut Registrar<'_, D>) -> Result<(), RegistryError> {
        let encoder: Arc<dyn Encoder<D>> = Arc::new(TrackEncoder {
            profile: Arc::clone(&self.profile),
        });
        registrar.encoder(&[MarkKind::Tone, MarkKind::Glide], &encoder)?;
        registrar.default_encoder(MarkKind::Tone)?;
        registrar.default_encoder(MarkKind::Glide)
    }
}

/// The encoder NUMISONA registers.
struct TrackEncoder {
    profile: Arc<PolarityProfile>,
}

impl<D: Borrow<DataBundle>> Encoder<D> for TrackEncoder {
    fn encode(
        &self,
        track: &TrackRequest,
        data: &D,
        _annotations: &[Annotation],
        cx: &mut CompileContext,
    ) -> Result<ScoreTrack, CompileError> {
        encode(track, data.borrow(), &self.profile, cx)
    }
}

/// Encodes one `tone` or `glide` track from the bundle's primary table.
///
/// # Errors
///
/// A [`CompileError::Diagnostic`] with an `SNR-ENG` code when the track cannot be encoded
/// (no table, an unknown field, a field no scale can read, a scale that cannot be fitted);
/// [`CompileError::Cancelled`] when the compile is cancelled.
pub fn encode(
    request: &TrackRequest,
    bundle: &DataBundle,
    profile: &PolarityProfile,
    cx: &mut CompileContext,
) -> Result<ScoreTrack, CompileError> {
    let track: TrackSpec = request.spec.parse()?;
    let at = request.spec.pointer().clone();
    let table = bundle
        .primary()
        .ok_or_else(|| error(&codes::NO_TABLE, "the data has no table to encode", &at))?;
    let concepts = ConceptTable::builtin().map_err(|err| CompileError::Engine {
        engine: ENGINE,
        message: err.to_string(),
    })?;
    let mut run = Run {
        request,
        track,
        fields: request
            .fields
            .as_ref()
            .map(SpecFragment::parse)
            .transpose()?
            .unwrap_or_default(),
        missing: request
            .missing
            .as_ref()
            .map(SpecFragment::parse)
            .transpose()?
            .unwrap_or(Missing::Silence),
        table,
        at,
        concepts,
        profile,
        diagnostics: Diagnostics::new(),
    };
    let substrate: Option<Substrate> = request
        .substrate
        .as_ref()
        .map(SpecFragment::parse)
        .transpose()?;
    let result = run.encode(substrate.as_ref(), cx);
    for diagnostic in run.diagnostics {
        cx.report(diagnostic);
    }
    result
}

fn error(code: &sonarithm_core::codes::CodeInfo, message: &str, at: &SpecPointer) -> CompileError {
    CompileError::from(Diagnostic::new(code, format!("NUMISONA: {message}")).at(at.clone()))
}

/// One track being encoded.
struct Run<'a> {
    request: &'a TrackRequest,
    track: TrackSpec,
    fields: BTreeMap<String, FieldSpec>,
    missing: Missing,
    table: &'a Table,
    at: SpecPointer,
    concepts: &'a ConceptTable,
    profile: &'a PolarityProfile,
    diagnostics: Diagnostics,
}

/// The rows of a track in play order, with their onsets.
struct Timeline {
    rows: Vec<usize>,
    onsets: Vec<f64>,
    /// The nominal step between points, used to hold a glide's last value.
    step: f64,
}

/// A channel's values in output units, one per row in play order.
#[derive(Clone, Debug, PartialEq)]
enum Values<T> {
    Constant(T),
    PerRow(Vec<Option<T>>),
}

impl<T: Clone> Values<T> {
    fn get(&self, index: usize) -> Option<T> {
        match self {
            Self::Constant(value) => Some(value.clone()),
            Self::PerRow(values) => values.get(index).cloned().flatten(),
        }
    }
}

/// The resolved pitch channel.
struct PitchChannel {
    values: Values<Cents>,
    range: Option<PitchRange>,
    /// The scale positions of the pitch field before polarity, for redundant encodings.
    positions: Option<Vec<Option<f64>>>,
}

impl Run<'_> {
    fn channel_at(&self, name: ChannelName) -> SpecPointer {
        self.at.key("channels").key(name.key())
    }

    fn encode(
        &mut self,
        substrate: Option<&Substrate>,
        cx: &mut CompileContext,
    ) -> Result<ScoreTrack, CompileError> {
        let rows = self.select_rows()?;
        cx.check_cancelled()?;
        let timeline = self.timeline(rows, substrate)?;
        let instrument = self.instrument();
        let mut out = ScoreTrack::new(
            self.request.id.clone(),
            TrackRole::Data,
            ENGINE,
            InstrumentRef(instrument.clone()),
        );
        let channels = self.track.channels.clone();
        let pitch = self.pitch(channels.pitch.as_ref(), &timeline)?;
        out.pitch_range = pitch.range;
        let glide = self.track.mark == Mark::Glide;
        let loudness = self.loudness(channels.loudness.as_ref(), &timeline)?;
        let pan = self.scalar(
            ChannelName::Pan,
            channels.pan.as_ref(),
            &timeline,
            0.0,
            [-1.0, 1.0],
            Some(RangeOrigin::Bounds),
        )?;
        let brightness = self.scalar(
            ChannelName::Brightness,
            channels.brightness.as_ref(),
            &timeline,
            DEFAULT_BRIGHTNESS,
            [0.0, 1.0],
            Some(RangeOrigin::Bounds),
        )?;
        let timbre = self.timbre(channels.timbre.as_ref(), &timeline, &instrument)?;
        self.spatial_checks(&channels);
        if channels.rate.is_some() {
            self.diagnostics.push(
                Diagnostic::new(
                    &codes::CHANNEL_NOT_RENDERED,
                    format!(
                        "NUMISONA: track `{}`: the `rate` channel has no place in Score IR version 1, so it is not encoded yet.",
                        self.request.id
                    ),
                )
                .at(self.channel_at(ChannelName::Rate)),
            );
        }
        let provenance = RowProvenance::read(self.table).ok();
        let source = Sources {
            table: self.table,
            provenance: provenance.as_ref(),
        };
        if glide {
            self.ignore_tone_only(&channels);
            out.events =
                events::glides(&timeline, &pitch, &loudness, &pan, &brightness, &source, cx)?;
        } else {
            let duration = self.duration(channels.duration.as_ref(), &timeline, &pitch)?;
            let taps = self.taps(channels.taps.as_ref(), &timeline, &pitch)?;
            let parts = NoteParts {
                pitch: &pitch.values,
                duration: &duration,
                taps: &taps,
                loudness: &loudness,
                pan: &pan,
                brightness: &brightness,
                timbre: &timbre,
            };
            out.events = self.notes(&timeline, &parts, &source, cx)?;
        }
        Ok(out)
    }

    /// The rows the track's `filter` keeps, in table order; every row without one.
    fn select_rows(&mut self) -> Result<Vec<usize>, CompileError> {
        let count = usize::try_from(self.table.rows().len()).unwrap_or(usize::MAX);
        let Some(filter) = self.track.filter.clone() else {
            return Ok((0..count).collect());
        };
        let at = self.at.key("filter").key("expr");
        let failed = |message: String| error(&codes::FILTER_FAILED, &message, &at);
        let batch = self
            .table
            .to_batch()
            .map_err(|err| failed(err.to_string()))?;
        let schema = batch.schema();
        let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();
        fields.push(Arc::new(Field::new(ROW_COLUMN, DataType::Int64, false)));
        let tagged_schema = Arc::new(Schema::new(fields).with_metadata(schema.metadata().clone()));
        let numbers = Int64Array::from_iter_values(0..i64::try_from(count).unwrap_or(i64::MAX));
        let mut columns: Vec<ArrayRef> = batch.columns().to_vec();
        columns.push(Arc::new(numbers));
        let tagged = RecordBatch::try_new(Arc::clone(&tagged_schema), columns)
            .map_err(|err| failed(err.to_string()))?;
        let tagged = self
            .table
            .next_generation(tagged_schema, vec![tagged])
            .map_err(|err| failed(err.to_string()))?;
        let mut ignored = Diagnostics::new();
        let transform = sonarithm_data::Transform::Filter {
            expr: filter.expr.clone(),
        };
        let kept = sonarithm_data::apply(&tagged, &transform, &mut ignored).map_err(|err| {
            failed(format!(
                "the track filter `{}` cannot be applied: {err}",
                filter.expr
            ))
        })?;
        let rows = kept
            .column(ROW_COLUMN)
            .map_err(|err| failed(err.to_string()))?;
        let rows = rows
            .as_primitive_opt::<Int64Type>()
            .map(|rows| {
                rows.values()
                    .iter()
                    .filter_map(|&row| usize::try_from(row).ok())
                    .collect()
            })
            .unwrap_or_default();
        Ok(rows)
    }

    /// The time field: the substrate's, else the field with `role: x`, else none.
    fn time_field(&mut self, substrate: Option<&Substrate>) -> Option<String> {
        let time_at = SpecPointer::root()
            .key("substrate")
            .key("time")
            .key("field");
        if let Some(field) = substrate
            .and_then(|s| s.time.as_ref())
            .and_then(|t| t.field.clone())
        {
            return Some(field);
        }
        let declared = self
            .fields
            .iter()
            .find(|(_, spec)| spec.role == Some(Role::X))
            .map(|(name, _)| name.clone());
        let inferred = || {
            self.table
                .schema()
                .fields()
                .iter()
                .find(|field| {
                    self.table
                        .field_meta(field.name())
                        .is_ok_and(|meta| meta.role == Some(sonarithm_data::Role::X))
                })
                .map(|field| field.name().clone())
        };
        let found = declared.or_else(inferred);
        let message = found.as_ref().map_or_else(
            || "time follows the row order, because the substrate names no time field and no field has role x.".to_owned(),
            |field| format!("time follows field `{field}`, which has role x (RFC-0001 §5)."),
        );
        self.report(&codes::DEFAULT_TIME, &message, time_at);
        found
    }

    /// Orders the rows by time and places them by the pace.
    fn timeline(
        &mut self,
        mut rows: Vec<usize>,
        substrate: Option<&Substrate>,
    ) -> Result<Timeline, CompileError> {
        let field = self.time_field(substrate);
        let mut keys: Option<Vec<f64>> = None;
        if let Some(field) = &field
            && let Column::Numbers(values) = self.column(field, &self.at.clone())?
        {
            let before = rows.len();
            rows.retain(|&row| values.get(row).is_some_and(|v| !v.is_nan()));
            if rows.len() < before {
                let message = format!(
                    "{} rows have no value in time field `{field}` and are left out.",
                    before - rows.len()
                );
                self.report(
                    &codes::ROWS_WITHOUT_TIME,
                    &message,
                    SpecPointer::root()
                        .key("substrate")
                        .key("time")
                        .key("field"),
                );
            }
            rows.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
            keys = Some(rows.iter().map(|&row| values[row]).collect());
        }
        let pace = substrate
            .and_then(|s| s.time.as_ref())
            .and_then(|t| t.pace)
            .unwrap_or(Pace::PointsPerSecond(3.0));
        let n = rows.len();
        #[expect(clippy::cast_precision_loss, reason = "row counts stay far below 2^52")]
        let (onsets, step): (Vec<f64>, f64) = match pace {
            Pace::PointsPerSecond(rate) => ((0..n).map(|k| k as f64 / rate).collect(), 1.0 / rate),
            Pace::Duration(total) => {
                let total = total.seconds();
                let count = n.max(1) as f64;
                (
                    (0..n).map(|k| k as f64 * total / count).collect(),
                    total / count,
                )
            }
            Pace::SecondsPerUnit(seconds) => {
                if let Some(keys) = &keys {
                    let first = keys.first().copied().unwrap_or(0.0);
                    (
                        keys.iter().map(|x| (x - first) * seconds).collect(),
                        seconds,
                    )
                } else {
                    let message = "`secondsPerUnit` needs a numeric time field; each row counts as one unit instead.";
                    self.report(
                        &codes::PACE_WITHOUT_UNITS,
                        message,
                        SpecPointer::root().key("substrate").key("time").key("pace"),
                    );
                    ((0..n).map(|k| k as f64 * seconds).collect(), seconds)
                }
            }
        };
        self.lint_pace(&onsets);
        Ok(Timeline { rows, onsets, step })
    }

    /// SDR-19: discrete marks above about 8 points per second.
    fn lint_pace(&mut self, onsets: &[f64]) {
        if self.track.mark != Mark::Tone || onsets.len() < 2 {
            return;
        }
        let span = onsets[onsets.len() - 1] - onsets[0];
        #[expect(clippy::cast_precision_loss, reason = "row counts stay far below 2^52")]
        let rate = (onsets.len() - 1) as f64 / span;
        if span > 0.0 && rate > LINT_POINTS_PER_SECOND {
            let message = format!(
                "the tones come at {rate:.1} per second, above the about 8 per second where estimates degrade (SDR-19). Slow the pace, or use `glide` (SDR-20)."
            );
            self.diagnostics.push(
                Diagnostic::new(
                    &codes::PACE_ABOVE_LINT,
                    format!("NUMISONA: track `{}`: {message}", self.request.id),
                )
                .at(SpecPointer::root().key("substrate").key("time").key("pace")),
            );
        }
    }

    /// The track instrument: the spec's, or the mallet default (SDR-29).
    fn instrument(&mut self) -> String {
        if let Some(instrument) = self.track.instrument.clone() {
            return instrument;
        }
        let message = format!(
            "the instrument is `{DEFAULT_INSTRUMENT}`, the harmonic mallet-like default (SDR-29)."
        );
        self.report(
            &codes::DEFAULT_INSTRUMENT,
            &message,
            self.at.key("instrument"),
        );
        DEFAULT_INSTRUMENT.to_owned()
    }

    /// A field's column.
    fn column(&self, field: &str, at: &SpecPointer) -> Result<Column, CompileError> {
        let array = self.table.column(field).map_err(|_| {
            error(&codes::UNKNOWN_FIELD, &format!(
                    "track `{}`: the data has no field `{field}`. Check the spelling against the data's columns.",
                    self.request.id
                ),
                at,
            )
        })?;
        columns::read(&array).map_err(|kind| {
            error(
                &codes::UNSUPPORTED_FIELD,
                &format!(
                    "track `{}`: field `{field}` holds {kind} values, which no channel reads.",
                    self.request.id
                ),
                at,
            )
        })
    }

    /// The field's concept: from the spec's `data.fields`, else from the data's metadata.
    fn concept(&self, field: &str) -> Option<(String, ConceptOrigin)> {
        if let Some(concept) = self.fields.get(field).and_then(|f| f.concept.clone()) {
            return Some((concept, ConceptOrigin::Spec));
        }
        self.table
            .field_meta(field)
            .ok()
            .and_then(|meta| meta.concept)
            .map(|concept| (concept.as_str().to_owned(), ConceptOrigin::Data))
    }

    /// The scale positions of a field's values, one per row in play order, with the
    /// off-scale and missing values reported.
    fn positions(
        &mut self,
        name: ChannelName,
        channel: &Channel,
        field: &str,
        timeline: &Timeline,
    ) -> Result<Vec<Option<Position>>, CompileError> {
        let at = self.channel_at(name);
        let column = self.column(field, &at.key("field"))?;
        let kind = scale_kind(channel);
        let scale = self.fit(&column, kind, channel, field, timeline, &at)?;
        let positions: Vec<Option<Position>> = timeline
            .rows
            .iter()
            .map(|&row| match (&column, &scale) {
                (Column::Numbers(values), scale) => scale.position(values[row]),
                (Column::Categories { values, .. }, Scale::Discrete(discrete)) => {
                    values[row].as_ref().and_then(|c| discrete.position(c))
                }
                (Column::Categories { .. }, _) => None,
            })
            .collect();
        let positions = self.fill_missing(positions, name, field);
        self.report_off_scale(&positions, timeline, name, field);
        Ok(positions)
    }

    /// Fits the channel's scale to the track's rows.
    fn fit(
        &mut self,
        column: &Column,
        kind: ScaleKind,
        channel: &Channel,
        field: &str,
        timeline: &Timeline,
        at: &SpecPointer,
    ) -> Result<Scale, CompileError> {
        let domain = domain(channel);
        let nice = channel.scale.as_ref().and_then(|s| s.nice).unwrap_or(false);
        let fitted = match column {
            Column::Numbers(values) => {
                let used: Vec<f64> = timeline.rows.iter().map(|&row| values[row]).collect();
                match kind {
                    ScaleKind::Quantile => QuantileScale::fit(&used).map(Scale::Quantile),
                    ScaleKind::Band | ScaleKind::Ordinal => {
                        Scale::fit_numeric(kind, domain, nice, &used)
                    }
                    _ => {
                        let transform = kind.transform().unwrap_or(Transform::Linear);
                        ContinuousScale::fit(transform, domain, nice, &used).map(Scale::Continuous)
                    }
                }
            }
            Column::Categories { values, order } => {
                let discrete = match kind {
                    ScaleKind::Band => DiscreteKind::Band,
                    ScaleKind::Ordinal => DiscreteKind::Ordinal,
                    _ => {
                        return Err(error(
                            &codes::FIELD_NOT_NUMERIC,
                            &format!(
                                "track `{}`: field `{field}` holds categories, which a continuous scale cannot place. Use a `band` or `ordinal` scale, or map the field to `timbre` (SDR-25).",
                                self.request.id
                            ),
                            &at.key("field"),
                        ));
                    }
                };
                let used = timeline.rows.iter().filter_map(|&row| values[row].clone());
                let categories = match (discrete, order) {
                    (DiscreteKind::Ordinal, Some(order)) => order.clone(),
                    (DiscreteKind::Ordinal, None) => sorted_categories(used),
                    (DiscreteKind::Band, _) => categories_in_order_of_appearance(used),
                };
                DiscreteScale::new(discrete, categories).map(Scale::Discrete)
            }
        };
        fitted.map_err(|err| {
            error(
                &codes::SCALE_FAILED,
                &format!(
                    "track `{}`: the scale of field `{field}` cannot be fitted: {err}.",
                    self.request.id
                ),
                &at.key("scale"),
            )
        })
    }

    /// Applies `data.missing` to one channel's positions (03 §5): silence keeps the gap,
    /// interpolation fills it linearly between the nearest values in play order.
    fn fill_missing(
        &mut self,
        mut positions: Vec<Option<Position>>,
        name: ChannelName,
        field: &str,
    ) -> Vec<Option<Position>> {
        let missing = positions.iter().filter(|p| p.is_none()).count();
        if missing == 0 {
            return positions;
        }
        let interpolate = self.missing == Missing::Interpolate;
        if interpolate {
            let known: Vec<usize> = (0..positions.len())
                .filter(|&i| positions[i].is_some())
                .collect();
            for pair in known.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let (Some(pa), Some(pb)) = (positions[a], positions[b]) else {
                    continue;
                };
                for (offset, slot) in positions[a + 1..b].iter_mut().enumerate() {
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "row counts stay far below 2^52"
                    )]
                    let w = (offset + 1) as f64 / (b - a) as f64;
                    *slot = Some(Position::within(pa.u + w * (pb.u - pa.u)));
                }
            }
        }
        let filled = missing - positions.iter().filter(|p| p.is_none()).count();
        let treatment = match self.missing {
            Missing::Interpolate => "interpolated where neighbours exist, silent at the ends",
            Missing::Earcon => "silent; the gap earcon arrives with AUDAXIS (B03)",
            _ => "silent",
        };
        let message = format!(
            "{missing} rows have no value in field `{field}` for `{}` ({filled} filled): they are {treatment} (03 §5).",
            name.key()
        );
        self.report(
            &codes::MISSING_VALUES,
            &message,
            SpecPointer::root().key("data").key("missing"),
        );
        positions
    }

    fn report_off_scale(
        &mut self,
        positions: &[Option<Position>],
        timeline: &Timeline,
        name: ChannelName,
        field: &str,
    ) {
        let off: Vec<usize> = positions
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_some_and(|p| p.off_scale != OffScale::Within))
            .map(|(i, _)| timeline.rows[i])
            .collect();
        if off.is_empty() {
            return;
        }
        let message = format!(
            "NUMISONA: track `{}`: {} values of field `{field}` lie outside the `{}` domain and sound at its nearest end (03 §5).",
            self.request.id,
            off.len(),
            name.key()
        );
        let mut diagnostic =
            Diagnostic::new(&codes::OFF_SCALE, message).at(self.channel_at(name).key("scale"));
        if let (Some(&first), Some(&last)) = (off.iter().min(), off.iter().max()) {
            let range = self.table.rows();
            diagnostic = diagnostic.rows(RowRange {
                start: first as u64,
                end: last as u64 + 1,
                ..range
            });
        }
        self.diagnostics.push(diagnostic);
    }

    /// The pitch channel (SDR-07..15).
    fn pitch(
        &mut self,
        channel: Option<&Channel>,
        timeline: &Timeline,
    ) -> Result<PitchChannel, CompileError> {
        let at = self.channel_at(ChannelName::Pitch);
        let Some(channel) = channel else {
            let middle = PitchMap::default_range().pitch(0.5);
            let message = format!(
                "the track maps nothing to pitch, so every note sounds at {:.1} Hz, the middle of the default range (SDR-08).",
                middle.to_hz().0
            );
            self.report(&codes::DEFAULT_PITCH, &message, at);
            return Ok(PitchChannel {
                values: Values::Constant(middle),
                range: None,
                positions: None,
            });
        };
        if let Some(value) = &channel.value {
            let cents = pitch_value(value)
                .map_or_else(|| PitchMap::default_range().pitch(0.5), PitchValue::cents);
            return Ok(PitchChannel {
                values: Values::Constant(cents),
                range: None,
                positions: None,
            });
        }
        let Some(field) = channel.field.clone() else {
            return Ok(PitchChannel {
                values: Values::Constant(PitchMap::default_range().pitch(0.5)),
                range: None,
                positions: None,
            });
        };
        let concept = self.concept(&field);
        let choice = match channel.polarity {
            Some(SpecPolarity::Positive) => PolarityChoice::Positive,
            Some(SpecPolarity::Negative) => PolarityChoice::Negative,
            _ => PolarityChoice::Auto,
        };
        let request = PolarityRequest {
            field: &field,
            concept: concept
                .as_ref()
                .map(|(name, origin)| (name.as_str(), *origin)),
            choice,
        };
        let (resolved, explained) = resolve(self.concepts, self.profile, &request, &at);
        self.diagnostics.extend(explained);
        let polarity = resolved.polarity;
        if channel.scale.as_ref().and_then(|s| s.scale_type) == Some(ScaleType::Psychophysical) {
            return self.psychophysical(channel, &field, polarity, concept.as_ref(), timeline);
        }
        let map = pitch_map(channel).map_err(|err| {
            error(
                &codes::SCALE_FAILED,
                &format!("track `{}`: {err}.", self.request.id),
                &at.key("range"),
            )
        })?;
        let positions = self.positions(ChannelName::Pitch, channel, &field, timeline)?;
        let values = positions
            .iter()
            .map(|p| p.map(|p| map.pitch(polarity.apply(p.u))))
            .collect();
        Ok(PitchChannel {
            values: Values::PerRow(values),
            range: Some(map.range()),
            positions: Some(positions.iter().map(|p| p.map(|p| p.u)).collect()),
        })
    }

    /// A psychophysical pitch mapping (SDR-11), clamped to the channel's range.
    fn psychophysical(
        &mut self,
        channel: &Channel,
        field: &str,
        polarity: Polarity,
        concept: Option<&(String, ConceptOrigin)>,
        timeline: &Timeline,
    ) -> Result<PitchChannel, CompileError> {
        let at = self.channel_at(ChannelName::Pitch);
        let scale = channel.scale.as_ref();
        let exponent = scale.and_then(|s| s.exponent).unwrap_or(f64::NAN);
        let reference = scale.and_then(|s| s.reference);
        let law = Psychophysical::new(
            exponent,
            reference.map_or(f64::NAN, |r| r.value),
            reference.map_or(f64::NAN, |r| r.hz),
        )
        .map_err(|err| {
            error(
                &codes::SCALE_FAILED,
                &format!("track `{}`: {err}.", self.request.id),
                &at.key("scale"),
            )
        })?;
        let column = self.column(field, &at.key("field"))?;
        let Column::Numbers(values) = column else {
            return Err(error(
                &codes::FIELD_NOT_NUMERIC,
                &format!(
                    "track `{}`: a psychophysical scale needs numbers, and field `{field}` holds categories.",
                    self.request.id
                ),
                &at.key("field"),
            ));
        };
        let range = if channel.range.is_some() {
            pitch_map(channel)
                .map_err(|err| {
                    error(
                        &codes::SCALE_FAILED,
                        &format!("track `{}`: {err}.", self.request.id),
                        &at.key("range"),
                    )
                })?
                .range()
        } else {
            let used: Vec<f64> = timeline.rows.iter().map(|&row| values[row]).collect();
            let domain = ContinuousScale::fit(Transform::Linear, Domain::Auto, false, &used)
                .map_or([f64::NAN; 2], |s| s.domain());
            law.range_for(domain).map_err(|err| {
                error(
                    &codes::SCALE_FAILED,
                    &format!("track `{}`: {err}.", self.request.id),
                    &at.key("scale"),
                )
            })?
        };
        let hint = concept
            .and_then(|(name, _)| self.concepts.get(name).map(|row| (name, row.exponent)))
            .and_then(|(name, m)| {
                m.map(|m| format!(" The concept table suggests m = {m} for `{name}`."))
            })
            .unwrap_or_default();
        let message = format!(
            "pitch follows f = {:.1} Hz · (d / {})^(1/{exponent}); {}.{hint}",
            reference.map_or(f64::NAN, |r| r.hz),
            reference.map_or(f64::NAN, |r| r.value),
            crate::scale::GUIDANCE
        );
        self.report(&codes::PSYCHOPHYSICAL, &message, at.key("scale"));
        let mut positions = Vec::with_capacity(timeline.rows.len());
        let mut off = 0_usize;
        let pitches = timeline
            .rows
            .iter()
            .map(|&row| {
                let pitch = law.pitch(values[row], range)?;
                off += usize::from(pitch.off_scale != OffScale::Within);
                let cents = match polarity {
                    Polarity::Positive => pitch.pitch,
                    Polarity::Negative => range.mirror(pitch.pitch),
                };
                let span = range.high.0 - range.low.0;
                positions.push(Some(if span > 0.0 {
                    (pitch.pitch.0 - range.low.0) / span
                } else {
                    0.5
                }));
                Some(cents)
            })
            .collect::<Vec<_>>();
        if off > 0 {
            let message = format!(
                "NUMISONA: track `{}`: {off} values of field `{field}` fall outside the pitch range under the psychophysical law and sound at its nearest end (03 §5).",
                self.request.id
            );
            self.diagnostics
                .push(Diagnostic::new(&codes::OFF_SCALE, message).at(at.key("range")));
        }
        Ok(PitchChannel {
            values: Values::PerRow(pitches),
            range: Some(range),
            positions: Some(positions),
        })
    }
}

/// The scale type of a channel; `linear` by default (RFC-0001 §6).
fn scale_kind(channel: &Channel) -> ScaleKind {
    let scale = channel.scale.as_ref();
    match scale.and_then(|s| s.scale_type) {
        Some(ScaleType::Log) => ScaleKind::Log,
        Some(ScaleType::Symlog) => ScaleKind::Symlog,
        Some(ScaleType::Sqrt) => ScaleKind::Sqrt,
        Some(ScaleType::Pow) => ScaleKind::Pow(scale.and_then(|s| s.exponent).unwrap_or(1.0)),
        Some(ScaleType::Quantile) => ScaleKind::Quantile,
        Some(ScaleType::Band) => ScaleKind::Band,
        Some(ScaleType::Ordinal) => ScaleKind::Ordinal,
        _ => ScaleKind::Linear,
    }
}

/// The scale domain of a channel; `auto` by default.
fn domain(channel: &Channel) -> Domain {
    match channel.scale.as_ref().and_then(|s| s.domain) {
        Some(SpecDomain::Range(range)) => Domain::Explicit(range),
        Some(SpecDomain::Keyword(DomainKeyword::ZeroBased)) => Domain::ZeroBased,
        _ => Domain::Auto,
    }
}

/// A constant pitch value in a pitch unit.
fn pitch_value(value: &ChannelValue) -> Option<PitchValue> {
    match value {
        ChannelValue::Unit(UnitValue::Hz(hz)) => Some(PitchValue::Hz(*hz)),
        ChannelValue::Unit(UnitValue::Midi(midi)) => Some(PitchValue::Midi(*midi)),
        ChannelValue::Unit(UnitValue::Cents(cents)) => Some(PitchValue::Cents(*cents)),
        _ => None,
    }
}

/// The pitch mapping of a channel: its space and range, or the defaults (SDR-08, SDR-09),
/// which the spec's resolved form normally writes out already.
fn pitch_map(channel: &Channel) -> Result<PitchMap, crate::pitch::PitchError> {
    let space = match channel.space {
        Some(SpecSpace::LinearFrequency) => PitchSpace::LinearFrequency,
        Some(SpecSpace::Mel) => PitchSpace::Mel,
        Some(SpecSpace::Erb) => PitchSpace::Erb,
        Some(SpecSpace::Bark) => PitchSpace::Bark,
        _ => PitchSpace::LogFrequency,
    };
    let [low, high] = DEFAULT_RANGE_HZ;
    let (low, high) = match channel.range {
        Some(ChannelRange::Unit(UnitRange::Midi([a, b]))) => {
            (PitchValue::Midi(a), PitchValue::Midi(b))
        }
        Some(ChannelRange::Unit(UnitRange::Cents([a, b]))) => {
            (PitchValue::Cents(a), PitchValue::Cents(b))
        }
        Some(ChannelRange::Unit(UnitRange::Hz([a, b]))) => (PitchValue::Hz(a), PitchValue::Hz(b)),
        _ => (PitchValue::Hz(low), PitchValue::Hz(high)),
    };
    PitchMap::new(space, low, high)
}

/// Where events point back to.
struct Sources<'a> {
    table: &'a Table,
    provenance: Option<&'a RowProvenance>,
}

impl Sources<'_> {
    fn confidence(&self, row: usize) -> Option<Confidence> {
        self.provenance.and_then(|p| p.get(row).confidence)
    }
}

/// The per-row channel values of a tone track.
struct NoteParts<'a> {
    pitch: &'a Values<Cents>,
    duration: &'a Values<f64>,
    taps: &'a Values<f64>,
    loudness: &'a Values<f64>,
    pan: &'a Values<f64>,
    brightness: &'a Values<f64>,
    timbre: &'a Values<String>,
}

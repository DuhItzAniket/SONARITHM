// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The announcement composer (08 §4, keyboard-and-navigation §8).
//!
//! An announcement is a list of parts (keyboard §8.1): value, x, series, position, level,
//! tags, provenance and hint. The listener chooses terse or verbose and the order of the
//! parts (A11Y-12). Each part is one catalogue message, and `announce-join` puts parts
//! together, so every announcement is one message as the pseudo-locale check requires.
//!
//! - **Units.** Speech uses spoken unit names (terse or verbose); the display text, for
//!   the log, the readout and braille, uses symbols. Both come from the same value and
//!   precision (DL-P12).
//! - **Provenance.** A value that is not stated plainly (03 §4 rule 3) always carries its
//!   qualifier: the part can be moved but not removed (keyboard §8.1, AGENTS rule 7).
//! - **Series.** The series name is left out when it has not changed since the last
//!   announcement (Zong et al. 2022).
//! - **Hints.** Verbose only, on the first visit to a kind of node.

use std::collections::BTreeSet;

use sonarithm_core::nav::{
    AxisName, NavKind, NavNode, NavNodeId, NavTree, Polarity, XType, XValue,
};
use sonarithm_core::provenance::Provenance;
use sonarithm_i18n::{
    Args, DateLength, DatePattern, I18nError, Localizer, Number, NumberOptions, UnitRef, UnitStyle,
};
use sonarithm_spec::sections::LegendPart;

use crate::earcons::EarconId;
use crate::nav::cursor::{Cursor, Edge, Invalid, Move, Outcome, WhereAmI};

/// How much an announcement says (A11Y-12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Verbosity {
    /// The value and little else.
    Terse,
    /// The full description.
    Verbose,
}

/// A part of an announcement (keyboard §8.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Part {
    /// The series name, when it changed.
    Series,
    /// The x label, date or category.
    X,
    /// The value with its unit.
    Value,
    /// "195 of 365".
    Position,
    /// The level on a level change, or the segment a step entered.
    Level,
    /// Insights at this node, and "nearest".
    Tags,
    /// The qualifier of a value that is not stated plainly. Always present when it applies.
    Provenance,
    /// A key hint, verbose only, on the first visit to a kind of node.
    Hint,
}

/// The key names hints speak. They follow the keymap, which the listener may change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyNames {
    /// The key that moves into a node (Down).
    pub explore: String,
    /// The key that activates a node (Enter).
    pub activate: String,
}

impl Default for KeyNames {
    fn default() -> Self {
        Self {
            explore: "Down".to_owned(),
            activate: "Enter".to_owned(),
        }
    }
}

/// The listener's announcement settings (A11Y-12; profiles in B03-T08).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Terse or verbose.
    pub verbosity: Verbosity,
    /// The part order at terse.
    pub terse: Vec<Part>,
    /// The part order at verbose.
    pub verbose: Vec<Part>,
    /// The key names hints use.
    pub keys: KeyNames,
}

impl Default for Settings {
    /// Keyboard §8.1 defaults. Terse: value, x, tags, provenance. Verbose: series, x,
    /// value, position, tags, provenance, hint. The level part leads both, so a level
    /// change is heard first.
    fn default() -> Self {
        Self {
            verbosity: Verbosity::Terse,
            terse: vec![
                Part::Level,
                Part::Value,
                Part::X,
                Part::Tags,
                Part::Provenance,
            ],
            verbose: vec![
                Part::Level,
                Part::Series,
                Part::X,
                Part::Value,
                Part::Position,
                Part::Tags,
                Part::Provenance,
                Part::Hint,
            ],
            keys: KeyNames::default(),
        }
    }
}

impl Settings {
    /// The part order for `verbosity`, with the provenance part added at the end if the
    /// listener removed it: it can be moved, never removed.
    #[must_use]
    pub fn order(&self, verbosity: Verbosity) -> Vec<Part> {
        let mut order = match verbosity {
            Verbosity::Terse => self.terse.clone(),
            Verbosity::Verbose => self.verbose.clone(),
        };
        let mut seen = BTreeSet::new();
        order.retain(|part| seen.insert(*part));
        if !order.contains(&Part::Provenance) {
            order.push(Part::Provenance);
        }
        order
    }
}

/// The queue class of an announcement (keyboard §9.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Class {
    /// Point, segment, series and lateral moves: only the latest survives.
    Navigation,
    /// Where am I, query answers, descriptions: superseded by the next command.
    Response,
    /// Modes, exports, errors: queued, never superseded by navigation.
    Status,
    /// Stream events: low priority, rate-limited (B06).
    Stream,
    /// The text equivalent of a boundary or invalid-action earcon.
    EarconText,
}

/// How the screen reader should treat it (08 §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Politeness {
    /// Navigation and status.
    Polite,
    /// Errors, device loss and true alarms only.
    Assertive,
}

/// A composed announcement.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Announcement {
    /// The queue class.
    pub class: Class,
    /// Polite or assertive.
    pub politeness: Politeness,
    /// The text to speak, with spoken unit names.
    pub speech: String,
    /// The same announcement with unit symbols, for the log, the readout and braille.
    pub display: String,
    /// The node it describes, for the announcement history (A11Y-24).
    pub node: Option<NavNodeId>,
    /// The earcon that goes with it.
    pub earcon: Option<EarconId>,
    /// Whether it is spoken; terse earcon texts only go to the log (keyboard §9.1).
    pub speak: bool,
    /// Whether it reports an error; errors are never dropped.
    pub error: bool,
}

impl Announcement {
    /// A status announcement from text the app already localized, such as "Export
    /// finished".
    #[must_use]
    pub fn status(speech: impl Into<String>, display: impl Into<String>) -> Self {
        Self {
            class: Class::Status,
            politeness: Politeness::Polite,
            speech: speech.into(),
            display: display.into(),
            node: None,
            earcon: None,
            speak: true,
            error: false,
        }
    }

    /// An error announcement: assertive, and never dropped.
    #[must_use]
    pub fn error(speech: impl Into<String>, display: impl Into<String>) -> Self {
        Self {
            politeness: Politeness::Assertive,
            error: true,
            ..Self::status(speech, display)
        }
    }
}

/// Which context cues the legend says are on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LegendCues {
    /// The tick track (SDR-16).
    pub ticks: bool,
    /// The reference tone (SDR-17).
    pub reference: bool,
}

fn yes(on: bool) -> &'static str {
    if on { "yes" } else { "no" }
}

/// The pieces of one announcement before ordering: a leading name and the parts.
#[derive(Default)]
struct Pieces {
    name: Option<String>,
    series: Option<String>,
    x: Option<String>,
    value: Option<String>,
    position: Option<String>,
    level: Option<String>,
    tags: Vec<String>,
    provenance: Option<String>,
    hint: Option<String>,
}

/// What the parts of one node say, decided once for both renderings.
#[derive(Clone, Copy)]
struct Plan {
    verbosity: Verbosity,
    with_series: bool,
    with_hint: bool,
}

/// Composes announcements for one localizer and settings.
#[derive(Debug)]
pub struct Composer<'l> {
    localizer: &'l Localizer,
    settings: Settings,
    last_series: Option<NavNodeId>,
    visited: BTreeSet<&'static str>,
    excluded: BTreeSet<NavNodeId>,
}

impl<'l> Composer<'l> {
    /// A composer.
    #[must_use]
    pub fn new(localizer: &'l Localizer, settings: Settings) -> Self {
        Self {
            localizer,
            settings,
            last_series: None,
            visited: BTreeSet::new(),
            excluded: BTreeSet::new(),
        }
    }

    /// The settings.
    #[must_use]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Changes the settings (a verbosity toggle or a new part order).
    pub fn set_settings(&mut self, settings: Settings) {
        self.settings = settings;
    }

    /// The announcement for a verb's outcome; `cursor` is the cursor after the verb, which
    /// says which series are filtered out.
    ///
    /// # Errors
    ///
    /// An [`I18nError`] if a catalogue message cannot be formatted.
    pub fn outcome(
        &mut self,
        tree: &NavTree,
        cursor: &Cursor,
        outcome: &Outcome,
    ) -> Result<Announcement, I18nError> {
        self.excluded = cursor.excluded().collect();
        let verbosity = self.settings.verbosity;
        match outcome {
            Outcome::Moved(m) => {
                let mut announcement = self.node_with(tree, m.to, Some(m), verbosity)?;
                announcement.earcon = outcome.earcon();
                Ok(announcement)
            }
            Outcome::Boundary { at, edge } => {
                let id = match verbosity {
                    Verbosity::Terse => "announce-boundary",
                    Verbosity::Verbose => "announce-boundary-verbose",
                };
                let text = self.message(id, &Args::new().text("edge", edge_key(*edge)))?;
                Ok(self.earcon_text(text, *at, EarconId::Boundary))
            }
            Outcome::Invalid { at, reason } => {
                let id = match verbosity {
                    Verbosity::Terse => "announce-invalid",
                    Verbosity::Verbose => "announce-invalid-verbose",
                };
                let text = self.message(id, &Args::new().text("reason", reason_key(*reason)))?;
                Ok(self.earcon_text(text, *at, EarconId::Invalid))
            }
            Outcome::Describe { node } => self.describe(tree, *node),
            Outcome::Toggled { series, filter, .. } => {
                let mut announcement = self.node_with(tree, *filter, None, Verbosity::Terse)?;
                announcement.class = Class::Response;
                announcement.node = Some(*series);
                Ok(announcement)
            }
        }
    }

    /// The announcement for arriving at `node` with no move context.
    ///
    /// # Errors
    ///
    /// As [`Composer::outcome`].
    pub fn node(&mut self, tree: &NavTree, node: NavNodeId) -> Result<Announcement, I18nError> {
        self.node_with(tree, node, None, self.settings.verbosity)
    }

    /// The full verbose description of `node` (Enter on a point), whatever the verbosity.
    ///
    /// # Errors
    ///
    /// As [`Composer::outcome`].
    pub fn describe(&mut self, tree: &NavTree, node: NavNodeId) -> Result<Announcement, I18nError> {
        let plan = Plan {
            verbosity: Verbosity::Verbose,
            with_series: true,
            with_hint: false,
        };
        let mut announcement = self.render(tree, node, None, plan)?;
        announcement.class = Class::Response;
        Ok(announcement)
    }

    /// The current value when playback pauses (A11Y-09, FR-051): "Paused" and the node.
    ///
    /// # Errors
    ///
    /// As [`Composer::outcome`].
    pub fn paused(&mut self, tree: &NavTree, node: NavNodeId) -> Result<Announcement, I18nError> {
        let mut announcement = self.node(tree, node)?;
        let paused = |localizer: &Localizer, rest: &str| {
            localizer.format(
                "announce-join",
                &Args::new()
                    .text("first", localizer.format("announce-paused", &Args::new())?)
                    .text("rest", rest),
            )
        };
        announcement.speech = paused(self.localizer, &announcement.speech)?;
        announcement.display = paused(self.localizer, &announcement.display)?;
        announcement.class = Class::Response;
        Ok(announcement)
    }

    /// The spoken legend (SDR-18, sound-design §7.3), verbose whatever the setting: the
    /// title, the x range, each series' value range with its unit, the pitch mapping with
    /// its polarity, the mode and the context cues that are on. `parts` are the spec's
    /// `context.legend.parts`; the mode and the cues are always said.
    ///
    /// # Errors
    ///
    /// As [`Composer::outcome`].
    pub fn legend(
        &self,
        tree: &NavTree,
        parts: &[LegendPart],
        mode: &str,
        cues: LegendCues,
    ) -> Result<Announcement, I18nError> {
        let mut speech = Vec::new();
        let mut display = Vec::new();
        for (units, out) in [
            (UnitStyle::Verbose, &mut speech),
            (UnitStyle::Symbol, &mut display),
        ] {
            *out = self.legend_parts(tree, parts, units)?;
            out.push(self.message("announce-legend-mode", &Args::new().text("mode", mode))?);
            out.push(
                self.message(
                    "announce-legend-cues",
                    &Args::new()
                        .text("ticks", yes(cues.ticks))
                        .text("reference", yes(cues.reference)),
                )?,
            );
        }
        Ok(Announcement {
            class: Class::Response,
            politeness: Politeness::Polite,
            speech: self.join(&speech)?,
            display: self.join(&display)?,
            node: Some(tree.root()),
            earcon: None,
            speak: true,
            error: false,
        })
    }

    fn legend_parts(
        &self,
        tree: &NavTree,
        parts: &[LegendPart],
        units: UnitStyle,
    ) -> Result<Vec<String>, I18nError> {
        let mut out = Vec::new();
        let has = |part: LegendPart| parts.contains(&part);
        if has(LegendPart::Title)
            && let Some(NavKind::Dataset(dataset)) = tree.node(tree.root()).map(|n| &n.kind)
            && let Some(title) = &dataset.title
        {
            out.push(self.text(title)?);
        }
        if has(LegendPart::XRange)
            && let Some(axes) = tree.landmark("axes")
            && let Some(&x_axis) = tree.children(axes).first()
            && let Some(node) = tree.node(x_axis)
            && let NavKind::Axis(axis) = &node.kind
            && let (Some(label), Some(range)) = (node.label.as_deref(), axis.range)
        {
            let from = self.x_label(tree, Some(&XValue::Number(range.min)), true)?;
            let to = self.x_label(tree, Some(&XValue::Number(range.max)), true)?;
            if let (Some(from), Some(to)) = (from, to) {
                out.push(
                    self.message(
                        "announce-legend-x",
                        &Args::new()
                            .text("field", label)
                            .text("range", self.range(from, to, false)?),
                    )?,
                );
            }
        }
        for series in tree.series() {
            let Some(node) = tree.node(series) else {
                continue;
            };
            let NavKind::Series(info) = &node.kind else {
                continue;
            };
            let label = node.label.clone().unwrap_or_default();
            if has(LegendPart::YRange)
                && let Some(range) = info.range
            {
                let unit = if has(LegendPart::Units) {
                    info.unit.as_deref().map(|text| UnitRef {
                        text,
                        ucum: info.ucum.as_deref(),
                    })
                } else {
                    None
                };
                let from = self.quantity(range.min, unit, units)?;
                let to = self.quantity(range.max, unit, units)?;
                out.push(
                    self.message(
                        "announce-legend-y",
                        &Args::new()
                            .text("series", label.as_str())
                            .text("range", self.range(from, to, false)?),
                    )?,
                );
            }
            if has(LegendPart::Polarity)
                && let Some(pitch) = info.pitch_hz
            {
                let hz = |v: f64| Number::with_options(v, NumberOptions::fixed(0));
                let polarity = match info.polarity {
                    Some(Polarity::Positive) => "positive",
                    Some(Polarity::Negative) => "negative",
                    None => "other",
                };
                out.push(
                    self.message(
                        "announce-legend-mapping",
                        &Args::new()
                            .number_with("low", hz(pitch.min))
                            .number_with("high", hz(pitch.max))
                            .text("polarity", polarity),
                    )?,
                );
            }
        }
        Ok(out)
    }

    /// The "where am I" report (keyboard §6, A11Y-07), verbose whatever the setting: every
    /// level of the path with its position, then the neighbours and the excluded series.
    ///
    /// # Errors
    ///
    /// As [`Composer::outcome`].
    pub fn where_am_i(
        &mut self,
        tree: &NavTree,
        report: &WhereAmI,
    ) -> Result<Announcement, I18nError> {
        let mut speech = Vec::new();
        let mut display = Vec::new();
        for step in &report.path {
            let Some(node) = tree.node(step.node) else {
                continue;
            };
            for (units, out) in [
                (UnitStyle::Verbose, &mut speech),
                (UnitStyle::Symbol, &mut display),
            ] {
                let name = self.name_of(tree, step.node, node, units)?;
                let position = (step.node != tree.root())
                    .then(|| {
                        self.message(
                            "announce-position-verbose",
                            &Args::new()
                                .text("kind", node.kind.name())
                                .number("index", count(step.position))
                                .number("total", count(step.of)),
                        )
                    })
                    .transpose()?;
                let pieces: Vec<String> = [name, position].into_iter().flatten().collect();
                if !pieces.is_empty() {
                    out.push(self.join(&pieces)?);
                }
                if let NavKind::Point(point) = &node.kind {
                    out.push(self.point_value(tree, step.node, point.value, units)?);
                }
            }
        }
        for (neighbour, id) in [
            (report.previous, "announce-previous"),
            (report.next, "announce-next"),
        ] {
            if let Some(n) = neighbour
                && let Some(NavKind::Point(p)) = tree.node(n).map(|node| &node.kind)
            {
                speech.push(self.message(
                    id,
                    &Args::new().text(
                        "text",
                        self.point_value(tree, n, p.value, UnitStyle::Verbose)?,
                    ),
                )?);
                display.push(self.message(
                    id,
                    &Args::new().text(
                        "text",
                        self.point_value(tree, n, p.value, UnitStyle::Symbol)?,
                    ),
                )?);
            }
        }
        let excluded = self.message(
            "announce-excluded",
            &Args::new().number("count", count(report.excluded.len() as u64)),
        )?;
        speech.push(excluded.clone());
        display.push(excluded);
        Ok(Announcement {
            class: Class::Response,
            politeness: Politeness::Polite,
            speech: self.join(&speech)?,
            display: self.join(&display)?,
            node: report.path.last().map(|s| s.node),
            earcon: None,
            speak: true,
            error: false,
        })
    }

    fn node_with(
        &mut self,
        tree: &NavTree,
        node: NavNodeId,
        mv: Option<&Move>,
        verbosity: Verbosity,
    ) -> Result<Announcement, I18nError> {
        let series = series_of(tree, node);
        let kind = tree.node(node).map_or("dataset", |n| n.kind.name());
        let plan = Plan {
            verbosity,
            with_series: series.is_some() && series != self.last_series,
            with_hint: verbosity == Verbosity::Verbose && !self.visited.contains(kind),
        };
        let announcement = self.render(tree, node, mv, plan)?;
        self.last_series = series.or(self.last_series);
        self.visited.insert(kind);
        Ok(announcement)
    }

    /// Renders `node` twice, with spoken units and with symbols.
    fn render(
        &self,
        tree: &NavTree,
        node: NavNodeId,
        mv: Option<&Move>,
        plan: Plan,
    ) -> Result<Announcement, I18nError> {
        let spoken = match plan.verbosity {
            Verbosity::Terse => UnitStyle::Terse,
            Verbosity::Verbose => UnitStyle::Verbose,
        };
        let speech = self.compose(tree, node, mv, plan, spoken)?;
        let display = self.compose(tree, node, mv, plan, UnitStyle::Symbol)?;
        Ok(Announcement {
            class: Class::Navigation,
            politeness: Politeness::Polite,
            speech,
            display,
            node: Some(node),
            earcon: None,
            speak: true,
            error: false,
        })
    }

    fn compose(
        &self,
        tree: &NavTree,
        id: NavNodeId,
        mv: Option<&Move>,
        plan: Plan,
        units: UnitStyle,
    ) -> Result<String, I18nError> {
        let Some(node) = tree.node(id) else {
            return Ok(String::new());
        };
        let pieces = self.pieces(tree, id, node, mv, plan, units)?;
        let mut parts: Vec<String> = pieces.name.into_iter().collect();
        for part in self.settings.order(plan.verbosity) {
            match part {
                Part::Series => parts.extend(pieces.series.clone()),
                Part::X => parts.extend(pieces.x.clone()),
                Part::Value => parts.extend(pieces.value.clone()),
                Part::Position => parts.extend(pieces.position.clone()),
                Part::Level => parts.extend(pieces.level.clone()),
                Part::Tags => parts.extend(pieces.tags.iter().cloned()),
                Part::Provenance => parts.extend(pieces.provenance.clone()),
                Part::Hint => parts.extend(pieces.hint.clone()),
            }
        }
        self.join(&parts)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one arm per node kind, each a few lines"
    )]
    fn pieces(
        &self,
        tree: &NavTree,
        id: NavNodeId,
        node: &NavNode,
        mv: Option<&Move>,
        plan: Plan,
        units: UnitStyle,
    ) -> Result<Pieces, I18nError> {
        let verbose = plan.verbosity == Verbosity::Verbose;
        let mut pieces = Pieces::default();
        if plan.with_series
            && let Some(series) = series_of(tree, id).filter(|&s| s != id)
        {
            pieces.series = tree
                .node(series)
                .and_then(|n| n.label.clone())
                .map(|label| self.text(&label))
                .transpose()?;
        }
        if let Some(m) = mv {
            if let Some(segment) = m.new_segment {
                pieces.level = self.x_span(tree, segment, verbose)?;
            } else if m.level_changed && names_level(&node.kind, verbose) {
                pieces.level = Some(self.message(
                    "announce-level",
                    &Args::new().text("kind", node.kind.name()),
                )?);
            }
            if m.nearest {
                pieces
                    .tags
                    .push(self.message("announce-nearest", &Args::new())?);
            }
        }
        let (index, total) = position(tree, id);
        let position_id = if verbose {
            "announce-position-verbose"
        } else {
            "announce-position"
        };
        let position_part = self.message(
            position_id,
            &Args::new()
                .text("kind", node.kind.name())
                .number("index", count(index))
                .number("total", count(total)),
        )?;
        let hint = |key_id: &str, key: &str| -> Result<Option<String>, I18nError> {
            if plan.with_hint {
                Ok(Some(self.message(key_id, &Args::new().text("key", key))?))
            } else {
                Ok(None)
            }
        };
        let keys = &self.settings.keys;
        match &node.kind {
            NavKind::Point(point) => {
                pieces.x = self.x_label(tree, point.x.as_ref(), verbose)?;
                pieces.value = Some(self.point_value(tree, id, point.value, units)?);
                pieces.position = Some(position_part);
                pieces.tags.extend(self.insight_tags(tree, id)?);
                pieces.provenance = self.provenance(point.provenance, verbose)?;
            }
            NavKind::Segment(segment) => {
                pieces.x = self.x_span(tree, id, verbose)?;
                pieces.value = Some(self.points_and_range(
                    tree,
                    id,
                    segment.points,
                    segment.range,
                    units,
                    verbose,
                )?);
                pieces.position = Some(position_part);
                pieces.hint = hint("announce-hint-points", &keys.explore)?;
            }
            NavKind::Series(series) => {
                pieces.name = node.label.as_deref().map(|l| self.text(l)).transpose()?;
                pieces.series = None;
                pieces.x = self.x_span(tree, id, verbose)?;
                pieces.value = Some(self.points_and_range(
                    tree,
                    id,
                    series.points,
                    series.range,
                    units,
                    verbose,
                )?);
                pieces.position = Some(position_part);
                if self.excluded.contains(&id) {
                    pieces.tags.push(self.inclusion(id)?);
                }
                pieces.hint = hint("announce-hint-explore", &keys.explore)?;
            }
            NavKind::Dataset(dataset) => {
                pieces.name = dataset.title.as_deref().map(|t| self.text(t)).transpose()?;
                let series = self.message(
                    "announce-series-count",
                    &Args::new().number("count", f64::from(dataset.series)),
                )?;
                let points = self.message(
                    "announce-points",
                    &Args::new().number("count", count(dataset.points)),
                )?;
                pieces.value = Some(self.join(&[series, points])?);
                pieces.hint = hint("announce-hint-explore", &keys.explore)?;
            }
            NavKind::Axes => {
                pieces.name = Some(self.message("announce-axes", &Args::new())?);
            }
            NavKind::Axis(axis) => {
                let which = self.message(
                    "announce-axis",
                    &Args::new().text("axis", if axis.axis == AxisName::X { "x" } else { "y" }),
                )?;
                let label = node.label.as_deref().map(|l| self.text(l)).transpose()?;
                pieces.name = Some(
                    self.join(
                        &[Some(which), label]
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>(),
                    )?,
                );
                pieces.value = match (axis.range, axis.axis) {
                    (Some(range), AxisName::Y) => {
                        let unit = axis.unit.as_deref().map(|text| UnitRef {
                            text,
                            ucum: axis.ucum.as_deref(),
                        });
                        Some(self.range(
                            self.quantity(range.min, unit, units)?,
                            self.quantity(range.max, unit, units)?,
                            false,
                        )?)
                    }
                    (Some(range), AxisName::X) => {
                        let from = self.x_label(tree, Some(&XValue::Number(range.min)), verbose)?;
                        let to = self.x_label(tree, Some(&XValue::Number(range.max)), verbose)?;
                        match (from, to) {
                            (Some(from), Some(to)) => Some(self.range(from, to, false)?),
                            _ => None,
                        }
                    }
                    _ => None,
                };
            }
            NavKind::Insights => {
                let insights: usize = tree
                    .children(id)
                    .iter()
                    .map(|&group| tree.children(group).len())
                    .sum();
                pieces.name = Some(self.message(
                    "announce-insight-count",
                    &Args::new().number("count", count(insights as u64)),
                )?);
            }
            NavKind::InsightGroup(_) => {
                pieces.name = node.label.as_deref().map(|l| self.text(l)).transpose()?;
                pieces.value = Some(self.message(
                    "announce-insight-count",
                    &Args::new().number("count", count(tree.children(id).len() as u64)),
                )?);
            }
            NavKind::Insight(insight) => {
                let word = self.message("announce-insight", &Args::new())?;
                let label = node.label.as_deref().map(|l| self.text(l)).transpose()?;
                pieces.name = Some(
                    self.join(
                        &[Some(word), label]
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>(),
                    )?,
                );
                if let Some(target) = insight.target
                    && let Some(NavKind::Point(point)) = tree.node(target).map(|n| &n.kind)
                {
                    pieces.value = Some(self.point_value(
                        tree,
                        target,
                        insight.value.or(point.value),
                        units,
                    )?);
                    pieces.x = self.x_label(tree, point.x.as_ref(), verbose)?;
                }
                pieces.position = Some(position_part);
                pieces.hint = hint("announce-hint-insight", &keys.activate)?;
            }
            NavKind::Filters => {
                pieces.name = Some(self.message("announce-filters", &Args::new())?);
            }
            NavKind::Filter(filter) => {
                pieces.name = node.label.as_deref().map(|l| self.text(l)).transpose()?;
                pieces.value = Some(self.inclusion(filter.series)?);
            }
            _ => {}
        }
        Ok(pieces)
    }

    /// "included" or "excluded" for a series.
    fn inclusion(&self, series: NavNodeId) -> Result<String, I18nError> {
        let included = if self.excluded.contains(&series) {
            "no"
        } else {
            "yes"
        };
        self.message(
            "announce-filter-state",
            &Args::new().text("included", included),
        )
    }

    /// "Insight" tags for the insights whose point this is.
    fn insight_tags(&self, tree: &NavTree, id: NavNodeId) -> Result<Vec<String>, I18nError> {
        tree.iter()
            .filter_map(|(_, node)| match &node.kind {
                NavKind::Insight(insight) if insight.target == Some(id) => node.label.as_deref(),
                _ => None,
            })
            .map(|label| self.text(label))
            .collect()
    }

    fn provenance(
        &self,
        provenance: Provenance,
        verbose: bool,
    ) -> Result<Option<String>, I18nError> {
        if provenance.class.states_plainly() {
            return Ok(None);
        }
        let class = class_key(provenance);
        if !verbose {
            return self
                .message(
                    "announce-provenance-terse",
                    &Args::new().text("class", class),
                )
                .map(Some);
        }
        let qualifier = self.message("announce-provenance", &Args::new().text("class", class))?;
        match provenance.confidence {
            Some(confidence) => self
                .message(
                    "announce-provenance-confidence",
                    &Args::new().text("qualifier", qualifier).number_with(
                        "confidence",
                        Number::with_options(f64::from(confidence.get()), NumberOptions::fixed(2)),
                    ),
                )
                .map(Some),
            None => Ok(Some(qualifier)),
        }
    }

    /// A point's value with its series unit, or "no value".
    fn point_value(
        &self,
        tree: &NavTree,
        point: NavNodeId,
        value: Option<f64>,
        units: UnitStyle,
    ) -> Result<String, I18nError> {
        let Some(value) = value else {
            return self.message("announce-missing", &Args::new());
        };
        let series = series_of(tree, point).and_then(|s| tree.node(s));
        let unit = series.and_then(|n| match &n.kind {
            NavKind::Series(s) => s.unit.as_deref().map(|text| UnitRef {
                text,
                ucum: s.ucum.as_deref(),
            }),
            _ => None,
        });
        self.quantity(value, unit, units)
    }

    fn quantity(
        &self,
        value: f64,
        unit: Option<UnitRef<'_>>,
        units: UnitStyle,
    ) -> Result<String, I18nError> {
        self.localizer.quantity(Number::new(value), unit, units)
    }

    /// "10 points. Range 3.1 to 28.7 degrees Celsius".
    fn points_and_range(
        &self,
        tree: &NavTree,
        id: NavNodeId,
        points: u64,
        range: Option<sonarithm_core::nav::ValueRange>,
        units: UnitStyle,
        verbose: bool,
    ) -> Result<String, I18nError> {
        let count_part = self.message(
            "announce-points",
            &Args::new().number("count", count(points)),
        )?;
        let Some(range) = range else {
            return Ok(count_part);
        };
        let from = self.point_value(
            tree,
            first_point(tree, id).unwrap_or(id),
            Some(range.min),
            units,
        )?;
        let to = self.point_value(
            tree,
            first_point(tree, id).unwrap_or(id),
            Some(range.max),
            units,
        )?;
        let range_part = self.range(from, to, verbose)?;
        self.join(&[count_part, range_part])
    }

    fn range(&self, from: String, to: String, verbose: bool) -> Result<String, I18nError> {
        let id = if verbose {
            "announce-range-verbose"
        } else {
            "announce-range"
        };
        self.message(id, &Args::new().text("from", from).text("to", to))
    }

    /// The x extent of a node as "from to to", or one label when it is a single position.
    fn x_span(
        &self,
        tree: &NavTree,
        id: NavNodeId,
        verbose: bool,
    ) -> Result<Option<String>, I18nError> {
        let points = match tree.node(id).map(|n| &n.kind) {
            Some(NavKind::Series(_)) => tree.points(id),
            _ => tree.children(id).to_vec(),
        };
        let x_of = |n: Option<&NavNodeId>| -> Option<XValue> {
            match tree.node(*n?).map(|node| &node.kind) {
                Some(NavKind::Point(p)) => p.x.clone(),
                _ => None,
            }
        };
        let (Some(first), Some(last)) = (x_of(points.first()), x_of(points.last())) else {
            return Ok(None);
        };
        let from = self.x_label(tree, Some(&first), verbose)?;
        if first == last {
            return Ok(from);
        }
        let to = self.x_label(tree, Some(&last), verbose)?;
        match (from, to) {
            (Some(from), Some(to)) => self.range(from, to, false).map(Some),
            _ => Ok(None),
        }
    }

    /// An x value formatted for its axis type.
    fn x_label(
        &self,
        tree: &NavTree,
        x: Option<&XValue>,
        verbose: bool,
    ) -> Result<Option<String>, I18nError> {
        let Some(x) = x else {
            return Ok(None);
        };
        let number = match x {
            XValue::Category(label) => return self.text(label).map(Some),
            XValue::Number(n) => *n,
        };
        let length = if verbose {
            DateLength::Long
        } else {
            DateLength::Medium
        };
        let pattern = if verbose {
            DatePattern::YearMonthDay
        } else {
            DatePattern::MonthDay
        };
        let text = match x_type(tree) {
            XType::Year => self.localizer.number(Number::with_options(
                number,
                NumberOptions::default().without_grouping(),
            ))?,
            XType::Date => self.date(days_floor(number), pattern, length)?,
            XType::Timestamp => {
                let days = (number / 86_400.0).floor();
                let date = self.date(days_floor(days), pattern, length)?;
                let seconds = number - days * 86_400.0;
                let field = |v: f64| {
                    Number::with_options(
                        v,
                        NumberOptions {
                            min_integer: Some(2),
                            max_fraction: Some(0),
                            ..NumberOptions::default()
                        },
                    )
                };
                let time = self.message(
                    "announce-time",
                    &Args::new()
                        .number_with("hours", field((seconds / 3600.0).floor()))
                        .number_with("minutes", field(((seconds % 3600.0) / 60.0).floor()))
                        .number_with("seconds", field((seconds % 60.0).floor())),
                )?;
                self.message(
                    "announce-date-time",
                    &Args::new().text("date", date).text("time", time),
                )?
            }
            _ => self.localizer.number(Number::new(number))?,
        };
        Ok(Some(text))
    }

    fn date(
        &self,
        days: i64,
        pattern: DatePattern,
        length: DateLength,
    ) -> Result<String, I18nError> {
        let (year, month, day) = civil_from_days(days);
        self.localizer.date((year, month, day), pattern, length)
    }

    /// The name of a node for "where am I": its label, or the dataset title.
    fn name_of(
        &self,
        tree: &NavTree,
        id: NavNodeId,
        node: &NavNode,
        units: UnitStyle,
    ) -> Result<Option<String>, I18nError> {
        match &node.kind {
            NavKind::Dataset(d) => d.title.as_deref().map(|t| self.text(t)).transpose(),
            NavKind::Point(p) => self.x_label(tree, p.x.as_ref(), true),
            NavKind::Segment(_) => self.x_span(tree, id, true),
            _ => {
                let _ = units;
                node.label.as_deref().map(|l| self.text(l)).transpose()
            }
        }
    }

    fn earcon_text(&self, text: String, at: NavNodeId, earcon: EarconId) -> Announcement {
        Announcement {
            class: Class::EarconText,
            politeness: Politeness::Polite,
            display: text.clone(),
            speech: text,
            node: Some(at),
            earcon: Some(earcon),
            // keyboard §9.1: spoken in verbose; in terse the earcon alone plays and the text
            // goes to the log.
            speak: self.settings.verbosity == Verbosity::Verbose,
            error: false,
        }
    }

    fn text(&self, text: &str) -> Result<String, I18nError> {
        self.message("announce-text", &Args::new().text("text", text))
    }

    fn message(&self, id: &str, args: &Args) -> Result<String, I18nError> {
        self.localizer.format(id, args)
    }

    /// Joins parts right to left, so the result is one message.
    fn join(&self, parts: &[String]) -> Result<String, I18nError> {
        let Some((last, rest)) = parts.split_last() else {
            return Ok(String::new());
        };
        rest.iter().rev().try_fold(last.clone(), |tail, head| {
            self.message(
                "announce-join",
                &Args::new().text("first", head.as_str()).text("rest", tail),
            )
        })
    }
}

/// Whether a level change names the new level. The names of axes, insights and the filter
/// group already say it, and at verbose so does the position ("Point 3 of 20").
fn names_level(kind: &NavKind, verbose: bool) -> bool {
    match kind {
        NavKind::Dataset(_) | NavKind::InsightGroup(_) | NavKind::Filter(_) => true,
        NavKind::Series(_) | NavKind::Segment(_) | NavKind::Point(_) => !verbose,
        _ => false,
    }
}

/// The series containing `node`.
fn series_of(tree: &NavTree, node: NavNodeId) -> Option<NavNodeId> {
    let mut current = Some(node);
    while let Some(id) = current {
        if matches!(tree.node(id).map(|n| &n.kind), Some(NavKind::Series(_))) {
            return Some(id);
        }
        current = tree.parent(id);
    }
    None
}

fn first_point(tree: &NavTree, node: NavNodeId) -> Option<NavNodeId> {
    series_of(tree, node).and_then(|s| tree.points(s).first().copied())
}

/// The position of a node among its peers, from 1: series among series, points among the
/// series' points, other nodes among their siblings.
fn position(tree: &NavTree, id: NavNodeId) -> (u64, u64) {
    let peers: Vec<NavNodeId> = match tree.node(id).map(|n| &n.kind) {
        Some(NavKind::Series(_)) => tree.series().collect(),
        Some(NavKind::Point(_)) => series_of(tree, id)
            .map(|s| tree.points(s))
            .unwrap_or_default(),
        _ => tree
            .parent(id)
            .map(|p| tree.children(p).to_vec())
            .unwrap_or_default(),
    };
    let index = peers.iter().position(|&p| p == id).unwrap_or(0);
    (index as u64 + 1, (peers.len() as u64).max(1))
}

/// The x type of the tree's x axis.
fn x_type(tree: &NavTree) -> XType {
    tree.landmark("axes")
        .and_then(|axes| {
            tree.children(axes)
                .iter()
                .find_map(|&id| match tree.node(id).map(|n| &n.kind) {
                    Some(NavKind::Axis(axis)) if axis.axis == AxisName::X => Some(axis.values),
                    _ => None,
                })
        })
        .unwrap_or(XType::Number)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "counts and positions stay far below 2^53"
)]
fn count(n: u64) -> f64 {
    n as f64
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "day numbers of real dates fit in i64; NaN becomes 0"
)]
fn days_floor(days: f64) -> i64 {
    days.floor() as i64
}

/// The proleptic Gregorian date of a day number (days since 1970-01-01), after Howard
/// Hinnant's `civil_from_days`.
fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (
        i32::try_from(year).unwrap_or(i32::MAX),
        u8::try_from(month).unwrap_or(1),
        u8::try_from(day).unwrap_or(1),
    )
}

fn edge_key(edge: Edge) -> &'static str {
    match edge {
        Edge::First => "first",
        Edge::Last => "last",
        Edge::Top => "top",
        Edge::Bottom => "bottom",
    }
}

fn reason_key(reason: Invalid) -> &'static str {
    match reason {
        Invalid::OneSeries => "one-series",
        Invalid::NotInSeries => "not-in-series",
        Invalid::NoValues => "no-values",
        Invalid::NoInsights => "no-insights",
        Invalid::NotFound => "not-found",
        Invalid::NoLandmark => "no-landmark",
        _ => "other",
    }
}

fn class_key(provenance: Provenance) -> &'static str {
    use sonarithm_core::provenance::ProvenanceClass as C;
    match provenance.class {
        C::Derived => "derived",
        C::Synthetic => "synthetic",
        C::CvEstimate => "cv_estimate",
        C::MlEstimate => "ml_estimate",
        C::CloudEstimate => "cloud_estimate",
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::civil_from_days;

    #[test]
    fn day_numbers_become_civil_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_648), (2026, 7, 14));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
    }
}

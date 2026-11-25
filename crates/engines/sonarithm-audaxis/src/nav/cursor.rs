// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The cursor and its movement verbs (08 §2, keyboard-and-navigation §4.2–§4.3), and
//! "where am I" (§6).
//!
//! One cursor points at one node of a [`NavTree`] (A11Y-10). Every verb resolves to one
//! discrete step (A11Y-27) and returns an [`Outcome`]:
//!
//! - **Moved**: the cursor is on a new node.
//! - **Boundary**: the move would leave the structure, so the cursor stays where it is and
//!   the boundary earcon plays (A11Y-08, Chart Reader's "bonk"). Moves never wrap.
//! - **Invalid**: the verb does not apply here, with a reason; the cursor stays and the
//!   invalid-action earcon plays.
//!
//! The cursor remembers its x position, so entering a child, going back up and moving to
//! another series keep the listener at the same place in time (keyboard §4.3). At point
//! level the siblings are all points of the series in x order: segments only group them,
//! and stepping past the last point of a segment continues into the next one.

use std::collections::BTreeSet;

use sonarithm_core::nav::{AxisName, NavKind, NavNodeId, NavTree, XValue};

use crate::earcons::EarconId;

/// Landmarks reachable with one key (keyboard §4.2: X, Y, I, F, D).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Landmark {
    /// The x axis.
    XAxis,
    /// The first value axis.
    YAxis,
    /// The insights group.
    Insights,
    /// The filters group.
    Filters,
    /// The dataset root.
    Dataset,
}

/// Where "go to x" goes.
#[derive(Clone, Debug, PartialEq)]
pub enum XTarget {
    /// A number, date (days since 1970-01-01) or instant (seconds since the epoch): the
    /// nearest point.
    Number(f64),
    /// A category label, compared without case.
    Category(String),
}

/// A movement verb (08 §2; keys in keyboard §4.2).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Verb {
    /// Spatial: the next node at this level, in x order (Right).
    Next,
    /// Spatial: the previous node at this level (Left).
    Previous,
    /// Spatial: forward by 10 % of the nodes at this level, at least one (Jump+Right).
    JumpForward,
    /// Spatial: back by 10 % (Jump+Left).
    JumpBack,
    /// Spatial: the first node at this level (Home).
    First,
    /// Spatial: the last node at this level (End).
    Last,
    /// Local: into the child at the current x position, else the first child (Down).
    Into,
    /// Local: out to the parent, keeping the x position (Up).
    Out,
    /// Enter: on a point, its full description; on an insight, its data; on a filter, the
    /// toggle; elsewhere, as [`Verb::Into`].
    Activate,
    /// Lateral: the next series at the same position (`PageDown`).
    NextSeries,
    /// Lateral: the previous series at the same position (`PageUp`).
    PreviousSeries,
    /// Targeted: the maximum point of the current series (Jump+Up).
    Maximum,
    /// Targeted: the minimum point of the current series (Jump+Down).
    Minimum,
    /// Targeted: the next insight of the current series, landing on its point (N).
    NextInsight,
    /// Targeted: the previous insight of the current series (Shift+N).
    PreviousInsight,
    /// Targeted: a landmark.
    Landmark(Landmark),
    /// Targeted: the point nearest an x value in the current series (G).
    GoTo(XTarget),
    /// Targeted: the next point equal to or crossing a value in the current series.
    FindValue(f64),
}

/// The edge a boundary move hit (keyboard §11: "First", "Last", "Top level", "No lower
/// level").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Edge {
    /// Nothing before this node.
    First,
    /// Nothing after this node.
    Last,
    /// Already at the top level.
    Top,
    /// No lower level.
    Bottom,
}

/// Why a verb does not apply here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Invalid {
    /// Lateral moves need a second series.
    OneSeries,
    /// The verb works inside a series, and the cursor is not in one.
    NotInSeries,
    /// The series has no values.
    NoValues,
    /// The series has no insights.
    NoInsights,
    /// No point matches.
    NotFound,
    /// The tree has no such landmark.
    NoLandmark,
    /// The verb has no meaning at this level.
    NotHere,
}

/// A successful move.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Move {
    /// Where the cursor was.
    pub from: NavNodeId,
    /// Where it is now.
    pub to: NavNodeId,
    /// Whether the level changed, so the announcement names the level.
    pub level_changed: bool,
    /// Whether the move landed on the nearest x rather than the same x ("nearest").
    pub nearest: bool,
    /// The segment a point move entered, so it can be named as context.
    pub new_segment: Option<NavNodeId>,
    /// The insight an insight move went to.
    pub insight: Option<NavNodeId>,
    /// Whether the new node is an insight's point or a landmark (`E-LANDMARK`).
    pub landmark: bool,
}

/// What a verb did.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Outcome {
    /// The cursor moved.
    Moved(Move),
    /// The cursor stays at `at`: the move would leave the structure.
    Boundary {
        /// The cursor's node.
        at: NavNodeId,
        /// The edge.
        edge: Edge,
    },
    /// The cursor stays at `at`: the verb does not apply.
    Invalid {
        /// The cursor's node.
        at: NavNodeId,
        /// Why.
        reason: Invalid,
    },
    /// Enter on a point: speak its full description; the cursor stays.
    Describe {
        /// The point.
        node: NavNodeId,
    },
    /// Enter on a filter: its series is now in or out of playback and view.
    Toggled {
        /// The filter.
        filter: NavNodeId,
        /// The series.
        series: NavNodeId,
        /// Whether the series is now included.
        included: bool,
    },
}

impl Outcome {
    /// The earcon this outcome plays, if any (A11Y-08).
    #[must_use]
    pub fn earcon(&self) -> Option<EarconId> {
        match self {
            Self::Boundary { .. } => Some(EarconId::Boundary),
            Self::Invalid { .. } => Some(EarconId::Invalid),
            Self::Moved(m) if m.landmark => Some(EarconId::Landmark),
            _ => None,
        }
    }

    /// The node the cursor is on after the verb.
    #[must_use]
    pub fn cursor(&self) -> NavNodeId {
        match self {
            Self::Moved(m) => m.to,
            Self::Boundary { at, .. } | Self::Invalid { at, .. } => *at,
            Self::Describe { node } => *node,
            Self::Toggled { filter, .. } => *filter,
        }
    }
}

/// One level of a "where am I" report: a node and its position among its peers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathStep {
    /// The node.
    pub node: NavNodeId,
    /// Its position, from 1: among the series for a series, among the series' points for a
    /// point, among its siblings otherwise.
    pub position: u64,
    /// The number of peers.
    pub of: u64,
}

/// A full orientation report (keyboard §6, A11Y-07): the path from the root to the cursor
/// with positions, and the neighbours at the cursor's level.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhereAmI {
    /// From the root (first) to the cursor (last).
    pub path: Vec<PathStep>,
    /// The previous node at the cursor's level.
    pub previous: Option<NavNodeId>,
    /// The next node at the cursor's level.
    pub next: Option<NavNodeId>,
    /// The series filtered out of playback and view.
    pub excluded: Vec<NavNodeId>,
}

/// The cursor: a node, the remembered x position and the excluded series.
#[derive(Clone, Debug, PartialEq)]
pub struct Cursor {
    node: NavNodeId,
    x: Option<f64>,
    series: Option<NavNodeId>,
    excluded: BTreeSet<NavNodeId>,
}

impl Cursor {
    /// A cursor on the root, with no x position and every series included.
    #[must_use]
    pub fn new() -> Self {
        Self {
            node: NavNodeId::ROOT,
            x: None,
            series: None,
            excluded: BTreeSet::new(),
        }
    }

    /// A cursor on `node`, or `None` if `tree` has no such node.
    #[must_use]
    pub fn at(tree: &NavTree, node: NavNodeId) -> Option<Self> {
        tree.node(node)?;
        let mut cursor = Self::new();
        cursor.land(tree, node);
        Some(cursor)
    }

    /// Moves the cursor to `node` from outside the verbs (a table row or a plot click,
    /// ADR-0017 "Cursor"), keeping the excluded series. Returns `false`, and leaves the
    /// cursor as it was, if `tree` has no such node.
    pub fn place(&mut self, tree: &NavTree, node: NavNodeId) -> bool {
        if tree.node(node).is_none() {
            return false;
        }
        self.land(tree, node);
        true
    }

    /// Sets the excluded series, such as after a recompile renumbered the nodes.
    pub fn set_excluded(&mut self, series: impl IntoIterator<Item = NavNodeId>) {
        self.excluded = series.into_iter().collect();
    }

    /// The node under the cursor.
    #[must_use]
    pub fn node(&self) -> NavNodeId {
        self.node
    }

    /// The series the cursor is in, or was last in (for lateral moves and targeted verbs).
    #[must_use]
    pub fn series(&self) -> Option<NavNodeId> {
        self.series
    }

    /// The remembered x position.
    #[must_use]
    pub fn x(&self) -> Option<f64> {
        self.x
    }

    /// Whether `series` is filtered out of playback and view.
    #[must_use]
    pub fn is_excluded(&self, series: NavNodeId) -> bool {
        self.excluded.contains(&series)
    }

    /// The series filtered out of playback and view.
    pub fn excluded(&self) -> impl Iterator<Item = NavNodeId> + '_ {
        self.excluded.iter().copied()
    }

    /// Applies `verb`. The cursor stays on a valid node whatever happens.
    pub fn apply(&mut self, tree: &NavTree, verb: &Verb) -> Outcome {
        if tree.node(self.node).is_none() {
            self.node = NavNodeId::ROOT;
        }
        let at = self.node;
        let resolved = match verb {
            Verb::Next => self.step(tree, 1),
            Verb::Previous => self.step(tree, -1),
            Verb::JumpForward => self.jump(tree, true),
            Verb::JumpBack => self.jump(tree, false),
            Verb::First => self.end(tree, false),
            Verb::Last => self.end(tree, true),
            Verb::Into => self.descend(tree),
            Verb::Out => self.ascend(tree),
            Verb::Activate => self.activate(tree),
            Verb::NextSeries => self.lateral(tree, 1),
            Verb::PreviousSeries => self.lateral(tree, -1),
            Verb::Maximum => self.extreme(tree, true),
            Verb::Minimum => self.extreme(tree, false),
            Verb::NextInsight => self.insight(tree, true),
            Verb::PreviousInsight => self.insight(tree, false),
            Verb::Landmark(landmark) => landmark_node(tree, *landmark)
                .map(Target::plain)
                .ok_or(Stop::Invalid(Invalid::NoLandmark)),
            Verb::GoTo(target) => self.go_to(tree, target),
            Verb::FindValue(value) => self.find(tree, *value),
        };
        match resolved {
            Ok(Target::Move {
                node,
                nearest,
                insight,
            }) => self.moved(tree, at, node, nearest, insight),
            Ok(Target::Describe) => Outcome::Describe { node: at },
            Ok(Target::Toggle { series }) => {
                let included = if self.excluded.remove(&series) {
                    true
                } else {
                    self.excluded.insert(series);
                    false
                };
                Outcome::Toggled {
                    filter: at,
                    series,
                    included,
                }
            }
            Err(Stop::Edge(edge)) => Outcome::Boundary { at, edge },
            Err(Stop::Invalid(reason)) => Outcome::Invalid { at, reason },
        }
    }

    /// The orientation report for the cursor (keyboard §6). Defined at every node.
    #[must_use]
    pub fn where_am_i(&self, tree: &NavTree) -> WhereAmI {
        let node = if tree.node(self.node).is_some() {
            self.node
        } else {
            NavNodeId::ROOT
        };
        let mut chain = vec![node];
        while let Some(parent) = chain.last().and_then(|&n| tree.parent(n)) {
            chain.push(parent);
        }
        chain.reverse();
        let path = chain
            .iter()
            .map(|&n| {
                let peers = peers(tree, n);
                let position = peers.iter().position(|&p| p == n).unwrap_or(0);
                PathStep {
                    node: n,
                    position: position as u64 + 1,
                    of: (peers.len() as u64).max(1),
                }
            })
            .collect();
        let level = siblings(tree, node);
        let index = level.iter().position(|&n| n == node);
        let previous = index
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| level.get(i).copied());
        let next = index.and_then(|i| level.get(i + 1).copied());
        WhereAmI {
            path,
            previous,
            next,
            excluded: self.excluded.iter().copied().collect(),
        }
    }

    fn moved(
        &mut self,
        tree: &NavTree,
        from: NavNodeId,
        to: NavNodeId,
        nearest: bool,
        insight: Option<NavNodeId>,
    ) -> Outcome {
        let old_segment = point_segment(tree, from);
        self.land(tree, to);
        let new_segment = point_segment(tree, to).filter(|_| old_segment.is_some());
        let new_segment = new_segment.filter(|segment| Some(*segment) != old_segment);
        let landmark = is_insight_target(tree, to) || insight.is_some();
        Outcome::Moved(Move {
            from,
            to,
            level_changed: tree.depth(from) != tree.depth(to),
            nearest,
            new_segment,
            insight,
            landmark,
        })
    }

    /// Puts the cursor on `node` and updates the remembered x position and series.
    fn land(&mut self, tree: &NavTree, node: NavNodeId) {
        self.node = node;
        if let Some(span) = tree.node(node).and_then(|n| n.span) {
            let is_point = matches!(tree.node(node).map(|n| &n.kind), Some(NavKind::Point(_)));
            if is_point || !self.x.is_some_and(|x| span.contains(x)) {
                self.x = Some(span.start);
            }
        }
        if let Some(series) = series_of(tree, node) {
            self.series = Some(series);
        }
    }

    fn step(&self, tree: &NavTree, delta: isize) -> Result<Target, Stop> {
        if self.node == NavNodeId::ROOT {
            return Err(Stop::Edge(if delta > 0 { Edge::Last } else { Edge::First }));
        }
        let level = siblings(tree, self.node);
        let index = level.iter().position(|&n| n == self.node).unwrap_or(0);
        offset(&level, index, delta).map(Target::plain)
    }

    fn jump(&self, tree: &NavTree, forward: bool) -> Result<Target, Stop> {
        if self.node == NavNodeId::ROOT {
            return Err(Stop::Edge(if forward { Edge::Last } else { Edge::First }));
        }
        let level = siblings(tree, self.node);
        // keyboard §4.2: 10 % of the nodes at this level, at least one.
        let stride = (level.len() / 10).max(1);
        let index = level.iter().position(|&n| n == self.node).unwrap_or(0);
        let edge = if forward { Edge::Last } else { Edge::First };
        let target = if forward {
            if index + 1 >= level.len() {
                return Err(Stop::Edge(edge));
            }
            (index + stride).min(level.len() - 1)
        } else {
            if index == 0 {
                return Err(Stop::Edge(edge));
            }
            index.saturating_sub(stride)
        };
        level
            .get(target)
            .copied()
            .map(Target::plain)
            .ok_or(Stop::Edge(edge))
    }

    fn end(&self, tree: &NavTree, last: bool) -> Result<Target, Stop> {
        let edge = if last { Edge::Last } else { Edge::First };
        if self.node == NavNodeId::ROOT {
            return Err(Stop::Edge(edge));
        }
        let level = siblings(tree, self.node);
        let target = if last { level.last() } else { level.first() };
        match target {
            Some(&n) if n != self.node => Ok(Target::plain(n)),
            _ => Err(Stop::Edge(edge)),
        }
    }

    fn descend(&self, tree: &NavTree) -> Result<Target, Stop> {
        let children = tree.children(self.node);
        if children.is_empty() {
            return Err(Stop::Edge(Edge::Bottom));
        }
        let at_x = self.x.and_then(|x| {
            children.iter().copied().find(|&c| {
                tree.node(c)
                    .and_then(|n| n.span)
                    .is_some_and(|s| s.contains(x))
            })
        });
        Ok(Target::plain(at_x.unwrap_or(children[0])))
    }

    fn ascend(&self, tree: &NavTree) -> Result<Target, Stop> {
        tree.parent(self.node)
            .map(Target::plain)
            .ok_or(Stop::Edge(Edge::Top))
    }

    fn activate(&self, tree: &NavTree) -> Result<Target, Stop> {
        match tree.node(self.node).map(|n| &n.kind) {
            Some(NavKind::Point(_)) => Ok(Target::Describe),
            Some(NavKind::Insight(insight)) => insight
                .target
                .map(|node| Target::Move {
                    node,
                    nearest: false,
                    insight: Some(self.node),
                })
                .ok_or(Stop::Invalid(Invalid::NotFound)),
            Some(NavKind::Filter(filter)) => Ok(Target::Toggle {
                series: filter.series,
            }),
            _ => self.descend(tree),
        }
    }

    fn lateral(&self, tree: &NavTree, delta: isize) -> Result<Target, Stop> {
        let all: Vec<NavNodeId> = tree.series().collect();
        if all.len() < 2 {
            return Err(Stop::Invalid(Invalid::OneSeries));
        }
        let current = series_of(tree, self.node).ok_or(Stop::Invalid(Invalid::NotInSeries))?;
        let index = all.iter().position(|&s| s == current).unwrap_or(0);
        let target = offset(&all, index, delta)?;
        let kind = tree.node(self.node).map(|n| &n.kind);
        match kind {
            Some(NavKind::Series(_)) => Ok(Target::plain(target)),
            Some(NavKind::Segment(_)) => {
                let segments = tree.children(target);
                nearest_by_span(tree, segments, self.x).ok_or(Stop::Invalid(Invalid::NotFound))
            }
            Some(NavKind::Point(_)) => {
                let points = tree.points(target);
                nearest_by_span(tree, &points, self.x).ok_or(Stop::Invalid(Invalid::NotFound))
            }
            _ => Err(Stop::Invalid(Invalid::NotInSeries)),
        }
    }

    fn extreme(&self, tree: &NavTree, maximum: bool) -> Result<Target, Stop> {
        if self.node == NavNodeId::ROOT {
            return Err(Stop::Invalid(Invalid::NotHere));
        }
        let series = series_of(tree, self.node).ok_or(Stop::Invalid(Invalid::NotInSeries))?;
        let mut best: Option<(NavNodeId, f64)> = None;
        for id in tree.points(series) {
            if let Some(value) = point_value(tree, id) {
                let better = best.is_none_or(|(_, b)| if maximum { value > b } else { value < b });
                if better {
                    best = Some((id, value));
                }
            }
        }
        best.map(|(id, _)| Target::plain(id))
            .ok_or(Stop::Invalid(Invalid::NoValues))
    }

    fn insight(&self, tree: &NavTree, next: bool) -> Result<Target, Stop> {
        let series = self
            .context_series(tree)
            .ok_or(Stop::Invalid(Invalid::NotInSeries))?;
        let points = tree.points(series);
        let mut found: Vec<(usize, NavNodeId, NavNodeId)> = insights(tree)
            .filter_map(|(insight, target)| {
                points
                    .iter()
                    .position(|&p| p == target)
                    .map(|index| (index, target, insight))
            })
            .collect();
        if found.is_empty() {
            return Err(Stop::Invalid(Invalid::NoInsights));
        }
        found.sort_by_key(|(index, _, insight)| (*index, *insight));
        let current = points.iter().position(|&p| p == self.node);
        let pick = if next {
            found
                .iter()
                .find(|(index, _, _)| current.is_none_or(|c| *index > c))
        } else {
            found
                .iter()
                .rev()
                .find(|(index, _, _)| current.is_none_or(|c| *index < c))
        };
        pick.map(|&(_, target, insight)| Target::Move {
            node: target,
            nearest: false,
            insight: Some(insight),
        })
        .ok_or(Stop::Edge(if next { Edge::Last } else { Edge::First }))
    }

    fn go_to(&self, tree: &NavTree, target: &XTarget) -> Result<Target, Stop> {
        let series = self
            .context_series(tree)
            .ok_or(Stop::Invalid(Invalid::NotInSeries))?;
        let points = tree.points(series);
        match target {
            XTarget::Number(x) => {
                nearest_by_span(tree, &points, Some(*x)).ok_or(Stop::Invalid(Invalid::NotFound))
            }
            XTarget::Category(label) => points
                .iter()
                .copied()
                .find(|&id| match tree.node(id).map(|n| &n.kind) {
                    Some(NavKind::Point(p)) => matches!(
                        &p.x,
                        Some(XValue::Category(c)) if c.to_lowercase() == label.to_lowercase()
                    ),
                    _ => false,
                })
                .map(Target::plain)
                .ok_or(Stop::Invalid(Invalid::NotFound)),
        }
    }

    fn find(&self, tree: &NavTree, value: f64) -> Result<Target, Stop> {
        let series = self
            .context_series(tree)
            .ok_or(Stop::Invalid(Invalid::NotInSeries))?;
        let points = tree.points(series);
        let start = points
            .iter()
            .position(|&p| p == self.node)
            .map_or(0, |i| i + 1);
        for k in start..points.len() {
            let Some(here) = point_value(tree, points[k]) else {
                continue;
            };
            let crossed = k > 0
                && point_value(tree, points[k - 1])
                    .is_some_and(|before| before.min(here) <= value && value <= before.max(here));
            if here.total_cmp(&value).is_eq() || crossed {
                return Ok(Target::plain(points[k]));
            }
        }
        Err(Stop::Invalid(Invalid::NotFound))
    }

    /// The series the cursor is in, else the last series it visited, else the first.
    fn context_series(&self, tree: &NavTree) -> Option<NavNodeId> {
        series_of(tree, self.node)
            .or(self.series)
            .or_else(|| tree.series().next())
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

/// Where a verb goes, before the cursor moves.
enum Target {
    Move {
        node: NavNodeId,
        nearest: bool,
        insight: Option<NavNodeId>,
    },
    Describe,
    Toggle {
        series: NavNodeId,
    },
}

impl Target {
    fn plain(node: NavNodeId) -> Self {
        Self::Move {
            node,
            nearest: false,
            insight: None,
        }
    }
}

/// Why a verb stopped.
enum Stop {
    Edge(Edge),
    Invalid(Invalid),
}

/// The node `delta` steps from `index` in `level`, or the edge it runs into.
fn offset(level: &[NavNodeId], index: usize, delta: isize) -> Result<NavNodeId, Stop> {
    let edge = if delta > 0 { Edge::Last } else { Edge::First };
    index
        .checked_add_signed(delta)
        .and_then(|i| level.get(i).copied())
        .ok_or(Stop::Edge(edge))
}

/// The nodes a spatial verb moves among: all points of the series for a point, the
/// node's siblings otherwise.
fn siblings(tree: &NavTree, node: NavNodeId) -> Vec<NavNodeId> {
    if matches!(tree.node(node).map(|n| &n.kind), Some(NavKind::Point(_)))
        && let Some(series) = series_of(tree, node)
    {
        return tree.points(series);
    }
    match tree.parent(node) {
        Some(parent) => tree.children(parent).to_vec(),
        None => vec![node],
    }
}

/// The peers a "where am I" position counts: the series for a series, the series' points
/// for a point, the siblings otherwise.
fn peers(tree: &NavTree, node: NavNodeId) -> Vec<NavNodeId> {
    match tree.node(node).map(|n| &n.kind) {
        Some(NavKind::Series(_)) => tree.series().collect(),
        _ => siblings(tree, node),
    }
}

/// The series that contains `node`, or `node` itself if it is a series.
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

/// A point's segment.
fn point_segment(tree: &NavTree, node: NavNodeId) -> Option<NavNodeId> {
    matches!(tree.node(node).map(|n| &n.kind), Some(NavKind::Point(_)))
        .then(|| tree.parent(node))
        .flatten()
}

fn point_value(tree: &NavTree, node: NavNodeId) -> Option<f64> {
    match tree.node(node).map(|n| &n.kind) {
        Some(NavKind::Point(p)) => p.value,
        _ => None,
    }
}

/// Every insight with a target point, as (insight, target).
fn insights(tree: &NavTree) -> impl Iterator<Item = (NavNodeId, NavNodeId)> + '_ {
    tree.iter().filter_map(|(id, node)| match &node.kind {
        NavKind::Insight(insight) => insight.target.map(|target| (id, target)),
        _ => None,
    })
}

fn is_insight_target(tree: &NavTree, node: NavNodeId) -> bool {
    insights(tree).any(|(_, target)| target == node)
}

/// The node whose span contains `x`, else the one nearest to it (ties go to the
/// earlier); `nearest` says whether it does not contain `x`. Without `x`, the first node.
fn nearest_by_span(tree: &NavTree, nodes: &[NavNodeId], x: Option<f64>) -> Option<Target> {
    let first = *nodes.first()?;
    let Some(x) = x else {
        return Some(Target::plain(first));
    };
    let distance = |id: NavNodeId| {
        tree.node(id)
            .and_then(|n| n.span)
            .map_or(f64::INFINITY, |s| {
                if s.contains(x) {
                    0.0
                } else {
                    (s.start - x).abs().min((s.end - x).abs())
                }
            })
    };
    let (best, gap) = nodes.iter().map(|&id| (id, distance(id))).fold(
        (first, f64::INFINITY),
        |(b, d), (id, dist)| {
            if dist < d { (id, dist) } else { (b, d) }
        },
    );
    Some(Target::Move {
        node: best,
        nearest: gap > 0.0,
        insight: None,
    })
}

/// The node of a landmark.
fn landmark_node(tree: &NavTree, landmark: Landmark) -> Option<NavNodeId> {
    let axis = |name: AxisName| {
        let axes = tree.landmark("axes")?;
        tree.children(axes)
            .iter()
            .copied()
            .find(|&id| matches!(tree.node(id).map(|n| &n.kind), Some(NavKind::Axis(a)) if a.axis == name))
    };
    match landmark {
        Landmark::XAxis => axis(AxisName::X),
        Landmark::YAxis => axis(AxisName::Y),
        Landmark::Insights => tree.landmark("insights"),
        Landmark::Filters => tree.landmark("filters"),
        Landmark::Dataset => Some(tree.root()),
    }
}

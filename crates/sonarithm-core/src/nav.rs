// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The navigation tree (08 §2, 05 §8): the structure a listener moves through.
//!
//! AUDAXIS builds one navigation tree per compiled Score from the Score, the data and the
//! annotations; the compiler returns it beside the Score, so the CLI, Python and the
//! desktop app expose the same structure. The tree holds data, not words: names come from
//! the spec and the data, and announcements are composed from the nodes by AUDAXIS with the
//! message catalogues (B03).
//!
//! | Level | Node | Children |
//! |---|---|---|
//! | 0 | [`NavKind::Dataset`] (the root) | Series 1…n, Axes, Insights, Filters |
//! | 1 | [`NavKind::Series`] | Segments |
//! | 1 | [`NavKind::Axes`] | One [`NavKind::Axis`] per axis |
//! | 1 | [`NavKind::Insights`] | One [`NavKind::InsightGroup`] per analyzer |
//! | 1 | [`NavKind::Filters`] | One [`NavKind::Filter`] per series |
//! | 2 | Segment, axis, insight group | Points, insights |
//! | 3 | [`NavKind::Point`], [`NavKind::Insight`] | — |
//!
//! The node payloads are plain records that AUDAXIS fills in; [`NavKind`], [`NavNode`] and
//! [`NavTree`] can grow without breaking callers.
//!
//! Nodes live in one vector in creation order; a node's parent always comes before it, so
//! the tree cannot contain a cycle. Every collection is ordered (x order for points), so the
//! same Score gives the same tree on every platform (ADR-0013).

use serde::{Deserialize, Serialize};

use crate::ids::{AnalyzerId, RowRange, RowRef, TrackId};
use crate::provenance::{Confidence, Provenance};
use crate::units::Seconds;

/// A node's index in its [`NavTree`]. The root is node 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NavNodeId(pub u32);

impl NavNodeId {
    /// The root node.
    pub const ROOT: Self = Self(0);

    fn index(self) -> usize {
        usize::try_from(self.0).unwrap_or(usize::MAX)
    }
}

/// A closed interval of positions on the ordering axis, `start <= end`.
///
/// Positions are x values for a numeric or temporal x field (in the column's unit),
/// category indices for a categorical one, and row positions when the series has no x
/// field. Moving into a child or to another series keeps the position (keyboard §4.3).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct XSpan {
    /// The first position.
    pub start: f64,
    /// The last position.
    pub end: f64,
}

impl XSpan {
    /// The span of one position.
    #[must_use]
    pub fn at(position: f64) -> Self {
        Self {
            start: position,
            end: position,
        }
    }

    /// Whether `position` lies inside the span, ends included.
    #[must_use]
    pub fn contains(&self, position: f64) -> bool {
        self.start <= position && position <= self.end
    }
}

/// The smallest and largest value of a set of points.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValueRange {
    /// The minimum.
    pub min: f64,
    /// The maximum.
    pub max: f64,
}

impl ValueRange {
    /// The range of the finite `values`, or `None` if there are none.
    #[must_use]
    pub fn of(values: impl IntoIterator<Item = f64>) -> Option<Self> {
        values
            .into_iter()
            .filter(|v| v.is_finite())
            .fold(None, |range, v| match range {
                None => Some(Self { min: v, max: v }),
                Some(Self { min, max }) => Some(Self {
                    min: min.min(v),
                    max: max.max(v),
                }),
            })
    }
}

/// What the x values of a series are, so they can be spoken in the right form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum XType {
    /// Plain numbers.
    Number,
    /// Calendar dates, as days since 1970-01-01.
    Date,
    /// Instants, as seconds since 1970-01-01T00:00:00 UTC.
    Timestamp,
    /// Categories, in their declared or first-seen order.
    Category,
    /// No x field: points follow the row order.
    RowOrder,
}

/// One x value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum XValue {
    /// A number, date or instant (see the axis's [`XType`]).
    Number(f64),
    /// A category label.
    Category(String),
}

/// Which axis an [`AxisNode`] describes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AxisName {
    /// The ordering axis.
    X,
    /// A value axis.
    Y,
}

/// The root: the dataset and its sonification.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DatasetNode {
    /// The spec's `name`, if it has one.
    pub title: Option<String>,
    /// The number of series.
    pub series: u32,
    /// The number of points in all series.
    pub points: u64,
}

/// A data track as a series of points.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeriesNode {
    /// The track.
    pub track: TrackId,
    /// The field whose values the points carry (the pitch field), if any.
    pub field: Option<String>,
    /// The unit of that field as the data declares it, such as `°C`.
    pub unit: Option<String>,
    /// The unit's UCUM code, such as `Cel`.
    pub ucum: Option<String>,
    /// The number of points.
    pub points: u64,
    /// The range of the point values.
    pub range: Option<ValueRange>,
}

/// One axis: its field, unit and extent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AxisNode {
    /// Which axis.
    pub axis: AxisName,
    /// The field on the axis, if any.
    pub field: Option<String>,
    /// The unit as the data declares it.
    pub unit: Option<String>,
    /// The unit's UCUM code.
    pub ucum: Option<String>,
    /// What the values are (the x axis); `Number` for value axes.
    pub values: XType,
    /// The extent of the values on the axis.
    pub range: Option<ValueRange>,
}

/// The insights of one analyzer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InsightGroupNode {
    /// The analyzer, such as `extrema`.
    pub analyzer: AnalyzerId,
}

/// One analyzer annotation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InsightNode {
    /// The analyzer that produced it.
    pub analyzer: AnalyzerId,
    /// The rows it covers.
    pub rows: RowRange,
    /// The value it reports, if any.
    pub value: Option<f64>,
    /// How confident the analyzer is.
    pub confidence: Confidence,
    /// The first point it covers, where "go to the data" lands.
    pub target: Option<NavNodeId>,
}

/// A filter that takes one series in or out of playback and view.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FilterNode {
    /// The series it filters.
    pub series: NavNodeId,
    /// That series' track.
    pub track: TrackId,
}

/// A run of consecutive points of one series.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SegmentNode {
    /// Whether the segment comes from an analysis (a trend segment) rather than even bins.
    pub semantic: bool,
    /// The number of points.
    pub points: u64,
    /// The range of the point values.
    pub range: Option<ValueRange>,
}

/// One data row of one series.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PointNode {
    /// The data row.
    pub row: RowRef,
    /// The point's position in its series, from 0, in x order.
    pub index: u64,
    /// When the point plays in the Score, if it is played.
    pub t: Option<Seconds>,
    /// The x value; `None` in row order.
    pub x: Option<XValue>,
    /// The value; `None` for a missing value (03 §5).
    pub value: Option<f64>,
    /// Where the value comes from (03 §4). Estimates must be announced as such.
    pub provenance: Provenance,
}

/// What a node is, with the data each kind carries. Serialized with a `kind` tag.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
#[non_exhaustive]
pub enum NavKind {
    /// The root.
    Dataset(DatasetNode),
    /// A data series.
    Series(SeriesNode),
    /// The group of axes.
    Axes,
    /// One axis.
    Axis(AxisNode),
    /// The group of insights.
    Insights,
    /// The insights of one analyzer.
    InsightGroup(InsightGroupNode),
    /// One insight.
    Insight(InsightNode),
    /// The group of series filters.
    Filters,
    /// One series filter.
    Filter(FilterNode),
    /// A segment of a series.
    Segment(SegmentNode),
    /// A data point.
    Point(PointNode),
}

impl NavKind {
    /// The kind's name, as in the serialized tag.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Dataset(_) => "dataset",
            Self::Series(_) => "series",
            Self::Axes => "axes",
            Self::Axis(_) => "axis",
            Self::Insights => "insights",
            Self::InsightGroup(_) => "insight-group",
            Self::Insight(_) => "insight",
            Self::Filters => "filters",
            Self::Filter(_) => "filter",
            Self::Segment(_) => "segment",
            Self::Point(_) => "point",
        }
    }
}

/// One node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NavNode {
    /// What the node is.
    #[serde(flatten)]
    pub kind: NavKind,
    /// A name from the spec or the data: a track ID, a field, a category, an insight label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The node's extent on the ordering axis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<XSpan>,
    /// The parent; `None` only for the root.
    pub parent: Option<NavNodeId>,
    /// The children, in navigation order.
    pub children: Vec<NavNodeId>,
}

impl NavNode {
    /// A node of `kind` with no label, extent, parent or children yet.
    #[must_use]
    pub fn new(kind: NavKind) -> Self {
        Self {
            kind,
            label: None,
            span: None,
            parent: None,
            children: Vec::new(),
        }
    }

    /// The same node with a label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The same node with an extent.
    #[must_use]
    pub fn with_span(mut self, span: Option<XSpan>) -> Self {
        self.span = span;
        self
    }
}

/// Why a [`NavTree`] is malformed.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum NavTreeError {
    /// The tree has no nodes.
    #[error("the tree has no root")]
    Empty,
    /// The root is not a dataset node, or has a parent.
    #[error("node 0 must be a dataset node without a parent")]
    BadRoot,
    /// A node names a node that does not exist.
    #[error("node {node} refers to missing node {missing}")]
    Dangling {
        /// The node.
        node: u32,
        /// The missing node.
        missing: u32,
    },
    /// A node's parent does not list it as a child, or lists it twice.
    #[error("node {node} and its parent disagree")]
    Inconsistent {
        /// The node.
        node: u32,
    },
    /// A node is not reachable from the root.
    #[error("node {node} is not reachable from the root")]
    Unreachable {
        /// The node.
        node: u32,
    },
}

/// The navigation tree of one compiled Score.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavTree {
    nodes: Vec<NavNode>,
}

impl NavTree {
    /// A tree with only its root.
    #[must_use]
    pub fn new(root: DatasetNode) -> Self {
        Self {
            nodes: vec![NavNode::new(NavKind::Dataset(root))],
        }
    }

    /// Adds `node` as the last child of `parent` and returns its ID. An unknown parent
    /// adds the node under the root.
    pub fn push(&mut self, parent: NavNodeId, mut node: NavNode) -> NavNodeId {
        let parent = if parent.index() < self.nodes.len() {
            parent
        } else {
            NavNodeId::ROOT
        };
        let id = NavNodeId(u32::try_from(self.nodes.len()).unwrap_or(u32::MAX));
        node.parent = Some(parent);
        node.children.clear();
        self.nodes.push(node);
        if let Some(parent) = self.nodes.get_mut(parent.index()) {
            parent.children.push(id);
        }
        id
    }

    /// The root node's ID.
    #[must_use]
    pub fn root(&self) -> NavNodeId {
        NavNodeId::ROOT
    }

    /// The number of nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the tree has no nodes (never true for a tree made with [`NavTree::new`]).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The node with this ID.
    #[must_use]
    pub fn node(&self, id: NavNodeId) -> Option<&NavNode> {
        self.nodes.get(id.index())
    }

    /// The node with this ID, for builders that fill in a node after adding its children.
    pub fn node_mut(&mut self, id: NavNodeId) -> Option<&mut NavNode> {
        self.nodes.get_mut(id.index())
    }

    /// Every node with its ID, in creation order.
    pub fn iter(&self) -> impl Iterator<Item = (NavNodeId, &NavNode)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (NavNodeId(u32::try_from(i).unwrap_or(u32::MAX)), node))
    }

    /// The parent of `id`.
    #[must_use]
    pub fn parent(&self, id: NavNodeId) -> Option<NavNodeId> {
        self.node(id).and_then(|node| node.parent)
    }

    /// The children of `id`, in navigation order.
    #[must_use]
    pub fn children(&self, id: NavNodeId) -> &[NavNodeId] {
        self.node(id).map_or(&[], |node| node.children.as_slice())
    }

    /// The depth of `id`: 0 for the root.
    #[must_use]
    pub fn depth(&self, id: NavNodeId) -> usize {
        let mut depth = 0;
        let mut current = id;
        while let Some(parent) = self.parent(current) {
            depth += 1;
            current = parent;
            if depth > self.nodes.len() {
                break;
            }
        }
        depth
    }

    /// The series nodes, in Score order.
    pub fn series(&self) -> impl Iterator<Item = NavNodeId> + '_ {
        self.children(self.root())
            .iter()
            .copied()
            .filter(|&id| matches!(self.node(id).map(|n| &n.kind), Some(NavKind::Series(_))))
    }

    /// The points of a series in x order, across its segments.
    #[must_use]
    pub fn points(&self, series: NavNodeId) -> Vec<NavNodeId> {
        self.children(series)
            .iter()
            .flat_map(|&segment| self.children(segment).iter().copied())
            .filter(|&id| matches!(self.node(id).map(|n| &n.kind), Some(NavKind::Point(_))))
            .collect()
    }

    /// The first node of a kind directly under the root, such as the axes group.
    #[must_use]
    pub fn landmark(&self, kind: &str) -> Option<NavNodeId> {
        self.children(self.root())
            .iter()
            .copied()
            .find(|&id| self.node(id).is_some_and(|n| n.kind.name() == kind))
    }

    /// Checks the tree's invariants: a dataset root without a parent, no dangling IDs,
    /// parents and children that agree, and every node reachable from the root.
    ///
    /// # Errors
    ///
    /// The first [`NavTreeError`] found.
    pub fn check(&self) -> Result<(), NavTreeError> {
        let root = self.nodes.first().ok_or(NavTreeError::Empty)?;
        if root.parent.is_some() || !matches!(root.kind, NavKind::Dataset(_)) {
            return Err(NavTreeError::BadRoot);
        }
        let count = self.nodes.len();
        for (id, node) in self.iter() {
            for &child in &node.children {
                let Some(child_node) = self.node(child) else {
                    return Err(NavTreeError::Dangling {
                        node: id.0,
                        missing: child.0,
                    });
                };
                if child_node.parent != Some(id) || child.index() <= id.index() {
                    return Err(NavTreeError::Inconsistent { node: child.0 });
                }
            }
            if let Some(parent) = node.parent {
                let Some(parent_node) = self.node(parent) else {
                    return Err(NavTreeError::Dangling {
                        node: id.0,
                        missing: parent.0,
                    });
                };
                let listed = parent_node.children.iter().filter(|&&c| c == id).count();
                if listed != 1 {
                    return Err(NavTreeError::Inconsistent { node: id.0 });
                }
            }
        }
        let mut reached = vec![false; count];
        let mut stack = vec![NavNodeId::ROOT];
        while let Some(id) = stack.pop() {
            if let Some(seen) = reached.get_mut(id.index()) {
                if *seen {
                    continue;
                }
                *seen = true;
            }
            stack.extend(self.children(id).iter().copied());
        }
        match reached.iter().position(|seen| !seen) {
            Some(index) => Err(NavTreeError::Unreachable {
                node: u32::try_from(index).unwrap_or(u32::MAX),
            }),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{Generation, TableId};

    fn point(row: u32) -> NavNode {
        NavNode::new(NavKind::Point(PointNode {
            row: RowRef {
                table: TableId(0),
                generation: Generation(0),
                row: u64::from(row),
            },
            index: u64::from(row),
            t: None,
            x: Some(XValue::Number(f64::from(row))),
            value: Some(1.0),
            provenance: Provenance::EXACT,
        }))
        .with_span(Some(XSpan::at(f64::from(row))))
    }

    fn small_tree() -> NavTree {
        let mut tree = NavTree::new(DatasetNode::default());
        let series = tree.push(
            NavNodeId::ROOT,
            NavNode::new(NavKind::Series(SeriesNode {
                track: TrackId::new("y"),
                field: None,
                unit: None,
                ucum: None,
                points: 3,
                range: None,
            })),
        );
        let segment = tree.push(
            series,
            NavNode::new(NavKind::Segment(SegmentNode {
                semantic: false,
                points: 3,
                range: None,
            })),
        );
        for row in 0..3 {
            tree.push(segment, point(row));
        }
        tree.push(NavNodeId::ROOT, NavNode::new(NavKind::Axes));
        tree
    }

    #[test]
    fn a_built_tree_passes_its_checks() {
        let tree = small_tree();
        assert_eq!(tree.check(), Ok(()));
        assert_eq!(tree.len(), 7);
        let series = tree.series().collect::<Vec<_>>();
        assert_eq!(series, [NavNodeId(1)]);
        assert_eq!(tree.points(NavNodeId(1)).len(), 3);
        assert_eq!(tree.depth(NavNodeId(3)), 3);
        assert_eq!(tree.landmark("axes"), Some(NavNodeId(6)));
    }

    #[test]
    fn checks_find_broken_trees() {
        let mut tree = small_tree();
        tree.nodes[3].parent = Some(NavNodeId(1));
        assert!(matches!(
            tree.check(),
            Err(NavTreeError::Inconsistent { .. })
        ));

        let mut tree = small_tree();
        tree.nodes[2].children.push(NavNodeId(99));
        assert!(matches!(tree.check(), Err(NavTreeError::Dangling { .. })));

        let mut tree = small_tree();
        let orphan = NavNode::new(NavKind::Axes);
        tree.nodes.push(orphan);
        assert!(matches!(
            tree.check(),
            Err(NavTreeError::Unreachable { .. })
        ));
    }

    #[test]
    fn serialized_nodes_carry_a_kind_tag() {
        let tree = small_tree();
        let json = serde_json::to_value(&tree).unwrap();
        assert_eq!(json["nodes"][0]["kind"], "dataset");
        assert_eq!(json["nodes"][3]["kind"], "point");
        let back: NavTree = serde_json::from_value(json).unwrap();
        assert_eq!(back, tree);
    }

    #[test]
    fn value_ranges_ignore_missing_values() {
        let range = ValueRange::of([3.0, f64::NAN, -1.0, 2.0]);
        assert_eq!(
            range,
            Some(ValueRange {
                min: -1.0,
                max: 3.0
            })
        );
        assert_eq!(ValueRange::of([f64::NAN]), None);
    }
}

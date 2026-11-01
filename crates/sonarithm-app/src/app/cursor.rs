// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The one cursor (B04-T06; 08 §1, A11Y-10, A11Y-11; ADR-0017 "Cursor").
//!
//! Rust owns the only cursor: a dataset row, and the chart explorer's node beside it. The
//! table, the plot and the explorer (B03-T09's `nav_move` and `nav_where`, in `nav.rs`)
//! all read and move this one. Every change,
//! whatever its source, goes out as a `cursor` event, so each view follows it. When the data
//! under it changes, the cursor follows: a closed or replaced dataset clears it, and a
//! dataset that lost rows (corrections, undo) clamps it to its last row.

use std::collections::BTreeSet;

use sonarithm_audaxis::announce::{Composer, Memory};
use sonarithm_audaxis::nav::Cursor;
use sonarithm_core::TableId;
use sonarithm_core::nav::{NavKind, NavNodeId, NavTree, XValue};

use super::{App, unknown_dataset};
use crate::codes;
use crate::ipc::{
    CursorOrigin, CursorPathItem, CursorState, Event, IpcError, PathLevel, Reply, SeriesRef,
};

/// Where the cursor is: a row of an open dataset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CursorAt {
    /// The dataset's session ID, such as `d1`.
    pub(super) dataset: String,
    /// The zero-based row in the dataset's first table.
    pub(super) row: u64,
}

/// The cursor and what was last sent about it.
#[derive(Debug)]
pub(super) struct CursorModel {
    pub(super) at: Option<CursorAt>,
    /// The explorer's node, x and series (B03-T09): on the row's point after a table or
    /// plot move, anywhere after a verb. `None` without a navigation tree, or when the row
    /// is not a point.
    pub(super) nav: Option<Cursor>,
    /// The series the filters took out of playback and view; kept when `nav` is `None`.
    pub(super) excluded: BTreeSet<NavNodeId>,
    /// What the explorer's announcements remember between commands (keyboard §8.1).
    pub(super) memory: Memory,
    seq: u64,
    /// The last state sent, without its `seq` and `origin`: a change is a difference here.
    sent: Option<CursorState>,
}

impl Default for CursorModel {
    /// No cursor, and the views already know it: the first command sends nothing.
    fn default() -> Self {
        Self {
            at: None,
            nav: None,
            excluded: BTreeSet::new(),
            memory: Memory::default(),
            seq: 0,
            sent: Some(CursorState::default()),
        }
    }
}

impl App {
    /// `cursor_set`: moves the one cursor to `row` of `dataset` and tells every view.
    ///
    /// What is spoken depends on where the move came from (ADR-0017 "Cursor", INFER,
    /// OD-30): from the table, the screen reader already reads the focused row, so nothing
    /// is announced; from the plot or the explorer, the point is announced as a move.
    pub(super) fn cursor_set(
        &mut self,
        dataset: &str,
        row: u64,
        origin: CursorOrigin,
    ) -> Result<Reply, IpcError> {
        let rows = self
            .datasets
            .values()
            .find(|open| open.summary.id == dataset)
            .map(|open| open.summary.rows)
            .ok_or_else(|| unknown_dataset(dataset))?;
        if row >= rows {
            return Err(IpcError::new(
                &codes::CURSOR_ROW,
                format!("Row {row} is outside the dataset, which has {rows} rows."),
            ));
        }
        self.cursor.at = Some(CursorAt {
            dataset: dataset.to_owned(),
            row,
        });
        self.place_nav();
        let state = self.send_cursor(Some(origin), true);
        // The table and braille routing keys are not announced: the screen reader already
        // reads the focused row or the braille field (INFER, OD-30).
        if !matches!(origin, CursorOrigin::Table | CursorOrigin::Braille)
            && let Some(node) = state.node.map(NavNodeId)
            && let Some((announcement, memory)) = self.cursor_announcement(node)
        {
            // The explorer's composer memory carries over, so a series named here is not
            // named again by the next explorer move (keyboard §8.1).
            self.cursor.memory = memory;
            self.transport.announce(announcement);
        }
        Ok(Reply::Cursor(state))
    }

    /// `cursor_get`: where the cursor is.
    pub(super) fn cursor_get(&self) -> Reply {
        let mut state = self.cursor_state();
        state.seq = self.cursor.seq;
        Reply::Cursor(state)
    }

    /// After every command: follows the data under the cursor (clears it when its dataset is
    /// gone, clamps it when the dataset shrank) and sends a `cursor` event if anything a
    /// view shows changed, such as the point node after a compile.
    pub(super) fn sync_cursor(&mut self) {
        let before = self.cursor.at.clone();
        if let Some(at) = self.cursor.at.clone() {
            let rows = self
                .datasets
                .values()
                .find(|open| open.summary.id == at.dataset)
                .map(|open| open.summary.rows);
            self.cursor.at = match rows {
                None | Some(0) => None,
                Some(rows) => Some(CursorAt {
                    row: at.row.min(rows - 1),
                    ..at
                }),
            };
        }
        if self.cursor.at != before {
            self.place_nav();
        }
        self.send_cursor(None, false);
    }

    /// Clears the cursor: the session's datasets were replaced (a project opened or a new
    /// one started), so a row of the old data means nothing.
    pub(super) fn clear_cursor(&mut self) {
        self.cursor.at = None;
        self.cursor.nav = None;
        self.cursor.excluded.clear();
        self.cursor.memory.clear();
    }

    /// The point of the cursor's row in the compiled Score, if the row is one.
    pub(super) fn row_point(&self) -> Option<NavNodeId> {
        self.row_point_in(self.cursor.nav.as_ref().and_then(Cursor::series))
    }

    /// The point of the cursor's row, in `prefer` when the row is a point there.
    pub(super) fn row_point_in(&self, prefer: Option<NavNodeId>) -> Option<NavNodeId> {
        let at = self.cursor.at.as_ref()?;
        let open = self
            .datasets
            .values()
            .find(|o| o.summary.id == at.dataset)?;
        let table = open.bundle.primary().map(sonarithm_data::Table::id)?;
        let tree = self.compiled.as_ref()?.nav()?;
        point_of(tree, table, at.row, prefer, &self.cursor.excluded)
    }

    /// Sends the cursor as an event when `always`, or when what it shows changed.
    pub(super) fn send_cursor(
        &mut self,
        origin: Option<CursorOrigin>,
        always: bool,
    ) -> CursorState {
        let mut state = self.cursor_state();
        let unchanged = self.cursor.sent.as_ref() == Some(&state);
        if always || !unchanged {
            self.cursor.seq += 1;
            self.cursor.sent = Some(state.clone());
            state.seq = self.cursor.seq;
            state.origin = origin;
            self.sink.emit(Event::Cursor(state.clone()));
        } else {
            state.seq = self.cursor.seq;
        }
        state
    }

    /// The cursor as views show it, with `seq` 0 and no origin: the dataset and row, the
    /// explorer's node (the row's point after a table or plot move), its x value, the path
    /// from the dataset to it and the excluded series.
    fn cursor_state(&self) -> CursorState {
        let mut state = CursorState::default();
        let tree = self.compiled.as_ref().and_then(|compiled| compiled.nav());
        // An explorer cursor can be on the root or a group before any row, and a filter
        // can exclude a series before any row.
        let explored = self.cursor.nav.is_some() || !self.cursor.excluded.is_empty();
        let dataset = match (&self.cursor.at, tree) {
            (Some(at), _) => Some(at.dataset.clone()),
            (None, Some(tree)) if explored => self.dataset_of_tree(tree),
            _ => None,
        };
        let Some(dataset) = dataset else {
            return state;
        };
        state.row = self.cursor.at.as_ref().map(|at| at.row);
        let open = self.datasets.values().find(|o| o.summary.id == dataset);
        state.dataset = Some(dataset);
        let Some(open) = open else {
            return state;
        };
        state.path.push(CursorPathItem {
            level: PathLevel::Dataset,
            label: open.summary.name.clone(),
        });
        let Some(tree) = tree else {
            return state;
        };
        let node = match &self.cursor.nav {
            Some(cursor) => Some(cursor.node()),
            None => self.row_point(),
        };
        if let Some(node) = node {
            state.node = Some(node.0);
            state.x = match tree.node(node).map(|n| &n.kind) {
                Some(NavKind::Point(point)) => match &point.x {
                    Some(XValue::Number(x)) => Some(*x),
                    _ => None,
                },
                _ => self.cursor.nav.as_ref().and_then(Cursor::x),
            };
            state.path.extend(path_of(tree, node));
            state.braille = self.braille_state(node);
        }
        state.excluded = self
            .cursor
            .excluded
            .iter()
            .map(|&id| SeriesRef {
                node: id.0,
                label: tree
                    .node(id)
                    .and_then(|node| node.label.clone())
                    .unwrap_or_default(),
            })
            .collect();
        state
    }

    /// The open dataset whose first table the tree's points read.
    fn dataset_of_tree(&self, tree: &NavTree) -> Option<String> {
        let table = tree.iter().find_map(|(_, node)| match &node.kind {
            NavKind::Point(point) => Some(point.row.table),
            _ => None,
        })?;
        self.datasets
            .values()
            .find(|open| open.bundle.primary().map(sonarithm_data::Table::id) == Some(table))
            .map(|open| open.summary.id.clone())
    }

    /// The announcement for arriving at `node`, in the listener's verbosity.
    fn cursor_announcement(
        &self,
        node: NavNodeId,
    ) -> Option<(sonarithm_audaxis::announce::Announcement, Memory)> {
        let tree = self.compiled.as_ref()?.nav()?;
        let localizer = self.localizer.as_ref()?;
        let mut composer = Composer::with_memory(
            localizer,
            self.composer_settings(),
            self.cursor.memory.clone(),
        );
        let announcement = composer.node(tree, node).ok()?;
        Some((announcement, composer.memory()))
    }
}

/// The point for `row` of `table`. A row read by several tracks is a point in each: the
/// cursor takes the one in `prefer` (the explorer's series), else the first in a series the
/// filters keep, else the first, as the playhead's current value does.
fn point_of(
    tree: &NavTree,
    table: TableId,
    row: u64,
    prefer: Option<NavNodeId>,
    excluded: &BTreeSet<NavNodeId>,
) -> Option<NavNodeId> {
    let is_row = |id: NavNodeId| {
        matches!(tree.node(id).map(|node| &node.kind),
            Some(NavKind::Point(point)) if point.row.table == table && point.row.row == row)
    };
    if let Some(series) = prefer
        && let Some(point) = tree.points(series).into_iter().find(|&p| is_row(p))
    {
        return Some(point);
    }
    let mut first = None;
    for (id, _) in tree.iter() {
        if is_row(id) {
            if series_of(tree, id).is_none_or(|series| !excluded.contains(&series)) {
                return Some(id);
            }
            first.get_or_insert(id);
        }
    }
    first
}

/// The series above `node`, if any.
fn series_of(tree: &NavTree, node: NavNodeId) -> Option<NavNodeId> {
    let mut at = Some(node);
    while let Some(id) = at {
        if matches!(tree.node(id).map(|n| &n.kind), Some(NavKind::Series(_))) {
            return Some(id);
        }
        at = tree.parent(id);
    }
    None
}

/// The levels above and at `node` below the dataset, outermost first. Labels are data
/// names (a track or field, a segment's span, the point's x, an axis' field), never UI
/// words: the UI names each level through its own catalogue, and a group (axes, insights,
/// filters) has an empty label.
fn path_of(tree: &NavTree, node: NavNodeId) -> Vec<CursorPathItem> {
    let mut items = Vec::new();
    let mut at = Some(node);
    while let Some(id) = at {
        let Some(current) = tree.node(id) else { break };
        let item = match &current.kind {
            NavKind::Series(_) => Some(CursorPathItem {
                level: PathLevel::Series,
                label: current.label.clone().unwrap_or_default(),
            }),
            NavKind::Segment(_) => Some(CursorPathItem {
                level: PathLevel::Segment,
                label: current.label.clone().unwrap_or_else(|| {
                    current.span.map_or_else(String::new, |span| {
                        format!("{}–{}", number(span.start), number(span.end))
                    })
                }),
            }),
            NavKind::Axes => Some(group(PathLevel::Axes)),
            NavKind::Insights => Some(group(PathLevel::Insights)),
            NavKind::Filters => Some(group(PathLevel::Filters)),
            NavKind::Axis(_) => Some(labelled(PathLevel::Axis, current.label.as_ref())),
            NavKind::InsightGroup(_) => {
                Some(labelled(PathLevel::InsightGroup, current.label.as_ref()))
            }
            NavKind::Insight(_) => Some(labelled(PathLevel::Insight, current.label.as_ref())),
            NavKind::Filter(_) => Some(labelled(PathLevel::Filter, current.label.as_ref())),
            NavKind::Point(point) => Some(CursorPathItem {
                level: PathLevel::Point,
                label: match &point.x {
                    Some(XValue::Number(x)) => number(*x),
                    Some(XValue::Category(name)) => name.clone(),
                    None => (point.index + 1).to_string(),
                },
            }),
            _ => None,
        };
        items.extend(item);
        at = current.parent;
    }
    items.reverse();
    items
}

fn group(level: PathLevel) -> CursorPathItem {
    CursorPathItem {
        level,
        label: String::new(),
    }
}

fn labelled(level: PathLevel, label: Option<&String>) -> CursorPathItem {
    CursorPathItem {
        level,
        label: label.cloned().unwrap_or_default(),
    }
}

/// A number as data text: no trailing zeros, no grouping (the UI formats for its locale).
fn number(value: f64) -> String {
    let text = format!("{value}");
    text.strip_suffix(".0").map_or(text.clone(), str::to_owned)
}

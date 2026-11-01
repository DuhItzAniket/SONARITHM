// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The one cursor (B04-T06; 08 §1, A11Y-10, A11Y-11; ADR-0017 "Cursor").
//!
//! Rust owns the only cursor: a dataset row. The table, the plot and later the chart
//! explorer (B03-T09's `nav_move` and `nav_where`) all read and move this one. Every change,
//! whatever its source, goes out as a `cursor` event, so each view follows it. When the data
//! under it changes, the cursor follows: a closed or replaced dataset clears it, and a
//! dataset that lost rows (corrections, undo) clamps it to its last row.

use sonarithm_audaxis::announce::Composer;
use sonarithm_core::TableId;
use sonarithm_core::nav::{NavKind, NavNodeId, NavTree, XValue};

use super::{App, unknown_dataset};
use crate::codes;
use crate::ipc::{CursorOrigin, CursorPathItem, CursorState, Event, IpcError, PathLevel, Reply};

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
    seq: u64,
    /// The last state sent, without its `seq` and `origin`: a change is a difference here.
    sent: Option<CursorState>,
}

impl Default for CursorModel {
    /// No cursor, and the views already know it: the first command sends nothing.
    fn default() -> Self {
        Self {
            at: None,
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
        let state = self.send_cursor(Some(origin), true);
        if origin != CursorOrigin::Table
            && let Some(node) = state.node.map(NavNodeId)
            && let Some(announcement) = self.cursor_announcement(node)
        {
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
        self.send_cursor(None, false);
    }

    /// Clears the cursor: the session's datasets were replaced (a project opened or a new
    /// one started), so a row of the old data means nothing.
    pub(super) fn clear_cursor(&mut self) {
        self.cursor.at = None;
    }

    /// Sends the cursor as an event when `always`, or when what it shows changed.
    fn send_cursor(&mut self, origin: Option<CursorOrigin>, always: bool) -> CursorState {
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
    /// point node of the compiled Score when the row is one, its x value and the path from
    /// the dataset to it.
    fn cursor_state(&self) -> CursorState {
        let mut state = CursorState::default();
        let Some(at) = &self.cursor.at else {
            return state;
        };
        state.dataset = Some(at.dataset.clone());
        state.row = Some(at.row);
        let Some(open) = self.datasets.values().find(|o| o.summary.id == at.dataset) else {
            return state;
        };
        state.path.push(CursorPathItem {
            level: PathLevel::Dataset,
            label: open.summary.name.clone(),
        });
        let table = open.bundle.primary().map(sonarithm_data::Table::id);
        let tree = self.compiled.as_ref().and_then(|compiled| compiled.nav());
        if let (Some(table), Some(tree)) = (table, tree)
            && let Some(node) = point_of(tree, table, at.row)
        {
            state.node = Some(node.0);
            if let Some(NavKind::Point(point)) = tree.node(node).map(|n| &n.kind)
                && let Some(XValue::Number(x)) = &point.x
            {
                state.x = Some(*x);
            }
            state.path.extend(path_of(tree, node));
        }
        state
    }

    /// The announcement for arriving at `node`, in the listener's verbosity.
    fn cursor_announcement(
        &self,
        node: NavNodeId,
    ) -> Option<sonarithm_audaxis::announce::Announcement> {
        let tree = self.compiled.as_ref()?.nav()?;
        let localizer = self.localizer.as_ref()?;
        Composer::new(localizer, self.composer_settings())
            .node(tree, node)
            .ok()
    }
}

/// The first series' point for `row` of `table`: a row read by several tracks is a point
/// in each, and the cursor takes the first, as the playhead's current value does.
fn point_of(tree: &NavTree, table: TableId, row: u64) -> Option<NavNodeId> {
    tree.iter().find_map(|(id, node)| match &node.kind {
        NavKind::Point(point) if point.row.table == table && point.row.row == row => Some(id),
        _ => None,
    })
}

/// The series, segment and point levels above and at `node`, outermost first. Labels are
/// data names (a track or field, a segment's span, the point's x), never UI words: the UI
/// names each level through its own catalogue.
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

/// A number as data text: no trailing zeros, no grouping (the UI formats for its locale).
fn number(value: f64) -> String {
    let text = format!("{value}");
    text.strip_suffix(".0").map_or(text.clone(), str::to_owned)
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The braille line on the one cursor (B03-T10; keyboard-and-navigation §10, §14; A11Y-21,
//! A11Y-27, A11Y-10).
//!
//! With braille output on, every `cursor` event carries the line for the cursor's node:
//! the review line (the composer's parts in symbol form, fitted to the user's cell count)
//! or, after Shift+B, the sparkline of the cursor's segment. A routing-key press on the
//! sparkline moves the one cursor to the point under the cell, with origin `braille`,
//! which is not announced, as for the table (INFER, OD-30). In review mode routing changes
//! nothing (keyboard §14). The cell count and the mode are app settings: they belong to
//! the user's device, not to a profile.

use sonarithm_audaxis::announce::Composer;
use sonarithm_audaxis::braille::{review_line, route, sparkline};
use sonarithm_core::nav::{NavKind, NavNodeId};

use super::App;
use crate::ipc::{BrailleMode, BrailleState, CursorOrigin, IpcError, Reply};

/// The braille modality's place in the effective modalities: sonification, speech,
/// braille, visual.
const BRAILLE: usize = 2;

impl App {
    /// The braille line for the cursor's `node`, or `None` when braille output is off or
    /// nothing has been compiled. In sparkline mode a node that is not a point shows its
    /// review line, and `mode` says which one is shown.
    pub(super) fn braille_state(&self, node: NavNodeId) -> Option<BrailleState> {
        if !self.interaction().modalities[BRAILLE] {
            return None;
        }
        let tree = self.compiled.as_ref()?.nav()?;
        let cells = self.settings.braille.cells;
        let fit = cells.map(usize::from);
        if self.settings.braille.mode == BrailleMode::Sparkline
            && let Some(line) = sparkline(tree, node, fit)
        {
            return Some(BrailleState {
                mode: BrailleMode::Sparkline,
                text: line.text,
                caret: u32::try_from(line.caret).unwrap_or(u32::MAX),
                cells,
            });
        }
        let localizer = self.localizer.as_ref()?;
        let fields = Composer::new(localizer, self.composer_settings())
            .review_fields(tree, node)
            .ok()?;
        Some(BrailleState {
            mode: BrailleMode::Review,
            text: review_line(&fields, fit),
            caret: 0,
            cells,
        })
    }

    /// `braille_route`: a routing-key press on cell `cell`. On a sparkline it moves the one
    /// cursor to the point under the cell (origin `braille`, not announced); anywhere else,
    /// or past the last cell, the cursor stays and the reply is where it is.
    ///
    /// # Errors
    ///
    /// As `cursor_set`, which cannot fail for a point of the compiled Score.
    pub(super) fn braille_route(&mut self, cell: u32) -> Result<Reply, IpcError> {
        let target = self.braille_target(cell);
        match target {
            Some((dataset, row)) => self.cursor_set(&dataset, row, CursorOrigin::Braille),
            None => Ok(self.cursor_get()),
        }
    }

    /// The dataset and row under sparkline cell `cell`, if the line is a sparkline.
    fn braille_target(&self, cell: u32) -> Option<(String, u64)> {
        if self.settings.braille.mode != BrailleMode::Sparkline {
            return None;
        }
        let tree = self.compiled.as_ref()?.nav()?;
        let node = self
            .cursor
            .nav
            .as_ref()
            .map(sonarithm_audaxis::nav::Cursor::node)
            .or_else(|| self.row_point())?;
        let line = sparkline(tree, node, self.settings.braille.cells.map(usize::from))?;
        let point = route(&line, usize::try_from(cell).ok()?)?;
        let NavKind::Point(point) = &tree.node(point)?.kind else {
            return None;
        };
        let dataset = self
            .datasets
            .values()
            .find(|open| {
                open.bundle.primary().map(sonarithm_data::Table::id) == Some(point.row.table)
            })?
            .summary
            .id
            .clone();
        Some((dataset, point.row.row))
    }
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The undo history of the command bus: every undoable command records the state before
//! and after it under an undo token (FR-111). B05-T01 adds spec patches as edits.

use std::collections::VecDeque;

/// The most edits kept; the oldest is forgotten first.
pub(crate) const DEPTH: usize = 200;

/// One recorded edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Edit<S> {
    /// The token the command returned.
    pub(crate) token: u64,
    /// What the edit did, such as `Set spec`.
    pub(crate) label: String,
    /// The state before.
    pub(crate) before: S,
    /// The state after.
    pub(crate) after: S,
}

/// Undo and redo stacks of states `S`.
#[derive(Clone, Debug)]
pub(crate) struct History<S> {
    undo: VecDeque<Edit<S>>,
    redo: Vec<Edit<S>>,
    next: u64,
}

impl<S> Default for History<S> {
    fn default() -> Self {
        Self {
            undo: VecDeque::new(),
            redo: Vec::new(),
            next: 1,
        }
    }
}

impl<S: Clone> History<S> {
    /// Records an edit and returns its token. A new edit clears the redo stack.
    pub(crate) fn record(&mut self, label: impl Into<String>, before: S, after: S) -> u64 {
        let token = self.next;
        self.next += 1;
        self.redo.clear();
        if self.undo.len() == DEPTH {
            self.undo.pop_front();
        }
        self.undo.push_back(Edit {
            token,
            label: label.into(),
            before,
            after,
        });
        token
    }

    /// Steps back: the edit undone, whose `before` is the state to restore.
    pub(crate) fn undo(&mut self) -> Option<Edit<S>> {
        let edit = self.undo.pop_back()?;
        self.redo.push(edit.clone());
        Some(edit)
    }

    /// Steps forward again: the edit redone, whose `after` is the state to restore.
    pub(crate) fn redo(&mut self) -> Option<Edit<S>> {
        let edit = self.redo.pop()?;
        self.undo.push_back(edit.clone());
        Some(edit)
    }

    /// Whether an undo is possible.
    #[must_use]
    pub(crate) fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether a redo is possible.
    #[must_use]
    pub(crate) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn tokens_count_up_and_redo_clears_on_new_edit() {
        let mut history = History::default();
        assert_eq!(history.record("a", 0, 1), 1);
        assert_eq!(history.record("b", 1, 2), 2);
        assert_eq!(history.undo().map(|e| e.token), Some(2));
        assert!(history.can_redo());
        assert_eq!(history.record("c", 1, 3), 3);
        assert!(!history.can_redo());
    }

    #[test]
    fn depth_is_bounded() {
        let mut history = History::default();
        for i in 0..(DEPTH + 10) {
            history.record("x", i, i + 1);
        }
        let mut undone = 0;
        while history.undo().is_some() {
            undone += 1;
        }
        assert_eq!(undone, DEPTH);
    }

    proptest! {
        /// Undoing every edit restores the first state; redoing them all restores the last.
        #[test]
        fn undo_all_then_redo_all_round_trips(steps in proptest::collection::vec(any::<u8>(), 1..50)) {
            let mut history = History::default();
            let mut state = 0_u32;
            for step in &steps {
                let next = state.wrapping_mul(31).wrapping_add(u32::from(*step));
                history.record("step", state, next);
                state = next;
            }
            let last = state;
            while let Some(edit) = history.undo() {
                state = edit.before;
            }
            prop_assert_eq!(state, 0);
            while let Some(edit) = history.redo() {
                state = edit.after;
            }
            prop_assert_eq!(state, last);
        }
    }
}

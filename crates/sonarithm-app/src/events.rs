// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Where events go: the host's [`EventSink`] (ADR-0017 §4).

use std::sync::{Arc, Mutex, PoisonError};

use crate::ipc::Event;

/// Receives events from the app core's threads. The Tauri host forwards each to the
/// channel [`Event::channel`] names; a test collects them.
///
/// `emit` is called from the control thread and the job worker, never from the audio
/// thread, and must not block for long.
pub trait EventSink: Send + Sync {
    /// Delivers one event.
    fn emit(&self, event: Event);
}

/// Drops every event: for hosts without a UI, such as a batch export.
#[derive(Clone, Copy, Debug, Default)]
pub struct NullSink;

impl EventSink for NullSink {
    fn emit(&self, _event: Event) {}
}

/// Keeps every event in order, for tests and for the CLI's `--json` log.
#[derive(Clone, Debug, Default)]
pub struct CollectingSink(Arc<Mutex<Vec<Event>>>);

impl CollectingSink {
    /// An empty collector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every event so far, oldest first.
    #[must_use]
    pub fn events(&self) -> Vec<Event> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Removes and returns every event so far.
    #[must_use]
    pub fn take(&self) -> Vec<Event> {
        std::mem::take(&mut *self.0.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

impl EventSink for CollectingSink {
    fn emit(&self, event: Event) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(event);
    }
}

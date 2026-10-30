// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Where events go: the host's [`EventSink`] (ADR-0017 §4).

use std::sync::{Arc, Mutex, PoisonError};

use crate::ipc::Event;
use crate::ipc::meters::MeterFrame;

/// Receives events from the app core's threads. The Tauri host forwards each to the
/// channel [`Event::channel`] names; a test collects them.
///
/// `emit` is called from the control thread and the job worker, never from the audio
/// thread, and must not block for long.
pub trait EventSink: Send + Sync {
    /// Delivers one event.
    fn emit(&self, event: Event);

    /// Delivers one meter frame for the `meters` channel (B01-T04). Meters are a hot path,
    /// so the host sends [`MeterFrame::encode`]'s bytes, never JSON (09 §2). By default the
    /// frame is dropped.
    fn meters(&self, frame: MeterFrame) {
        let _ = frame;
    }
}

/// Drops every event: for hosts without a UI, such as a batch export.
#[derive(Clone, Copy, Debug, Default)]
pub struct NullSink;

impl EventSink for NullSink {
    fn emit(&self, _event: Event) {}
}

/// Keeps every event in order, for tests and for the CLI's `--json` log.
#[derive(Clone, Debug, Default)]
pub struct CollectingSink {
    events: Arc<Mutex<Vec<Event>>>,
    meters: Arc<Mutex<Vec<MeterFrame>>>,
}

impl CollectingSink {
    /// An empty collector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every event so far, oldest first.
    #[must_use]
    pub fn events(&self) -> Vec<Event> {
        self.events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Removes and returns every event so far.
    #[must_use]
    pub fn take(&self) -> Vec<Event> {
        std::mem::take(&mut *self.events.lock().unwrap_or_else(PoisonError::into_inner))
    }

    /// Every meter frame so far, oldest first.
    #[must_use]
    pub fn meter_frames(&self) -> Vec<MeterFrame> {
        self.meters
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl EventSink for CollectingSink {
    fn emit(&self, event: Event) {
        self.events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(event);
    }

    fn meters(&self, frame: MeterFrame) {
        self.meters
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(frame);
    }
}

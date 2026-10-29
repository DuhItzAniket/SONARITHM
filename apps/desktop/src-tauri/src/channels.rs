// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Event channels (ADR-0017 §4): the app core hands every event to the [`Router`], which
//! sends it to the webview's subscribers of the event's channel. The app core calls `emit`
//! from its control and job threads, never from the audio thread.

use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};

use sonarithm_app::EventSink;
use sonarithm_app::ipc::{Channel, Event};

/// Where one subscription's events go: a Tauri channel in the app, a recorder in tests.
pub(crate) trait Subscriber: Send + Sync {
    /// Sends one event; `false` once the webview side has gone (the page reloaded).
    fn send(&self, event: &Event) -> bool;
}

impl Subscriber for tauri::ipc::Channel<Event> {
    fn send(&self, event: &Event) -> bool {
        tauri::ipc::Channel::send(self, event.clone()).is_ok()
    }
}

/// The subscriptions, by ID, in subscription order.
#[derive(Default)]
pub(crate) struct Router {
    inner: Mutex<Subscriptions>,
}

#[derive(Default)]
struct Subscriptions {
    next: u64,
    by_id: BTreeMap<u64, (Channel, Box<dyn Subscriber>)>,
}

impl Router {
    /// Adds a subscriber for `channel`; returns its ID.
    pub(crate) fn subscribe(&self, channel: Channel, subscriber: Box<dyn Subscriber>) -> u64 {
        let mut subs = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        subs.next += 1;
        let id = subs.next;
        subs.by_id.insert(id, (channel, subscriber));
        id
    }

    /// Removes a subscriber; unknown IDs are ignored.
    pub(crate) fn unsubscribe(&self, id: u64) {
        self.inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .by_id
            .remove(&id);
    }
}

impl EventSink for Router {
    fn emit(&self, event: Event) {
        let channel = event.channel();
        let mut subs = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        // Subscribers whose page has gone are dropped on their first failed send.
        subs.by_id
            .retain(|_, (wanted, subscriber)| *wanted != channel || subscriber.send(&event));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sonarithm_app::ipc::Event;

    use super::*;

    /// Records what it receives, or reports a closed page.
    #[derive(Clone, Default)]
    struct Recorder {
        got: Arc<Mutex<Vec<Event>>>,
        closed: bool,
    }

    impl Recorder {
        fn count(&self) -> usize {
            self.got.lock().map_or(0, |v| v.len())
        }
    }

    impl Subscriber for Recorder {
        fn send(&self, event: &Event) -> bool {
            if self.closed {
                return false;
            }
            self.got
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(event.clone());
            true
        }
    }

    fn app_notice() -> Event {
        Event::PlaybackFinished { duration_s: 1.0 }
    }

    #[test]
    fn events_reach_only_subscribers_of_their_channel() {
        let router = Router::default();
        let app = Recorder::default();
        let clock = Recorder::default();
        router.subscribe(Channel::App, Box::new(app.clone()));
        router.subscribe(Channel::Clock, Box::new(clock.clone()));
        router.emit(app_notice());
        assert_eq!(app.count(), 1);
        assert_eq!(clock.count(), 0);
    }

    #[test]
    fn unsubscribed_and_closed_subscribers_stop_receiving() {
        let router = Router::default();
        let kept = Recorder::default();
        let gone = Recorder::default();
        let closed = Recorder {
            closed: true,
            ..Recorder::default()
        };
        let id = router.subscribe(Channel::App, Box::new(gone.clone()));
        router.subscribe(Channel::App, Box::new(closed));
        router.subscribe(Channel::App, Box::new(kept.clone()));
        router.unsubscribe(id);
        router.emit(app_notice());
        router.emit(app_notice());
        assert_eq!(kept.count(), 2);
        assert_eq!(gone.count(), 0);
        let left = router.inner.lock().map_or(0, |s| s.by_id.len());
        assert_eq!(left, 1, "the closed subscriber is dropped");
    }
}

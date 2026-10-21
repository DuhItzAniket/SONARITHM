// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Rate limits and coalescing for stream announcements (A11Y-16, keyboard-and-navigation
//! §9.1; B06-T05).
//!
//! Stream events (threshold crossings, anomalies, connects and disconnects) arrive as fast
//! as the data does; a screen reader must not. Per stream, the [`RateLimiter`] speaks at
//! most one notice every [`INTERVAL`], coalescing the events since the last one into a
//! summary ("3 threshold events in 10 seconds"), and drops events older than [`MAX_AGE`],
//! which are stale by the time they could be spoken. Errors and user-configured alarms are
//! never rate-limited, coalesced or dropped, whatever the stream's state.
//!
//! Each stream can be paused (its notices wait), hidden (its notices go to the log only,
//! not to speech) or stopped (its pending notices are dropped). Every notice, spoken or
//! not, is returned for the announcement log, the text equivalent (A11Y-24).
//!
//! Time is passed in by the caller (seconds on any monotonic clock), so the limiter is
//! deterministic and testable. The 5 s and 10 s values are the keymap proposal of §9.1,
//! not validated yet: TODO(verify) in co-design round 1 (B07-T04).

use std::collections::BTreeMap;

use sonarithm_core::Seconds;

/// The shortest gap between two spoken notices of one stream (§9.1 proposal).
pub const INTERVAL: Seconds = Seconds(5.0);

/// Events older than this when a notice could be spoken are dropped (§9.1 proposal).
pub const MAX_AGE: Seconds = Seconds(10.0);

/// What a stream event is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EventKind {
    /// A crossed threshold.
    Threshold,
    /// A detected anomaly.
    Anomaly,
    /// The stream connected or reconnected.
    Connected,
    /// The stream lost its connection.
    Disconnected,
    /// An error: never dropped.
    Error,
    /// A user-configured alarm (SDR-35): never dropped.
    Alarm,
}

impl EventKind {
    /// Whether events of this kind bypass the limits.
    #[must_use]
    pub const fn is_urgent(self) -> bool {
        matches!(self, Self::Error | Self::Alarm)
    }
}

/// One stream event.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct StreamEvent {
    /// The stream's ID.
    pub stream: String,
    /// What happened.
    pub kind: EventKind,
    /// When, in seconds on the caller's clock.
    pub at: Seconds,
    /// The event in words, such as `CPU crossed 90 %`.
    pub detail: String,
}

impl StreamEvent {
    /// An event.
    #[must_use]
    pub fn new(
        stream: impl Into<String>,
        kind: EventKind,
        at: Seconds,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            stream: stream.into(),
            kind,
            at,
            detail: detail.into(),
        }
    }
}

/// A notice for the announcer and the log: one event, or a summary of several.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Notice {
    /// The stream's ID.
    pub stream: String,
    /// The kind: the events' kind, or the most frequent one in a summary.
    pub kind: EventKind,
    /// How many events it covers.
    pub count: u32,
    /// The first and last event's times.
    pub span: (Seconds, Seconds),
    /// The latest event's words.
    pub detail: String,
    /// Whether it is spoken (`false` for a hidden stream: log only).
    pub spoken: bool,
    /// An error or alarm, delivered at once.
    pub urgent: bool,
}

/// A stream's state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum StreamState {
    /// Notices are spoken.
    #[default]
    Active,
    /// Notices wait until resumed (stale ones are then dropped).
    Paused,
    /// Notices go to the log only.
    Hidden,
}

#[derive(Debug, Default)]
struct Stream {
    state: StreamState,
    last_spoken: Option<Seconds>,
    pending: Vec<StreamEvent>,
}

/// Per-stream limits, coalescing and controls.
#[derive(Debug, Default)]
pub struct RateLimiter {
    streams: BTreeMap<String, Stream>,
    urgent: Vec<Notice>,
    dropped: u64,
}

impl RateLimiter {
    /// A limiter with no streams.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes an event. Errors and alarms are queued for the next [`RateLimiter::poll`]
    /// whatever the stream's state; other events wait for their stream's next notice.
    pub fn push(&mut self, event: StreamEvent) {
        let stream = self.streams.entry(event.stream.clone()).or_default();
        if event.kind.is_urgent() {
            self.urgent.push(Notice {
                stream: event.stream,
                kind: event.kind,
                count: 1,
                span: (event.at, event.at),
                detail: event.detail,
                spoken: true,
                urgent: true,
            });
        } else {
            stream.pending.push(event);
        }
    }

    /// The notices due at `now`: urgent ones first, then at most one per stream whose
    /// interval has passed. Events older than [`MAX_AGE`] are dropped first.
    pub fn poll(&mut self, now: Seconds) -> Vec<Notice> {
        let mut out = std::mem::take(&mut self.urgent);
        for (id, stream) in &mut self.streams {
            let before = stream.pending.len();
            stream.pending.retain(|e| now.0 - e.at.0 <= MAX_AGE.0);
            self.dropped += (before - stream.pending.len()) as u64;
            if stream.pending.is_empty() || stream.state == StreamState::Paused {
                continue;
            }
            let hidden = stream.state == StreamState::Hidden;
            if !hidden
                && stream
                    .last_spoken
                    .is_some_and(|last| now.0 - last.0 < INTERVAL.0)
            {
                continue;
            }
            let events = std::mem::take(&mut stream.pending);
            out.push(summary(id, &events, !hidden));
            if !hidden {
                stream.last_spoken = Some(now);
            }
        }
        out
    }

    /// Holds a stream's notices until [`RateLimiter::resume`].
    pub fn pause(&mut self, stream: &str) {
        self.streams.entry(stream.to_owned()).or_default().state = StreamState::Paused;
    }

    /// Speaks a paused or hidden stream's notices again.
    pub fn resume(&mut self, stream: &str) {
        self.streams.entry(stream.to_owned()).or_default().state = StreamState::Active;
    }

    /// Sends a stream's notices to the log only.
    pub fn hide(&mut self, stream: &str) {
        self.streams.entry(stream.to_owned()).or_default().state = StreamState::Hidden;
    }

    /// Forgets a stream and drops its pending events; urgent notices already queued stay.
    pub fn stop(&mut self, stream: &str) {
        if let Some(removed) = self.streams.remove(stream) {
            self.dropped += removed.pending.len() as u64;
        }
    }

    /// A stream's state.
    #[must_use]
    pub fn state(&self, stream: &str) -> StreamState {
        self.streams
            .get(stream)
            .map(|s| s.state)
            .unwrap_or_default()
    }

    /// Events dropped so far as stale or because their stream stopped.
    #[must_use]
    pub const fn dropped(&self) -> u64 {
        self.dropped
    }
}

/// One notice for `events` (oldest first, at least one).
fn summary(stream: &str, events: &[StreamEvent], spoken: bool) -> Notice {
    let mut counts: BTreeMap<EventKind, u32> = BTreeMap::new();
    for event in events {
        *counts.entry(event.kind).or_default() += 1;
    }
    // The most frequent kind; ties go to the first in `EventKind` order.
    let kind = counts
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map_or(EventKind::Threshold, |(kind, _)| *kind);
    let first = events.first().map_or(Seconds(0.0), |e| e.at);
    let last = events.last().map_or(first, |e| e.at);
    Notice {
        stream: stream.to_owned(),
        kind,
        count: u32::try_from(events.len()).unwrap_or(u32::MAX),
        span: (first, last),
        detail: events.last().map(|e| e.detail.clone()).unwrap_or_default(),
        spoken,
        urgent: false,
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn event(stream: &str, kind: EventKind, at: f64) -> StreamEvent {
        StreamEvent::new(stream, kind, Seconds(at), format!("{kind:?} at {at}"))
    }

    #[test]
    fn a11y_16_events_coalesce_into_one_notice_per_interval() {
        let mut limiter = RateLimiter::new();
        limiter.push(event("cpu", EventKind::Threshold, 0.0));
        assert_eq!(limiter.poll(Seconds(0.0)).len(), 1);
        for at in [1.0, 2.0, 3.0] {
            limiter.push(event("cpu", EventKind::Threshold, at));
        }
        assert_eq!(
            limiter.poll(Seconds(3.0)),
            Vec::new(),
            "within the interval"
        );
        let notices = limiter.poll(Seconds(5.0));
        assert_eq!(notices.len(), 1);
        assert_eq!(notices[0].count, 3);
        assert_eq!(notices[0].span, (Seconds(1.0), Seconds(3.0)));
        assert_eq!(notices[0].detail, "Threshold at 3");
    }

    #[test]
    fn a11y_16_stale_events_are_dropped() {
        let mut limiter = RateLimiter::new();
        limiter.pause("cpu");
        limiter.push(event("cpu", EventKind::Anomaly, 0.0));
        limiter.push(event("cpu", EventKind::Anomaly, 8.0));
        limiter.resume("cpu");
        let notices = limiter.poll(Seconds(12.0));
        assert_eq!(notices.len(), 1);
        assert_eq!(notices[0].count, 1);
        assert_eq!(limiter.dropped(), 1);
    }

    #[test]
    fn a11y_16_hidden_streams_go_to_the_log_and_alarms_always_speak() {
        let mut limiter = RateLimiter::new();
        limiter.hide("net");
        limiter.push(event("net", EventKind::Disconnected, 0.0));
        limiter.push(event("net", EventKind::Alarm, 0.5));
        let notices = limiter.poll(Seconds(1.0));
        assert_eq!(notices.len(), 2);
        assert!(notices[0].urgent && notices[0].spoken);
        assert!(!notices[1].spoken);
        limiter.pause("net");
        limiter.push(event("net", EventKind::Error, 2.0));
        assert!(limiter.poll(Seconds(2.0))[0].urgent);
        limiter.stop("net");
        assert_eq!(limiter.state("net"), StreamState::Active);
    }

    /// One step of a scripted session.
    #[derive(Clone, Debug)]
    enum Step {
        Push(u8, u8, f64),
        Poll(f64),
        Pause(u8),
        Resume(u8),
        Hide(u8),
        Stop(u8),
    }

    fn step() -> impl Strategy<Value = Step> {
        prop_oneof![
            6 => (0_u8..3, 0_u8..6, 0.0_f64..3.0).prop_map(|(s, k, dt)| Step::Push(s, k, dt)),
            4 => (0.0_f64..4.0).prop_map(Step::Poll),
            1 => (0_u8..3).prop_map(Step::Pause),
            1 => (0_u8..3).prop_map(Step::Resume),
            1 => (0_u8..3).prop_map(Step::Hide),
            1 => (0_u8..3).prop_map(Step::Stop),
        ]
    }

    const KINDS: [EventKind; 6] = [
        EventKind::Threshold,
        EventKind::Anomaly,
        EventKind::Connected,
        EventKind::Disconnected,
        EventKind::Error,
        EventKind::Alarm,
    ];

    /// What the property test counts.
    #[derive(Default)]
    struct Tally {
        urgent_in: u64,
        urgent_out: u64,
        other_in: u64,
        other_out: u64,
        last_spoken: BTreeMap<String, f64>,
    }

    impl Tally {
        fn check(&mut self, notices: Vec<Notice>, now: f64) -> Result<(), TestCaseError> {
            for notice in notices {
                if notice.urgent {
                    self.urgent_out += 1;
                    continue;
                }
                self.other_out += u64::from(notice.count);
                prop_assert!(now - notice.span.0.0 <= MAX_AGE.0 + 1e-9);
                if notice.spoken {
                    if let Some(last) = self.last_spoken.get(&notice.stream) {
                        prop_assert!(now - last >= INTERVAL.0 - 1e-9);
                    }
                    self.last_spoken.insert(notice.stream.clone(), now);
                }
            }
            Ok(())
        }
    }

    proptest! {
        /// Whatever happens: per stream, spoken non-urgent notices are at least the interval
        /// apart; no notice holds an event older than the maximum age; every error and
        /// alarm comes out exactly once; and every other event is either in a notice or
        /// counted as dropped.
        #[test]
        fn a11y_16_limits_hold_for_any_session(steps in proptest::collection::vec(step(), 1..120)) {
            let mut limiter = RateLimiter::new();
            let mut now = 0.0_f64;
            let mut tally = Tally::default();
            for step in steps {
                match step {
                    Step::Push(s, k, dt) => {
                        now += dt;
                        let kind = KINDS[usize::from(k)];
                        if kind.is_urgent() { tally.urgent_in += 1 } else { tally.other_in += 1 }
                        limiter.push(event(&format!("s{s}"), kind, now));
                    }
                    Step::Poll(dt) => {
                        now += dt;
                        tally.check(limiter.poll(Seconds(now)), now)?;
                    }
                    Step::Pause(s) => limiter.pause(&format!("s{s}")),
                    Step::Resume(s) => limiter.resume(&format!("s{s}")),
                    Step::Hide(s) => limiter.hide(&format!("s{s}")),
                    Step::Stop(s) => {
                        limiter.stop(&format!("s{s}"));
                        tally.last_spoken.remove(&format!("s{s}"));
                    }
                }
            }
            // Drain: resume everything and wait out the interval.
            for s in 0..3 {
                limiter.resume(&format!("s{s}"));
            }
            now += INTERVAL.0;
            tally.check(limiter.poll(Seconds(now)), now)?;
            prop_assert_eq!(tally.urgent_out, tally.urgent_in);
            prop_assert_eq!(tally.other_out + limiter.dropped(), tally.other_in);
        }
    }
}

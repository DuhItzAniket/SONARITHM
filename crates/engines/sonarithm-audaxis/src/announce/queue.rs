// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The announcement queue and history (keyboard-and-navigation §9.1, 08 §4; A11Y-17,
//! A11Y-24).
//!
//! | Class | Priority | Policy |
//! |---|---|---|
//! | Navigation, earcon text | High | A new one supersedes every pending navigation or response: only the latest position is spoken |
//! | Response | High | Superseded by the next command (navigation or response) |
//! | Status | Normal | Queued after the current announcement; never superseded. Errors are never dropped |
//! | Stream | Low | Dropped when older than [`STREAM_MAX_AGE_MS`]; rate limits and coalescing come with B06 |
//!
//! Every announcement, spoken or not, is written to the history with its time and the node
//! it describes; the history is the text equivalent for Deaf and hard-of-hearing users
//! (A11Y-24). Time comes from the caller, in milliseconds since the session started, so the
//! queue reads no clock (ADR-0013).

use std::collections::VecDeque;

use sonarithm_core::nav::NavNodeId;

use super::compose::{Announcement, Class};
use crate::earcons::EarconId;

/// Stream announcements older than this are dropped (keyboard §9.1 proposal: 10 s).
pub const STREAM_MAX_AGE_MS: u64 = 10_000;

/// History entries kept; the oldest go first.
pub const HISTORY_LIMIT: usize = 10_000;

/// Delivery priority (keyboard §9.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// Stream events.
    Low,
    /// Status.
    Normal,
    /// Navigation, responses and earcon text.
    High,
}

impl Class {
    /// The class's priority.
    #[must_use]
    pub fn priority(self) -> Priority {
        match self {
            Self::Navigation | Self::Response | Self::EarconText => Priority::High,
            Self::Status => Priority::Normal,
            Self::Stream => Priority::Low,
        }
    }

    /// Whether a newer command makes it stale.
    fn supersedable(self) -> bool {
        matches!(self, Self::Navigation | Self::Response | Self::EarconText)
    }
}

/// One history entry.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct HistoryEntry {
    /// The order it arrived in, from 0.
    pub seq: u64,
    /// When it arrived, in milliseconds since the session started.
    pub at_ms: u64,
    /// Its class.
    pub class: Class,
    /// The display text (unit symbols).
    pub text: String,
    /// The node it describes, for "go to" from the history.
    pub node: Option<NavNodeId>,
    /// The earcon that went with it.
    pub earcon: Option<EarconId>,
    /// Whether it was meant to be spoken.
    pub spoken: bool,
    /// Whether a newer announcement replaced it before it was delivered.
    pub superseded: bool,
    /// Whether it reported an error.
    pub error: bool,
}

/// An announcement ready for the announcer route (B03-T04).
#[derive(Clone, Debug, PartialEq)]
pub struct Delivery {
    /// Its order.
    pub seq: u64,
    /// The announcement.
    pub announcement: Announcement,
    /// Whether it should cut off speech in progress (navigation, responses, earcon text).
    pub interrupt: bool,
}

#[derive(Clone, Debug)]
struct Pending {
    seq: u64,
    at_ms: u64,
    announcement: Announcement,
}

/// The queue of announcements waiting to be spoken, and the history of all of them.
#[derive(Clone, Debug, Default)]
pub struct AnnouncementQueue {
    pending: Vec<Pending>,
    history: VecDeque<HistoryEntry>,
    next_seq: u64,
}

impl AnnouncementQueue {
    /// An empty queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an announcement that arrived at `at_ms` and returns its order. It always goes
    /// to the history; it waits to be spoken only if it is spoken at all.
    pub fn push(&mut self, announcement: Announcement, at_ms: u64) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        if announcement.speak && announcement.class.supersedable() {
            let superseded: Vec<u64> = self
                .pending
                .iter()
                .filter(|p| p.announcement.class.supersedable() && !p.announcement.error)
                .map(|p| p.seq)
                .collect();
            self.pending.retain(|p| !superseded.contains(&p.seq));
            for entry in self
                .history
                .iter_mut()
                .filter(|e| superseded.contains(&e.seq))
            {
                entry.superseded = true;
            }
        }
        self.history.push_back(HistoryEntry {
            seq,
            at_ms,
            class: announcement.class,
            text: announcement.display.clone(),
            node: announcement.node,
            earcon: announcement.earcon,
            spoken: announcement.speak,
            superseded: false,
            error: announcement.error,
        });
        while self.history.len() > HISTORY_LIMIT {
            self.history.pop_front();
        }
        if announcement.speak {
            self.pending.push(Pending {
                seq,
                at_ms,
                announcement,
            });
        }
        seq
    }

    /// The next announcement to deliver: the highest priority first, then the oldest.
    pub fn pop(&mut self) -> Option<Delivery> {
        let index = self
            .pending
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.announcement
                    .class
                    .priority()
                    .cmp(&b.announcement.class.priority())
                    .then(b.seq.cmp(&a.seq))
            })
            .map(|(i, _)| i)?;
        let pending = self.pending.remove(index);
        Some(Delivery {
            seq: pending.seq,
            interrupt: pending.announcement.class.supersedable(),
            announcement: pending.announcement,
        })
    }

    /// Drops stream announcements older than [`STREAM_MAX_AGE_MS`] at `now_ms`. Errors stay.
    pub fn expire(&mut self, now_ms: u64) {
        let stale: Vec<u64> = self
            .pending
            .iter()
            .filter(|p| {
                p.announcement.class == Class::Stream
                    && !p.announcement.error
                    && now_ms.saturating_sub(p.at_ms) > STREAM_MAX_AGE_MS
            })
            .map(|p| p.seq)
            .collect();
        self.pending.retain(|p| !stale.contains(&p.seq));
        for entry in self.history.iter_mut().filter(|e| stale.contains(&e.seq)) {
            entry.superseded = true;
        }
    }

    /// The announcements waiting, oldest first.
    pub fn pending(&self) -> impl Iterator<Item = &Announcement> {
        self.pending.iter().map(|p| &p.announcement)
    }

    /// Every announcement so far, oldest first (up to [`HISTORY_LIMIT`]).
    pub fn history(&self) -> impl Iterator<Item = &HistoryEntry> {
        self.history.iter()
    }
}

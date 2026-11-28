// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Announcements (07 AUDAXIS, `announce`): the composer, which turns nodes and verb
//! outcomes into catalogue text, and the queue with its supersede policy and history.

pub mod compose;
pub mod queue;

pub use compose::{
    Announcement, Class, Composer, KeyNames, LegendCues, Memory, Part, Politeness, Query, Settings,
    Verbosity,
};
pub use queue::{AnnouncementQueue, Delivery, HistoryEntry, Priority};

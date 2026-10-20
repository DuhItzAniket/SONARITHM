// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The study runner as a library: protocol files, trial sequencing, the session state
//! machine and the JSONL log (evaluation-plan "`sonarithm study run`"; FR-084).
//!
//! The library decides what happens and what is logged; it does no I/O, reads no clock and
//! makes no sound. A front end (the `sonarithm study run` subcommand) reads the protocol,
//! binds each trial's stimulus into its condition's spec, renders and plays it, passes the
//! monotonic time of every action, and appends every returned record to the log at once.
//!
//! **Research use (OD-04).** The project has no ethics review route, so this runner must
//! not collect data from participants as research until one exists. Its pilot runs with
//! scripted responses.

pub mod log;
pub mod protocol;
pub mod schedule;
pub mod session;

pub use log::{Event, LOG_SCHEMA, LogRecord, parse_log};
pub use protocol::{
    Block, Condition, FORMAT, Order, Protocol, ResponseSpec, Task, TaskKind, is_code,
};
pub use schedule::{Schedule, Trial};
pub use session::{AudioContext, Prepared, Scored, Session, SessionError, SessionInfo, State};

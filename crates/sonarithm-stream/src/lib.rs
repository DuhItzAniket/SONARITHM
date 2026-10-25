// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Live stream adapters for SONARITHM (07 SONOTRACE, 02 §2; B06-T02).
//!
//! A stream connects to one [`Endpoint`] (a WebSocket URL, an OSC port, a serial port, an
//! MQTT topic or a file to follow), decodes each message into a row with a
//! [`MessageSchema`], and appends the rows to a bounded [`StreamTable`] (03 §8) that
//! SONOTRACE reads. Memory stays bounded by the table's capacity whatever the source sends.
//!
//! **Consent (ADR-0012, A11Y-16, AGENTS.md rule 9).** Nothing in this crate opens a
//! connection on its own. [`connect`] needs a [`Consent`], and a `Consent` exists only
//! through [`Consent::user_approved`], which a host calls from the handler of the user's
//! own confirmation of that endpoint. A spec, a project file or a default can describe a
//! stream ([`Endpoint::from_parts`]) but never open it.
//!
//! **Threads.** Each connection reads on its own thread with synchronous clients; the
//! WebSocket, OSC, serial and file adapters need no async runtime. The thread reconnects
//! with exponential backoff, and [`Connection::stop`] (or dropping the connection) ends it
//! within one read timeout.
//!
//! | Feature | Default | Transport |
//! |---|---|---|
//! | `websocket` | on | `ws://` URLs (no TLS yet) |
//! | `osc` | on | OSC 1.0 over UDP, on a local port |
//! | `serial` | off | Serial ports, one message per line |
//! | `mqtt` | off | MQTT 3.1.1 topics, without TLS |
//! | (always) | — | `tail`: lines appended to a file |

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod connection;
pub mod decode;
mod endpoint;
mod rows;
pub mod schema;
mod transports;

pub use connection::{
    Connection, ConnectionState, ConnectionStatus, Options, TableReader, connect,
};
pub use endpoint::{Consent, DEFAULT_BAUD, Endpoint};
pub use rows::RowBuffer;
pub use schema::{FieldKind, MessageSchema};
pub use sonarithm_data::{Capacity, StreamTable};

/// Why a stream could not be set up or decoded.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum StreamError {
    /// The spec's stream source is incomplete or names an unsupported transport.
    #[error("the stream source is not usable: {0}")]
    Source(String),
    /// This build has no adapter for the transport (its feature is off).
    #[error("this build cannot connect {0} streams")]
    Unsupported(&'static str),
    /// The message schema is not usable.
    #[error("the message schema is not usable: {0}")]
    Schema(String),
    /// A message could not be decoded into a row.
    #[error("a message could not be read: {0}")]
    Decode(String),
    /// The stream table refused the rows.
    #[error("the stream table refused the rows: {0}")]
    Table(String),
}

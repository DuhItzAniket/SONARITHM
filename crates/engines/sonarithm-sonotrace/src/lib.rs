// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SONOTRACE, SONARITHM's streaming and transport engine (07 SONOTRACE; B06).
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`live`] | Live encoding: encodes a [`StreamTable`]'s window with the track's encoder and emits the events of new rows only, by sequence number (05 §6) |
//! | [`rate_limit`] | Stream announcements: one per stream every 5 s, coalesced, stale after 10 s; errors and alarms never dropped; pause, hide and stop (A11Y-16) |
//! | [`transport`] | Scrub grains that preview the value at the cursor (A11Y-09), and long-series windowing |
//! | engine | The `stream` source: a spec's `{"kind": "stream", …}` reads the window of the stream the host connected |
//!
//! The network adapters that fill a stream live in `sonarithm-stream` (B06-T02), behind a
//! consent gate; this crate never opens a connection and builds for wasm32. Like every
//! engine it depends only on core, spec and data (02 §3).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod codes;
pub mod live;
pub mod rate_limit;
pub mod transport;

use std::sync::Arc;

use sonarithm_core::engine::{
    CompileContext, CompileError, Engine, EngineManifest, Registrar, RegistryError, Source,
    SourceKind, SpecFragment,
};
use sonarithm_core::{Diagnostic, EngineId, EngineVersion, TableId};
use sonarithm_data::bundle::SourceKind as DataSourceKind;
use sonarithm_data::{BundleId, DataBundle, SourceDescriptor, StreamTable, Table};

/// The engine ID.
pub const ENGINE: EngineId = EngineId::from_static("sonotrace");

/// The table ID a stream window gets in the bundle a `stream` source returns.
pub const WINDOW_TABLE: TableId = TableId(0);

/// The SONOTRACE engine. It registers the `stream` source; with [`Sonotrace::with_window`]
/// the source returns that window.
#[derive(Clone, Debug, Default)]
pub struct Sonotrace {
    window: Option<Arc<Table>>,
}

impl Sonotrace {
    /// The engine with no stream connected: a `stream` source fails with `SNR-ENG-200`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The engine whose `stream` source returns the rows `stream` holds now.
    ///
    /// # Errors
    ///
    /// The data layer's error if the window cannot form a table.
    pub fn with_window(stream: &StreamTable) -> Result<Self, sonarithm_data::DataError> {
        let table = stream.snapshot().to_table(WINDOW_TABLE)?;
        Ok(Self {
            window: Some(Arc::new(table)),
        })
    }

    /// The engine's manifest.
    #[must_use]
    pub fn manifest() -> EngineManifest {
        let mut manifest = EngineManifest::new(
            ENGINE,
            "SONOTRACE",
            EngineVersion(env!("CARGO_PKG_VERSION").to_owned()),
        );
        manifest.spec_keys = vec!["data.source.kind=stream".to_owned(), "playback".to_owned()];
        manifest
    }
}

impl Engine<DataBundle> for Sonotrace {
    fn manifest(&self) -> EngineManifest {
        Self::manifest()
    }

    fn register(&self, registrar: &mut Registrar<'_, DataBundle>) -> Result<(), RegistryError> {
        registrar.source(
            SourceKind::Stream,
            Arc::new(StreamSource {
                window: self.window.clone(),
            }),
        )
    }
}

/// The `stream` source.
struct StreamSource {
    window: Option<Arc<Table>>,
}

impl Source<DataBundle> for StreamSource {
    fn ingest(
        &self,
        source: &SpecFragment,
        _cx: &mut CompileContext,
    ) -> Result<DataBundle, CompileError> {
        let parsed: sonarithm_spec::Source = source.parse()?;
        let sonarithm_spec::Source::Stream { transport, .. } = parsed else {
            return Err(CompileError::Engine {
                engine: ENGINE,
                message: "the stream source was given another kind of source".to_owned(),
            });
        };
        let Some(window) = &self.window else {
            return Err(CompileError::from(
                Diagnostic::new(
                    &codes::STREAM_NOT_CONNECTED,
                    format!(
                        "The spec reads a {} stream, but no stream is connected.",
                        transport_name(transport)
                    ),
                )
                .at(source.pointer().clone()),
            ));
        };
        Ok(DataBundle::single(
            BundleId(0),
            SourceDescriptor::new(DataSourceKind::Memory, "stream", "stream"),
            Table::clone(window),
        ))
    }
}

fn transport_name(transport: sonarithm_spec::data::Transport) -> &'static str {
    match transport {
        sonarithm_spec::data::Transport::Websocket => "WebSocket",
        sonarithm_spec::data::Transport::Mqtt => "MQTT",
        sonarithm_spec::data::Transport::Serial => "serial",
        sonarithm_spec::data::Transport::Osc => "OSC",
        sonarithm_spec::data::Transport::Tail => "file-tail",
        _ => "live",
    }
}

#[cfg(test)]
mod tests;

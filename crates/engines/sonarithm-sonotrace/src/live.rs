// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Live encoding (05 §6 "Streaming data"; B06-T03).
//!
//! A [`Live`] follows one track over a [`StreamTable`]. Each [`Live::advance`] encodes the
//! stream's current window with the track's encoder, so shapers and scales that need global
//! information see the sliding window, then keeps only the events of rows the track has
//! not played yet, identified by sequence number through each event's source row. A row's
//! event is therefore emitted exactly once, and the same recorded input in the same
//! batches gives the same event sequence (SDR-01, ADR-0013).
//!
//! Encoding the whole window per update costs O(window) per update instead of O(new rows).
//! The A06 encoders reach about 2 million rows per second (PB-04), so a 10 s window of a
//! 1 kHz source updated 20 times a second is about a tenth of that. A native
//! `encode_incremental` on the core `Encoder` trait would change a cross-engine contract
//! and needs an RFC; the PB-11 soak (B06-T07) decides whether it is needed.

use sonarithm_core::engine::{CompileContext, CompileError, Encoder, TrackRequest};
use sonarithm_core::score::ScoreEvent;
use sonarithm_core::{Diagnostic, Diagnostics, TableId};
use sonarithm_data::bundle::SourceKind as DataSourceKind;
use sonarithm_data::{BundleId, DataBundle, SourceDescriptor, StreamTable};

use crate::codes;

/// The table ID the window gets while it is encoded.
const WINDOW: TableId = TableId(0);

/// One event of a new row.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct LiveEvent {
    /// The sequence number of the row that produced it.
    pub seq: u64,
    /// The event, timed within the window it was encoded in.
    pub event: ScoreEvent,
}

/// What one [`Live::advance`] produced.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct LiveUpdate {
    /// The first sequence number this update covers.
    pub from: u64,
    /// One past the last sequence number it covers; the next update starts here.
    pub to: u64,
    /// Rows the track never played because the stream dropped them first (backpressure).
    pub skipped: u64,
    /// The new rows' events, by sequence number, then start time.
    pub events: Vec<LiveEvent>,
    /// The encoder's diagnostics, `SNR-DATA-071` for skipped rows and `SNR-ENG-201` for
    /// events without a row.
    pub diagnostics: Diagnostics,
}

/// One track followed live over a stream.
#[derive(Clone, Debug)]
pub struct Live {
    track: TrackRequest,
    cursor: u64,
}

impl Live {
    /// Follows `track` from the stream's first row.
    #[must_use]
    pub fn new(track: TrackRequest) -> Self {
        Self { track, cursor: 0 }
    }

    /// Follows `track` from sequence number `seq`, skipping older rows.
    #[must_use]
    pub fn from_seq(track: TrackRequest, seq: u64) -> Self {
        Self { track, cursor: seq }
    }

    /// The next sequence number the track will play.
    #[must_use]
    pub const fn cursor(&self) -> u64 {
        self.cursor
    }

    /// Encodes the stream's window and returns the events of rows from the cursor on, then
    /// moves the cursor past them.
    ///
    /// # Errors
    ///
    /// The encoder's [`CompileError`]; the cursor does not move.
    pub fn advance(
        &mut self,
        stream: &StreamTable,
        encoder: &dyn Encoder<DataBundle>,
        cx: &mut CompileContext,
    ) -> Result<LiveUpdate, CompileError> {
        let window = stream.snapshot();
        let new = stream.since(self.cursor);
        let mut diagnostics = Diagnostics::new();
        if let Some(diagnostic) = new.diagnostic() {
            diagnostics.push(diagnostic);
        }
        let mut update = LiveUpdate {
            from: new.first_seq(),
            to: new.next_seq(),
            skipped: new.skipped(),
            events: Vec::new(),
            diagnostics,
        };
        if new.num_rows() == 0 {
            self.cursor = update.to;
            return Ok(update);
        }
        let table = window
            .to_table(WINDOW)
            .map_err(|err| CompileError::from(err.to_diagnostic()))?;
        let bundle = DataBundle::single(
            BundleId(0),
            SourceDescriptor::new(DataSourceKind::Memory, "stream", "stream"),
            table,
        );
        let track = encoder.encode(&self.track, &bundle, &[], cx)?;
        for diagnostic in cx.take_diagnostics() {
            update.diagnostics.push(diagnostic);
        }
        let first = window.first_seq();
        let mut without_row = 0_u64;
        for event in track.events {
            let Some(row) = source_row(&event) else {
                without_row += 1;
                continue;
            };
            let seq = first + row;
            if seq >= update.from && seq < update.to {
                update.events.push(LiveEvent { seq, event });
            }
        }
        if without_row > 0 {
            update.diagnostics.push(Diagnostic::new(
                &codes::LIVE_EVENT_WITHOUT_ROW,
                format!(
                    "Live mode skipped {without_row} event(s) of track `{}` that have no data row.",
                    self.track.id
                ),
            ));
        }
        update.events.sort_by(|a, b| {
            a.seq
                .cmp(&b.seq)
                .then(a.event.start().0.total_cmp(&b.event.start().0))
        });
        self.cursor = update.to;
        Ok(update)
    }
}

/// The window row an event comes from: a note's row, or the last row of a glide's contour
/// (the row that completed it).
fn source_row(event: &ScoreEvent) -> Option<u64> {
    match event {
        ScoreEvent::Note(note) => note.src.map(|src| src.row),
        ScoreEvent::Glide(glide) => glide.src.map(|range| range.end.saturating_sub(1)),
        _ => None,
    }
}

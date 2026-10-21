// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::sync::Arc;

use arrow_array::{Array, Float64Array, RecordBatch};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use proptest::prelude::*;
use serde_json::json;
use sonarithm_core::engine::{
    Annotation, Budget, CancelToken, CompileContext, CompileError, Encoder, MarkKind, Registry,
    SourceKind, SpecFragment, TrackRequest,
};
use sonarithm_core::score::{
    Breakpoint, Glide, InstrumentRef, Note, ScoreEvent, ScoreTrack, TrackRole,
};
use sonarithm_core::{Cents, Db, RowRef, Seconds, SpecPointer, TrackId};
use sonarithm_data::{Capacity, DataBundle, StreamTable};

use super::live::{Live, LiveEvent};
use super::transport::{GRAIN, scrub_grain, window, windows};
use super::*;

/// A minimal encoder for tests (engines cannot use NUMISONA, 02 §3 rule 2): one note per
/// row, pitch from `v`, onsets 0.1 s apart from the window's start.
struct RowNotes;

impl Encoder<DataBundle> for RowNotes {
    fn encode(
        &self,
        track: &TrackRequest,
        data: &DataBundle,
        _annotations: &[Annotation],
        _cx: &mut CompileContext,
    ) -> Result<ScoreTrack, CompileError> {
        let mut out = ScoreTrack::new(
            track.id.clone(),
            TrackRole::Data,
            ENGINE,
            InstrumentRef("mallet".into()),
        );
        let Some(table) = data.primary() else {
            return Ok(out);
        };
        for batch in table.batches() {
            let v = batch.column(1).as_any().downcast_ref::<Float64Array>();
            let Some(v) = v else { continue };
            for i in 0..v.len() {
                let row = out.events.len() as u64;
                #[expect(clippy::cast_precision_loss, reason = "small test counts")]
                let mut note =
                    Note::new(Seconds(row as f64 * 0.1), Seconds(0.1), Cents(v.value(i)));
                note.src = Some(RowRef {
                    table: table.id(),
                    generation: table.generation(),
                    row,
                });
                out.events.push(ScoreEvent::Note(note));
            }
        }
        Ok(out)
    }
}

fn schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("t", DataType::Float64, false),
        Field::new("v", DataType::Float64, false),
    ]))
}

/// A recorded stream: batches whose `v` is 10 × the row's sequence number.
fn recording(sizes: &[usize]) -> Vec<RecordBatch> {
    let mut seq = 0_u64;
    sizes
        .iter()
        .map(|n| {
            #[expect(clippy::cast_precision_loss, reason = "small test counts")]
            let rows: Vec<f64> = (seq..seq + *n as u64).map(|s| s as f64).collect();
            seq += *n as u64;
            let v: Vec<f64> = rows.iter().map(|r| r * 10.0).collect();
            RecordBatch::try_new(
                schema(),
                vec![
                    Arc::new(Float64Array::from(rows)),
                    Arc::new(Float64Array::from(v)),
                ],
            )
            .unwrap()
        })
        .collect()
}

fn context() -> CompileContext {
    CompileContext::new(7, Budget::default(), CancelToken::new())
}

fn track() -> TrackRequest {
    TrackRequest::new(
        TrackId::new("live"),
        MarkKind::Tone,
        SpecFragment::new(
            SpecPointer::root().key("tracks").index(0),
            &json!({"id": "live"}),
        )
        .unwrap(),
    )
}

/// Plays `batches` through a stream of `limit` rows, advancing after every `every` appends.
fn replay(batches: &[RecordBatch], limit: u64, every: usize) -> (Vec<LiveEvent>, u64) {
    let mut stream = StreamTable::new(TableId(1), "live", schema(), Capacity::rows(limit)).unwrap();
    let mut live = Live::new(track());
    let mut cx = context();
    let mut events = Vec::new();
    let mut skipped = 0;
    for (i, batch) in batches.iter().enumerate() {
        stream.append(batch.clone()).unwrap();
        if (i + 1) % every == 0 {
            let update = live.advance(&stream, &RowNotes, &mut cx).unwrap();
            skipped += update.skipped;
            events.extend(update.events);
        }
    }
    let update = live.advance(&stream, &RowNotes, &mut cx).unwrap();
    skipped += update.skipped;
    events.extend(update.events);
    (events, skipped)
}

#[test]
fn sdr_01_recorded_input_replays_to_an_identical_event_sequence() {
    let batches = recording(&[3, 0, 7, 1, 12, 4, 9]);
    let first = replay(&batches, 16, 2);
    let second = replay(&batches, 16, 2);
    assert_eq!(first, second);
    assert_ne!(first.0.len(), 0);
}

proptest! {
    /// Every row's event is emitted once, in sequence order, unless the stream dropped
    /// the row first, which the update reports.
    #[test]
    fn live_emits_each_row_once(
        sizes in proptest::collection::vec(0_usize..20, 1..20),
        limit in 1_u64..30,
        every in 1_usize..4,
    ) {
        let batches = recording(&sizes);
        let total: u64 = sizes.iter().map(|n| *n as u64).sum();
        let (events, skipped) = replay(&batches, limit, every);
        prop_assert_eq!(events.len() as u64 + skipped, total);
        prop_assert!(events.windows(2).all(|w| w[0].seq < w[1].seq));
        for event in &events {
            let ScoreEvent::Note(note) = &event.event else { unreachable!() };
            // The note carries the value of its own row.
            #[expect(clippy::cast_precision_loss, reason = "small test counts")]
            let expected = event.seq as f64 * 10.0;
            prop_assert!((note.pitch.0 - expected).abs() < 1e-9);
        }
    }
}

#[test]
fn an_empty_or_unchanged_stream_gives_no_events() {
    let mut stream = StreamTable::new(TableId(1), "s", schema(), Capacity::rows(8)).unwrap();
    let mut live = Live::new(track());
    let mut cx = context();
    let update = live.advance(&stream, &RowNotes, &mut cx).unwrap();
    assert_eq!(update.events, Vec::new());
    stream.append(recording(&[3]).remove(0)).unwrap();
    assert_eq!(
        live.advance(&stream, &RowNotes, &mut cx)
            .unwrap()
            .events
            .len(),
        3
    );
    assert_eq!(
        live.advance(&stream, &RowNotes, &mut cx).unwrap().events,
        Vec::new()
    );
    assert_eq!(live.cursor(), 3);
}

fn fixture() -> ScoreTrack {
    let mut track = ScoreTrack::new(
        TrackId::new("data"),
        TrackRole::Data,
        ENGINE,
        InstrumentRef("mallet".into()),
    );
    for (i, cents) in [-1200.0, -600.0, 0.0, 700.0].into_iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "four notes")]
        let mut note = Note::new(Seconds(i as f64 * 0.5), Seconds(0.25), Cents(cents));
        note.gain = Db(-3.0);
        track.events.push(ScoreEvent::Note(note));
    }
    let mut a = Breakpoint::new(Seconds(2.5), Cents(0.0));
    a.gain = Db(-6.0);
    let b = Breakpoint::new(Seconds(3.5), Cents(1200.0));
    track.events.push(ScoreEvent::Glide(Glide::new(
        Seconds(2.5),
        Seconds(3.5),
        vec![a, b],
    )));
    track
}

#[test]
fn golden_scrub_grains_land_at_the_cursor() {
    let track = fixture();
    let grains: Vec<Note> = [0.0, 0.1, 0.3, 0.6, 1.9, 2.5, 3.0, 3.5, 9.0]
        .into_iter()
        .filter_map(|t| scrub_grain(&track, Seconds(t)))
        .collect();
    for (grain, t) in grains
        .iter()
        .zip([0.0, 0.1, 0.3, 0.6, 1.9, 2.5, 3.0, 3.5, 9.0])
    {
        assert_eq!(grain.t, Seconds(t), "the grain starts at the cursor");
        assert_eq!(grain.dur, GRAIN);
    }
    insta::assert_json_snapshot!("scrub_grains", grains);
}

#[test]
fn a11y_09_scrub_grain_previews_the_local_value() {
    let track = fixture();
    // Inside the second note.
    let grain = scrub_grain(&track, Seconds(0.6)).unwrap();
    assert!((grain.pitch.0 + 600.0).abs() < 1e-9);
    assert!((grain.gain.0 + 3.0).abs() < 1e-9);
    // Between notes: the last note before the cursor.
    assert!((scrub_grain(&track, Seconds(0.8)).unwrap().pitch.0 + 600.0).abs() < 1e-9);
    // Halfway along the glide: halfway in cents and dB.
    let grain = scrub_grain(&track, Seconds(3.0)).unwrap();
    assert!((grain.pitch.0 - 600.0).abs() < 1e-9);
    assert!((grain.gain.0 + 3.0).abs() < 1e-9);
    // A track without notes has nothing to preview.
    let empty = ScoreTrack::new(
        TrackId::new("e"),
        TrackRole::Data,
        ENGINE,
        InstrumentRef("mallet".into()),
    );
    assert!(scrub_grain(&empty, Seconds(1.0)).is_none());
}

#[test]
fn a_window_moves_its_part_of_the_track_to_zero() {
    let track = fixture();
    let part = window(&track, Seconds(1.0), Seconds(3.0));
    let starts: Vec<f64> = part.events.iter().map(|e| e.start().0).collect();
    assert_eq!(starts, [0.0, 0.5, 1.5]);
    let ScoreEvent::Glide(glide) = &part.events[2] else {
        unreachable!("the glide is cut, not dropped");
    };
    assert_eq!((glide.t0, glide.t1), (Seconds(1.5), Seconds(2.0)));
    let last = glide.curve.last().unwrap();
    assert!(
        (last.pitch.0 - 600.0).abs() < 1e-9,
        "cut at the window edge"
    );
    assert_eq!(windows(Seconds(2.5), Seconds(1.0)).len(), 3);
    assert_eq!(
        windows(Seconds(2.5), Seconds(1.0))[2],
        (Seconds(2.0), Seconds(2.5))
    );
    assert_eq!(windows(Seconds(1.0), Seconds(0.0)), Vec::new());
}

fn stream_source() -> SpecFragment {
    SpecFragment::new(
        SpecPointer::root().key("data").key("source"),
        &json!({"kind": "stream", "transport": "websocket", "url": "ws://localhost:9000"}),
    )
    .unwrap()
}

#[test]
fn the_stream_source_reads_the_connected_window() {
    let mut stream = StreamTable::new(TableId(1), "live", schema(), Capacity::rows(4)).unwrap();
    for batch in recording(&[3, 3]) {
        stream.append(batch).unwrap();
    }
    let engine = Sonotrace::with_window(&stream).unwrap();
    let mut registry = Registry::<DataBundle>::new();
    registry.add_engine(&engine).unwrap();
    let (owner, source) = registry.source(SourceKind::Stream).unwrap();
    assert_eq!(owner, &ENGINE);
    let bundle = source.ingest(&stream_source(), &mut context()).unwrap();
    assert_eq!(
        bundle.primary().map(sonarithm_data::Table::num_rows),
        Some(4)
    );
}

#[test]
fn a_stream_source_without_a_stream_says_so() {
    let mut registry = Registry::<DataBundle>::new();
    registry.add_engine(&Sonotrace::new()).unwrap();
    let (_, source) = registry.source(SourceKind::Stream).unwrap();
    let Err(CompileError::Diagnostic(diagnostic)) = source.ingest(&stream_source(), &mut context())
    else {
        unreachable!("expected a diagnostic");
    };
    assert_eq!(diagnostic.code.as_str(), "SNR-ENG-200");
    assert!(diagnostic.message.contains("WebSocket"));
}

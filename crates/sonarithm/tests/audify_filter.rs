// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! An `audify` track honours its `filter` (#506 review): it plays only the rows the filter
//! keeps, as every encoder does through `play_rows`, and the detectors analyse that series,
//! so its insights are about the rows that sound (RFC-0009 §1).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use serde_json::json;
use sonarithm::score::ScoreEvent;
use sonarithm::{Sonarithm, Spec};

/// 60 rows over `t`: a small hump peaking at t = 10 (5.0), then a larger one peaking at
/// t = 45 (20.0). The filter `t < 30` keeps the first 30 rows, so the peak at t = 45 does
/// not sound.
fn spec(filter: Option<&str>) -> Spec {
    let rows: Vec<serde_json::Value> = (0..60_u32)
        .map(|t| {
            let x = f64::from(t);
            let y = if t < 30 {
                5.0 - (x - 10.0).abs() * 0.25
            } else {
                20.0 - (x - 45.0).abs() * 0.5
            };
            json!({ "t": t, "y": y })
        })
        .collect();
    let mut track = json!({ "id": "a", "mark": "audify" });
    if let Some(expr) = filter {
        track["filter"] = json!({ "expr": expr });
    }
    Spec::from_json_str(
        json!({
            "specVersion": "0.1",
            "data": { "source": { "kind": "inline", "values": rows } },
            "substrate": { "time": { "field": "t" } },
            "tracks": [track],
            "analysis": { "detect": ["extrema"] }
        })
        .to_string(),
    )
    .unwrap()
}

fn audify(compiled: &sonarithm::Compiled) -> &sonarithm::score::Audify {
    compiled
        .score()
        .tracks
        .iter()
        .flat_map(|track| &track.events)
        .find_map(|event| match event {
            ScoreEvent::Audify(audify) => Some(audify),
            _ => None,
        })
        .unwrap()
}

#[test]
fn a_filtered_audify_track_plays_and_analyses_only_its_rows() {
    let compiled = Sonarithm::new().compile(&spec(Some("t < 30"))).unwrap();
    let played = audify(&compiled);
    assert_eq!(played.samples.len(), 30, "one sample per kept row");
    let src = played.src.unwrap();
    assert_eq!((src.start, src.end), (0, 30));
    // The detectors analysed the audify track's own series: its maximum is the peak that
    // sounds (t = 10), not the filtered-out one at t = 45.
    let maximum = compiled
        .annotations()
        .iter()
        .find(|a| a.label == "maximum")
        .unwrap();
    assert_eq!(maximum.range.start, 10);
    assert_eq!(
        maximum
            .track
            .as_ref()
            .map(sonarithm_core::ids::TrackId::as_str),
        Some("a")
    );
    assert_eq!(maximum.field.as_deref(), Some("y"));
}

#[test]
fn an_unfiltered_audify_track_plays_every_row() {
    let compiled = Sonarithm::new().compile(&spec(None)).unwrap();
    assert_eq!(audify(&compiled).samples.len(), 60);
    let maximum = compiled
        .annotations()
        .iter()
        .find(|a| a.label == "maximum")
        .unwrap();
    assert_eq!(maximum.range.start, 45);
}

/// A filter that cannot be applied is refused with `SNR-ENG-466`, never ignored (it used to
/// be ignored). (A filter on a field the data does not have is not an error: the data layer
/// warns and keeps no rows.)
#[test]
fn an_audify_filter_that_cannot_be_applied_is_refused() {
    let Err(err) = Sonarithm::new().compile(&spec(Some("t <"))) else {
        panic!("a filter on a missing field must be refused");
    };
    let diagnostics = err.diagnostics();
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(codes.contains(&"SNR-ENG-466"), "{codes:?}");
}

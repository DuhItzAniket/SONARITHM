// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The human-readable review form of a Score, for insta snapshots (05 §4.3,
//! testing-strategy §4).
//!
//! Rounding keeps diffs legible: time to 1 µs, pitch to 0.01 cents, gain to 0.01 dB, and
//! every other float to 1e-6. Rounding is half to even. Object keys are sorted. A track
//! with more than [`ReviewOptions::max_events`] events is summarized: its event count,
//! time extent, the SHA-256 of its full-precision events, and the first and last
//! [`ReviewOptions::edge_events`] events. The review form never replaces the canonical
//! hash; see [`super::canonical`].

use serde_json::{Map, Number, Value, json};

use super::Score;
use crate::digest::Sha256Digest;

/// Controls when tracks are summarized.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviewOptions {
    /// Tracks with more events than this are summarized.
    pub max_events: usize,
    /// Events kept at each end of a summarized track.
    pub edge_events: usize,
}

impl Default for ReviewOptions {
    /// Summarize above 64 events, keeping 8 at each end.
    fn default() -> Self {
        Self {
            max_events: 64,
            edge_events: 8,
        }
    }
}

/// Keys whose values are pitches in cents or gains in dB, rounded to 0.01. Every other
/// float, times included, is rounded to 1e-6.
const HUNDREDTHS_KEYS: &[&str] = &["pitch", "low", "high", "gain"];

impl Score {
    /// The review form as pretty-printed JSON with a trailing newline.
    #[must_use]
    pub fn review_json(&self, options: &ReviewOptions) -> String {
        let mut value = match serde_json::to_value(self) {
            Ok(value) => value,
            Err(err) => json!({ "error": err.to_string() }),
        };
        summarize(self, &mut value, options);
        round(&mut value, None);
        let mut text = serde_json::to_string_pretty(&value)
            .unwrap_or_else(|err| format!("{{\"error\": \"{err}\"}}"));
        text.push('\n');
        text
    }
}

/// Replaces the event arrays of long tracks with summaries.
fn summarize(score: &Score, value: &mut Value, options: &ReviewOptions) {
    let Some(tracks) = value.get_mut("tracks").and_then(Value::as_array_mut) else {
        return;
    };
    for (track, track_value) in score.tracks.iter().zip(tracks.iter_mut()) {
        let count = track.events.len();
        if count <= options.max_events {
            continue;
        }
        let Some(events) = track_value.get_mut("events") else {
            continue;
        };
        let Some(all) = events.as_array() else {
            continue;
        };
        let edge = options.edge_events.min(count);
        let first = all.iter().take(edge).cloned().collect::<Vec<_>>();
        let last = all.iter().skip(count - edge).cloned().collect::<Vec<_>>();
        let hash = serde_json::to_vec(&track.events)
            .map(|bytes| Sha256Digest::of(&bytes).to_string())
            .unwrap_or_default();
        let first_t = track.events.first().map(|event| event.start().0);
        let last_t = track.events.last().map(|event| event.start().0);
        *events = json!({
            "summary": {
                "count": count,
                "first_t": first_t,
                "last_t": last_t,
                "sha256": hash,
                "first": first,
                "last": last,
            }
        });
    }
}

/// Rounds every float according to the key it is stored under.
fn round(value: &mut Value, key: Option<&str>) {
    match value {
        Value::Number(number) if number.is_f64() => {
            let step = if key.is_some_and(|key| HUNDREDTHS_KEYS.contains(&key)) {
                100.0
            } else {
                1e6
            };
            if let Some(x) = number.as_f64()
                && let Some(rounded) = Number::from_f64((x * step).round_ties_even() / step)
            {
                *number = rounded;
            }
        }
        Value::Array(items) => {
            for item in items {
                round(item, key);
            }
        }
        Value::Object(map) => round_map(map),
        _ => {}
    }
}

fn round_map(map: &mut Map<String, Value>) {
    for (key, item) in map.iter_mut() {
        round(item, Some(key.as_str()));
    }
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Live playback of a stream (B06-T07; FR-005, PB-11).
//!
//! `live_start` plays an open stream through the session's spec, whose source must be a
//! stream. A worker thread compiles the spec against the stream's newest window about
//! once a second ([`sonarithm::Sonarithm::compile_stream`]) and places each window's
//! Score on the stream's own timeline ([`sonarithm_core::Score::offset`]): a row keeps
//! its Score time from one window to the next. The transport swaps each Score in while it
//! plays (`Player::swap`), so playback goes on from the same position and never stops at
//! a window's end; it trails the newest data by [`LAG`].
//!
//! The worker reads the stream through a snapshot ([`sonarithm_data::Snapshot`]), so the
//! connection keeps appending while a window compiles. Memory stays bounded by the
//! stream's own capacity; the compile sees only the spec's `window` when it has one.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use sonarithm::{Datasets, Snapshot, Sonarithm, Spec};
use sonarithm_core::Seconds;
use sonarithm_spec::{Pace, Source};
use sonarithm_stream::TableReader;

use crate::codes;
use crate::ipc::IpcError;
use crate::transport::LiveFeed;

/// How often the worker compiles the newest window.
pub(crate) const PERIOD: Duration = Duration::from_secs(1);

/// How far the playhead trails the newest data, in Score seconds: one compile period
/// plus room for a late window. A proposal, like every live default, until the B06
/// evaluation (sound-design §10.7; TODO(verify) with HP-11).
pub(crate) const LAG: f64 = 2.0;

/// How a live spec spaces rows in time (its pace, 07 NUMISONA `encode`).
#[derive(Clone, Debug, PartialEq)]
enum Timing {
    /// `secondsPerUnit` over a time field: Score time follows the field.
    PerUnit { field: String, seconds: f64 },
    /// `pointsPerSecond`, or units without a time field: every row is one step.
    PerRow { seconds: f64 },
}

/// What the worker needs from the effective spec.
pub(crate) struct LivePlan {
    spec: Spec,
    timing: Timing,
    /// The time field and the spec's `window` in seconds, when both are given.
    window: Option<(String, f64)>,
}

impl std::fmt::Debug for LivePlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LivePlan")
            .field("timing", &self.timing)
            .field("window", &self.window)
            .finish_non_exhaustive()
    }
}

/// Reads the live timing from the effective spec text.
///
/// # Errors
///
/// `SNR-APP-073` when the source is not a stream or the pace is a total duration; the
/// spec's own error when it does not parse.
pub(crate) fn plan(effective: &str) -> Result<LivePlan, IpcError> {
    let refused = |detail: &str| {
        IpcError::new(
            &codes::LIVE_SPEC,
            format!("The spec cannot play live: {detail}."),
        )
    };
    let parsed = sonarithm_spec::parse(effective).map_err(|diagnostics| {
        diagnostics
            .iter()
            .next()
            .map_or_else(|| refused("it does not parse"), IpcError::from_diagnostic)
    })?;
    let (parsed, _) = sonarithm_spec::defaults::apply_defaults(&parsed);
    let Source::Stream { window, .. } = &parsed.data.source else {
        return Err(refused("its data source is not a stream"));
    };
    let time = parsed.substrate.as_ref().and_then(|s| s.time.as_ref());
    let field = time.and_then(|t| t.field.clone());
    let pace = time.and_then(|t| t.pace).unwrap_or(Pace::PointsPerSecond(
        sonarithm_spec::defaults::DEFAULT_POINTS_PER_SECOND,
    ));
    let timing = match (pace, &field) {
        (Pace::SecondsPerUnit(seconds), Some(field)) => Timing::PerUnit {
            field: field.clone(),
            seconds,
        },
        (Pace::SecondsPerUnit(seconds), None) => Timing::PerRow { seconds },
        (Pace::PointsPerSecond(rate), _) => Timing::PerRow {
            seconds: 1.0 / rate,
        },
        _ => {
            return Err(refused(
                "a pace of a total duration has no fixed place for new rows; use secondsPerUnit or pointsPerSecond",
            ));
        }
    };
    let spec = Spec::from_json_str(effective).map_err(|err| refused(&err.to_string()))?;
    Ok(LivePlan {
        spec,
        timing,
        window: field.zip(window.map(sonarithm_spec::values::DurationValue::seconds)),
    })
}

/// The worker thread; it stops when dropped.
pub(crate) struct LiveWorker {
    stop: Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for LiveWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveWorker").finish_non_exhaustive()
    }
}

impl LiveWorker {
    /// Starts compiling `plan` against `reader`'s stream and feeding the transport.
    pub(crate) fn spawn(
        snr: Sonarithm,
        plan: LivePlan,
        datasets: Datasets,
        reader: TableReader,
        feed: LiveFeed,
    ) -> Result<Self, IpcError> {
        let (stop, stopped) = mpsc::channel();
        let mut run = Run {
            snr,
            plan,
            datasets,
            reader,
            feed,
            start: None,
            seen: None,
            last_error: None,
        };
        let thread = std::thread::Builder::new()
            .name("sonarithm-live".to_owned())
            .spawn(move || {
                loop {
                    run.step();
                    match stopped.recv_timeout(PERIOD) {
                        Err(RecvTimeoutError::Timeout) => {}
                        Ok(()) | Err(RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .map_err(|err| {
                IpcError::new(
                    &codes::CONTROL_THREAD,
                    format!("Live playback could not start: {err}."),
                )
            })?;
        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for LiveWorker {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct Run {
    snr: Sonarithm,
    plan: LivePlan,
    datasets: Datasets,
    reader: TableReader,
    feed: LiveFeed,
    /// The first window's start: Score time 0 of the stream's timeline.
    start: Option<f64>,
    /// One past the newest row compiled.
    seen: Option<u64>,
    last_error: Option<String>,
}

impl Run {
    /// Compiles the newest window if rows arrived since the last one.
    fn step(&mut self) {
        let Some(snapshot) = self.reader.read(sonarithm_data::StreamTable::snapshot) else {
            return;
        };
        if snapshot.num_rows() == 0 || self.seen == Some(snapshot.next_seq()) {
            return;
        }
        self.seen = Some(snapshot.next_seq());
        let window = match &self.plan.window {
            Some((field, seconds)) => snapshot.within(field, *seconds),
            None => snapshot,
        };
        let offset = self.offset(&window);
        match self
            .snr
            .compile_stream(&self.plan.spec, &self.datasets, &window)
        {
            Ok(compiled) => {
                let mut score = compiled.into_score();
                score.offset(Seconds(offset));
                self.feed.score(score);
                self.last_error = None;
            }
            Err(err) => {
                let error = crate::app::first_error(&err);
                let message = format!("A live window did not compile: {}", error.message);
                if self.last_error.as_ref() != Some(&message) {
                    self.last_error = Some(message.clone());
                    self.feed
                        .notice(IpcError::new(&codes::LIVE_COMPILE, message));
                }
            }
        }
    }

    /// Where `window` starts on the stream's timeline, in Score seconds.
    fn offset(&mut self, window: &Snapshot) -> f64 {
        let (here, scale) = match &self.plan.timing {
            Timing::PerUnit { field, seconds } => {
                let Some((earliest, _)) = window.time_span(field) else {
                    return 0.0;
                };
                (earliest, *seconds)
            }
            #[expect(
                clippy::cast_precision_loss,
                reason = "sequence numbers stay far below 2^52"
            )]
            Timing::PerRow { seconds } => (window.first_seq() as f64, *seconds),
        };
        let start = *self.start.get_or_insert(here);
        (here - start).max(0.0) * scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(source: &str, substrate: &str) -> String {
        format!(
            r#"{{"specVersion": "0.1", "data": {{"source": {source}}}, {substrate} "tracks": [{{"id": "v", "mark": "tone", "channels": {{"pitch": {{"field": "v"}}}}}}]}}"#
        )
    }

    const STREAM: &str =
        r#"{"kind": "stream", "transport": "tail", "path": "/data/live.csv", "window": {"s": 10}}"#;

    #[test]
    fn the_timing_follows_the_pace() {
        let per_unit = plan(&spec(
            STREAM,
            r#""substrate": {"time": {"field": "t", "pace": {"secondsPerUnit": 0.5}}},"#,
        ))
        .unwrap();
        assert_eq!(
            per_unit.timing,
            Timing::PerUnit {
                field: "t".to_owned(),
                seconds: 0.5
            }
        );
        assert_eq!(per_unit.window, Some(("t".to_owned(), 10.0)));
        // The spec's default pace, 3 points per second (SDR-19), steps by rows.
        let default = plan(&spec(STREAM, "")).unwrap();
        assert_eq!(default.timing, Timing::PerRow { seconds: 1.0 / 3.0 });
        assert_eq!(default.window, None);
    }

    #[test]
    fn only_a_stream_spec_with_a_fixed_pace_plays_live() {
        let inline = spec(r#"{"kind": "inline", "values": [{"v": 1}]}"#, "");
        assert_eq!(plan(&inline).unwrap_err().code, "SNR-APP-073");
        let total = spec(
            STREAM,
            r#""substrate": {"time": {"field": "t", "pace": {"duration": {"s": 10}}}},"#,
        );
        assert_eq!(plan(&total).unwrap_err().code, "SNR-APP-073");
    }
}

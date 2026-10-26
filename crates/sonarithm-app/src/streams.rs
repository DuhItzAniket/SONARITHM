// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Live streams in the app core (B06-T05; A11Y-16, ADR-0012).
//!
//! A stream opens in two steps. `stream_offer` describes a source and returns a one-time
//! token; nothing connects. `stream_connect` with that token is the user's approval, so
//! the host sends it only from the consent prompt, and one approval opens one connection
//! (`sonarithm_stream::Consent`).
//!
//! Open streams live on the transport's control thread ([`LiveStreams`]). Every tick it
//! turns connection changes into stream events, passes them through SONOTRACE's
//! [`RateLimiter`] (at most one notice per stream every 5 s, summed up; errors and alarms
//! never dropped) and hands the notices to the speech sequencer as stream announcements.
//! Each stream can be paused, resumed, hidden and stopped at any time.

use std::collections::BTreeMap;
use std::path::Path;

use sonarithm_audaxis::announce::{Announcement, Class};
use sonarithm_core::Seconds;
use sonarithm_i18n::{Args, Localizer};
use sonarithm_sonotrace::rate_limit::{EventKind, Notice, RateLimiter, StreamEvent, StreamState};
use sonarithm_stream::{Connection, ConnectionState, Endpoint};

use crate::codes;
use crate::ipc::{IpcError, StreamConnection, StreamInfo, StreamNotices};

/// Rows a stream keeps: memory stays bounded at any rate (PB-11), and at 1 kHz this is
/// 100 s of history. Live playback compiles only the spec's `window` of it (B06-T07).
pub(crate) const STREAM_ROWS: u64 = 100_000;

/// The transport name and target of `endpoint` as the protocol shows them: a file by its
/// name alone (ADR-0017 §7), URLs without user information.
pub(crate) fn target(endpoint: &Endpoint) -> (&'static str, String, Option<String>) {
    match endpoint {
        Endpoint::WebSocket { url } => ("websocket", without_userinfo(url), None),
        Endpoint::Osc { port } => ("osc", port.to_string(), None),
        Endpoint::Serial { port, baud } => ("serial", format!("{port}@{baud}"), None),
        Endpoint::Mqtt { url, topic } => ("mqtt", without_userinfo(url), Some(topic.clone())),
        Endpoint::Tail { path } => (
            "tail",
            path.file_name()
                .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
            None,
        ),
        _ => ("unknown", String::new(), None),
    }
}

/// `scheme://user:password@host/path` → `scheme://host/path`.
fn without_userinfo(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_owned();
    };
    let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let host = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    format!("{scheme}://{host}{path}")
}

/// The endpoint of a stream source the user chose. A file to follow must be an absolute
/// path in an existing folder; it is canonicalised here (ADR-0017 §7).
///
/// # Errors
///
/// `SNR-APP-071` for an unknown transport, a missing or bad address, or a file outside an
/// existing folder.
pub(crate) fn endpoint(
    transport: &str,
    url: Option<&str>,
    port: Option<&str>,
    topic: Option<&str>,
    path: Option<&str>,
) -> Result<Endpoint, IpcError> {
    let refused = |detail: String| {
        IpcError::new(
            &codes::STREAM_SOURCE,
            format!("The stream source is not valid: {detail}."),
        )
    };
    let path = match path {
        Some(path) if transport == "tail" => {
            let path = Path::new(path);
            let name = path.file_name().filter(|_| path.is_absolute());
            let folder = path.parent().and_then(|folder| folder.canonicalize().ok());
            let (Some(name), Some(folder)) = (name, folder) else {
                return Err(refused(
                    "a file to follow must be in an existing folder".to_owned(),
                ));
            };
            Some(folder.join(name).to_string_lossy().into_owned())
        }
        other => other.map(str::to_owned),
    };
    Endpoint::from_parts(transport, url, port, topic, path.as_deref())
        .map_err(|err| refused(err.to_string()))
}

/// What the user does to a stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StreamAction {
    Pause,
    Resume,
    Hide,
    Stop,
}

struct Live {
    transport: &'static str,
    target: String,
    connection: Connection,
    last: ConnectionState,
}

/// The open streams and their notices, on the control thread.
pub(crate) struct LiveStreams {
    streams: BTreeMap<String, Live>,
    /// Names outlive their streams, for notices queued before a stop.
    names: BTreeMap<String, String>,
    limiter: RateLimiter,
    localizer: Option<Localizer>,
}

impl LiveStreams {
    pub(crate) fn new() -> Self {
        Self {
            streams: BTreeMap::new(),
            names: BTreeMap::new(),
            limiter: RateLimiter::new(),
            localizer: Localizer::new("en-US").ok(),
        }
    }

    pub(crate) fn open(&mut self, id: String, endpoint: &Endpoint, connection: Connection) {
        let (transport, target, _) = target(endpoint);
        self.names.insert(id.clone(), target.clone());
        self.streams.insert(
            id,
            Live {
                transport,
                target,
                connection,
                last: ConnectionState::Connecting,
            },
        );
    }

    /// Pauses, resumes, hides or stops stream `id` (A11Y-16).
    pub(crate) fn act(&mut self, id: &str, action: StreamAction) -> Result<(), IpcError> {
        let Some(live) = self.streams.get_mut(id) else {
            return Err(IpcError::new(
                &codes::UNKNOWN_STREAM,
                format!("No open stream has the ID {id}."),
            ));
        };
        match action {
            StreamAction::Pause => {
                live.connection.pause();
                self.limiter.pause(id);
            }
            StreamAction::Resume => {
                live.connection.resume();
                self.limiter.resume(id);
            }
            StreamAction::Hide => self.limiter.hide(id),
            StreamAction::Stop => {
                if let Some(mut live) = self.streams.remove(id) {
                    live.connection.stop();
                }
                self.limiter.stop(id);
            }
        }
        Ok(())
    }

    /// A read handle on stream `id`'s rows (B06-T07).
    pub(crate) fn reader(&self, id: &str) -> Result<sonarithm_stream::TableReader, IpcError> {
        self.streams
            .get(id)
            .map(|live| live.connection.reader())
            .ok_or_else(|| {
                IpcError::new(
                    &codes::UNKNOWN_STREAM,
                    format!("No open stream has the ID {id}."),
                )
            })
    }

    /// Stops every stream (shutdown).
    pub(crate) fn stop_all(&mut self) {
        for (id, mut live) in std::mem::take(&mut self.streams) {
            live.connection.stop();
            self.limiter.stop(&id);
        }
    }

    pub(crate) fn list(&self) -> Vec<StreamInfo> {
        self.streams
            .iter()
            .map(|(id, live)| {
                let status = live.connection.status();
                StreamInfo {
                    id: id.clone(),
                    transport: live.transport.to_owned(),
                    target: live.target.clone(),
                    connection: match status.state {
                        ConnectionState::Connecting => StreamConnection::Connecting,
                        ConnectionState::Connected => StreamConnection::Connected,
                        ConnectionState::Paused => StreamConnection::Paused,
                        ConnectionState::Reconnecting => StreamConnection::Reconnecting,
                        _ => StreamConnection::Stopped,
                    },
                    notices: match self.limiter.state(id) {
                        StreamState::Paused => StreamNotices::Paused,
                        StreamState::Hidden => StreamNotices::Hidden,
                        _ => StreamNotices::Active,
                    },
                    received: status.received,
                    invalid: status.invalid,
                }
            })
            .collect()
    }

    /// One tick at `now`: connection changes become events, and the notices due become
    /// stream announcements (errors and alarms as errors, hidden ones log only).
    pub(crate) fn tick(&mut self, now: Seconds) -> Vec<Announcement> {
        for (id, live) in &mut self.streams {
            let status = live.connection.status();
            let event = match (live.last, status.state) {
                (before, after) if before == after => None,
                // Resuming is the user's own action, not news.
                (ConnectionState::Paused, ConnectionState::Connected)
                | (_, ConnectionState::Paused) => None,
                (_, ConnectionState::Connected) => Some((EventKind::Connected, String::new())),
                // Never connected: the first failure is an error, spoken whatever the state.
                (ConnectionState::Connecting, ConnectionState::Reconnecting) => Some((
                    EventKind::Error,
                    status.last_error.clone().unwrap_or_default(),
                )),
                (_, ConnectionState::Reconnecting) => Some((
                    EventKind::Disconnected,
                    status.last_error.clone().unwrap_or_default(),
                )),
                _ => None,
            };
            live.last = status.state;
            if let Some((kind, detail)) = event {
                self.limiter
                    .push(StreamEvent::new(id.clone(), kind, now, detail));
            }
        }
        self.limiter
            .poll(now)
            .iter()
            .filter_map(|notice| self.announcement(notice))
            .collect()
    }

    /// The announcement for a notice, from the `stream-*` messages.
    fn announcement(&self, notice: &Notice) -> Option<Announcement> {
        let localizer = self.localizer.as_ref()?;
        let name = self
            .names
            .get(&notice.stream)
            .cloned()
            .unwrap_or_else(|| notice.stream.clone());
        let args = Args::new()
            .text("stream", name)
            .text("detail", notice.detail.clone());
        let text = match (notice.kind, notice.count) {
            (EventKind::Alarm, _) => localizer.format("stream-alarm", &args),
            (EventKind::Error, _) => localizer.format("stream-error", &args),
            (EventKind::Connected, 1) => localizer.format("stream-connected", &args),
            (EventKind::Disconnected, 1) => localizer.format("stream-disconnected", &args),
            (_, 1) => localizer.format("stream-event", &args),
            (kind, count) => {
                let seconds = (notice.span.1.0 - notice.span.0.0).ceil().max(1.0);
                localizer.format(
                    "stream-summary",
                    &args
                        .text("kind", kind_key(kind))
                        .number("count", f64::from(count))
                        .number("seconds", seconds),
                )
            }
        }
        .ok()?;
        let mut announcement = if notice.urgent {
            Announcement::error(text.clone(), text)
        } else {
            let mut status = Announcement::status(text.clone(), text);
            status.class = Class::Stream;
            status
        };
        announcement.speak = notice.spoken;
        Some(announcement)
    }
}

/// The `$kind` selector of `stream-summary`.
fn kind_key(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Threshold => "threshold",
        EventKind::Anomaly => "anomaly",
        EventKind::Connected => "connected",
        EventKind::Disconnected => "disconnected",
        EventKind::Error => "error",
        _ => "alarm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_lose_their_user_information() {
        assert_eq!(
            without_userinfo("ws://ann:secret@example.org:9000/feed"),
            "ws://example.org:9000/feed"
        );
        assert_eq!(
            without_userinfo("ws://localhost:9000"),
            "ws://localhost:9000"
        );
        assert_eq!(without_userinfo("not a url"), "not a url");
    }

    #[test]
    fn a_file_to_follow_is_named_alone() {
        let dir = std::env::temp_dir();
        let path = dir.join("sensor.csv");
        let endpoint = endpoint("tail", None, None, None, path.to_str()).unwrap();
        assert_eq!(target(&endpoint), ("tail", "sensor.csv".to_owned(), None));
        let relative = endpoint_err("tail", Some("sensor.csv"));
        assert_eq!(relative, "SNR-APP-071");
        let nowhere = dir.join("no such folder").join("sensor.csv");
        assert_eq!(endpoint_err("tail", nowhere.to_str()), "SNR-APP-071");
        assert_eq!(endpoint_err("carrier pigeon", None), "SNR-APP-071");
    }

    fn endpoint_err(transport: &str, path: Option<&str>) -> String {
        match endpoint(transport, None, None, None, path) {
            Ok(_) => "ok".to_owned(),
            Err(err) => err.code,
        }
    }
}

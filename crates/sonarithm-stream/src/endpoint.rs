// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Where a stream comes from, and the user's consent to connect to it.

use std::path::PathBuf;

use crate::StreamError;

/// A stream's source. Describing one opens nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Endpoint {
    /// A WebSocket server, `ws://host:port/path`.
    WebSocket {
        /// The URL.
        url: String,
    },
    /// OSC packets sent to a local UDP port.
    Osc {
        /// The local port to listen on.
        port: u16,
    },
    /// A serial port, one message per line.
    Serial {
        /// The port's name, such as `COM3` or `/dev/ttyUSB0`.
        port: String,
        /// The baud rate.
        baud: u32,
    },
    /// An MQTT topic on a broker, `mqtt://host:port`.
    Mqtt {
        /// The broker URL.
        url: String,
        /// The topic.
        topic: String,
    },
    /// Lines appended to a file.
    Tail {
        /// The file.
        path: PathBuf,
    },
}

/// The baud rate when a serial `port` does not give one (`COM3@115200`).
pub const DEFAULT_BAUD: u32 = 9600;

impl Endpoint {
    /// The endpoint a spec's `{"kind": "stream", …}` source describes (RFC-0001: the
    /// `transport` name and its address key). Nothing connects.
    ///
    /// # Errors
    ///
    /// [`StreamError::Source`] for an unknown transport, a missing address key, or an
    /// address that does not parse.
    pub fn from_parts(
        transport: &str,
        url: Option<&str>,
        port: Option<&str>,
        topic: Option<&str>,
        path: Option<&str>,
    ) -> Result<Self, StreamError> {
        let missing = |key: &str| StreamError::Source(format!("`{key}` is missing"));
        match transport {
            "websocket" => {
                let url = url.ok_or_else(|| missing("url"))?;
                if !url.starts_with("ws://") {
                    return Err(StreamError::Source(
                        "only ws:// URLs are supported so far (no TLS)".to_owned(),
                    ));
                }
                Ok(Self::WebSocket {
                    url: url.to_owned(),
                })
            }
            "osc" => {
                let port = port.ok_or_else(|| missing("port"))?;
                let port = port.parse::<u16>().map_err(|_| {
                    StreamError::Source(format!("`{port}` is not a UDP port number"))
                })?;
                Ok(Self::Osc { port })
            }
            "serial" => {
                let port = port.ok_or_else(|| missing("port"))?;
                let (name, baud) = match port.split_once('@') {
                    Some((name, baud)) => (
                        name,
                        baud.parse::<u32>().map_err(|_| {
                            StreamError::Source(format!("`{baud}` is not a baud rate"))
                        })?,
                    ),
                    None => (port, DEFAULT_BAUD),
                };
                Ok(Self::Serial {
                    port: name.to_owned(),
                    baud,
                })
            }
            "mqtt" => Ok(Self::Mqtt {
                url: url.ok_or_else(|| missing("url"))?.to_owned(),
                topic: topic.ok_or_else(|| missing("topic"))?.to_owned(),
            }),
            "tail" => Ok(Self::Tail {
                path: PathBuf::from(path.ok_or_else(|| missing("path"))?),
            }),
            other => Err(StreamError::Source(format!(
                "`{other}` is not a stream transport"
            ))),
        }
    }

    /// The transport's name, such as `WebSocket`.
    #[must_use]
    pub const fn transport(&self) -> &'static str {
        match self {
            Self::WebSocket { .. } => "WebSocket",
            Self::Osc { .. } => "OSC",
            Self::Serial { .. } => "serial",
            Self::Mqtt { .. } => "MQTT",
            Self::Tail { .. } => "file",
        }
    }

    /// The question a consent prompt asks, in one sentence.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::WebSocket { url } => format!("Connect to the WebSocket server at {url}?"),
            Self::Osc { port } => format!("Listen for OSC messages on UDP port {port}?"),
            Self::Serial { port, baud } => format!("Read the serial port {port} at {baud} baud?"),
            Self::Mqtt { url, topic } => {
                format!("Subscribe to the MQTT topic {topic} on the broker at {url}?")
            }
            Self::Tail { path } => format!(
                "Follow the file {}?",
                path.file_name()
                    .map_or_else(|| "chosen".into(), |n| n.to_string_lossy())
            ),
        }
    }
}

/// The user's approval to connect to one endpoint (ADR-0012: per-action consent).
///
/// [`crate::connect`] takes it by value, so each approval opens one connection.
#[derive(Debug)]
pub struct Consent {
    endpoint: Endpoint,
}

impl Consent {
    /// Records that the user approved connecting to `endpoint`.
    ///
    /// Call this only from the handler of the user's own confirmation: the connect
    /// dialog's button after it showed [`Endpoint::describe`], or a command-line flag the
    /// user typed. Never from a spec, a project file, a setting's default or a reconnect.
    #[must_use]
    pub fn user_approved(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }

    /// The approved endpoint.
    #[must_use]
    pub const fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// Uses up the approval: one approval opens one connection.
    pub(crate) fn into_endpoint(self) -> Endpoint {
        self.endpoint
    }
}

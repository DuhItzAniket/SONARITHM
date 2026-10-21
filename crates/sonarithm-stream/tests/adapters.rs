// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Adapter tests against local loopback servers (B06-T02): every connection stays on
//! 127.0.0.1 or a temporary file.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::io::Write as _;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use arrow_array::{Array, Float64Array};
use sonarithm_stream::{
    Capacity, Connection, ConnectionState, Consent, Endpoint, Options, connect,
};

/// Waits until `done` holds for the connection's table, at most 10 s.
fn wait_rows(connection: &Connection, rows: u64) -> u64 {
    let start = Instant::now();
    loop {
        let held = connection
            .read(sonarithm_stream::StreamTable::next_seq)
            .unwrap_or(0);
        if held >= rows || start.elapsed() > Duration::from_secs(10) {
            return held;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn values(connection: &Connection, field: &str) -> Vec<f64> {
    connection
        .read(|table| {
            let snapshot = table.snapshot();
            let index = snapshot.schema().index_of(field).unwrap();
            snapshot
                .batches()
                .iter()
                .flat_map(|b| {
                    let column = b
                        .column(index)
                        .as_any()
                        .downcast_ref::<Float64Array>()
                        .unwrap();
                    (0..column.len())
                        .map(|i| column.value(i))
                        .collect::<Vec<_>>()
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(feature = "websocket")]
mod websocket {
    use std::net::{TcpListener, TcpStream};

    use serde_json::json;
    use sonarithm_stream::MessageSchema;

    use tungstenite::Message;

    use super::*;

    /// Serves `rounds` connections; each sends `messages` and closes.
    fn server(rounds: usize, messages: Vec<String>) -> (u16, std::thread::JoinHandle<usize>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let thread = std::thread::spawn(move || {
            let mut served = 0;
            for _ in 0..rounds {
                let Ok((stream, _)) = listener.accept() else {
                    break;
                };
                let mut socket = tungstenite::accept(stream).unwrap();
                for message in &messages {
                    socket.send(Message::text(message.clone())).unwrap();
                }
                std::thread::sleep(Duration::from_millis(100));
                let _ = socket.close(None);
                let _ = socket.flush();
                served += 1;
            }
            served
        });
        (port, thread)
    }

    #[test]
    fn a_websocket_stream_fills_its_table_and_reconnects() {
        let messages: Vec<String> = (0..5)
            .map(|i| json!({ "t": i, "cpu": i * 10, "note": "x" }).to_string())
            .collect();
        let (port, server) = server(2, messages);
        let endpoint = Endpoint::from_parts(
            "websocket",
            Some(&format!("ws://127.0.0.1:{port}/cpu")),
            None,
            None,
            None,
        )
        .unwrap();
        let mut options = Options::new(Capacity::rows(100));
        options.backoff = Duration::from_millis(50);
        let mut connection = connect(Consent::user_approved(endpoint), options).unwrap();
        // Two rounds of five: the second arrives after the reconnect.
        assert_eq!(wait_rows(&connection, 10), 10);
        assert_eq!(
            &values(&connection, "cpu")[..5],
            [0.0, 10.0, 20.0, 30.0, 40.0]
        );
        connection.stop();
        assert_eq!(connection.status().state, ConnectionState::Stopped);
        assert_eq!(server.join().unwrap(), 2);
    }

    #[test]
    fn a16_no_connection_opens_without_the_users_consent() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("ws://127.0.0.1:{port}/");
        // Describing the stream, and asking the user, open nothing.
        let endpoint = Endpoint::from_parts("websocket", Some(&url), None, None, None).unwrap();
        assert!(endpoint.describe().contains(&format!("127.0.0.1:{port}")));
        std::thread::sleep(Duration::from_millis(300));
        assert!(matches!(
            listener.accept(),
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock
        ));
        // Only the consent opens it.
        let _connection = connect(
            Consent::user_approved(endpoint),
            Options::new(Capacity::rows(10)),
        )
        .unwrap();
        let start = Instant::now();
        let accepted = loop {
            match listener.accept() {
                Ok((stream, _)) => break Some(stream),
                Err(_) if start.elapsed() < Duration::from_secs(5) => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break None,
            }
        };
        let _: Option<TcpStream> = accepted;
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "the approved connection arrived"
        );
    }

    #[test]
    fn invalid_messages_are_counted_and_the_window_stays_bounded() {
        let mut messages: Vec<String> = (0..20).map(|i| json!({ "v": i }).to_string()).collect();
        messages.insert(3, "not json".to_owned());
        messages.insert(5, json!({ "v": "high" }).to_string());
        let (port, server) = server(1, messages);
        let schema = MessageSchema::from_json_schema(&json!({
            "type": "object", "properties": { "v": { "type": "number" } }, "required": ["v"]
        }))
        .unwrap();
        let endpoint = Endpoint::WebSocket {
            url: format!("ws://127.0.0.1:{port}/"),
        };
        let options = Options::new(Capacity::rows(4)).with_schema(schema);
        let mut connection = connect(Consent::user_approved(endpoint), options).unwrap();
        assert_eq!(wait_rows(&connection, 20), 20);
        connection.stop();
        let status = connection.status();
        // "not json" never becomes a message; the text value fails the schema.
        assert_eq!(status.invalid, 1);
        assert_eq!(connection.read(sonarithm_stream::StreamTable::len), Some(4));
        assert_eq!(values(&connection, "v"), [16.0, 17.0, 18.0, 19.0]);
        server.join().unwrap();
    }
}

#[cfg(feature = "osc")]
#[test]
fn an_osc_stream_reads_packets_sent_to_its_port() {
    use std::net::UdpSocket;

    use rosc::{OscMessage, OscPacket, OscType};

    let probe = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);
    let endpoint = Endpoint::Osc { port };
    let mut connection = connect(
        Consent::user_approved(endpoint),
        Options::new(Capacity::rows(100)),
    )
    .unwrap();
    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
    let start = Instant::now();
    // Send until the listener is up and the rows arrive.
    while connection
        .read(sonarithm_stream::StreamTable::next_seq)
        .unwrap_or(0)
        < 3
        && start.elapsed() < Duration::from_secs(10)
    {
        for value in [1.0_f32, 2.0, 3.0] {
            let packet = OscPacket::Message(OscMessage {
                addr: "/sensor/temp".into(),
                args: vec![OscType::Float(value)],
            });
            sender
                .send_to(
                    &rosc::encoder::encode(&packet).unwrap(),
                    ("127.0.0.1", port),
                )
                .unwrap();
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    connection.stop();
    let v0 = values(&connection, "v0");
    assert!(v0.len() >= 3, "{v0:?}");
    assert!(v0.iter().all(|v| [1.0, 2.0, 3.0].contains(v)));
}

/// A file in the system temporary folder, removed on drop.
struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn a_followed_file_streams_lines_appended_after_it_started() {
    let file = TempFile(
        std::env::temp_dir().join(format!("sonarithm-stream-tail-{}.log", std::process::id())),
    );
    std::fs::write(&file.0, "100,1000\n").unwrap();
    let mut connection = connect(
        Consent::user_approved(Endpoint::Tail {
            path: file.0.clone(),
        }),
        Options::new(Capacity::rows(100)),
    )
    .unwrap();
    // Let the reader reach the end before appending.
    let start = Instant::now();
    while connection.status().state != ConnectionState::Connected
        && start.elapsed() < Duration::from_secs(5)
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut writer = std::fs::OpenOptions::new()
        .append(true)
        .open(&file.0)
        .unwrap();
    writer
        .write_all(b"1,10\n2,20\n{\"v0\": 3, \"v1\": 30}\n4,4")
        .unwrap();
    writer.flush().unwrap();
    assert_eq!(wait_rows(&connection, 3), 3);
    // The partial last line waits for its newline.
    writer.write_all(b"0\n").unwrap();
    writer.flush().unwrap();
    assert_eq!(wait_rows(&connection, 4), 4);
    connection.stop();
    assert_eq!(values(&connection, "v1"), [10.0, 20.0, 30.0, 40.0]);
}

#[test]
fn pausing_discards_messages_until_resumed() {
    let file = TempFile(
        std::env::temp_dir().join(format!("sonarithm-stream-pause-{}.log", std::process::id())),
    );
    std::fs::write(&file.0, "").unwrap();
    let mut connection = connect(
        Consent::user_approved(Endpoint::Tail {
            path: file.0.clone(),
        }),
        Options::new(Capacity::rows(100)),
    )
    .unwrap();
    let start = Instant::now();
    while connection.status().state != ConnectionState::Connected
        && start.elapsed() < Duration::from_secs(5)
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    connection.pause();
    assert_eq!(connection.status().state, ConnectionState::Paused);
    let mut writer = std::fs::OpenOptions::new()
        .append(true)
        .open(&file.0)
        .unwrap();
    writer.write_all(b"1\n2\n").unwrap();
    writer.flush().unwrap();
    let start = Instant::now();
    while connection.status().discarded < 2 && start.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(10));
    }
    connection.resume();
    writer.write_all(b"3\n").unwrap();
    writer.flush().unwrap();
    assert_eq!(wait_rows(&connection, 1), 1);
    connection.stop();
    assert_eq!(connection.status().discarded, 2);
    assert_eq!(values(&connection, "v0"), [3.0]);
}

#[test]
fn unsupported_and_incomplete_sources_are_refused() {
    let err = Endpoint::from_parts(
        "websocket",
        Some("wss://example.org/feed"),
        None,
        None,
        None,
    )
    .unwrap_err();
    assert!(err.to_string().contains("ws://"));
    assert!(Endpoint::from_parts("osc", None, None, None, None).is_err());
    assert!(Endpoint::from_parts("carrier-pigeon", None, None, None, None).is_err());
    let serial = Endpoint::from_parts("serial", None, Some("COM3@115200"), None, None).unwrap();
    assert_eq!(
        serial,
        Endpoint::Serial {
            port: "COM3".into(),
            baud: 115_200
        }
    );
    #[cfg(not(feature = "serial"))]
    assert!(
        connect(
            Consent::user_approved(serial),
            Options::new(Capacity::rows(1))
        )
        .is_err()
    );
}

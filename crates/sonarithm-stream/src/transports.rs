// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The transports' readers. Each `open` connects (or binds), each `read` waits at most
//! [`READ_TIMEOUT`], so a stop request is honoured quickly.

use std::fs::File;
use std::io::{BufRead as _, BufReader, Seek as _, SeekFrom};
use std::path::PathBuf;
use std::time::Duration;

use serde_json::{Map, Value};

use crate::StreamError;
use crate::connection::Reader;
use crate::decode;
use crate::endpoint::Endpoint;

/// The longest a read waits before the thread checks for a stop request.
pub(crate) const READ_TIMEOUT: Duration = Duration::from_millis(100);

/// The reader for `endpoint`. Nothing connects until the thread calls `open`.
pub(crate) fn open(endpoint: &Endpoint) -> Result<Box<dyn Reader>, StreamError> {
    match endpoint {
        Endpoint::Tail { path } => Ok(Box::new(Tail {
            path: path.clone(),
            reader: None,
            position: None,
        })),
        #[cfg(feature = "websocket")]
        Endpoint::WebSocket { url } => Ok(Box::new(websocket::WebSocket::new(url.clone()))),
        #[cfg(feature = "osc")]
        Endpoint::Osc { port } => Ok(Box::new(osc::Osc::new(*port))),
        #[cfg(feature = "serial")]
        Endpoint::Serial { port, baud } => Ok(Box::new(serial::Serial::new(port.clone(), *baud))),
        #[cfg(feature = "mqtt")]
        Endpoint::Mqtt { url, topic } => Ok(Box::new(mqtt::Mqtt::new(url.clone(), topic.clone()))),
        #[allow(unreachable_patterns, reason = "reachable only when a feature is off")]
        other => Err(StreamError::Unsupported(other.transport())),
    }
}

/// Splits complete lines off `pending` and decodes each; a partial line stays.
#[cfg(feature = "serial")]
fn lines(pending: &mut Vec<u8>, out: &mut Vec<Map<String, Value>>) {
    while let Some(end) = pending.iter().position(|b| *b == b'\n') {
        let line: Vec<u8> = pending.drain(..=end).collect();
        if let Ok(text) = std::str::from_utf8(&line)
            && let Ok(object) = decode::line(text)
        {
            out.push(object);
        }
    }
    if pending.len() > decode::MAX_MESSAGE {
        pending.clear(); // an endless line: drop it
    }
}

/// Lines appended to a file, from the end it had when the stream started.
struct Tail {
    path: PathBuf,
    reader: Option<BufReader<File>>,
    position: Option<u64>,
}

impl Reader for Tail {
    fn open(&mut self) -> Result<(), String> {
        let mut file = File::open(&self.path).map_err(|err| err.to_string())?;
        let end = file.seek(SeekFrom::End(0)).map_err(|err| err.to_string())?;
        // Resume where we were, unless the file was truncated or replaced by a shorter one.
        let start = self.position.filter(|p| *p <= end).unwrap_or(end);
        file.seek(SeekFrom::Start(start))
            .map_err(|err| err.to_string())?;
        self.position = Some(start);
        self.reader = Some(BufReader::new(file));
        Ok(())
    }

    fn read(&mut self) -> Result<Vec<Map<String, Value>>, String> {
        let reader = self.reader.as_mut().ok_or("the file is not open")?;
        let mut out = Vec::new();
        let mut line = String::new();
        loop {
            line.clear();
            let read = reader.read_line(&mut line).map_err(|err| err.to_string())?;
            if read == 0 || !line.ends_with('\n') {
                // At the end, or a partial line: go back to its start and wait.
                if read > 0 {
                    reader
                        .seek_relative(-i64::try_from(read).unwrap_or(0))
                        .map_err(|err| err.to_string())?;
                }
                break;
            }
            if let Some(p) = self.position.as_mut() {
                *p += read as u64;
            }
            if let Ok(object) = decode::line(&line) {
                out.push(object);
            }
        }
        if out.is_empty() {
            std::thread::sleep(READ_TIMEOUT);
        }
        Ok(out)
    }
}

#[cfg(feature = "websocket")]
mod websocket {
    use std::net::TcpStream;

    use serde_json::{Map, Value};
    use tungstenite::stream::MaybeTlsStream;
    use tungstenite::{Message, WebSocket as Socket};

    use super::READ_TIMEOUT;
    use crate::connection::Reader;
    use crate::decode;

    pub(super) struct WebSocket {
        url: String,
        socket: Option<Socket<MaybeTlsStream<TcpStream>>>,
    }

    impl WebSocket {
        pub(super) fn new(url: String) -> Self {
            Self { url, socket: None }
        }
    }

    impl Reader for WebSocket {
        fn open(&mut self) -> Result<(), String> {
            self.socket = None;
            let (mut socket, _) =
                tungstenite::connect(self.url.as_str()).map_err(|err| err.to_string())?;
            if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
                stream
                    .set_read_timeout(Some(READ_TIMEOUT))
                    .map_err(|err| err.to_string())?;
            }
            self.socket = Some(socket);
            Ok(())
        }

        fn read(&mut self) -> Result<Vec<Map<String, Value>>, String> {
            let socket = self.socket.as_mut().ok_or("not connected")?;
            let mut out = Vec::new();
            match socket.read() {
                Ok(Message::Text(text)) => {
                    if let Ok(object) = decode::json_object(text.as_bytes()) {
                        out.push(object);
                    }
                }
                Ok(Message::Binary(bytes)) => {
                    if let Ok(object) = decode::json_object(&bytes) {
                        out.push(object);
                    }
                }
                Ok(Message::Close(_)) => return Err("the server closed the stream".to_owned()),
                Ok(_) => {}
                Err(tungstenite::Error::Io(err))
                    if matches!(
                        err.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    // Sends any pending pong.
                    let _ = socket.flush();
                }
                Err(err) => return Err(err.to_string()),
            }
            Ok(out)
        }
    }
}

#[cfg(feature = "osc")]
mod osc {
    use std::net::UdpSocket;

    use serde_json::{Map, Value};

    use super::READ_TIMEOUT;
    use crate::connection::Reader;
    use crate::decode;

    pub(super) struct Osc {
        port: u16,
        socket: Option<UdpSocket>,
        buffer: Vec<u8>,
    }

    impl Osc {
        pub(super) fn new(port: u16) -> Self {
            Self {
                port,
                socket: None,
                buffer: vec![0; 65_536],
            }
        }
    }

    impl Reader for Osc {
        fn open(&mut self) -> Result<(), String> {
            // Any interface: OSC usually comes from another device on the network. The
            // consent prompt says so (Endpoint::describe).
            let socket = UdpSocket::bind(("0.0.0.0", self.port)).map_err(|err| err.to_string())?;
            socket
                .set_read_timeout(Some(READ_TIMEOUT))
                .map_err(|err| err.to_string())?;
            self.socket = Some(socket);
            Ok(())
        }

        fn read(&mut self) -> Result<Vec<Map<String, Value>>, String> {
            let socket = self.socket.as_ref().ok_or("not listening")?;
            match socket.recv(&mut self.buffer) {
                Ok(n) => Ok(self
                    .buffer
                    .get(..n)
                    .and_then(|packet| decode::osc(packet).ok())
                    .unwrap_or_default()),
                Err(err)
                    if matches!(
                        err.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    Ok(Vec::new())
                }
                Err(err) => Err(err.to_string()),
            }
        }
    }
}

#[cfg(feature = "serial")]
mod serial {
    use std::io::Read as _;

    use serde_json::{Map, Value};

    use super::{READ_TIMEOUT, lines};
    use crate::connection::Reader;

    pub(super) struct Serial {
        port: String,
        baud: u32,
        handle: Option<Box<dyn serialport::SerialPort>>,
        pending: Vec<u8>,
    }

    impl Serial {
        pub(super) fn new(port: String, baud: u32) -> Self {
            Self {
                port,
                baud,
                handle: None,
                pending: Vec::new(),
            }
        }
    }

    impl Reader for Serial {
        fn open(&mut self) -> Result<(), String> {
            self.pending.clear();
            let handle = serialport::new(&self.port, self.baud)
                .timeout(READ_TIMEOUT)
                .open()
                .map_err(|err| err.to_string())?;
            self.handle = Some(handle);
            Ok(())
        }

        fn read(&mut self) -> Result<Vec<Map<String, Value>>, String> {
            let handle = self.handle.as_mut().ok_or("the port is not open")?;
            let mut chunk = [0_u8; 4096];
            let mut out = Vec::new();
            match handle.read(&mut chunk) {
                Ok(0) => return Err("the port closed".to_owned()),
                Ok(n) => {
                    self.pending
                        .extend_from_slice(chunk.get(..n).unwrap_or_default());
                    lines(&mut self.pending, &mut out);
                }
                Err(err) if err.kind() == std::io::ErrorKind::TimedOut => {}
                Err(err) => return Err(err.to_string()),
            }
            Ok(out)
        }
    }
}

#[cfg(feature = "mqtt")]
mod mqtt {
    use rumqttc::{Client, Connection, Event, MqttOptions, Packet, QoS};
    use serde_json::{Map, Value};

    use super::READ_TIMEOUT;
    use crate::connection::Reader;
    use crate::decode;

    pub(super) struct Mqtt {
        url: String,
        topic: String,
        session: Option<(Client, Connection)>,
    }

    impl Mqtt {
        pub(super) fn new(url: String, topic: String) -> Self {
            Self {
                url,
                topic,
                session: None,
            }
        }
    }

    fn host_port(url: &str) -> Result<(String, u16), String> {
        let rest = url
            .strip_prefix("mqtt://")
            .ok_or("only mqtt:// URLs are supported so far (no TLS)")?;
        let rest = rest.split('/').next().unwrap_or(rest);
        match rest.rsplit_once(':') {
            Some((host, port)) => Ok((
                host.to_owned(),
                port.parse()
                    .map_err(|_| format!("`{port}` is not a port"))?,
            )),
            None => Ok((rest.to_owned(), 1883)),
        }
    }

    impl Reader for Mqtt {
        fn open(&mut self) -> Result<(), String> {
            self.session = None;
            let (host, port) = host_port(&self.url)?;
            let id = format!("sonarithm-{}", std::process::id());
            let (client, connection) = Client::new(MqttOptions::new(id, host, port), 64);
            client
                .subscribe(self.topic.as_str(), QoS::AtMostOnce)
                .map_err(|err| err.to_string())?;
            self.session = Some((client, connection));
            Ok(())
        }

        fn read(&mut self) -> Result<Vec<Map<String, Value>>, String> {
            let (_, connection) = self.session.as_mut().ok_or("not connected")?;
            match connection.recv_timeout(READ_TIMEOUT) {
                Ok(Ok(Event::Incoming(Packet::Publish(publish)))) => {
                    Ok(decode::json_object(&publish.payload)
                        .map(|object| vec![object])
                        .unwrap_or_default())
                }
                Ok(Ok(_)) | Err(_) => Ok(Vec::new()),
                Ok(Err(err)) => Err(err.to_string()),
            }
        }
    }
}

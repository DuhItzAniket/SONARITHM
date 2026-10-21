// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A live connection: its thread, reconnects, pause and stop, and the stream table it fills.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde_json::{Map, Value};
use sonarithm_core::TableId;
use sonarithm_data::{Capacity, StreamTable};

use crate::StreamError;
use crate::decode::row;
use crate::endpoint::{Consent, Endpoint};
use crate::rows::RowBuffer;
use crate::schema::MessageSchema;
use crate::transports;

/// How a connection runs.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Options {
    /// The message schema, or `None` to take it from the first valid message.
    pub schema: Option<MessageSchema>,
    /// How much the stream table keeps (03 §8).
    pub capacity: Capacity,
    /// Append once this many rows wait…
    pub flush_rows: usize,
    /// …or this long after the last append.
    pub flush_every: Duration,
    /// The first reconnect delay; it doubles up to `max_backoff`.
    pub backoff: Duration,
    /// The longest reconnect delay.
    pub max_backoff: Duration,
}

impl Options {
    /// Defaults: the schema from the first message, `capacity`, appends every 256 rows or
    /// 50 ms, reconnects after 0.25 s doubling to 30 s.
    #[must_use]
    pub fn new(capacity: Capacity) -> Self {
        Self {
            schema: None,
            capacity,
            flush_rows: 256,
            flush_every: Duration::from_millis(50),
            backoff: Duration::from_millis(250),
            max_backoff: Duration::from_secs(30),
        }
    }

    /// The same options with a message schema.
    #[must_use]
    pub fn with_schema(mut self, schema: MessageSchema) -> Self {
        self.schema = Some(schema);
        self
    }
}

/// Where a connection is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConnectionState {
    /// Opening for the first time.
    Connecting,
    /// Receiving.
    Connected,
    /// Connected, but discarding messages until resumed (A11Y-16).
    Paused,
    /// Lost; trying again after a delay.
    Reconnecting,
    /// Stopped by the user; the thread has ended or is ending.
    Stopped,
}

/// A connection's counters and state.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ConnectionStatus {
    /// The state.
    pub state: ConnectionState,
    /// Messages received.
    pub received: u64,
    /// Messages dropped because they did not fit the schema.
    pub invalid: u64,
    /// Messages discarded while paused.
    pub discarded: u64,
    /// Connection attempts that failed since the last success.
    pub failed_attempts: u32,
    /// The last error, in words.
    pub last_error: Option<String>,
}

/// One open stream. Dropping it stops the thread.
pub struct Connection {
    endpoint: Endpoint,
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

struct Shared {
    stop: AtomicBool,
    paused: AtomicBool,
    status: Mutex<ConnectionStatus>,
    table: Mutex<Option<StreamTable>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Opens the approved endpoint on a thread of its own and starts filling a stream table.
///
/// The only way to open a stream: `consent` comes from the user's explicit approval
/// ([`Consent::user_approved`]).
///
/// # Errors
///
/// [`StreamError::Unsupported`] when this build lacks the transport's feature;
/// [`StreamError::Table`] when the thread cannot start.
pub fn connect(consent: Consent, options: Options) -> Result<Connection, StreamError> {
    let endpoint = consent.into_endpoint();
    let source = transports::open(&endpoint)?;
    let shared = Arc::new(Shared {
        stop: AtomicBool::new(false),
        paused: AtomicBool::new(false),
        status: Mutex::new(ConnectionStatus {
            state: ConnectionState::Connecting,
            received: 0,
            invalid: 0,
            discarded: 0,
            failed_attempts: 0,
            last_error: None,
        }),
        table: Mutex::new(None),
    });
    let worker = Arc::clone(&shared);
    let name = endpoint.transport().to_ascii_lowercase();
    let thread = std::thread::Builder::new()
        .name(format!("sonarithm-stream-{name}"))
        .spawn(move || run(source, &options, &worker, &name))
        .map_err(|err| StreamError::Table(format!("the stream thread could not start: {err}")))?;
    Ok(Connection {
        endpoint,
        shared,
        thread: Some(thread),
    })
}

impl Connection {
    /// The endpoint.
    #[must_use]
    pub const fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// The current status.
    #[must_use]
    pub fn status(&self) -> ConnectionStatus {
        lock(&self.shared.status).clone()
    }

    /// Runs `read` on the stream table, once the first rows have arrived.
    pub fn read<R>(&self, read: impl FnOnce(&StreamTable) -> R) -> Option<R> {
        lock(&self.shared.table).as_ref().map(read)
    }

    /// Keeps the connection but discards messages until [`Connection::resume`] (A11Y-16).
    pub fn pause(&self) {
        self.shared.paused.store(true, Ordering::Release);
        let mut status = lock(&self.shared.status);
        if status.state == ConnectionState::Connected {
            status.state = ConnectionState::Paused;
        }
    }

    /// Takes messages again after a pause.
    pub fn resume(&self) {
        self.shared.paused.store(false, Ordering::Release);
        let mut status = lock(&self.shared.status);
        if status.state == ConnectionState::Paused {
            status.state = ConnectionState::Connected;
        }
    }

    /// Closes the connection and ends its thread; the rows received stay readable.
    pub fn stop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        lock(&self.shared.status).state = ConnectionState::Stopped;
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.stop();
    }
}

/// A transport's reading side, used only on the connection's thread.
pub(crate) trait Reader: Send {
    /// Opens or reopens the connection.
    fn open(&mut self) -> Result<(), String>;
    /// The messages that arrived, waiting about 100 ms at most; an error means the
    /// connection was lost.
    fn read(&mut self) -> Result<Vec<Map<String, Value>>, String>;
}

/// The thread: connect, read, append, reconnect, until stopped.
fn run(mut source: Box<dyn Reader>, options: &Options, shared: &Shared, name: &str) {
    let stopped = || shared.stop.load(Ordering::Acquire);
    let mut schema = options.schema.clone();
    let mut buffer = schema.clone().map(RowBuffer::new);
    let mut delay = options.backoff;
    while !stopped() {
        if let Err(error) = source.open() {
            let mut status = lock(&shared.status);
            status.failed_attempts = status.failed_attempts.saturating_add(1);
            status.last_error = Some(error);
            status.state = if status.received == 0 && status.failed_attempts == 1 {
                ConnectionState::Connecting
            } else {
                ConnectionState::Reconnecting
            };
            drop(status);
            sleep_unless_stopped(delay, shared);
            delay = (delay * 2).min(options.max_backoff);
            continue;
        }
        delay = options.backoff;
        {
            let mut status = lock(&shared.status);
            status.failed_attempts = 0;
            status.state = if shared.paused.load(Ordering::Acquire) {
                ConnectionState::Paused
            } else {
                ConnectionState::Connected
            };
        }
        let mut last_flush = Instant::now();
        while !stopped() {
            match source.read() {
                Ok(messages) => {
                    for message in messages {
                        take(&message, &mut schema, &mut buffer, shared);
                    }
                }
                Err(error) => {
                    let mut status = lock(&shared.status);
                    status.last_error = Some(error);
                    status.state = ConnectionState::Reconnecting;
                    break;
                }
            }
            let due = buffer.as_ref().is_some_and(|b| {
                b.len() >= options.flush_rows
                    || (!b.is_empty() && last_flush.elapsed() >= options.flush_every)
            });
            if due {
                flush(&mut buffer, options, shared, name);
                last_flush = Instant::now();
            }
        }
        flush(&mut buffer, options, shared, name);
    }
    flush(&mut buffer, options, shared, name);
}

/// Decodes one message into the buffer, creating the schema from it if there is none.
fn take(
    message: &Map<String, Value>,
    schema: &mut Option<MessageSchema>,
    buffer: &mut Option<RowBuffer>,
    shared: &Shared,
) {
    let mut status = lock(&shared.status);
    status.received += 1;
    if shared.paused.load(Ordering::Acquire) {
        status.discarded += 1;
        return;
    }
    if schema.is_none() {
        let Ok(inferred) = MessageSchema::infer(message) else {
            status.invalid += 1;
            return;
        };
        *buffer = Some(RowBuffer::new(inferred.clone()));
        *schema = Some(inferred);
    }
    let (Some(schema), Some(buffer)) = (schema.as_ref(), buffer.as_mut()) else {
        return;
    };
    match row(message, schema).and_then(|cells| buffer.push(cells)) {
        Ok(()) => {}
        Err(_) => status.invalid += 1,
    }
}

/// Appends the waiting rows to the table, creating it on the first append.
fn flush(buffer: &mut Option<RowBuffer>, options: &Options, shared: &Shared, name: &str) {
    let Some(buffer) = buffer.as_mut() else {
        return;
    };
    if buffer.is_empty() {
        return;
    }
    let Ok(batch) = buffer.take() else {
        return;
    };
    let mut table = lock(&shared.table);
    if table.is_none() {
        match StreamTable::new(
            TableId(0),
            name,
            buffer.schema().arrow(),
            options.capacity.clone(),
        ) {
            Ok(created) => *table = Some(created),
            Err(err) => {
                lock(&shared.status).last_error = Some(err.to_string());
                return;
            }
        }
    }
    if let Some(table) = table.as_mut()
        && let Err(err) = table.append(batch)
    {
        lock(&shared.status).last_error = Some(err.to_string());
    }
}

fn sleep_unless_stopped(total: Duration, shared: &Shared) {
    let step = Duration::from_millis(20);
    let start = Instant::now();
    while start.elapsed() < total && !shared.stop.load(Ordering::Acquire) {
        std::thread::sleep(step);
    }
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Test helpers shared by the unit and integration tests. Not part of the API.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// A fresh folder under the system temporary folder, removed on drop.
#[derive(Debug)]
pub struct TempDir(PathBuf);

impl TempDir {
    /// Creates `sonarithm-app-<label>-<process>-<n>`.
    ///
    /// # Panics
    ///
    /// If the folder cannot be created (tests only).
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "a test helper; failing loudly is the point"
    )]
    pub fn new(label: &str) -> Self {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("sonarithm-app-{label}-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("creating a temporary folder");
        Self(path)
    }

    /// The folder.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A synthetic live source (B06-T07): a thread that appends one JSON line per row to a
/// file at a fixed rate, `{"t": seconds, "v": value}`, for a `tail` stream to follow.
/// It stops when dropped.
#[derive(Debug)]
pub struct FileFeed {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    rows: std::sync::Arc<AtomicU64>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl FileFeed {
    /// Starts appending `rate` rows per second to `path` (created if missing), in a write
    /// every 10 ms. `t` counts seconds from the first row; `v` is a slow sine.
    ///
    /// # Panics
    ///
    /// If the file cannot be opened or the thread cannot start (tests only).
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "a test helper; failing loudly is the point"
    )]
    pub fn start(path: &Path, rate: f64) -> Self {
        use std::fmt::Write as _;
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("opening the feed file");
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let rows = std::sync::Arc::new(AtomicU64::new(0));
        let (stopping, written) = (stop.clone(), rows.clone());
        let thread = std::thread::Builder::new()
            .name("file-feed".to_owned())
            .spawn(move || {
                let start = std::time::Instant::now();
                let mut n = 0_u64;
                let mut text = String::new();
                while !stopping.load(Ordering::Acquire) {
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "row counts of a test feed"
                    )]
                    let due = (start.elapsed().as_secs_f64() * rate) as u64;
                    text.clear();
                    while n < due {
                        #[expect(clippy::cast_precision_loss, reason = "row counts of a test feed")]
                        let t = n as f64 / rate;
                        let v = (t * 0.5).sin();
                        let _ = writeln!(text, "{{\"t\": {t}, \"v\": {v}}}");
                        n += 1;
                    }
                    if !text.is_empty() && file.write_all(text.as_bytes()).is_ok() {
                        written.store(n, Ordering::Release);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            })
            .expect("starting the feed thread");
        Self {
            stop,
            rows,
            thread: Some(thread),
        }
    }

    /// Rows written so far.
    #[must_use]
    pub fn rows(&self) -> u64 {
        self.rows.load(Ordering::Acquire)
    }
}

impl Drop for FileFeed {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

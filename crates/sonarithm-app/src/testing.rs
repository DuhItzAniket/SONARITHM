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

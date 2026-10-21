// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The platform speech engine through the `tts` crate (feature `system`).

use crate::{Speaker, SpeechError};

/// The platform's speech engine: `WinRT` on Windows, `AVSpeechSynthesizer` on macOS,
/// speech-dispatcher on Linux. Create it on the thread that uses it.
pub struct System {
    tts: tts::Tts,
    reports_speaking: bool,
}

impl System {
    /// Opens the platform's default speech engine.
    ///
    /// # Errors
    ///
    /// [`SpeechError::Unavailable`] when the platform has no engine or it cannot start.
    pub fn new() -> Result<Self, SpeechError> {
        let tts = tts::Tts::default().map_err(|err| SpeechError::Unavailable(err.to_string()))?;
        let reports_speaking = tts.supported_features().is_speaking;
        Ok(Self {
            tts,
            reports_speaking,
        })
    }
}

impl std::fmt::Debug for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("System")
            .field("reports_speaking", &self.reports_speaking)
            .finish_non_exhaustive()
    }
}

impl Speaker for System {
    fn speak(&mut self, text: &str, interrupt: bool) -> Result<(), SpeechError> {
        self.tts
            .speak(text, interrupt)
            .map(|_| ())
            .map_err(|err| SpeechError::Failed(err.to_string()))
    }

    fn stop(&mut self) -> Result<(), SpeechError> {
        self.tts
            .stop()
            .map(|_| ())
            .map_err(|err| SpeechError::Failed(err.to_string()))
    }

    fn is_speaking(&self) -> Option<bool> {
        if self.reports_speaking {
            self.tts.is_speaking().ok()
        } else {
            None
        }
    }
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Optional self-voicing for SONARITHM (08 §4, A11Y-19; B03-T07).
//!
//! Speech goes through the listener's screen reader by default (A11Y-19). Self-voicing is
//! for listeners without one, or who prefer it: SONARITHM then speaks through the
//! platform's speech engine itself. Because it controls the timing, it keeps speech and
//! sonification sequenced and ducks sonification while it speaks (A11Y-20); the ducker is
//! a stage of the render graph (`sonarithm-render`, 06 §3).
//!
//! A [`Speaker`] is created on the thread that uses it, because platform engines are not
//! always `Send`. [`Recorder`] is a silent speaker for tests and headless runs; the
//! `system` feature adds `System`, the platform engine through the `tts` crate. Speech
//! never goes into an exported audio file (08 §4): exports carry a `WebVTT` sidecar instead.
//!
//! | Feature | Default | Speaker |
//! |---|---|---|
//! | (always) | — | [`Recorder`]: silent, records what it is asked to say |
//! | `system` | off | `System`: `WinRT` on Windows, `AVSpeechSynthesizer` on macOS, speech-dispatcher on Linux |

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::sync::{Arc, Mutex, PoisonError};

pub use sonarithm_core::{WordsPerMinute, spoken_duration};

#[cfg(feature = "system")]
mod system;
#[cfg(feature = "system")]
pub use system::System;

/// Something that speaks text aloud.
pub trait Speaker {
    /// Speaks `text`. With `interrupt`, what is being spoken stops first; otherwise the
    /// text follows it.
    ///
    /// # Errors
    ///
    /// [`SpeechError::Failed`] when the engine refuses the text.
    fn speak(&mut self, text: &str, interrupt: bool) -> Result<(), SpeechError>;

    /// Stops speaking and drops what is queued ("stop all audio", A11Y-15).
    ///
    /// # Errors
    ///
    /// [`SpeechError::Failed`] when the engine cannot stop.
    fn stop(&mut self) -> Result<(), SpeechError>;

    /// Whether it is speaking, when the engine can tell; `None` when it cannot (macOS
    /// reports no end of speech, keyboard-and-navigation §9.1), so the caller estimates
    /// with [`spoken_duration`].
    fn is_speaking(&self) -> Option<bool>;
}

/// Why speech failed.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SpeechError {
    /// No speech engine could be opened.
    #[error("no speech engine is available: {0}")]
    Unavailable(String),
    /// The engine failed a request.
    #[error("the speech engine failed: {0}")]
    Failed(String),
}

/// One request a [`Recorder`] received.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Utterance {
    /// The text.
    pub text: String,
    /// Whether it interrupted what was being spoken.
    pub interrupt: bool,
}

#[derive(Debug, Default)]
struct Log {
    said: Vec<Utterance>,
    stops: u32,
    speaking: Option<bool>,
}

/// A silent [`Speaker`] that records what it is asked to say. Clones share one log, so a
/// test keeps a clone while the app owns the speaker.
#[derive(Clone, Debug, Default)]
pub struct Recorder {
    log: Arc<Mutex<Log>>,
}

impl Recorder {
    /// An empty recorder that cannot tell whether it is speaking, like macOS.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn log(&self) -> std::sync::MutexGuard<'_, Log> {
        self.log.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Everything it was asked to say, in order.
    #[must_use]
    pub fn said(&self) -> Vec<Utterance> {
        self.log().said.clone()
    }

    /// How many times it was asked to stop.
    #[must_use]
    pub fn stops(&self) -> u32 {
        self.log().stops
    }

    /// Sets what [`Speaker::is_speaking`] reports.
    pub fn set_speaking(&self, speaking: Option<bool>) {
        self.log().speaking = speaking;
    }
}

impl Speaker for Recorder {
    fn speak(&mut self, text: &str, interrupt: bool) -> Result<(), SpeechError> {
        self.log().said.push(Utterance {
            text: text.to_owned(),
            interrupt,
        });
        Ok(())
    }

    fn stop(&mut self) -> Result<(), SpeechError> {
        self.log().stops += 1;
        Ok(())
    }

    fn is_speaking(&self) -> Option<bool> {
        self.log().speaking
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recorder_shares_its_log_with_its_clones() {
        let recorder = Recorder::new();
        let mut speaker: Box<dyn Speaker> = Box::new(recorder.clone());
        assert_eq!(speaker.speak("Paused, 12.5", true), Ok(()));
        assert_eq!(speaker.stop(), Ok(()));
        assert_eq!(speaker.is_speaking(), None);
        recorder.set_speaking(Some(true));
        assert_eq!(speaker.is_speaking(), Some(true));
        assert_eq!(
            recorder.said(),
            [Utterance {
                text: "Paused, 12.5".to_owned(),
                interrupt: true
            }]
        );
        assert_eq!(recorder.stops(), 1);
    }
}

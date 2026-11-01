// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Speech and sonification sequencing (08 §4, keyboard-and-navigation §9.2; B03-T07).
//!
//! AUDAXIS composes announcements and queues them by class ([`AnnouncementQueue`]); the
//! [`Sequencer`] decides when each one may be spoken, against the cue and the transport:
//!
//! 1. **Cue, then speech.** After a navigation step the point cue plays first; nothing is
//!    spoken before it ends (A11Y-20).
//! 2. **Key repeat.** While an arrow key auto-repeats, only cues play; the latest position
//!    is spoken when the key is released.
//! 3. **During playback** the cursor follows the playhead silently: navigation and earcon
//!    texts are logged, not spoken, and status and stream notices wait for the pause.
//!    Errors and alarms are spoken at once. On pause the current value is announced
//!    (A11Y-09; the app core composes it).
//! 4. **Responses during playback** (where am I, repeat) pause playback first, or, with
//!    "duck instead of pause", play on 20 dB down while they are spoken.
//! 5. **Speech over sonification** is ducked: whatever is spoken while sonification plays
//!    holds it [`DUCK_LEVEL`](sonarithm_render::DUCK_LEVEL) down until the speech ends
//!    (A11Y-20). Screen readers report no end of speech, so the end is estimated from the
//!    text ([`spoken_duration`]); a self-voicing engine that reports it extends the duck
//!    while it still speaks.
//!
//! Time is passed in, in milliseconds on any monotonic clock, so the sequencer is
//! deterministic and its rules are tested without a clock (ADR-0013).

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sonarithm_audaxis::announce::queue::STREAM_MAX_AGE_MS;
use sonarithm_audaxis::announce::{Announcement, AnnouncementQueue, Class, Delivery, Politeness};
use sonarithm_core::Seconds;
use sonarithm_speech::{Speaker, SpeechError, WordsPerMinute, spoken_duration};
use ts_rs::TS;

use crate::ipc::{AnnouncementClass, AnnouncementInfo};

/// Where announcements are spoken (A11Y-19).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum SpeechRoute {
    /// The listener's screen reader, through the host's announcer (the default).
    #[default]
    ScreenReader,
    /// SONARITHM's own voice, through the platform speech engine (optional self-voicing).
    SelfVoicing,
}

/// How the sequencer behaves; from the settings.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct SequencerConfig {
    /// Where announcements go.
    pub route: SpeechRoute,
    /// A response during playback ducks instead of pausing (keyboard §9.2 item 4).
    pub duck_instead_of_pause: bool,
    /// The speaking rate estimates assume.
    pub rate: WordsPerMinute,
}

impl Default for SequencerConfig {
    fn default() -> Self {
        Self {
            route: SpeechRoute::ScreenReader,
            duck_instead_of_pause: false,
            rate: WordsPerMinute::DEFAULT,
        }
    }
}

/// What the sequencer asks the control thread to do, in order.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Output {
    /// Pause playback before the next delivery.
    Pause,
    /// Engage (`true`) or release the ducker.
    Duck(bool),
    /// Speak this announcement now, through the route.
    Deliver(Delivery),
}

/// Sequences speech with sonification; see the module documentation.
#[derive(Debug)]
pub struct Sequencer {
    queue: AnnouncementQueue,
    config: SequencerConfig,
    /// Nothing but errors and alarms is spoken before this time: the end of the cue.
    cue_end_ms: u64,
    /// An arrow key auto-repeats.
    repeating: bool,
    /// Status and stream notices taken out of the queue during playback, with the time
    /// they were taken, to speak after it.
    held: Vec<(Delivery, u64)>,
    /// When the speech delivered so far is estimated to end.
    speech_end_ms: u64,
    ducked: bool,
    /// The last announcement spoken, for "repeat".
    last: Option<Announcement>,
}

impl Sequencer {
    /// An idle sequencer.
    #[must_use]
    pub fn new(config: SequencerConfig) -> Self {
        Self {
            queue: AnnouncementQueue::new(),
            config,
            cue_end_ms: 0,
            repeating: false,
            held: Vec::new(),
            speech_end_ms: 0,
            ducked: false,
            last: None,
        }
    }

    /// Its configuration.
    #[must_use]
    pub const fn config(&self) -> SequencerConfig {
        self.config
    }

    /// Changes its configuration.
    pub const fn set_config(&mut self, config: SequencerConfig) {
        self.config = config;
    }

    /// Queues `announcement`, composed at `now_ms`.
    pub fn push(&mut self, announcement: Announcement, now_ms: u64) {
        self.queue.push(announcement, now_ms);
    }

    /// An announcement while speech is off (B03-T09): it goes to the history and is
    /// returned for the log, never spoken. Errors and alarms are queued as usual, since
    /// they are never muted (INFER, OD-30); then `None`.
    pub fn push_muted(&mut self, announcement: Announcement, now_ms: u64) -> Option<Delivery> {
        if urgent(&announcement) {
            self.queue.push(announcement, now_ms);
            return None;
        }
        let mut quiet = announcement;
        quiet.speak = false;
        let seq = self.queue.push(quiet.clone(), now_ms);
        Some(Delivery {
            seq,
            announcement: quiet,
            interrupt: false,
        })
    }

    /// A point cue of `length` started at `now_ms`: speech waits for its end (§9.2 item 1).
    pub fn cue(&mut self, now_ms: u64, length: Seconds) {
        self.cue_end_ms = self.cue_end_ms.max(now_ms + millis(length));
    }

    /// An arrow key starts (`true`) or stops auto-repeating (§9.2 item 2).
    pub const fn key_repeat(&mut self, active: bool) {
        self.repeating = active;
    }

    /// Queues the last spoken announcement again as a response ("repeat", keyboard §5).
    /// Returns whether there was one.
    pub fn repeat(&mut self, now_ms: u64) -> bool {
        let Some(mut last) = self.last.clone() else {
            return false;
        };
        last.class = Class::Response;
        self.queue.push(last, now_ms);
        true
    }

    /// Drops everything pending and releases the duck ("stop all audio", A11Y-15).
    pub fn silence(&mut self) -> Vec<Output> {
        while self.queue.pop().is_some() {}
        self.held.clear();
        self.speech_end_ms = 0;
        self.cue_end_ms = 0;
        if self.ducked {
            self.ducked = false;
            return vec![Output::Duck(false)];
        }
        Vec::new()
    }

    /// The announcement history, for the log (A11Y-24).
    #[cfg(test)]
    pub(crate) fn history(
        &self,
    ) -> impl Iterator<Item = &sonarithm_audaxis::announce::HistoryEntry> {
        self.queue.history()
    }

    /// Whether the ducker is engaged.
    #[must_use]
    pub const fn ducked(&self) -> bool {
        self.ducked
    }

    /// One step at `now_ms`. `playing` is the transport; `speaking` is what a self-voicing
    /// engine reports (`None` when it cannot tell, or for the screen reader).
    pub fn poll(&mut self, now_ms: u64, playing: bool, speaking: Option<bool>) -> Vec<Output> {
        let mut out = Vec::new();
        self.queue.expire(now_ms);
        if self.ducked && speaking != Some(true) && now_ms >= self.speech_end_ms {
            self.ducked = false;
            out.push(Output::Duck(false));
        }
        let mut playing = playing;
        let urgent_waiting = self.queue.pending().any(urgent);
        let waiting_for_cue = now_ms < self.cue_end_ms;
        // While a cue plays or a key repeats, leave the queue alone so supersession keeps
        // only the latest; errors and alarms still go out.
        let holding = waiting_for_cue || (self.repeating && !playing);
        if holding && !urgent_waiting {
            return out;
        }
        let mut ready = Vec::new();
        while let Some(delivery) = self.queue.pop() {
            let announcement = &delivery.announcement;
            if urgent(announcement) {
                ready.push(delivery);
            } else if holding {
                // Taken out only to reach an error behind it.
                self.held.push((delivery, now_ms));
            } else if playing {
                match announcement.class {
                    Class::Response => {
                        if !self.config.duck_instead_of_pause {
                            out.push(Output::Pause);
                            playing = false;
                        }
                        ready.push(delivery);
                    }
                    // The cursor follows the playhead silently (§9.2 item 3): logged in the
                    // queue's history, not spoken.
                    Class::Navigation | Class::EarconText => {}
                    _ => self.held.push((delivery, now_ms)),
                }
            } else {
                ready.push(delivery);
            }
        }
        if !playing && !holding {
            let released = std::mem::take(&mut self.held);
            ready.extend(released.into_iter().filter_map(|(delivery, at)| {
                let stale = delivery.announcement.class == Class::Stream
                    && now_ms.saturating_sub(at) > STREAM_MAX_AGE_MS;
                (!stale).then_some(delivery)
            }));
        }
        // Only the newest navigation, response or earcon text survives (A11Y-17), also
        // across items held back and items still queued.
        let newest = ready
            .iter()
            .filter(|delivery| delivery.interrupt && !urgent(&delivery.announcement))
            .map(|delivery| delivery.seq)
            .max();
        ready.retain(|delivery| {
            !delivery.interrupt || urgent(&delivery.announcement) || Some(delivery.seq) == newest
        });
        // Highest priority first, then oldest (the queue's order across both sources).
        ready.sort_by_key(|delivery| {
            (
                std::cmp::Reverse(delivery.announcement.class.priority()),
                delivery.seq,
            )
        });
        for delivery in ready {
            self.deliver(delivery, now_ms, playing, &mut out);
        }
        out
    }

    fn deliver(&mut self, delivery: Delivery, now_ms: u64, playing: bool, out: &mut Vec<Output>) {
        let length = millis(spoken_duration(
            &delivery.announcement.speech,
            self.config.rate,
        ));
        let start = if delivery.interrupt {
            now_ms
        } else {
            self.speech_end_ms.max(now_ms)
        };
        self.speech_end_ms = start + length;
        if playing && !self.ducked {
            self.ducked = true;
            out.push(Output::Duck(true));
        }
        if delivery.announcement.class != Class::Stream {
            self.last = Some(delivery.announcement.clone());
        }
        out.push(Output::Deliver(delivery));
    }
}

/// Makes the self-voicing [`Speaker`] on the thread that uses it: platform speech engines
/// are not always `Send` (B03-T07). The host passes one in
/// [`AppConfig::with_speaker`](crate::AppConfig::with_speaker); the desktop app's builds
/// make a `sonarithm_speech::System`, tests a `Recorder`.
#[derive(Clone)]
pub struct SpeakerSource(Arc<MakeSpeaker>);

type MakeSpeaker = dyn Fn() -> Result<Box<dyn Speaker>, SpeechError> + Send + Sync;

impl SpeakerSource {
    /// A source that calls `make` once, when self-voicing first speaks.
    pub fn new(
        make: impl Fn() -> Result<Box<dyn Speaker>, SpeechError> + Send + Sync + 'static,
    ) -> Self {
        Self(Arc::new(make))
    }

    /// Makes a speaker.
    ///
    /// # Errors
    ///
    /// The engine's error when none can be opened.
    pub fn make(&self) -> Result<Box<dyn Speaker>, SpeechError> {
        (self.0)()
    }
}

impl std::fmt::Debug for SpeakerSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SpeakerSource")
    }
}

/// The protocol form of a delivery.
pub(crate) fn info(delivery: &Delivery, self_voiced: bool) -> AnnouncementInfo {
    let announcement = &delivery.announcement;
    AnnouncementInfo {
        seq: delivery.seq,
        speech: announcement.speech.clone(),
        display: announcement.display.clone(),
        class: match announcement.class {
            Class::Navigation => AnnouncementClass::Navigation,
            Class::Response => AnnouncementClass::Response,
            Class::Status => AnnouncementClass::Status,
            Class::Stream => AnnouncementClass::Stream,
            Class::EarconText => AnnouncementClass::EarconText,
        },
        assertive: announcement.politeness == Politeness::Assertive,
        interrupt: delivery.interrupt,
        self_voiced,
        node: announcement.node.map(|node| node.0),
        muted: false,
    }
}

/// Errors and alarms: spoken whatever the cue, the key or the transport (08 §4).
fn urgent(announcement: &Announcement) -> bool {
    announcement.error || announcement.politeness == Politeness::Assertive
}

fn millis(duration: Seconds) -> u64 {
    let ms = (duration.0 * 1_000.0).ceil();
    if ms.is_finite() && ms > 0.0 {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a positive, finite millisecond count"
        )]
        let ms = ms as u64;
        ms
    } else {
        0
    }
}

#[cfg(test)]
#[path = "speech/tests.rs"]
mod tests;

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Conformance tests for speech and sonification sequencing (A11Y-09, A11Y-20; keyboard
//! §9.2).

use sonarithm_audaxis::announce::{Announcement, Class};
use sonarithm_core::Seconds;
use sonarithm_speech::WordsPerMinute;

use super::{Output, Sequencer, SequencerConfig, SpeechRoute};

/// 60 words per minute: one word takes exactly one second.
fn config() -> SequencerConfig {
    SequencerConfig {
        rate: WordsPerMinute(60.0),
        ..SequencerConfig::default()
    }
}

fn of(class: Class, text: &str) -> Announcement {
    let mut announcement = Announcement::status(text, text);
    announcement.class = class;
    announcement
}

/// The texts delivered, in order.
fn spoken(outputs: &[Output]) -> Vec<String> {
    outputs
        .iter()
        .filter_map(|output| match output {
            Output::Deliver(delivery) => Some(delivery.announcement.speech.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a11y_20_the_announcement_follows_the_cue() {
    let mut sequencer = Sequencer::new(config());
    sequencer.cue(1_000, Seconds(0.15));
    sequencer.push(of(Class::Navigation, "12.5"), 1_000);
    assert_eq!(
        sequencer.poll(1_100, false, None),
        Vec::<Output>::new(),
        "the cue is still playing"
    );
    assert_eq!(spoken(&sequencer.poll(1_150, false, None)), ["12.5"]);
}

#[test]
fn a11y_20_errors_do_not_wait_for_the_cue() {
    let mut sequencer = Sequencer::new(config());
    sequencer.cue(0, Seconds(0.15));
    sequencer.push(of(Class::Navigation, "12.5"), 0);
    sequencer.push(Announcement::error("Device lost", "Device lost"), 0);
    assert_eq!(spoken(&sequencer.poll(10, false, None)), ["Device lost"]);
    assert_eq!(spoken(&sequencer.poll(150, false, None)), ["12.5"]);
}

#[test]
fn a11y_20_only_cues_play_while_a_key_repeats() {
    let mut sequencer = Sequencer::new(config());
    sequencer.key_repeat(true);
    for (at, value) in [(0, "1"), (50, "2"), (100, "3")] {
        sequencer.cue(at, Seconds(0.03));
        sequencer.push(of(Class::Navigation, value), at);
        assert_eq!(sequencer.poll(at + 40, false, None), Vec::<Output>::new());
    }
    sequencer.key_repeat(false);
    assert_eq!(
        spoken(&sequencer.poll(200, false, None)),
        ["3"],
        "the position on release"
    );
    let superseded = sequencer.history().filter(|entry| entry.superseded).count();
    assert_eq!(
        superseded, 2,
        "the steps in between are logged as superseded"
    );
}

#[test]
fn a11y_20_speech_waits_for_the_pause_except_errors_and_alarms() {
    let mut sequencer = Sequencer::new(config());
    sequencer.push(of(Class::Status, "Compiled"), 0);
    sequencer.push(of(Class::Navigation, "Landmark crossed"), 0);
    sequencer.push(of(Class::EarconText, "Boundary"), 0);
    sequencer.push(Announcement::error("Device lost", "Device lost"), 0);
    let outputs = sequencer.poll(10, true, None);
    assert_eq!(spoken(&outputs), ["Device lost"]);
    assert_eq!(
        outputs[0],
        Output::Duck(true),
        "spoken over playback: ducked"
    );
    assert_eq!(
        spoken(&sequencer.poll(20, true, None)),
        Vec::<String>::new()
    );
    assert_eq!(
        spoken(&sequencer.poll(30, false, None)),
        ["Compiled"],
        "status after the pause; the cursor followed the playhead silently"
    );
}

#[test]
fn a11y_20_a_response_during_playback_pauses_it_first() {
    let mut sequencer = Sequencer::new(config());
    sequencer.push(of(Class::Response, "Point 3 of 12"), 0);
    let outputs = sequencer.poll(10, true, None);
    assert_eq!(outputs.first(), Some(&Output::Pause));
    assert_eq!(spoken(&outputs), ["Point 3 of 12"]);
    assert!(
        !outputs.contains(&Output::Duck(true)),
        "paused, so nothing to duck"
    );
}

#[test]
fn a11y_20_duck_instead_of_pause_keeps_sonification_down_while_speaking() {
    let mut sequencer = Sequencer::new(SequencerConfig {
        duck_instead_of_pause: true,
        ..config()
    });
    // Four words at one word per second.
    sequencer.push(of(Class::Response, "Point 3 of 12"), 0);
    let outputs = sequencer.poll(0, true, None);
    assert_eq!(outputs.first(), Some(&Output::Duck(true)));
    assert!(!outputs.contains(&Output::Pause));
    assert!(sequencer.ducked());
    assert_eq!(
        sequencer.poll(3_999, true, None),
        Vec::<Output>::new(),
        "still speaking"
    );
    assert_eq!(sequencer.poll(4_000, true, None), [Output::Duck(false)]);
}

#[test]
fn a11y_20_self_voicing_holds_the_duck_while_the_engine_speaks() {
    let mut sequencer = Sequencer::new(SequencerConfig {
        route: SpeechRoute::SelfVoicing,
        ..config()
    });
    sequencer.push(Announcement::error("Alarm", "Alarm"), 0);
    assert_eq!(sequencer.poll(0, true, Some(true))[0], Output::Duck(true));
    assert_eq!(
        sequencer.poll(5_000, true, Some(true)),
        Vec::<Output>::new(),
        "the engine still speaks past the estimate"
    );
    assert_eq!(
        sequencer.poll(5_010, true, Some(false)),
        [Output::Duck(false)]
    );
}

#[test]
fn queued_speech_is_estimated_back_to_back() {
    let mut sequencer = Sequencer::new(SequencerConfig {
        duck_instead_of_pause: true,
        ..config()
    });
    sequencer.push(Announcement::error("One two", "One two"), 0);
    sequencer.push(Announcement::error("Three", "Three"), 0);
    let outputs = sequencer.poll(0, true, None);
    assert_eq!(spoken(&outputs), ["One two", "Three"]);
    assert_eq!(sequencer.poll(2_999, true, None), Vec::<Output>::new());
    assert_eq!(sequencer.poll(3_000, true, None), [Output::Duck(false)]);
}

#[test]
fn stale_stream_notices_are_dropped_after_playback() {
    let mut sequencer = Sequencer::new(config());
    sequencer.push(of(Class::Stream, "3 threshold events"), 0);
    assert_eq!(spoken(&sequencer.poll(0, true, None)), Vec::<String>::new());
    assert_eq!(
        spoken(&sequencer.poll(10_001, false, None)),
        Vec::<String>::new(),
        "older than 10 s"
    );
}

#[test]
fn repeat_speaks_the_last_announcement_again_as_a_response() {
    let mut sequencer = Sequencer::new(config());
    assert!(!sequencer.repeat(0));
    sequencer.push(of(Class::Navigation, "12.5"), 0);
    assert_eq!(spoken(&sequencer.poll(0, false, None)), ["12.5"]);
    assert!(sequencer.repeat(100));
    let outputs = sequencer.poll(100, true, None);
    assert_eq!(outputs.first(), Some(&Output::Pause));
    assert_eq!(spoken(&outputs), ["12.5"]);
}

#[test]
fn stop_all_silences_everything_and_releases_the_duck() {
    let mut sequencer = Sequencer::new(config());
    sequencer.push(Announcement::error("Alarm", "Alarm"), 0);
    sequencer.push(of(Class::Status, "Compiled"), 0);
    let _ = sequencer.poll(0, true, None);
    assert_eq!(sequencer.silence(), [Output::Duck(false)]);
    assert_eq!(
        spoken(&sequencer.poll(10, false, None)),
        Vec::<String>::new()
    );
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The point cue (B03-T09; sound-design §3.2; keyboard-and-navigation §9.2).
//!
//! The one cursor drives the cue as it drives the announcement and the highlight (A11Y-10):
//! a move onto a point, from the table, the plot or the explorer, plays that point's tone,
//! then the announcement follows it (A11Y-20; the sequencer waits for the cue's end).
//!
//! The cue is the point's own note in the compiled Score (its pitch, gain, pan and the
//! track's instrument), so a step and the overview sound alike (sound-design §3.2) and the
//! sound still comes from the Score (AGENTS rule 1). The transport plays it through the
//! render graph's allocation-free `Command::Cue`; nothing here runs on the audio thread.

use sonarithm_core::nav::{NavKind, NavNodeId, NavTree};
use sonarithm_core::{Db, Pan, Score, ScoreEvent, Seconds};
use sonarithm_render::Instrument;
use sonarithm_render::graph::Cue;

use super::App;

/// The point cue's length, ramps included: sound-design §3.2's proposal (120 ms), short
/// enough for the announcement to follow promptly. A proposal, not a rule.
const POINT_CUE: Seconds = Seconds(0.12);

/// How far a note's onset may sit from its point's time and still be its note.
const ONSET_TOLERANCE_S: f64 = 1e-9;

impl App {
    /// Plays the point cue of `node` if it is a point and sonification is on. Returns
    /// whether a cue went to the transport (which drops it during playback, keyboard §9.2
    /// item 3).
    pub(super) fn play_cue(&self, node: NavNodeId) -> bool {
        // Sonification off silences cues and playback together (sound-design §3.2).
        if !self.interaction().modalities[0] {
            return false;
        }
        let Some(compiled) = &self.compiled else {
            return false;
        };
        let Some(cue) = compiled
            .nav()
            .and_then(|tree| cue_of(compiled.score(), tree, node))
        else {
            return false;
        };
        self.transport.cue(cue);
        true
    }
}

/// The cue of a point: the note its row produced in its series' track. `None` for a node
/// that is not a point, or a point with no note (a missing value, a glide's breakpoint).
pub(super) fn cue_of(score: &Score, tree: &NavTree, node: NavNodeId) -> Option<Cue> {
    let NavKind::Point(point) = &tree.node(node)?.kind else {
        return None;
    };
    let t = point.t?.0;
    let mut at = tree.parent(node);
    let series = loop {
        let id = at?;
        if let NavKind::Series(series) = &tree.node(id)?.kind {
            break series;
        }
        at = tree.parent(id);
    };
    let track = score.tracks.iter().find(|track| track.id == series.track)?;
    // Events are sorted by start: find the first at the point's time, then its note.
    let first = track
        .events
        .partition_point(|event| event.start().0 < t - ONSET_TOLERANCE_S);
    let played = track.events[first..]
        .iter()
        .take_while(|event| event.start().0 <= t + ONSET_TOLERANCE_S)
        .find_map(|event| match event {
            ScoreEvent::Note(candidate) if candidate.src == Some(point.row) => Some(candidate),
            _ => None,
        })?;
    Some(Cue {
        frequency: played.pitch.to_hz(),
        gain: Db(track.gain.0 + played.gain.0),
        pan: Pan::clamped(track.pan.0 + played.pan.0),
        instrument: Instrument::from_name(&track.instrument.0).unwrap_or_default(),
        duration: POINT_CUE,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;
    use sonarithm_core::ScoreEvent;
    use sonarithm_core::nav::NavKind;

    use super::cue_of;
    use crate::testing::TempDir;
    use crate::{App, AppConfig, AudioOutput, CollectingSink};

    /// Sound-design §3.2: a point's cue is its own note, with the same pitch, level and
    /// stereo position as playback gives it.
    #[test]
    fn sound_design_3_2_the_point_cue_is_the_points_own_note() {
        let dir = TempDir::new("cue-of");
        let config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
        let mut app = App::new(config, Arc::new(CollectingSink::new())).unwrap();
        let csv = dir.path().join("w.csv");
        std::fs::write(&csv, "day,temp\n1,8.5\n2,9.25\n3,7.0\n").unwrap();
        let spec = json!({
            "specVersion": "0.1",
            "data": { "source": { "kind": "dataset", "id": "d1" } },
            "substrate": { "time": { "field": "day" } },
            "tracks": [{ "id": "t", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
        })
        .to_string();
        for request in [
            json!({ "command": "dataset_open", "args": { "path": csv } }),
            json!({ "command": "spec_set", "args": { "text": spec } }),
            json!({ "command": "compile" }),
        ] {
            app.dispatch_json(&request.to_string()).unwrap();
        }
        let compiled = app.compiled.as_ref().unwrap();
        let (score, tree) = (compiled.score(), compiled.nav().unwrap());
        let mut checked = 0;
        for (id, node) in tree.iter() {
            let NavKind::Point(point) = &node.kind else {
                assert!(cue_of(score, tree, id).is_none(), "only points have cues");
                continue;
            };
            let cue = cue_of(score, tree, id).unwrap();
            let track = &score.tracks[0];
            let note = track
                .events
                .iter()
                .find_map(|event| match event {
                    ScoreEvent::Note(note) if note.src == Some(point.row) => Some(note),
                    _ => None,
                })
                .unwrap();
            assert!((cue.frequency.0 - note.pitch.to_hz().0).abs() < 1e-9);
            assert!((cue.gain.0 - (track.gain.0 + note.gain.0)).abs() < 1e-9);
            assert!((cue.pan.0 - (track.pan.0 + note.pan.0).clamp(-1.0, 1.0)).abs() < 1e-9);
            assert!((cue.duration.0 - 0.12).abs() < 1e-12);
            checked += 1;
        }
        assert_eq!(checked, 3);
    }
}

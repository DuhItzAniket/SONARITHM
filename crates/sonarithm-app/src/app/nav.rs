// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The chart explorer's verbs on the one cursor (B03-T09; ADR-0017 "Navigation";
//! keyboard-and-navigation §4 and §6).
//!
//! `nav_move` and `nav_where` read and move the cursor of `cursor_set`, never a second one
//! (A11Y-10). The cursor model keeps AUDAXIS's cursor beside the row: a verb moves the
//! node, and the row follows it (a point's own row, or the series' point nearest the
//! cursor's x), so the table and the plot follow the explorer as the explorer follows them.
//! The outcome is announced on the `announcements` channel and nowhere else, so nothing is
//! spoken twice (ADR-0017 "Cursor", INFER, OD-30).
//!
//! A filter takes its series out of playback and view (keyboard §3, the Filters row): the
//! transport plays a copy of the compiled Score without that series' track. Exports still
//! render the whole Score, because the filter is session state, not part of the spec.

use std::collections::BTreeSet;
use std::sync::Arc;

use sonarithm_audaxis::announce::Composer;
use sonarithm_audaxis::nav::{Cursor, Edge, Invalid, Landmark, Outcome, Verb, XTarget};
use sonarithm_core::{Score, TrackId};
use sonarithm_core::nav::{NavKind, NavNodeId, NavTree};

use super::App;
use super::cursor::CursorAt;
use crate::codes;
use crate::ipc::{
    CursorOrigin, IpcError, NavLandmark, NavOutcome, NavResult, NavSpan, NavVerb, NavX, Reply,
};

/// What a verb changed, worked out while the tree is borrowed.
struct Step {
    outcome: Outcome,
    row: Option<CursorAt>,
    excluded_changed: bool,
    announcement: Option<sonarithm_audaxis::announce::Announcement>,
    highlight: Option<NavSpan>,
}

impl App {
    /// `nav_move`: applies an explorer verb to the one cursor, moves the row with it,
    /// announces the outcome and sends the `cursor` event with origin `nav`.
    ///
    /// # Errors
    ///
    /// `SNR-APP-090` when no compiled Score has a navigation tree.
    pub(super) fn nav_move(&mut self, verb: &NavVerb) -> Result<Reply, IpcError> {
        let verb = to_verb(verb);
        let settings = self.composer_settings();
        let tree = self
            .compiled
            .as_ref()
            .and_then(sonarithm::Compiled::nav)
            .ok_or_else(no_tree)?;
        let mut cursor = self.cursor.nav.take().unwrap_or_else(|| {
            let mut cursor = Cursor::new();
            cursor.set_excluded(self.cursor.excluded.iter().copied());
            cursor
        });
        let before: BTreeSet<NavNodeId> = cursor.excluded().collect();
        let outcome = cursor.apply(tree, &verb);
        let excluded: BTreeSet<NavNodeId> = cursor.excluded().collect();
        let row = row_after(tree, &cursor).and_then(|(table, row)| {
            self.datasets
                .values()
                .find(|open| open.bundle.primary().map(sonarithm_data::Table::id) == Some(table))
                .map(|open| CursorAt {
                    dataset: open.summary.id.clone(),
                    row,
                })
        });
        let mut memory = self.cursor.memory.clone();
        let announcement = self.localizer.as_ref().and_then(|localizer| {
            let mut composer = Composer::with_memory(localizer, settings, memory.clone());
            let announcement = composer.outcome(tree, &cursor, &outcome).ok();
            memory = composer.memory();
            announcement
        });
        let highlight = tree.node(cursor.node()).and_then(|node| node.span).map(|span| NavSpan {
            start: span.start,
            end: span.end,
        });
        let step = Step {
            outcome,
            row,
            excluded_changed: excluded != before,
            announcement,
            highlight,
        };
        self.cursor.memory = memory;
        self.cursor.excluded = excluded;
        self.cursor.nav = Some(cursor);
        self.finish_move(step)
    }

    /// Stores the row, reloads playback after a toggle, announces and replies.
    fn finish_move(&mut self, step: Step) -> Result<Reply, IpcError> {
        if let Some(at) = step.row {
            self.cursor.at = Some(at);
        }
        if step.excluded_changed {
            self.load_playback();
        }
        let announced = step.announcement.is_some();
        if let Some(announcement) = step.announcement {
            self.transport.announce(announcement);
        }
        let cursor = self.send_cursor(Some(CursorOrigin::Nav), true);
        let (outcome, edge, reason, included) = match step.outcome {
            Outcome::Moved(_) => (NavOutcome::Moved, None, None, None),
            Outcome::Boundary { edge, .. } => {
                (NavOutcome::Boundary, Some(edge_name(edge)), None, None)
            }
            Outcome::Invalid { reason, .. } => {
                (NavOutcome::Invalid, None, Some(reason_name(reason)), None)
            }
            Outcome::Describe { .. } => (NavOutcome::Describe, None, None, None),
            Outcome::Toggled { included, .. } => (NavOutcome::Toggled, None, None, Some(included)),
            _ => (NavOutcome::Invalid, None, Some("notHere".to_owned()), None),
        };
        Ok(Reply::Nav(NavResult {
            cursor,
            outcome,
            edge,
            reason,
            included,
            earcon: step.outcome.earcon().map(|earcon| earcon.id().to_owned()),
            highlight: step.highlight,
            announced,
        }))
    }

    /// `nav_where`: announces where the cursor is (keyboard §6, A11Y-07). The cursor does
    /// not move and no `cursor` event is sent.
    ///
    /// # Errors
    ///
    /// `SNR-APP-090` when no compiled Score has a navigation tree.
    pub(super) fn nav_where(&self) -> Result<Reply, IpcError> {
        let settings = self.composer_settings();
        let tree = self
            .compiled
            .as_ref()
            .and_then(sonarithm::Compiled::nav)
            .ok_or_else(no_tree)?;
        let report = match &self.cursor.nav {
            Some(cursor) => cursor.where_am_i(tree),
            None => {
                let mut cursor = Cursor::new();
                cursor.set_excluded(self.cursor.excluded.iter().copied());
                cursor.where_am_i(tree)
            }
        };
        let announcement = self.localizer.as_ref().and_then(|localizer| {
            Composer::with_memory(localizer, settings, self.cursor.memory.clone())
                .where_am_i(tree, &report)
                .ok()
        });
        let queued = announcement.is_some();
        if let Some(announcement) = announcement {
            self.transport.announce(announcement);
        }
        Ok(Reply::Announce { queued })
    }

    /// While a filter excludes a series, an export still renders the whole compiled Score;
    /// this says so when an export starts, until the export dialog (B05-T05) shows it.
    pub(super) fn announce_export_scope(&self) {
        if self.cursor.excluded.is_empty() {
            return;
        }
        let Some(text) = self.localizer.as_ref().and_then(|localizer| {
            localizer
                .format("announce-export-all-series", &sonarithm_i18n::Args::new())
                .ok()
        }) else {
            return;
        };
        self.transport.announce(
            sonarithm_audaxis::announce::Announcement::status(text.clone(), text),
        );
    }

    /// Loads the playback Score into the transport, with the tracks it leaves out.
    pub(super) fn load_playback(&self) {
        let excluded = self
            .excluded_tracks()
            .iter()
            .map(ToString::to_string)
            .collect();
        self.transport.load(self.playback_score(), excluded);
    }

    /// The compiled Score's tracks of the series the filters took out, in track order.
    fn excluded_tracks(&self) -> BTreeSet<TrackId> {
        let Some(tree) = self.compiled.as_ref().and_then(sonarithm::Compiled::nav) else {
            return BTreeSet::new();
        };
        self.cursor
            .excluded
            .iter()
            .filter_map(|&id| match tree.node(id).map(|node| &node.kind) {
                Some(NavKind::Series(series)) => Some(series.track.clone()),
                _ => None,
            })
            .collect()
    }

    /// The Score the transport plays: the compiled Score with the excluded series' tracks
    /// removed, and nothing else changed (the header, the markers and every other track are
    /// the compiled Score's own), so it is still the compiler's Score IR (ADR-0005).
    pub(super) fn playback_score(&self) -> Option<Arc<Score>> {
        let compiled = self.compiled.as_ref()?;
        let mut score = compiled.score().clone();
        let dropped = self.excluded_tracks();
        if !dropped.is_empty() {
            score.tracks.retain(|track| !dropped.contains(&track.id));
        }
        Some(Arc::new(score))
    }

    /// The labels of the excluded series, read before a compile replaces the tree.
    pub(super) fn excluded_labels(&self) -> BTreeSet<String> {
        let Some(tree) = self.compiled.as_ref().and_then(sonarithm::Compiled::nav) else {
            return BTreeSet::new();
        };
        self.cursor
            .excluded
            .iter()
            .filter_map(|&id| tree.node(id).and_then(|node| node.label.clone()))
            .collect()
    }

    /// After a compile: node IDs may have changed, so the explorer's node is found again
    /// from the row, the excluded series by their labels, and the composer forgets which
    /// series it named last.
    pub(super) fn renumber_nav(&mut self, labels: &BTreeSet<String>) {
        self.cursor.memory.clear();
        self.cursor.nav = None;
        let Some(tree) = self.compiled.as_ref().and_then(sonarithm::Compiled::nav) else {
            self.cursor.excluded.clear();
            return;
        };
        self.cursor.excluded = tree
            .series()
            .filter(|&id| {
                tree.node(id)
                    .and_then(|node| node.label.as_ref())
                    .is_some_and(|label| labels.contains(label))
            })
            .collect();
        self.place_nav();
    }

    /// Puts the explorer's node on the cursor row's point, keeping the excluded series;
    /// no node when the row is not a point.
    pub(super) fn place_nav(&mut self) {
        let point = self.row_point();
        let nav = match (self.compiled.as_ref().and_then(sonarithm::Compiled::nav), point) {
            (Some(tree), Some(point)) => {
                let mut cursor = self.cursor.nav.take().unwrap_or_default();
                cursor.set_excluded(self.cursor.excluded.iter().copied());
                cursor.place(tree, point).then_some(cursor)
            }
            _ => None,
        };
        self.cursor.nav = nav;
    }
}

/// The row a cursor's node stands for: a point's own row; for a series or a segment, the
/// series' point nearest the cursor's x. Other nodes (axes, insights, filters) leave the
/// row where it was.
fn row_after(tree: &NavTree, cursor: &Cursor) -> Option<(sonarithm_core::TableId, u64)> {
    let node = tree.node(cursor.node())?;
    let point = match &node.kind {
        NavKind::Point(_) => cursor.node(),
        NavKind::Series(_) | NavKind::Segment(_) => {
            let x = cursor.x()?;
            let series = cursor.series()?;
            tree.points(series).into_iter().min_by(|&a, &b| {
                let distance = |id: NavNodeId| {
                    tree.node(id)
                        .and_then(|n| n.span)
                        .map_or(f64::INFINITY, |span| (span.start - x).abs())
                };
                distance(a).total_cmp(&distance(b))
            })?
        }
        _ => return None,
    };
    match &tree.node(point)?.kind {
        NavKind::Point(p) => Some((p.row.table, p.row.row)),
        _ => None,
    }
}

fn to_verb(verb: &NavVerb) -> Verb {
    match verb {
        NavVerb::Next => Verb::Next,
        NavVerb::Previous => Verb::Previous,
        NavVerb::JumpForward => Verb::JumpForward,
        NavVerb::JumpBack => Verb::JumpBack,
        NavVerb::First => Verb::First,
        NavVerb::Last => Verb::Last,
        NavVerb::Into => Verb::Into,
        NavVerb::Out => Verb::Out,
        NavVerb::Activate => Verb::Activate,
        NavVerb::NextSeries => Verb::NextSeries,
        NavVerb::PreviousSeries => Verb::PreviousSeries,
        NavVerb::Maximum => Verb::Maximum,
        NavVerb::Minimum => Verb::Minimum,
        NavVerb::NextInsight => Verb::NextInsight,
        NavVerb::PreviousInsight => Verb::PreviousInsight,
        NavVerb::Landmark { landmark } => Verb::Landmark(match landmark {
            NavLandmark::XAxis => Landmark::XAxis,
            NavLandmark::YAxis => Landmark::YAxis,
            NavLandmark::Insights => Landmark::Insights,
            NavLandmark::Filters => Landmark::Filters,
            NavLandmark::Dataset => Landmark::Dataset,
        }),
        NavVerb::GoTo { x } => Verb::GoTo(match x {
            NavX::Number(value) => XTarget::Number(*value),
            NavX::Category(name) => XTarget::Category(name.clone()),
        }),
        NavVerb::FindValue { value } => Verb::FindValue(*value),
    }
}

fn edge_name(edge: Edge) -> String {
    match edge {
        Edge::First => "first",
        Edge::Last => "last",
        Edge::Top => "top",
        Edge::Bottom => "bottom",
    }
    .to_owned()
}

fn reason_name(reason: Invalid) -> String {
    match reason {
        Invalid::OneSeries => "oneSeries",
        Invalid::NotInSeries => "notInSeries",
        Invalid::NoValues => "noValues",
        Invalid::NoInsights => "noInsights",
        Invalid::NotFound => "notFound",
        Invalid::NoLandmark => "noLandmark",
        _ => "notHere",
    }
    .to_owned()
}

fn no_tree() -> IpcError {
    IpcError::new(
        &codes::NAV_NO_TREE,
        "There is nothing to explore yet. Compile the spec first.",
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use crate::testing::TempDir;
    use crate::{App, AppConfig, AudioOutput, CollectingSink};

    /// Keyboard section 3 (Filters): a filtered series leaves playback, while the compiled
    /// Score, which exports render, keeps every track.
    #[test]
    fn b03_t09_playback_leaves_out_an_excluded_series_and_the_export_score_does_not() {
        let dir = TempDir::new("nav-playback");
        let config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
        let mut app = App::new(config, Arc::new(CollectingSink::new())).unwrap();
        let csv = dir.path().join("w.csv");
        std::fs::write(&csv, "day,temp,rain\n1,8.5,3\n2,9.25,0\n").unwrap();
        let spec = json!({
            "specVersion": "0.1",
            "data": { "source": { "kind": "dataset", "id": "d1" } },
            "substrate": { "time": { "field": "day" } },
            "tracks": [
                { "id": "temp", "mark": "tone", "channels": { "pitch": { "field": "temp" } } },
                { "id": "rain", "mark": "tone", "channels": { "pitch": { "field": "rain" } } }
            ]
        })
        .to_string();
        for request in [
            json!({ "command": "dataset_open", "args": { "path": csv } }),
            json!({ "command": "spec_set", "args": { "text": spec } }),
            json!({ "command": "compile" }),
            json!({ "command": "nav_move", "args": { "verb": { "verb": "landmark", "landmark": "filters" } } }),
            json!({ "command": "nav_move", "args": { "verb": { "verb": "into" } } }),
        ] {
            app.dispatch_json(&request.to_string()).unwrap();
        }
        let canonical = app.compiled.as_ref().unwrap().score().clone();
        assert_eq!(
            app.playback_score().unwrap().canonical_json().unwrap(),
            canonical.canonical_json().unwrap(),
            "nothing excluded: playback is the compiled Score, byte for byte"
        );
        let toggled = app
            .dispatch_json(&json!({ "command": "nav_move", "args": { "verb": { "verb": "activate" } } }).to_string())
            .unwrap();
        assert_eq!(toggled["included"], false);
        let playing = app.playback_score().unwrap();
        // Only one track is gone: the excluded series' own.
        let gone: Vec<&str> = canonical
            .tracks
            .iter()
            .filter(|track| !playing.tracks.iter().any(|p| p.id == track.id))
            .map(|track| track.id.as_str())
            .collect();
        assert_eq!(gone.len(), 1, "{gone:?}");
        assert_eq!(playing.tracks.len(), canonical.tracks.len() - 1);
        // Everything else is the compiled Score's own, bit for bit and in order.
        let kept = canonical.tracks.iter().filter(|track| track.id.as_str() != gone[0]);
        for (playing, compiled) in playing.tracks.iter().zip(kept) {
            assert_eq!(bytes(playing), bytes(compiled), "track {}", compiled.id);
        }
        assert_eq!(bytes(&playing.header), bytes(&canonical.header));
        assert_eq!(bytes(&playing.markers), bytes(&canonical.markers));
        // The compiled Score, which exports render, keeps every track.
        assert_eq!(
            app.compiled.as_ref().unwrap().score().canonical_json().unwrap(),
            canonical.canonical_json().unwrap()
        );
        // The transport says what it leaves out, so what plays is not taken for the Score.
        let transport = app
            .dispatch_json(&json!({ "command": "transport_state" }).to_string())
            .unwrap();
        assert_eq!(transport["excludedTracks"], json!([gone[0]]));
    }

    /// Serialized bytes, for a bit-for-bit comparison (serde_json writes every f64 so that
    /// it reads back exactly).
    fn bytes<T: serde::Serialize>(value: &T) -> Vec<u8> {
        serde_json::to_vec(value).unwrap()
    }
}

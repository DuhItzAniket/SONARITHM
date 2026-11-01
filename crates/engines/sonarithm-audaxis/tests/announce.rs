// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! B03-T03: the announcement composer and queue (keyboard-and-navigation §8, §9.1);
//! A11Y-02, A11Y-12, A11Y-17, A11Y-24, FR-044, FR-051 and AGENTS rule 7.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

mod support;

use std::fmt::Write as _;

use proptest::prelude::*;
use sonarithm_audaxis::announce::{
    Announcement, AnnouncementQueue, Class, Composer, Part, Settings, Verbosity,
};
use sonarithm_audaxis::nav::{Cursor, Landmark, Outcome, Verb, XTarget};
use sonarithm_core::nav::{NavKind, NavNodeId, NavTree};
use sonarithm_core::provenance::{Confidence, Provenance, ProvenanceClass};
use sonarithm_i18n::{Localizer, PSEUDO_LOCALE, check_pseudo};
use support::weather;

fn english() -> Localizer {
    Localizer::new("en-US").unwrap()
}

/// A walk through the weather chart: into a point, along it, to an insight, past the end,
/// across series, onto a missing value, to the landmarks and filters.
fn walk() -> Vec<(&'static str, Verb)> {
    vec![
        ("Down", Verb::Into),
        ("Down", Verb::Into),
        ("Down", Verb::Into),
        ("Right", Verb::Next),
        ("N", Verb::NextInsight),
        ("End", Verb::Last),
        ("Right", Verb::Next),
        ("PageDown", Verb::NextSeries),
        ("G 5", Verb::GoTo(XTarget::Number(5.0))),
        ("Left", Verb::Previous),
        ("PageUp", Verb::PreviousSeries),
        ("PageUp", Verb::PreviousSeries),
        ("Up", Verb::Out),
        ("Up", Verb::Out),
        ("Enter", Verb::Activate),
        ("X", Verb::Landmark(Landmark::XAxis)),
        ("Right", Verb::Next),
        ("I", Verb::Landmark(Landmark::Insights)),
        ("Down", Verb::Into),
        ("Down", Verb::Into),
        ("Enter", Verb::Activate),
        ("F", Verb::Landmark(Landmark::Filters)),
        ("Down", Verb::Into),
        ("Enter", Verb::Activate),
        ("D", Verb::Landmark(Landmark::Dataset)),
    ]
}

/// Runs the walk and returns every announcement with its key.
fn announcements(localizer: &Localizer, settings: Settings) -> Vec<(&'static str, Announcement)> {
    let tree = weather();
    let mut cursor = Cursor::new();
    let mut composer = Composer::new(localizer, settings);
    let mut out = vec![("(start)", composer.node(&tree, cursor.node()).unwrap())];
    for (key, verb) in walk() {
        let outcome = cursor.apply(&tree, &verb);
        out.push((key, composer.outcome(&tree, &cursor, &outcome).unwrap()));
        if key == "G 5" {
            let report = cursor.where_am_i(&tree);
            out.push(("W", composer.where_am_i(&tree, &report).unwrap()));
        }
    }
    let report = cursor.where_am_i(&tree);
    out.push(("W", composer.where_am_i(&tree, &report).unwrap()));
    let point = tree.points(tree.series().next().unwrap())[7];
    out.push(("(pause)", composer.paused(&tree, point).unwrap()));
    out.push(("(describe)", composer.describe(&tree, point).unwrap()));
    out
}

fn transcript(announcements: &[(&str, Announcement)]) -> String {
    let mut text = String::new();
    for (key, a) in announcements {
        let earcon = a.earcon.map_or_else(String::new, |e| format!(" [{e}]"));
        let log = if a.speak { "" } else { " (log only)" };
        let _ = writeln!(text, "{key:<10} {:?}{earcon}{log}", a.class);
        let _ = writeln!(text, "  speech:  {}", a.speech);
        let _ = writeln!(text, "  display: {}", a.display);
    }
    text
}

/// B03-T03 acceptance: announcement snapshots at terse and verbose.
#[test]
fn a11y_12_announcement_snapshots_at_terse_and_verbose() {
    let localizer = english();
    for (name, verbosity) in [("terse", Verbosity::Terse), ("verbose", Verbosity::Verbose)] {
        let settings = Settings {
            verbosity,
            ..Settings::default()
        };
        insta::assert_snapshot!(name, transcript(&announcements(&localizer, settings)));
    }
}

/// A11Y-12: the part order is the listener's.
#[test]
fn a11y_12_part_order_is_configurable() {
    let tree = weather();
    let point = tree.points(tree.series().next().unwrap())[3];
    let localizer = english();
    let compose = |order: Vec<Part>| {
        let settings = Settings {
            terse: order,
            ..Settings::default()
        };
        Composer::new(&localizer, settings)
            .node(&tree, point)
            .unwrap()
            .speech
    };
    assert_eq!(compose(vec![Part::Value, Part::X]), "19 degrees C. 3");
    assert_eq!(compose(vec![Part::X, Part::Value]), "3. 19 degrees C");
}

/// Keyboard §8.1: speech uses unit names, the display text keeps symbols, with the same
/// value and precision (DL-P12).
#[test]
fn a11y_12_units_are_spoken_in_full_at_verbose_and_shown_as_symbols() {
    let tree = weather();
    let point = tree.points(tree.series().next().unwrap())[3];
    let localizer = english();
    let verbose = Settings {
        verbosity: Verbosity::Verbose,
        ..Settings::default()
    };
    let a = Composer::new(&localizer, verbose)
        .node(&tree, point)
        .unwrap();
    assert!(a.speech.contains("19 degrees Celsius"), "{}", a.speech);
    assert!(a.display.contains("19 °C"), "{}", a.display);
}

/// Keyboard §8.1: the series name is left out when it has not changed.
#[test]
fn the_series_name_is_left_out_when_unchanged() {
    let tree = weather();
    let points = tree.points(tree.series().next().unwrap());
    let localizer = english();
    let verbose = Settings {
        verbosity: Verbosity::Verbose,
        ..Settings::default()
    };
    let mut composer = Composer::new(&localizer, verbose);
    let first = composer.node(&tree, points[0]).unwrap();
    let second = composer.node(&tree, points[1]).unwrap();
    assert!(first.speech.starts_with("Temperature."), "{}", first.speech);
    assert!(!second.speech.contains("Temperature"), "{}", second.speech);
}

/// Keyboard §8.1 across commands: a host that builds a composer per command (the app's
/// `nav_move`, B03-T09) keeps the series left out by carrying the composer's memory.
#[test]
fn the_series_name_stays_left_out_across_composers_with_memory() {
    let tree = weather();
    let points = tree.points(tree.series().next().unwrap());
    let localizer = english();
    let verbose = Settings {
        verbosity: Verbosity::Verbose,
        ..Settings::default()
    };
    let mut first = Composer::new(&localizer, verbose.clone());
    assert!(
        first
            .node(&tree, points[0])
            .unwrap()
            .speech
            .starts_with("Temperature.")
    );
    let mut memory = first.memory();
    let second = Composer::with_memory(&localizer, verbose.clone(), memory.clone())
        .node(&tree, points[1])
        .unwrap();
    assert!(!second.speech.contains("Temperature"), "{}", second.speech);
    memory.clear();
    let fresh = Composer::with_memory(&localizer, verbose, memory)
        .node(&tree, points[1])
        .unwrap();
    assert!(fresh.speech.starts_with("Temperature."), "{}", fresh.speech);
}

fn with_provenance(tree: &NavTree, point: NavNodeId, provenance: Provenance) -> NavTree {
    let mut tree = tree.clone();
    if let Some(NavKind::Point(p)) = tree.node_mut(point).map(|n| &mut n.kind) {
        p.provenance = provenance;
    }
    tree
}

/// AGENTS rule 7, 03 §4 rule 3: every value that is not stated plainly carries its
/// qualifier, at both verbosities and in every part order, even one without the part.
#[test]
fn a11y_02_every_estimate_carries_its_qualifier() {
    let base = weather();
    let point = base.points(base.series().next().unwrap())[3];
    let localizer = english();
    let classes = [
        (ProvenanceClass::Derived, "derived"),
        (ProvenanceClass::Synthetic, "synthetic"),
        (ProvenanceClass::CvEstimate, "estimated"),
        (ProvenanceClass::MlEstimate, "estimated"),
        (ProvenanceClass::CloudEstimate, "estimated"),
    ];
    let orders = [
        Settings::default().terse,
        vec![Part::Value],
        vec![Part::Provenance, Part::Value, Part::X],
    ];
    for (class, word) in classes {
        let provenance = Provenance {
            class,
            confidence: Confidence::new(0.82),
        };
        let tree = with_provenance(&base, point, provenance);
        for verbosity in [Verbosity::Terse, Verbosity::Verbose] {
            for order in &orders {
                let settings = Settings {
                    verbosity,
                    terse: order.clone(),
                    verbose: order.clone(),
                    ..Settings::default()
                };
                let a = Composer::new(&localizer, settings)
                    .node(&tree, point)
                    .unwrap();
                assert!(
                    a.speech.contains(word),
                    "{class:?} {verbosity:?} {order:?}: {}",
                    a.speech
                );
                assert!(a.display.contains(word), "{class:?}: {}", a.display);
                if verbosity == Verbosity::Verbose {
                    assert!(a.speech.contains("confidence 0.82"), "{}", a.speech);
                }
            }
        }
    }
    for class in [
        ProvenanceClass::Exact,
        ProvenanceClass::Vector,
        ProvenanceClass::UserEdited,
    ] {
        let tree = with_provenance(
            &base,
            point,
            Provenance {
                class,
                confidence: None,
            },
        );
        let a = Composer::new(&localizer, Settings::default())
            .node(&tree, point)
            .unwrap();
        assert!(
            !a.speech.contains("estimated") && !a.speech.contains("derived"),
            "{}",
            a.speech
        );
    }
}

/// NFR-09: every composed announcement passes the pseudo-locale check, so none is built
/// by concatenation outside the catalogue.
#[test]
fn nfr_09_every_announcement_passes_the_pseudo_locale_check() {
    let pseudo = Localizer::new(PSEUDO_LOCALE).unwrap();
    for verbosity in [Verbosity::Terse, Verbosity::Verbose] {
        let settings = Settings {
            verbosity,
            ..Settings::default()
        };
        for (key, a) in announcements(&pseudo, settings) {
            check_pseudo(&a.speech).unwrap_or_else(|e| panic!("{key} speech: {e}"));
            check_pseudo(&a.display).unwrap_or_else(|e| panic!("{key} display: {e}"));
        }
    }
}

/// Keyboard §9.1: earcon text is spoken in verbose, and only logged in terse.
#[test]
fn earcon_text_is_logged_in_terse_and_spoken_in_verbose() {
    let tree = weather();
    let localizer = english();
    for (verbosity, spoken) in [(Verbosity::Terse, false), (Verbosity::Verbose, true)] {
        let mut composer = Composer::new(
            &localizer,
            Settings {
                verbosity,
                ..Settings::default()
            },
        );
        let boundary = Cursor::new().apply(&tree, &Verb::Out);
        let a = composer.outcome(&tree, &Cursor::new(), &boundary).unwrap();
        assert_eq!(a.class, Class::EarconText);
        assert_eq!(a.speak, spoken);
        let mut queue = AnnouncementQueue::new();
        queue.push(a, 0);
        assert_eq!(queue.history().count(), 1, "logged either way (A11Y-24)");
        assert_eq!(queue.pending().count(), usize::from(spoken));
    }
}

fn navigation(text: &str) -> Announcement {
    let mut a = Announcement::status(text, text);
    a.class = Class::Navigation;
    a
}

#[derive(Clone, Debug)]
enum Event {
    Navigation,
    Response,
    Status,
    Error,
    Stream,
    Pop,
    Expire(u64),
}

fn event() -> impl Strategy<Value = Event> {
    prop_oneof![
        4 => Just(Event::Navigation),
        1 => Just(Event::Response),
        1 => Just(Event::Status),
        1 => Just(Event::Error),
        1 => Just(Event::Stream),
        2 => Just(Event::Pop),
        1 => (0_u64..30_000).prop_map(Event::Expire),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// A11Y-17: only the latest navigation announcement survives; errors are never dropped;
    /// every announcement reaches the history (A11Y-24).
    #[test]
    fn a11y_17_only_the_latest_navigation_survives_and_errors_are_never_dropped(
        events in prop::collection::vec(event(), 1..80),
    ) {
        let mut queue = AnnouncementQueue::new();
        let mut errors = Vec::new();
        let mut delivered = Vec::new();
        let mut latest_command = None;
        let mut pushed = 0;
        for (t, event) in events.iter().enumerate() {
            let at = t as u64 * 1_000;
            let text = format!("#{t}");
            let announcement = match event {
                Event::Navigation => navigation(&text),
                Event::Response => {
                    let mut a = navigation(&text);
                    a.class = Class::Response;
                    a
                }
                Event::Status => Announcement::status(&text, &text),
                Event::Error => Announcement::error(&text, &text),
                Event::Stream => {
                    let mut a = Announcement::status(&text, &text);
                    a.class = Class::Stream;
                    a
                }
                Event::Pop => {
                    if let Some(d) = queue.pop() {
                        delivered.push(d.announcement.speech);
                    }
                    continue;
                }
                Event::Expire(now) => {
                    queue.expire(*now);
                    continue;
                }
            };
            if matches!(event, Event::Navigation | Event::Response) {
                latest_command = Some(text.clone());
            }
            if matches!(event, Event::Error) {
                errors.push(text.clone());
            }
            queue.push(announcement, at);
            pushed += 1;
        }
        let commands: Vec<&Announcement> = queue
            .pending()
            .filter(|a| matches!(a.class, Class::Navigation | Class::Response))
            .collect();
        prop_assert!(commands.len() <= 1);
        if let Some(a) = commands.first() {
            prop_assert_eq!(Some(&a.speech), latest_command.as_ref());
        }
        let pending: Vec<&String> = queue.pending().map(|a| &a.speech).collect();
        for error in &errors {
            prop_assert!(pending.contains(&error) || delivered.contains(error), "{error} dropped");
        }
        prop_assert_eq!(queue.history().count(), pushed);
    }
}

/// Status announcements wait behind navigation but are not superseded by it.
#[test]
fn status_waits_and_navigation_goes_first() {
    let mut queue = AnnouncementQueue::new();
    queue.push(
        Announcement::status("Export finished", "Export finished"),
        0,
    );
    queue.push(navigation("a"), 1);
    queue.push(navigation("b"), 2);
    let first = queue.pop().unwrap();
    assert_eq!(first.announcement.speech, "b");
    assert!(first.interrupt);
    assert_eq!(queue.pop().unwrap().announcement.speech, "Export finished");
    assert!(queue.pop().is_none());
    let superseded: Vec<bool> = queue.history().map(|e| e.superseded).collect();
    assert_eq!(superseded, [false, true, false]);
}

/// The history links each announcement to its node.
#[test]
fn a11y_24_history_entries_link_to_their_node() {
    let tree = weather();
    let localizer = english();
    let mut composer = Composer::new(&localizer, Settings::default());
    let mut cursor = Cursor::new();
    let mut queue = AnnouncementQueue::new();
    for (t, verb) in [Verb::Into, Verb::Into, Verb::Into, Verb::Next]
        .iter()
        .enumerate()
    {
        let outcome = cursor.apply(&tree, verb);
        if let Outcome::Moved(m) = outcome {
            let a = composer.outcome(&tree, &cursor, &outcome).unwrap();
            queue.push(a, t as u64);
            assert_eq!(queue.history().last().unwrap().node, Some(m.to));
        }
    }
}

/// The first series of the weather tree with its points estimated from a chart image:
/// `estimated` points from the start, confidence 0.9 except a dip to 0.3 at points 8
/// to 10.
fn estimated_series(estimated: usize) -> (NavTree, NavNodeId) {
    let base = weather();
    let series = base.series().next().unwrap();
    let mut tree = base.clone();
    for (i, point) in base.points(series).iter().enumerate().take(estimated) {
        let confidence = if (8..11).contains(&i) { 0.3 } else { 0.9 };
        tree = with_provenance(
            &tree,
            *point,
            Provenance {
                class: ProvenanceClass::CvEstimate,
                confidence: Confidence::new(confidence),
            },
        );
    }
    (tree, series)
}

fn speak(tree: &NavTree, node: NavNodeId, verbosity: Verbosity) -> String {
    let localizer = english();
    let settings = Settings {
        verbosity,
        ..Settings::default()
    };
    Composer::new(&localizer, settings)
        .node(tree, node)
        .unwrap()
        .speech
}

/// C02-T09, 03 §4 rule 3: a series or segment of estimates says so at both verbosities,
/// and the verbose series names where it is least certain, relative to the series (OD-07).
#[test]
fn a11y_02_series_and_segments_of_estimates_say_so() {
    let (tree, series) = estimated_series(usize::MAX);
    let total = tree.points(series).len();
    let terse = speak(&tree, series, Verbosity::Terse);
    assert!(
        terse.contains("estimated") && !terse.contains("partly"),
        "{terse}"
    );
    let verbose = speak(&tree, series, Verbosity::Verbose);
    assert!(
        verbose.contains(&format!(
            "{total} of {total} points estimated from a chart image, median confidence 0.90"
        )),
        "{verbose}"
    );
    assert!(
        verbose.contains("lowest confidence around") && verbose.contains("median 0.30"),
        "{verbose}"
    );
    // Some points estimated: "partly"; the series' other points are stated plainly.
    let (partly, series) = estimated_series(10);
    assert!(speak(&partly, series, Verbosity::Terse).contains("partly estimated"));
    let verbose = speak(&partly, series, Verbosity::Verbose);
    assert!(
        verbose.contains(&format!("10 of {total} points estimated")),
        "{verbose}"
    );
    // Segments carry the mix but not the stretch.
    if let Some(segment) = tree
        .children(series)
        .iter()
        .copied()
        .find(|id| matches!(tree.node(*id).map(|n| &n.kind), Some(NavKind::Segment(_))))
    {
        let verbose = speak(&tree, segment, Verbosity::Verbose);
        assert!(
            verbose.contains("points estimated from a chart image"),
            "{verbose}"
        );
        assert!(!verbose.contains("lowest confidence"), "{verbose}");
    }
    // A series with no estimates says nothing of the kind.
    let plain = weather();
    let first = plain.series().next().unwrap();
    assert!(!speak(&plain, first, Verbosity::Verbose).contains("estimated"));
}

/// C02-T09: the estimated series, a segment of it and one estimated point, at terse and
/// verbose, all estimated and partly estimated, as the listener hears them.
#[test]
fn estimates_at_terse_and_verbose() {
    let mut text = String::new();
    for (label, estimated) in [("all estimated", usize::MAX), ("first 10 estimated", 10)] {
        let (tree, series) = estimated_series(estimated);
        let segment = tree
            .children(series)
            .iter()
            .copied()
            .find(|id| matches!(tree.node(*id).map(|n| &n.kind), Some(NavKind::Segment(_))));
        let point = tree.points(series)[9];
        for verbosity in [Verbosity::Terse, Verbosity::Verbose] {
            let _ = writeln!(text, "## {label}, {verbosity:?}");
            let _ = writeln!(text, "series: {}", speak(&tree, series, verbosity));
            if let Some(segment) = segment {
                let _ = writeln!(text, "segment: {}", speak(&tree, segment, verbosity));
            }
            let _ = writeln!(text, "point 9: {}", speak(&tree, point, verbosity));
        }
    }
    insta::assert_snapshot!(text);
}

/// Keyboard §7 (B03-T09): the query menu answers about the cursor's series, as a response,
/// in both renderings, and asks nothing of the cursor itself.
#[test]
fn keyboard_7_queries_answer_about_the_cursors_series() {
    use sonarithm_audaxis::announce::Query;
    let tree = weather();
    let series = tree.series().next().unwrap();
    let cursor = Cursor::at(&tree, tree.points(series)[2]).unwrap();
    let before = cursor.clone();
    let localizer = english();
    let mut composer = Composer::new(&localizer, Settings::default());
    let ask = |composer: &mut Composer<'_>, query: Query| {
        composer.query(&tree, &cursor, &query).unwrap()
    };
    let max = ask(&mut composer, Query::Maximum);
    assert_eq!(max.class, Class::Response);
    assert!(max.speech.starts_with("Maximum of Temperature: "), "{}", max.speech);
    assert!(max.speech.contains("degrees Celsius"), "{}", max.speech);
    assert!(max.display.contains("°C"), "{}", max.display);
    let min = ask(&mut composer, Query::Minimum);
    assert!(min.speech.starts_with("Minimum of Temperature: "), "{}", min.speech);
    assert_ne!(min.node, max.node);
    let range = ask(&mut composer, Query::Range);
    assert!(range.speech.starts_with("Range of Temperature: "), "{}", range.speech);
    assert!(range.speech.contains(", span "), "{}", range.speech);
    let at = ask(&mut composer, Query::ValueAt(XTarget::Number(5.0)));
    assert!(at.speech.starts_with("At "), "{}", at.speech);
    let summary = ask(&mut composer, Query::Summary);
    assert!(summary.speech.contains("Temperature"), "{}", summary.speech);
    assert_eq!(cursor, before, "a query never moves the cursor");
}

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

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Conformance tests for SDR-12..15 and the concept table's evidence rules (A06-T03).

use proptest::prelude::*;
use sonarithm_core::diagnostic::Severity;

use super::*;
use crate::pitch::{PitchMap, PitchSpace, PitchValue};

fn table() -> &'static ConceptTable {
    ConceptTable::builtin().unwrap()
}

fn at() -> SpecPointer {
    SpecPointer::root()
        .key("tracks")
        .index(0)
        .key("channels")
        .key("pitch")
}

fn auto(concept: Option<(&str, ConceptOrigin)>) -> (ResolvedPolarity, Vec<Diagnostic>) {
    let request = PolarityRequest {
        field: "value",
        concept,
        choice: PolarityChoice::Auto,
    };
    resolve(table(), &PolarityProfile::default(), &request, &at())
}

fn polarity_of(concept: &str) -> Polarity {
    auto(Some((concept, ConceptOrigin::Spec))).0.polarity
}

fn codes_of(diagnostics: &[Diagnostic]) -> Vec<&str> {
    diagnostics.iter().map(|d| d.code.as_str()).collect()
}

/// A06-T03: every row of the shipped table cites its evidence and names its rules, and
/// the table parses under its own checks.
#[test]
fn every_concept_row_cites_evidence() {
    let table = table();
    assert!(table.rows().count() >= 15);
    for row in table.rows() {
        assert!(row.evidence.len() > 20, "{}", row.name);
        assert!(!row.rules.is_empty(), "{}", row.name);
        if row.votes.is_some() {
            assert!(row.evidence.contains("Walker 2007"), "{}", row.name);
        }
    }
}

/// The concepts the table flags are exactly the ones spec validation warns about
/// (`SNR-SPEC-042`), so both layers agree.
#[test]
fn flagged_concepts_match_spec_validation() {
    let flagged: BTreeSet<&str> = table()
        .rows()
        .filter(|row| row.flagged && !row.name.starts_with("graph"))
        .map(|row| row.name.as_str())
        .collect();
    let spec: BTreeSet<&str> = sonarithm_spec::validate::FLAGGED_CONCEPTS
        .iter()
        .copied()
        .collect();
    assert_eq!(flagged, spec);
    for row in table().rows().filter(|row| row.name.starts_with("graph")) {
        assert!(row.flagged && row.rules == ["SDR-41"], "{}", row.name);
    }
}

/// SDR-12 (DEFAULT): positive polarity for velocity, pressure, temperature, urgency,
/// danger, proximity and generic quantities.
#[test]
fn sdr_12_positive_polarity_for_generic_and_listed_concepts() {
    for concept in [
        "velocity",
        "pressure",
        "temperature",
        "urgency",
        "danger",
        "proximity",
        "generic",
    ] {
        assert_eq!(polarity_of(concept), Polarity::Positive, "{concept}");
        assert!(!table().get(concept).unwrap().flagged, "{concept}");
    }
    // No concept, or an unknown one: the generic default, explained.
    let (none, diagnostics) = auto(None);
    assert_eq!(
        (none.polarity, none.source),
        (Polarity::Positive, PolaritySource::Generic)
    );
    assert_eq!(codes_of(&diagnostics), ["SNR-ENG-020"]);
    assert_eq!(diagnostics[0].rules, [RuleId::from_static("SDR-12")]);
    let (unknown, diagnostics) = auto(Some(("brightness_of_stars", ConceptOrigin::Data)));
    assert_eq!(unknown.source, PolaritySource::Generic);
    assert_eq!(codes_of(&diagnostics), ["SNR-ENG-022"]);
}

/// SDR-13 (DEFAULT): negative polarity for mass (strong evidence) and for size (split, so
/// flagged).
#[test]
fn sdr_13_negative_polarity_for_mass_and_flagged_size() {
    assert_eq!(polarity_of("mass"), Polarity::Negative);
    assert!(!table().get("mass").unwrap().flagged);
    assert_eq!(polarity_of("size"), Polarity::Negative);
    let size = table().get("size").unwrap();
    assert!(size.flagged);
    assert_eq!(
        size.votes,
        Some(Votes {
            positive: 9,
            none: 0,
            negative: 10
        })
    );
    let (_, diagnostics) = auto(Some(("mass", ConceptOrigin::Spec)));
    let info = &diagnostics[0];
    assert_eq!(
        (info.severity, info.rules.clone()),
        (Severity::Info, vec![RuleId::from_static("SDR-13")])
    );
    assert!(info.message.contains("2 positive, 0 none, 17 negative"));
}

/// SDR-14 (MUST): money and attractiveness are flagged. A flagged concept read from the
/// data warns with the listener counts (from the spec, validation warns instead), and
/// attractiveness falls back to the generic default because pitch is best avoided.
#[test]
fn sdr_14_ambiguous_concepts_are_flagged_with_their_evidence() {
    for concept in ["money", "attractiveness"] {
        assert!(table().get(concept).unwrap().flagged, "{concept}");
        let (_, from_spec) = auto(Some((concept, ConceptOrigin::Spec)));
        assert_eq!(codes_of(&from_spec), ["SNR-ENG-020"], "{concept}");
        let (_, from_data) = auto(Some((concept, ConceptOrigin::Data)));
        assert_eq!(
            codes_of(&from_data),
            ["SNR-ENG-020", "SNR-ENG-021"],
            "{concept}"
        );
        assert_eq!(from_data[1].severity, Severity::Warning);
    }
    let money = auto(Some(("money", ConceptOrigin::Data))).1;
    assert!(
        money[0]
            .message
            .contains("blind and low-vision listeners lean negative")
    );
    assert!(money[1].message.contains("9 positive, 2 none, 8 negative"));
    let attractiveness = auto(Some(("attractiveness", ConceptOrigin::Spec)));
    assert_eq!(attractiveness.0.polarity, Polarity::Positive);
    assert!(attractiveness.1[0].message.contains("best avoided"));
    // Graph concepts await a study (SDR-41), with or without a row of their own.
    for concept in ["graph.degree", "graph.betweenness"] {
        let (resolved, diagnostics) = auto(Some((concept, ConceptOrigin::Data)));
        assert_eq!(resolved.polarity, Polarity::Positive);
        assert_eq!(codes_of(&diagnostics), ["SNR-ENG-020", "SNR-ENG-021"]);
        assert!(diagnostics[0].message.contains("SDR-41"));
    }
}

/// SDR-15 (MUST): the one-key flip inverts every resolution, explicit or automatic, and a
/// profile can set a concept's polarity for `auto`; explicit spec polarity beats the
/// profile's concept preference but not the flip.
#[test]
fn sdr_15_profile_overrides_and_the_one_key_flip() {
    let mut profile = PolarityProfile::default();
    profile
        .concepts
        .insert("money".to_owned(), Polarity::Negative);
    let request = |choice| PolarityRequest {
        field: "price",
        concept: Some(("money", ConceptOrigin::Spec)),
        choice,
    };
    let (by_profile, diagnostics) =
        resolve(table(), &profile, &request(PolarityChoice::Auto), &at());
    assert_eq!(
        (by_profile.polarity, by_profile.source),
        (Polarity::Negative, PolaritySource::Profile)
    );
    assert_eq!(codes_of(&diagnostics), ["SNR-ENG-023"]);
    let (explicit, _) = resolve(table(), &profile, &request(PolarityChoice::Positive), &at());
    assert_eq!(explicit.polarity, Polarity::Positive);

    profile.flipped_fields.insert("price".to_owned());
    for (choice, unflipped) in [
        (PolarityChoice::Auto, Polarity::Negative),
        (PolarityChoice::Positive, Polarity::Positive),
        (PolarityChoice::Negative, Polarity::Negative),
    ] {
        let (resolved, diagnostics) = resolve(table(), &profile, &request(choice), &at());
        assert!(resolved.flipped);
        assert_eq!(resolved.polarity, unflipped.flipped(), "{choice:?}");
        assert_eq!(diagnostics.last().unwrap().code.as_str(), "SNR-ENG-023");
        assert_eq!(
            diagnostics.last().unwrap().location,
            Some(at().key("polarity"))
        );
    }
    assert_eq!(Polarity::Positive.flipped().flipped(), Polarity::Positive);
}

#[test]
fn invalid_tables_are_rejected() {
    let row = |name: &str, extra: &str| {
        format!(
            "[[concept]]
name = \"{name}\"
polarity = \"positive\"
flagged = false
rules = [\"SDR-12\"]
evidence = \"A source.\"
{extra}
"
        )
    };
    let generic = row("generic", "");
    let parse = |text: &str| {
        ConceptTable::parse(&format!(
            "schema = 1
{text}"
        ))
    };
    assert!(parse(&generic).is_ok());
    assert!(matches!(parse(""), Err(ConceptTableError::Parse(_))));
    assert!(matches!(
        ConceptTable::parse(&format!(
            "schema = 2
{generic}"
        )),
        Err(ConceptTableError::Parse(_))
    ));
    for bad in [
        row("velocity", ""),
        format!("{generic}{}", row("Velocity", "")),
        format!("{generic}{}", row("velocity", "exponent = -1.0")),
        format!("{generic}{generic}"),
        generic.replace("A source.", " "),
        generic.replace("[\"SDR-12\"]", "[]"),
        generic.replace("[\"SDR-12\"]", "[\"SDR-1\"]"),
        generic.replace("\"positive\"", "\"avoid\""),
    ] {
        assert!(
            matches!(parse(&bad), Err(ConceptTableError::Invalid { .. })),
            "{bad}"
        );
    }
}

proptest! {
    /// SDR-15 and 05 §4.2 invariant 4: flipping polarity mirrors every pitch within the
    /// range in the pitch space's coordinate; in log-frequency (the default) that is the
    /// mirror in cents, `low + high − pitch`.
    #[test]
    fn sdr_15_polarity_flip_symmetry(
        space in prop::sample::select(PitchSpace::ALL.to_vec()),
        low in 50.0..5000.0_f64,
        ratio in 1.01..20.0_f64,
        u in 0.0..=1.0_f64,
    ) {
        let high = (low * ratio).min(20_000.0);
        let map = PitchMap::new(space, PitchValue::Hz(low), PitchValue::Hz(high)).unwrap();
        let positive = map.pitch(Polarity::Positive.apply(u));
        let negative = map.pitch(Polarity::Negative.apply(u));
        let range = map.range();
        prop_assert!(range.contains(positive) && range.contains(negative));
        let x = |pitch: sonarithm_core::units::Cents| space.coordinate(pitch.to_hz());
        let (x0, x1) = (x(range.low), x(range.high));
        let mirrored = x0 + x1 - x(positive);
        prop_assert!((x(negative) - mirrored).abs() <= 1e-9 * (x1 - x0).abs().max(1.0));
        if space == PitchSpace::LogFrequency {
            prop_assert!((negative.0 - range.mirror(positive).0).abs() < 1e-9);
        }
        // Flipping twice changes nothing.
        let twice = Polarity::Negative.flipped().flipped().apply(u);
        prop_assert_eq!(map.pitch(twice), negative);
    }
}

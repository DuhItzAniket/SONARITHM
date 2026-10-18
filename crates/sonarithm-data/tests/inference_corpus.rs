// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A04-T05 acceptance: type, role and concept inference reach at least 95% precision on the
//! inference corpus (`tests/corpus/csv/expected.json`, 03 §3).
//!
//! Precision counts only what inference asserted: a field type always, a role or concept
//! only when one was inferred. A role or concept the manifest expects but inference left
//! out lowers recall, which is printed but not gated. The labels were written together
//! with the rules, so the corpus guards behaviour and documents it; it is not an
//! independent benchmark. It keeps one known miss (`27_weather.csv`, `wind_dir`).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use sonarithm_data::{Concept, CsvOptions, FieldType, Format, ReadOptions, Role, read_bytes};

/// The precision every part of inference must reach (A04-T05).
const REQUIRED_PRECISION: f64 = 0.95;

#[derive(Deserialize)]
struct Expected {
    #[serde(default)]
    options: CsvOptions,
    fields: BTreeMap<String, Field>,
}

#[derive(Deserialize)]
struct Field {
    #[serde(rename = "type")]
    ftype: String,
    role: Option<String>,
    concept: Option<String>,
}

#[derive(Default)]
struct Score {
    asserted: u32,
    correct: u32,
    expected: u32,
    found: u32,
    misses: Vec<String>,
}

impl Score {
    fn record(&mut self, place: &str, inferred: Option<&str>, expected: Option<&str>) {
        if expected.is_some() {
            self.expected += 1;
        }
        let Some(inferred) = inferred else { return };
        self.asserted += 1;
        if Some(inferred) == expected {
            self.correct += 1;
            self.found += 1;
        } else {
            self.misses.push(format!(
                "{place}: inferred {inferred}, expected {expected:?}"
            ));
        }
    }

    fn precision(&self) -> f64 {
        f64::from(self.correct) / f64::from(self.asserted.max(1))
    }

    fn recall(&self) -> f64 {
        f64::from(self.found) / f64::from(self.expected.max(1))
    }
}

#[test]
fn inference_reaches_95_percent_precision_on_the_corpus() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/csv");
    let manifest = std::fs::read_to_string(dir.join("expected.json")).unwrap();
    let corpus: BTreeMap<String, Expected> = serde_json::from_str(&manifest).unwrap();
    let (mut types, mut roles, mut concepts) =
        (Score::default(), Score::default(), Score::default());
    for (file, expected) in &corpus {
        let bytes = std::fs::read(dir.join(file)).unwrap();
        let options = ReadOptions::new().with_csv(expected.options.clone());
        let loaded = read_bytes(&bytes, Format::Csv, file, &options).unwrap();
        let table = loaded.bundle.primary().unwrap();
        for (name, field) in &expected.fields {
            let meta = table.field_meta(name).unwrap();
            let place = format!("{file} `{name}`");
            types.record(
                &place,
                meta.ftype.map(FieldType::as_str),
                Some(&field.ftype),
            );
            roles.record(&place, meta.role.map(Role::as_str), field.role.as_deref());
            concepts.record(
                &place,
                meta.concept.as_ref().map(Concept::as_str),
                field.concept.as_deref(),
            );
        }
    }
    let report: Vec<String> = [("type", &types), ("role", &roles), ("concept", &concepts)]
        .iter()
        .map(|(part, score)| {
            format!(
                "{part}: precision {:.3} ({}/{}), recall {:.3}; misses: {:?}",
                score.precision(),
                score.correct,
                score.asserted,
                score.recall(),
                score.misses
            )
        })
        .collect();
    println!("{}", report.join("\n"));
    for (part, score) in [("type", &types), ("role", &roles), ("concept", &concepts)] {
        assert!(
            score.precision() >= REQUIRED_PRECISION,
            "{part} precision below 95%:\n{}",
            report.join("\n")
        );
    }
    assert!(
        types.asserted >= 70,
        "the corpus has {} fields",
        types.asserted
    );
}

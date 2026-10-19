// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Synthetic example datasets for tests, templates and documentation.
//!
//! Each is generated from a formula and a fixed seed, so it is identical on every platform
//! (ADR-0013): sines come from [`sonarithm_core::math`] and noise from a seeded `ChaCha8`
//! stream. They follow the planned example datasets of docs/research-paper/datasets.md
//! (DS-001 to DS-005) and add the shapes the templates need (scatter, histogram, box,
//! heat map). They are not the registered gallery datasets, which A07 ships.
//!
//! | Function | Shape | Fields |
//! |---|---|---|
//! | [`temperature`] | DS-001-like daily series | `day` (x), `temp` (°C, concept `temperature`) |
//! | [`prices`] | DS-002-like price series, 20 points | `week` (x), `price` ($, concept `money`) |
//! | [`regions`] | DS-003-like three series, long form | `month` (x), `region` (series), `rain` (mm) |
//! | [`fruit`] | DS-004-like bars | `fruit` (category), `sold` |
//! | [`signal`] | DS-005-like dense signal with transients, 2000 samples | `t` (x), `amplitude` |
//! | [`scatter`] | two correlated measures and a group | `height`, `weight`, `group` |
//! | [`waiting_times`] | 200 raw values for a histogram | `minutes` |
//! | [`reaction_times`] | values in three groups for box plots | `condition`, `ms` |
//! | [`grid`] | a 6 × 8 heat map | `row`, `column`, `level` |
//! | [`estimated`] | a series with estimated values (03 §4) | `x`, `y`, `__snr_prov`, `__snr_conf` |

use std::collections::BTreeMap;

use serde_json::{Value, json};
use sonarithm_core::engine::TrackRng;
use sonarithm_core::ids::TrackId;
use sonarithm_core::math;
use sonarithm_data::{
    Concept, DataBundle, DataError, FieldMeta, FieldType, Format, ReadOptions, Role, Unit,
    read_bytes,
};

/// A seeded noise stream in [−1, 1).
fn noise(seed: u64, name: &str) -> impl FnMut() -> f64 {
    let mut rng = TrackRng::new(seed, &TrackId::new(name));
    move || 2.0 * rng.next_f64() - 1.0
}

/// Rounds to `digits` decimals (exact IEEE rounding, platform-independent).
fn round(value: f64, digits: i32) -> f64 {
    let scale = math::pow(10.0, f64::from(digits));
    (value * scale).round() / scale
}

fn meta(ftype: FieldType) -> FieldMeta {
    FieldMeta::new().with_type(ftype)
}

/// Builds a bundle from records with declared field semantics. The data layer's inference
/// fills only what is not declared (03 §3).
fn bundle(
    name: &str,
    rows: &[Value],
    fields: BTreeMap<String, FieldMeta>,
) -> Result<DataBundle, DataError> {
    let bytes = serde_json::to_vec(&Value::Array(rows.to_vec())).unwrap_or_default();
    let options = ReadOptions::new().with_fields(fields);
    read_bytes(&bytes, Format::Json, &format!("{name}.json"), &options).map(|loaded| loaded.bundle)
}

fn x_field() -> FieldMeta {
    meta(FieldType::Quantitative).with_role(Role::X)
}

/// A daily temperature series over 120 days: a seasonal curve plus seeded noise
/// (DS-001-like; concept `temperature`, positive polarity, SDR-12).
///
/// # Errors
///
/// A [`DataError`] only if the data layer rejects the generated records.
pub fn temperature() -> Result<DataBundle, DataError> {
    let mut noise = noise(1, "DS-001");
    let rows: Vec<Value> = (0..120)
        .map(|day| {
            let season = math::sin(2.0 * std::f64::consts::PI * f64::from(day) / 365.0);
            json!({ "day": day, "temp": round(8.0 + 12.0 * season + 1.5 * noise(), 2) })
        })
        .collect();
    let mut fields = BTreeMap::new();
    fields.insert("day".to_owned(), x_field());
    let temp = meta(FieldType::Quantitative)
        .with_label("Temperature")
        .with_concept(Concept::new("temperature").unwrap_or_else(Concept::generic));
    let temp = Unit::new("°C").map_or(temp.clone(), |unit| temp.with_unit(unit));
    fields.insert("temp".to_owned(), temp);
    bundle("temperature", &rows, fields)
}

/// A 20-point price series, as in Smith & Walker 2005 (DS-002-like; concept `money`,
/// flagged, SDR-14).
///
/// # Errors
///
/// As [`temperature`].
pub fn prices() -> Result<DataBundle, DataError> {
    let mut noise = noise(2, "DS-002");
    let mut price = 50.0;
    let rows: Vec<Value> = (0..20)
        .map(|week| {
            let trend = if week < 10 { 1.5 } else { -1.0 };
            price = round(price + trend + 2.0 * noise(), 2);
            json!({ "week": week, "price": price })
        })
        .collect();
    let mut fields = BTreeMap::new();
    fields.insert("week".to_owned(), x_field());
    let price = meta(FieldType::Quantitative)
        .with_label("Price")
        .with_concept(Concept::new("money").unwrap_or_else(Concept::generic));
    let price = Unit::new("$").map_or(price.clone(), |unit| price.with_unit(unit));
    fields.insert("price".to_owned(), price);
    bundle("prices", &rows, fields)
}

/// Monthly rainfall for three regions in long form (DS-003-like).
///
/// # Errors
///
/// As [`temperature`].
pub fn regions() -> Result<DataBundle, DataError> {
    let mut noise = noise(3, "DS-003");
    let mut rows = Vec::new();
    for month in 1..=12 {
        for (region, base, phase) in [
            ("north", 80.0, 0.0),
            ("south", 40.0, 3.0),
            ("east", 60.0, 6.0),
        ] {
            let season = math::cos(2.0 * std::f64::consts::PI * (f64::from(month) - phase) / 12.0);
            let rain = round(base + 25.0 * season + 5.0 * noise(), 1);
            rows.push(json!({ "month": month, "region": region, "rain": rain }));
        }
    }
    let mut fields = BTreeMap::new();
    fields.insert("month".to_owned(), x_field());
    fields.insert(
        "region".to_owned(),
        meta(FieldType::Nominal).with_role(Role::Series),
    );
    let rain = meta(FieldType::Quantitative).with_label("Rainfall");
    let rain = Unit::new("mm").map_or(rain.clone(), |unit| rain.with_unit(unit));
    fields.insert("rain".to_owned(), rain);
    bundle("regions", &rows, fields)
}

/// Sales by fruit, for bar charts (DS-004-like).
///
/// # Errors
///
/// As [`temperature`].
pub fn fruit() -> Result<DataBundle, DataError> {
    let rows: Vec<Value> = [
        ("apple", 42),
        ("banana", 57),
        ("cherry", 18),
        ("date", 33),
        ("fig", 9),
    ]
    .iter()
    .map(|(fruit, sold)| json!({ "fruit": fruit, "sold": sold }))
    .collect();
    let mut fields = BTreeMap::new();
    fields.insert(
        "fruit".to_owned(),
        meta(FieldType::Nominal).with_role(Role::Category),
    );
    fields.insert(
        "sold".to_owned(),
        meta(FieldType::Quantitative).with_label("Units sold"),
    );
    bundle("fruit", &rows, fields)
}

/// A dense signal: 2000 samples of noise with three transients (DS-005-like), for the
/// density switch (SDR-20).
///
/// # Errors
///
/// As [`temperature`].
pub fn signal() -> Result<DataBundle, DataError> {
    let mut noise = noise(5, "DS-005");
    let rows: Vec<Value> = (0..2000)
        .map(|i| {
            let transient = [400, 1100, 1650]
                .iter()
                .map(|&at| {
                    let d = f64::from(i - at);
                    if d >= 0.0 {
                        5.0 * math::exp(-d / 20.0)
                    } else {
                        0.0
                    }
                })
                .sum::<f64>();
            json!({ "t": i, "amplitude": round(0.3 * noise() + transient, 3) })
        })
        .collect();
    let mut fields = BTreeMap::new();
    fields.insert("t".to_owned(), x_field());
    fields.insert("amplitude".to_owned(), meta(FieldType::Quantitative));
    bundle("signal", &rows, fields)
}

/// Height and weight of 40 people in two groups, for scatter plots.
///
/// # Errors
///
/// As [`temperature`].
pub fn scatter() -> Result<DataBundle, DataError> {
    let mut noise = noise(6, "scatter");
    let rows: Vec<Value> = (0..40)
        .map(|i| {
            let height = round(155.0 + 35.0 * (f64::from(i) / 39.0) + 4.0 * noise(), 1);
            let weight = round(0.9 * height - 85.0 + 6.0 * noise(), 1);
            let group = if i % 2 == 0 { "a" } else { "b" };
            json!({ "height": height, "weight": weight, "group": group })
        })
        .collect();
    let mut fields = BTreeMap::new();
    fields.insert("height".to_owned(), x_field());
    fields.insert(
        "weight".to_owned(),
        meta(FieldType::Quantitative).with_role(Role::Y),
    );
    fields.insert(
        "group".to_owned(),
        meta(FieldType::Nominal).with_role(Role::Series),
    );
    bundle("scatter", &rows, fields)
}

/// 200 waiting times in minutes, skewed, for histograms.
///
/// # Errors
///
/// As [`temperature`].
pub fn waiting_times() -> Result<DataBundle, DataError> {
    let mut noise = noise(7, "waiting");
    let rows: Vec<Value> = (0..200)
        .map(|_| {
            let u = f64::midpoint(noise(), 1.0);
            json!({ "minutes": round(-6.0 * math::ln(1.0 - 0.98 * u), 2) })
        })
        .collect();
    let mut fields = BTreeMap::new();
    fields.insert(
        "minutes".to_owned(),
        meta(FieldType::Quantitative).with_label("Waiting time"),
    );
    bundle("waiting", &rows, fields)
}

/// Reaction times in three conditions, 30 each, for box plots.
///
/// # Errors
///
/// As [`temperature`].
pub fn reaction_times() -> Result<DataBundle, DataError> {
    let mut noise = noise(8, "reaction");
    let mut rows = Vec::new();
    for (condition, centre, spread) in [
        ("easy", 420.0, 40.0),
        ("medium", 510.0, 60.0),
        ("hard", 640.0, 90.0),
    ] {
        for _ in 0..30 {
            rows.push(json!({ "condition": condition, "ms": round(centre + spread * noise(), 0) }));
        }
    }
    let mut fields = BTreeMap::new();
    fields.insert(
        "condition".to_owned(),
        meta(FieldType::Nominal).with_role(Role::Category),
    );
    fields.insert(
        "ms".to_owned(),
        meta(FieldType::Quantitative).with_label("Reaction time"),
    );
    bundle("reaction", &rows, fields)
}

/// A 6 × 8 grid of levels with a hot spot, for heat maps.
///
/// # Errors
///
/// As [`temperature`].
pub fn grid() -> Result<DataBundle, DataError> {
    let mut rows = Vec::new();
    for row in 0..6 {
        for column in 0..8 {
            let d = f64::from((row - 2) * (row - 2) + (column - 5) * (column - 5));
            rows.push(json!({ "row": row, "column": column, "level": round(10.0 * math::exp(-d / 6.0), 2) }));
        }
    }
    let mut fields = BTreeMap::new();
    fields.insert("row".to_owned(), meta(FieldType::Ordinal));
    fields.insert(
        "column".to_owned(),
        meta(FieldType::Ordinal).with_role(Role::X),
    );
    fields.insert("level".to_owned(), meta(FieldType::Quantitative));
    bundle("grid", &rows, fields)
}

/// Ten points, three of them estimated from a raster chart with their confidence (03 §4).
///
/// # Errors
///
/// As [`temperature`].
pub fn estimated() -> Result<DataBundle, DataError> {
    let rows: Vec<Value> = (0..10)
        .map(|x| {
            let estimated = matches!(x, 3 | 4 | 7);
            json!({
                "x": x,
                "y": f64::from(x * x) / 4.0,
                "__snr_prov": if estimated { "cv_estimate" } else { "exact" },
                "__snr_conf": if estimated { json!(0.75) } else { Value::Null },
            })
        })
        .collect();
    let mut fields = BTreeMap::new();
    fields.insert("x".to_owned(), x_field());
    fields.insert("y".to_owned(), meta(FieldType::Quantitative));
    bundle("estimated", &rows, fields)
}

/// Every example by name, in a fixed order.
///
/// # Errors
///
/// As [`temperature`].
pub fn all() -> Result<Vec<(&'static str, DataBundle)>, DataError> {
    Ok(vec![
        ("temperature", temperature()?),
        ("prices", prices()?),
        ("regions", regions()?),
        ("fruit", fruit()?),
        ("signal", signal()?),
        ("scatter", scatter()?),
        ("waiting_times", waiting_times()?),
        ("reaction_times", reaction_times()?),
        ("grid", grid()?),
        ("estimated", estimated()?),
    ])
}

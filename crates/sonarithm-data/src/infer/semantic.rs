// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Field type, role and concept inference (03 §3).
//!
//! Inference fills only what neither the file nor the spec declares, and reports every
//! part it fills as an `Info` diagnostic (SNR-DATA-030 type, 031 role, 032 concept), so a
//! screen-reader user hears what was assumed. It is a suggestion: a declaration in the
//! spec's `data.fields` always wins.
//!
//! The rules are deliberately simple and readable; their precision is measured on the
//! inference corpus in `tests/corpus/csv` (A04-T05 acceptance: ≥ 95%).

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::{Int64Type, UInt8Type};
use arrow_array::{
    Array, ArrayRef, DictionaryArray, Float64Array, RecordBatch, StringArray, UInt8Array,
};
use arrow_schema::{DataType, Field, Schema};
use sonarithm_core::{Diagnostic, Diagnostics};

use crate::codes;
use crate::error::DataError;
use crate::meta::{Concept, FieldMeta, FieldType, Role};
use crate::table::Table;

/// More distinct values than this make an integer field quantitative (03 §3.1).
const MAX_ORDINAL_LEVELS: usize = 12;

/// Fields whose names start with this prefix are reserved (provenance, 03 §4).
pub(crate) const RESERVED_PREFIX: &str = "__snr_";

/// Ordered label sets that make a text field ordinal (03 §3.1). Matching ignores case.
const ORDERED_LABELS: &[&[&str]] = &[
    &[
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ],
    &[
        "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
    ],
    &[
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
    ],
    &["mon", "tue", "wed", "thu", "fri", "sat", "sun"],
    &["very low", "low", "medium", "high", "very high"],
    &["low", "mid", "high"],
    &["extra small", "small", "medium", "large", "extra large"],
    &["xs", "s", "m", "l", "xl", "xxl"],
    &["never", "rarely", "sometimes", "often", "always"],
    &[
        "strongly disagree",
        "disagree",
        "neutral",
        "agree",
        "strongly agree",
    ],
    &["poor", "fair", "good", "very good", "excellent"],
    &["none", "mild", "moderate", "severe"],
];

/// Name tokens and units that suggest a concept (03 §3.3). The concept table that turns a
/// concept into defaults is NUMISONA data; this list only recognizes names.
const CONCEPT_WORDS: &[(&str, &[&str], &[&str])] = &[
    // (concept, name tokens, unit texts)
    (
        "temperature",
        &[
            "temp",
            "temperature",
            "temperatur",
            "celsius",
            "fahrenheit",
            "kelvin",
        ],
        &["°c", "°f", "degc", "degf", "cel", "[degf]", "℃", "℉"],
    ),
    (
        "velocity",
        &["speed", "velocity", "mph", "kph", "kmh", "knots"],
        &["m/s", "km/h", "mph", "kn", "kt", "ft/s"],
    ),
    (
        "pressure",
        &["pressure", "hpa", "kpa", "mbar", "psi", "mmhg"],
        &["hpa", "kpa", "pa", "mbar", "bar", "psi", "mmhg", "atm"],
    ),
    (
        "money",
        &[
            "price", "cost", "costs", "revenue", "sales", "income", "profit", "salary", "wage",
            "wages", "gdp", "usd", "eur", "gbp", "dollar", "dollars", "euro", "euros", "budget",
            "spend", "spending", "fare", "fee", "fees", "payment",
        ],
        &["$", "€", "£", "¥", "₹", "usd", "eur", "gbp", "jpy", "inr"],
    ),
    (
        "mass",
        &["mass"],
        &["kg", "g", "mg", "lb", "lbs", "oz", "t"],
    ),
    ("size", &["size", "area", "volume"], &[]),
    ("danger", &["danger", "risk", "hazard", "threat"], &[]),
    ("urgency", &["urgency", "priority"], &[]),
    ("proximity", &["proximity"], &[]),
    ("attractiveness", &["attractiveness"], &[]),
    ("graph.degree", &["degree", "indegree", "outdegree"], &[]),
    (
        "graph.centrality",
        &["centrality", "betweenness", "pagerank"],
        &[],
    ),
];

/// Infers missing field semantics and returns the table with them in its schema.
pub(crate) fn infer_table(
    table: &Table,
    declared: &BTreeMap<String, FieldMeta>,
    diagnostics: &mut Diagnostics,
) -> Result<Table, DataError> {
    if table.batches().is_empty() {
        return Ok(table.clone());
    }
    let batch = table.to_batch()?;
    let rows = batch.num_rows();
    let mut fields: Vec<FieldInfo> = Vec::with_capacity(batch.num_columns());
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(batch.num_columns());
    for (field, column) in table.schema().fields().iter().zip(batch.columns()) {
        let (mut meta, _) = FieldMeta::read(field);
        if let Some(declared) = declared.get(field.name()) {
            merge(&mut meta, declared);
        }
        let mut column = Arc::clone(column);
        let reserved = field.name().starts_with(RESERVED_PREFIX);
        let facts = Facts::of(field.name(), &column);
        if !reserved && meta.ftype.is_none() {
            let (ftype, reason, replacement) =
                field_type(field.name(), &column, &facts, meta.unit.is_some());
            diagnostics.push(Diagnostic::new(
                &codes::TYPE_INFERRED,
                format!("Field `{}` is {} ({reason}).", field.name(), ftype.as_str()),
            ));
            meta.ftype = Some(ftype);
            if let Some(replacement) = replacement {
                column = replacement;
            }
        }
        fields.push(FieldInfo {
            name: field.name().clone(),
            nullable: field.is_nullable(),
            metadata: field.metadata().clone(),
            meta,
            facts,
            reserved,
        });
        columns.push(column);
    }
    assign_roles(&mut fields, rows, diagnostics);
    assign_concepts(&mut fields, diagnostics);

    let schema_fields: Vec<Field> = fields
        .iter()
        .zip(&columns)
        .map(|(info, column)| {
            info.meta.apply(
                Field::new(&info.name, column.data_type().clone(), info.nullable)
                    .with_metadata(info.metadata.clone()),
            )
        })
        .collect();
    let schema =
        Arc::new(Schema::new(schema_fields).with_metadata(table.schema().metadata().clone()));
    let row_count = arrow_array::RecordBatchOptions::new().with_row_count(Some(rows));
    let batch = RecordBatch::try_new_with_options(Arc::clone(&schema), columns, &row_count)
        .map_err(|source| DataError::MalformedTable {
            table: table.name().to_owned(),
            source,
        })?;
    Table::new(table.id(), table.name(), schema, vec![batch])
}

/// Declared parts win over what the file says.
fn merge(meta: &mut FieldMeta, declared: &FieldMeta) {
    if declared.ftype.is_some() {
        meta.ftype = declared.ftype;
    }
    if declared.unit.is_some() {
        meta.unit.clone_from(&declared.unit);
    }
    if declared.concept.is_some() {
        meta.concept.clone_from(&declared.concept);
    }
    if declared.role.is_some() {
        meta.role = declared.role;
    }
    if declared.label.is_some() {
        meta.label.clone_from(&declared.label);
    }
    if declared.description.is_some() {
        meta.description.clone_from(&declared.description);
    }
}

struct FieldInfo {
    name: String,
    nullable: bool,
    metadata: arrow_schema::Metadata,
    meta: FieldMeta,
    facts: Facts,
    reserved: bool,
}

/// What the rules need to know about a column.
#[derive(Clone, Debug, Default)]
struct Facts {
    /// Lowercase name tokens, with a bracketed unit removed.
    tokens: Vec<String>,
    /// Distinct non-missing values; for numbers counted only up to `MAX_ORDINAL_LEVELS + 1`.
    distinct: usize,
    /// Every non-missing value is different (and there are at least two); text only.
    all_distinct: bool,
    /// Non-missing values strictly increase in row order.
    increasing: bool,
    kind: Kind,
}

/// The broad storage kind of a column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Kind {
    Numeric,
    Temporal,
    Text,
    Boolean,
    #[default]
    Other,
}

impl Facts {
    fn of(name: &str, column: &ArrayRef) -> Self {
        let data_type = column.data_type();
        let kind = if data_type.is_numeric() {
            Kind::Numeric
        } else if data_type.is_temporal() {
            Kind::Temporal
        } else if matches!(
            data_type,
            DataType::Utf8 | DataType::LargeUtf8 | DataType::Dictionary(..)
        ) {
            Kind::Text
        } else if *data_type == DataType::Boolean {
            Kind::Boolean
        } else {
            Kind::Other
        };
        let mut facts = Self {
            tokens: tokens(name),
            kind,
            ..Self::default()
        };
        if let Some(strings) = column.as_string_opt::<i32>() {
            let mut seen = HashSet::new();
            let mut duplicate = false;
            for value in strings.iter().flatten() {
                duplicate |= !seen.insert(value);
            }
            facts.distinct = seen.len();
            facts.all_distinct = !duplicate && seen.len() > 1;
        } else if kind == Kind::Numeric
            && let Some(values) = numbers(column)
        {
            // Distinct values are counted only up to the ordinal limit: that is all the
            // rules need, and it keeps inference cheap on large columns.
            let mut seen = HashSet::new();
            let mut increasing = true;
            let mut count = 0_usize;
            let mut previous: Option<f64> = None;
            for value in values.iter().flatten() {
                if seen.len() <= MAX_ORDINAL_LEVELS {
                    seen.insert(value.to_bits());
                }
                if let Some(p) = previous {
                    increasing &= value > p;
                }
                previous = Some(value);
                count += 1;
            }
            facts.distinct = seen.len();
            facts.increasing = increasing && count > 1;
        }
        facts
    }

    fn has(&self, words: &[&str]) -> bool {
        self.tokens
            .iter()
            .any(|token| words.contains(&token.as_str()))
    }

    fn is(&self, words: &[&str]) -> bool {
        self.tokens.len() == 1 && self.has(words)
    }
}

/// A numeric or temporal column as `f64` values, used only to count distinct values and
/// check order.
pub(crate) fn numbers(column: &ArrayRef) -> Option<Float64Array> {
    use arrow_array::types::{
        Date32Type, Date64Type, DurationMicrosecondType, DurationMillisecondType,
        DurationNanosecondType, DurationSecondType, Float32Type, Float64Type, Int8Type, Int16Type,
        Int32Type, TimestampMicrosecondType, TimestampMillisecondType, TimestampNanosecondType,
        TimestampSecondType, UInt16Type, UInt32Type, UInt64Type,
    };
    use arrow_schema::TimeUnit;

    macro_rules! to_f64 {
        ($type:ty) => {{
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_lossless,
                reason = "only compared and counted; the lints fire for some types only"
            )]
            let values: Float64Array = column
                .as_primitive::<$type>()
                .iter()
                .map(|v| v.map(|v| v as f64))
                .collect();
            Some(values)
        }};
    }
    match column.data_type() {
        DataType::Float64 => Some(column.as_primitive::<Float64Type>().clone()),
        DataType::Float32 => to_f64!(Float32Type),
        DataType::Int8 => to_f64!(Int8Type),
        DataType::Int16 => to_f64!(Int16Type),
        DataType::Int32 => to_f64!(Int32Type),
        DataType::Int64 => to_f64!(Int64Type),
        DataType::UInt8 => to_f64!(UInt8Type),
        DataType::UInt16 => to_f64!(UInt16Type),
        DataType::UInt32 => to_f64!(UInt32Type),
        DataType::UInt64 => to_f64!(UInt64Type),
        DataType::Date32 => to_f64!(Date32Type),
        DataType::Date64 => to_f64!(Date64Type),
        DataType::Timestamp(TimeUnit::Second, _) => to_f64!(TimestampSecondType),
        DataType::Timestamp(TimeUnit::Millisecond, _) => to_f64!(TimestampMillisecondType),
        DataType::Timestamp(TimeUnit::Microsecond, _) => to_f64!(TimestampMicrosecondType),
        DataType::Timestamp(TimeUnit::Nanosecond, _) => to_f64!(TimestampNanosecondType),
        DataType::Duration(TimeUnit::Second) => to_f64!(DurationSecondType),
        DataType::Duration(TimeUnit::Millisecond) => to_f64!(DurationMillisecondType),
        DataType::Duration(TimeUnit::Microsecond) => to_f64!(DurationMicrosecondType),
        DataType::Duration(TimeUnit::Nanosecond) => to_f64!(DurationNanosecondType),
        _ => None,
    }
}

/// Lowercase tokens of a field name: split at non-alphanumerics and lower-to-upper case
/// changes, with a trailing bracketed unit removed.
fn tokens(name: &str) -> Vec<String> {
    let name = match name.rfind(['(', '[']) {
        Some(start) if start > 0 && name.trim_end().ends_with([')', ']']) => &name[..start],
        _ => name,
    };
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut previous_lower = false;
    for c in name.chars() {
        if !c.is_alphanumeric() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            previous_lower = false;
            continue;
        }
        if c.is_uppercase() && previous_lower && !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
        previous_lower = c.is_lowercase() || c.is_ascii_digit();
        current.extend(c.to_lowercase());
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

const YEAR_WORDS: &[&str] = &["year", "yr", "years"];
const ID_WORDS: &[&str] = &[
    "id", "key", "uuid", "guid", "code", "zip", "postcode", "sku", "iso",
];
const GROUP_WORDS: &[&str] = &["type", "group", "class", "kind", "category"];
const LABEL_WORDS: &[&str] = &["label", "name", "title", "description"];
const X_WORDS: &[&str] = &[
    "x",
    "time",
    "t",
    "date",
    "datetime",
    "timestamp",
    "step",
    "index",
    "idx",
    "frame",
    "month",
    "day",
    "week",
    "quarter",
];
const LOWER_WORDS: &[&str] = &["lower", "low", "lo", "min", "minimum", "lcl"];
const UPPER_WORDS: &[&str] = &["upper", "high", "hi", "max", "maximum", "ucl"];
const SERIES_WORDS: &[&str] = &[
    "series", "variable", "sensor", "station", "site", "region", "country", "city", "location",
    "source",
];

/// The field type, the reason in words, and a replacement array for ordinal labels.
fn field_type(
    name: &str,
    column: &ArrayRef,
    facts: &Facts,
    has_unit: bool,
) -> (FieldType, String, Option<ArrayRef>) {
    let data_type = column.data_type();
    if facts.kind == Kind::Temporal {
        return (FieldType::Temporal, "dates or times".to_owned(), None);
    }
    if facts.kind == Kind::Boolean {
        return (FieldType::Nominal, "true or false".to_owned(), None);
    }
    if data_type.is_integer() {
        if facts.has(YEAR_WORDS) {
            return (
                FieldType::Temporal,
                "whole numbers named as years".to_owned(),
                None,
            );
        }
        if facts.has(ID_WORDS) || facts.has(GROUP_WORDS) {
            return (
                FieldType::Nominal,
                format!("whole numbers named `{name}`"),
                None,
            );
        }
        if !has_unit && small_scale(column, facts) {
            return (
                FieldType::Ordinal,
                format!("a small scale of {} whole-number levels", facts.distinct),
                None,
            );
        }
        return (
            FieldType::Quantitative,
            describe_numbers(facts, has_unit),
            None,
        );
    }
    if data_type.is_numeric() {
        return (
            FieldType::Quantitative,
            describe_numbers(facts, has_unit),
            None,
        );
    }
    if let Some(strings) = column.as_string_opt::<i32>()
        && let Some((labels, array)) = ordered_labels(strings)
    {
        return (
            FieldType::Ordinal,
            format!("ordered labels: {}", labels.join(", ")),
            Some(array),
        );
    }
    (
        FieldType::Nominal,
        format!("text with {} distinct values", facts.distinct),
        None,
    )
}

fn describe_numbers(facts: &Facts, has_unit: bool) -> String {
    if has_unit {
        "numbers with a unit".to_owned()
    } else if facts.distinct > MAX_ORDINAL_LEVELS {
        format!("numbers with more than {MAX_ORDINAL_LEVELS} distinct values")
    } else {
        "numbers".to_owned()
    }
}

/// At most 12 levels, in a contiguous integer range, each used about twice on average.
fn small_scale(column: &ArrayRef, facts: &Facts) -> bool {
    let Some(ints) = column.as_primitive_opt::<Int64Type>() else {
        return false;
    };
    let non_null = ints.len() - ints.null_count();
    let (Some(min), Some(max)) = (ints.iter().flatten().min(), ints.iter().flatten().max()) else {
        return false;
    };
    let span = max.saturating_sub(min).saturating_add(1);
    facts.distinct >= 2
        && facts.distinct <= MAX_ORDINAL_LEVELS
        && span <= i64::try_from(MAX_ORDINAL_LEVELS).unwrap_or(i64::MAX)
        && non_null >= 2 * facts.distinct
}

/// Text whose distinct values all belong to one ordered label set: the labels in order (as
/// written in the data) and the column re-encoded as a dictionary in that order, so a key
/// is the label's rank.
fn ordered_labels(strings: &StringArray) -> Option<(Vec<String>, ArrayRef)> {
    let mut distinct: Vec<String> = Vec::new();
    for value in strings.iter().flatten() {
        let key = value.trim().to_lowercase();
        if !distinct.contains(&key) {
            distinct.push(key);
            if distinct.len() > MAX_ORDINAL_LEVELS {
                return None;
            }
        }
    }
    if distinct.len() < 2 {
        return None;
    }
    let set = ORDERED_LABELS
        .iter()
        .find(|set| distinct.iter().all(|value| set.contains(&value.as_str())))?;
    // The labels present, in the set's order, spelled as in the data.
    let mut order: Vec<String> = Vec::new();
    for label in *set {
        if distinct.iter().any(|d| d == label)
            && let Some(spelling) = strings
                .iter()
                .flatten()
                .find(|v| v.trim().to_lowercase() == *label)
        {
            order.push(spelling.trim().to_owned());
        }
    }
    let keys: UInt8Array = strings
        .iter()
        .map(|value| {
            let value = value?.trim().to_lowercase();
            order
                .iter()
                .position(|label| label.to_lowercase() == value)
                .and_then(|rank| u8::try_from(rank).ok())
        })
        .collect();
    let values = Arc::new(StringArray::from(order.clone()));
    let array = DictionaryArray::<UInt8Type>::try_new(keys, values).ok()?;
    Some((order, Arc::new(array)))
}

/// Roles (03 §3.4): first from names, then the x axis, then the remaining fields in order.
/// `x`, `y`, `series` and `label` are given to one field each; declared roles count.
fn assign_roles(fields: &mut [FieldInfo], rows: usize, diagnostics: &mut Diagnostics) {
    let mut chosen: Vec<(usize, Role, &'static str)> = Vec::new();
    for (index, info) in fields.iter().enumerate() {
        if info.is_open()
            && let Some(role) = role_from_name(info)
        {
            chosen.push((index, role, "from its name"));
        }
    }
    apply(fields, &chosen, diagnostics);
    if !has_role(fields, Role::X)
        && let Some((index, why)) = x_axis(fields, rows)
    {
        apply(fields, &[(index, Role::X, why)], diagnostics);
    }
    for index in 0..fields.len() {
        let role = fields
            .get(index)
            .filter(|f| f.is_open())
            .and_then(|info| remaining_role(info, fields, rows));
        if let Some((role, why)) = role {
            apply(fields, &[(index, role, why)], diagnostics);
        }
    }
}

impl FieldInfo {
    /// Not reserved and without a role yet.
    fn is_open(&self) -> bool {
        !self.reserved && self.meta.role.is_none()
    }
}

fn has_role(fields: &[FieldInfo], role: Role) -> bool {
    fields.iter().any(|f| f.meta.role == Some(role))
}

/// Gives roles and reports them; a unique role already taken is skipped.
fn apply(fields: &mut [FieldInfo], chosen: &[(usize, Role, &str)], diagnostics: &mut Diagnostics) {
    for &(index, role, why) in chosen {
        let unique = matches!(role, Role::X | Role::Y | Role::Series);
        if unique && has_role(fields, role) {
            continue;
        }
        if let Some(info) = fields.get_mut(index) {
            info.meta.role = Some(role);
            diagnostics.push(Diagnostic::new(
                &codes::ROLE_INFERRED,
                format!("Field `{}` has role {} ({why}).", info.name, role.as_str()),
            ));
        }
    }
}

/// A role the field's name states.
fn role_from_name(info: &FieldInfo) -> Option<Role> {
    let f = &info.facts;
    let quantitative = info.meta.ftype == Some(FieldType::Quantitative);
    if f.is(&["x"]) {
        Some(Role::X)
    } else if f.is(&["y"]) {
        Some(Role::Y)
    } else if quantitative && f.tokens.len() <= 3 && f.has(LOWER_WORDS) {
        Some(Role::Lower)
    } else if quantitative && f.tokens.len() <= 3 && f.has(UPPER_WORDS) {
        Some(Role::Upper)
    } else if info.meta.ftype == Some(FieldType::Nominal) && f.has(ID_WORDS) {
        Some(Role::Id)
    } else if f.kind == Kind::Text && f.has(LABEL_WORDS) {
        Some(Role::Label)
    } else if info.meta.ftype == Some(FieldType::Nominal)
        && f.kind == Kind::Text
        && !f.all_distinct
        && f.has(SERIES_WORDS)
    {
        Some(Role::Series)
    } else if quantitative && info.meta.unit.is_none() && f.is(&["weight", "weights", "wt"]) {
        Some(Role::Weight)
    } else {
        None
    }
}

/// The x axis: the first temporal field, else the first field named like an axis, else
/// the first strictly increasing number field (with at least four rows, so that order is
/// unlikely by chance, and another number field to plot against it).
fn x_axis(fields: &[FieldInfo], rows: usize) -> Option<(usize, &'static str)> {
    let open = |f: &&FieldInfo| f.is_open();
    let position = |predicate: &dyn Fn(&FieldInfo) -> bool| {
        fields
            .iter()
            .enumerate()
            .filter(|(_, f)| open(f))
            .find(|(_, f)| predicate(f))
            .map(|(i, _)| i)
    };
    if let Some(index) = position(&|f| f.meta.ftype == Some(FieldType::Temporal)) {
        return Some((index, "the first date or time field"));
    }
    if let Some(index) = position(&|f| f.facts.is(X_WORDS)) {
        return Some((index, "named like an axis"));
    }
    let numbers = fields
        .iter()
        .filter(|f| !f.reserved && f.facts.kind == Kind::Numeric)
        .count();
    if rows >= 4 && numbers >= 2 {
        return position(&|f| f.facts.kind == Kind::Numeric && f.facts.increasing)
            .map(|index| (index, "the first increasing number field"));
    }
    None
}

/// The role of a field that names and the x axis did not settle.
fn remaining_role(
    info: &FieldInfo,
    fields: &[FieldInfo],
    rows: usize,
) -> Option<(Role, &'static str)> {
    let f = &info.facts;
    let any_quantitative = fields
        .iter()
        .any(|g| g.meta.ftype == Some(FieldType::Quantitative));
    match info.meta.ftype? {
        FieldType::Temporal => Some((Role::Time, "another date or time field")),
        FieldType::Quantitative => {
            (!has_role(fields, Role::Y)).then_some((Role::Y, "the first quantitative field"))
        }
        FieldType::Ordinal if f.kind != Kind::Text => (!any_quantitative
            && !has_role(fields, Role::Y))
        .then_some((Role::Y, "the first numeric scale")),
        FieldType::Nominal | FieldType::Ordinal if f.kind == Kind::Text => {
            if f.all_distinct {
                (rows >= 3 && !has_role(fields, Role::Label))
                    .then_some((Role::Label, "every value is different"))
            } else if info.meta.ftype == Some(FieldType::Nominal)
                && has_role(fields, Role::X)
                && !has_role(fields, Role::Series)
                && !f.has(GROUP_WORDS)
            {
                Some((Role::Series, "groups rows along the x axis"))
            } else {
                Some((Role::Category, "groups rows"))
            }
        }
        FieldType::Nominal if f.kind == Kind::Boolean || f.has(GROUP_WORDS) => {
            Some((Role::Category, "groups rows"))
        }
        _ => None,
    }
}

/// Concepts for quantitative fields, from units first, then name tokens.
fn assign_concepts(fields: &mut [FieldInfo], diagnostics: &mut Diagnostics) {
    for info in fields.iter_mut() {
        if info.reserved
            || info.meta.concept.is_some()
            || info.meta.ftype != Some(FieldType::Quantitative)
        {
            continue;
        }
        let unit = info
            .meta
            .unit
            .as_ref()
            .map(|u| u.text.trim().to_lowercase());
        let by_unit = unit.as_deref().and_then(|unit| {
            CONCEPT_WORDS
                .iter()
                .find(|(_, _, units)| units.contains(&unit))
                .map(|(c, _, _)| (*c, "its unit"))
        });
        let by_name = || {
            CONCEPT_WORDS
                .iter()
                .find(|(_, words, _)| info.facts.has(words))
                .map(|(c, _, _)| (*c, "its name"))
        };
        if let Some((name, why)) = by_unit.or_else(by_name)
            && let Some(concept) = Concept::new(name)
        {
            diagnostics.push(Diagnostic::new(
                &codes::CONCEPT_INFERRED,
                format!(
                    "Field `{}` measures {} (from {why}); its default polarity comes from the concept table.",
                    info.name,
                    concept.as_str()
                ),
            ));
            info.meta.concept = Some(concept);
        }
    }
}

#[cfg(test)]
mod tests {
    use arrow_array::{BooleanArray, Int64Array};

    use super::*;
    use sonarithm_core::TableId;

    fn table(columns: Vec<(&str, ArrayRef)>) -> Table {
        let fields: Vec<Field> = columns
            .iter()
            .map(|(n, a)| Field::new(*n, a.data_type().clone(), true))
            .collect();
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(fields)),
            columns.into_iter().map(|(_, a)| a).collect(),
        )
        .unwrap();
        Table::from_batch(TableId(0), "t", batch)
    }

    fn infer(table: &Table) -> (Table, Diagnostics) {
        let mut diagnostics = Diagnostics::new();
        let inferred = infer_table(table, &BTreeMap::new(), &mut diagnostics).unwrap();
        (inferred, diagnostics)
    }

    fn meta(table: &Table, name: &str) -> FieldMeta {
        table.field_meta(name).unwrap()
    }

    #[test]
    fn names_split_into_lowercase_tokens() {
        assert_eq!(tokens("windSpeed_max (m/s)"), ["wind", "speed", "max"]);
        assert_eq!(tokens("CO2 ppm"), ["co2", "ppm"]);
        assert_eq!(tokens("Höhe [m]"), ["höhe"]);
        assert_eq!(tokens("(m)"), ["m"]);
    }

    #[test]
    fn a_time_series_gets_x_y_and_series() {
        let dates = Arc::new(arrow_array::Date32Array::from(vec![0, 1, 2, 3]));
        let temps = Arc::new(Float64Array::from(vec![1.5, 2.0, 2.5, 1.0]));
        let station = Arc::new(StringArray::from(vec!["n", "s", "n", "s"]));
        let (t, diagnostics) = infer(&table(vec![
            ("date", dates),
            ("temp_c", temps),
            ("station", station),
        ]));
        assert_eq!(meta(&t, "date").ftype, Some(FieldType::Temporal));
        assert_eq!(meta(&t, "date").role, Some(Role::X));
        assert_eq!(meta(&t, "temp_c").role, Some(Role::Y));
        assert_eq!(meta(&t, "temp_c").concept, Concept::new("temperature"));
        assert_eq!(meta(&t, "station").role, Some(Role::Series));
        let codes: Vec<_> = diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes.iter().filter(|c| **c == "SNR-DATA-030").count(), 3);
        assert!(
            diagnostics
                .iter()
                .all(|d| d.severity == sonarithm_core::Severity::Info)
        );
    }

    #[test]
    fn integer_scales_years_and_identifiers() {
        let rating = Arc::new(Int64Array::from(vec![1, 2, 3, 4, 5, 1, 2, 3, 4, 5]));
        let year = Arc::new(Int64Array::from((2000..2010).collect::<Vec<_>>()));
        let group_id = Arc::new(Int64Array::from(vec![1, 1, 2, 2, 3, 3, 1, 2, 3, 1]));
        let count = Arc::new(Int64Array::from(
            (0..10).map(|i| i * 7 % 13).collect::<Vec<_>>(),
        ));
        let (t, _) = infer(&table(vec![
            ("rating", rating),
            ("year", year),
            ("group_id", group_id),
            ("count", count),
        ]));
        assert_eq!(meta(&t, "rating").ftype, Some(FieldType::Ordinal));
        assert_eq!(meta(&t, "year").ftype, Some(FieldType::Temporal));
        assert_eq!(meta(&t, "year").role, Some(Role::X));
        assert_eq!(meta(&t, "group_id").ftype, Some(FieldType::Nominal));
        assert_eq!(meta(&t, "group_id").role, Some(Role::Id));
        assert_eq!(meta(&t, "count").ftype, Some(FieldType::Quantitative));
    }

    #[test]
    fn ordered_labels_become_ranked_dictionaries() {
        let level = Arc::new(StringArray::from(vec![
            Some("High"),
            Some("low"),
            None,
            Some("Medium"),
            Some("low"),
        ]));
        let (t, _) = infer(&table(vec![("level", level)]));
        assert_eq!(meta(&t, "level").ftype, Some(FieldType::Ordinal));
        let column = t.column("level").unwrap();
        let dict = column.as_dictionary::<UInt8Type>();
        let values = dict.values().as_string::<i32>();
        assert_eq!(
            values.iter().flatten().collect::<Vec<_>>(),
            ["low", "Medium", "High"]
        );
        assert_eq!(
            dict.keys().iter().collect::<Vec<_>>(),
            [Some(2), Some(0), None, Some(1), Some(0)]
        );
    }

    #[test]
    fn declarations_win_and_are_not_reported() {
        let v = Arc::new(Float64Array::from(vec![1.0, 2.0, 3.0]));
        let flags = Arc::new(BooleanArray::from(vec![true, false, true]));
        let t = table(vec![("v", v), ("flag", flags)]);
        let declared = BTreeMap::from([(
            "v".to_owned(),
            FieldMeta::new()
                .with_type(FieldType::Ordinal)
                .with_role(Role::Weight),
        )]);
        let mut diagnostics = Diagnostics::new();
        let t = infer_table(&t, &declared, &mut diagnostics).unwrap();
        assert_eq!(meta(&t, "v").ftype, Some(FieldType::Ordinal));
        assert_eq!(meta(&t, "v").role, Some(Role::Weight));
        assert!(
            diagnostics.iter().all(|d| !d.message.contains("Field `v`")),
            "{diagnostics:?}"
        );
        assert_eq!(meta(&t, "flag").role, Some(Role::Category));
    }

    #[test]
    fn reserved_columns_are_left_alone() {
        let prov = Arc::new(StringArray::from(vec!["exact", "exact"]));
        let (t, diagnostics) = infer(&table(vec![("__snr_prov", prov)]));
        assert_eq!(meta(&t, "__snr_prov"), FieldMeta::new());
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn concepts_come_from_units_before_names() {
        let a = Arc::new(Float64Array::from(vec![1.0, 2.0, 3.0]));
        let fields = [
            Field::new("level", DataType::Float64, true),
            Field::new("price_usd", DataType::Float64, true),
        ];
        let unit = FieldMeta::new().with_unit(crate::meta::Unit::new("hPa").unwrap());
        let schema = Arc::new(Schema::new(vec![
            unit.apply(fields[0].clone()),
            fields[1].clone(),
        ]));
        let batch = RecordBatch::try_new(schema, vec![a.clone(), a]).unwrap();
        let (t, _) = infer(&Table::from_batch(TableId(0), "t", batch));
        assert_eq!(meta(&t, "level").concept, Concept::new("pressure"));
        assert_eq!(meta(&t, "price_usd").concept, Concept::new("money"));
    }
}

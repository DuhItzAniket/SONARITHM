// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! JSON and NDJSON sources (10 §2): records or columns, found at a JSON pointer.
//!
//! Accepted shapes: an array of objects (records), an object of equal-length arrays
//! (columns), an array of arrays (rows, with a header row when the first row is all
//! strings) and an array of scalars (one `value` field). Without a pointer, a root object
//! with exactly one array member is read through that member, as many web APIs wrap their
//! data in `{"data": [...]}`. Nested objects are flattened with `.` (`pos.x`); nested arrays
//! are kept as JSON text. Values then go through the same type inference as CSV, so a
//! column of date strings becomes dates.

use std::collections::HashMap;
use std::fmt::Write as _;

use serde::Deserialize;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use sonarithm_core::{Diagnostic, Diagnostics};

use super::ReadOptions;
use super::columns::{RawColumn, text_table, unique_names};
use crate::bundle::Format;
use crate::codes;
use crate::error::DataError;
use crate::infer::text::TextOptions;
use crate::table::Table;

/// Options for JSON and NDJSON sources, as written in a spec's `data.source.options`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct JsonOptions {
    /// An RFC 6901 JSON pointer to the records or columns, such as `/data/items` (JSON
    /// only).
    pub pointer: Option<String>,
    /// A `strftime`-style format for date strings that are not ISO-8601.
    pub date_format: Option<String>,
}

impl JsonOptions {
    /// The default options: no pointer, ISO-8601 dates.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The same options reading the table at a JSON pointer.
    #[must_use]
    pub fn with_pointer(mut self, pointer: impl Into<String>) -> Self {
        self.pointer = Some(pointer.into());
        self
    }

    /// The same options with a `strftime`-style date format.
    #[must_use]
    pub fn with_date_format(mut self, format: impl Into<String>) -> Self {
        self.date_format = Some(format.into());
        self
    }
}

/// A JSON value that keeps object members in document order.
#[derive(Clone, Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl<'de> Deserialize<'de> for Json {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(JsonVisitor)
    }
}

struct JsonVisitor;

impl<'de> Visitor<'de> for JsonVisitor {
    type Value = Json;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a JSON value")
    }

    fn visit_unit<E>(self) -> Result<Json, E> {
        Ok(Json::Null)
    }

    fn visit_none<E>(self) -> Result<Json, E> {
        Ok(Json::Null)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Json, E> {
        Ok(Json::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Json, E> {
        Ok(Json::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Json, E> {
        Ok(Json::Number(value.into()))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Json, E> {
        serde_json::Number::from_f64(value)
            .map(Json::Number)
            .ok_or_else(|| E::custom("a JSON number must be finite"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Json, E> {
        Ok(Json::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Json, E> {
        Ok(Json::String(value))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Json, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(Json::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Json, A::Error> {
        let mut members: Vec<(String, Json)> = Vec::new();
        while let Some((key, value)) = map.next_entry::<String, Json>()? {
            // A repeated key: the last value wins, as in most JSON readers.
            match members.iter_mut().find(|(k, _)| *k == key) {
                Some(slot) => slot.1 = value,
                None => members.push((key, value)),
            }
        }
        Ok(Json::Object(members))
    }
}

/// Reads JSON or NDJSON into a table and the diagnostics that describe what was assumed.
pub(super) fn read_json(
    bytes: &[u8],
    name: &str,
    format: Format,
    options: &ReadOptions,
) -> Result<(Table, Diagnostics), DataError> {
    let json = &options.json;
    let unreadable = |reason: String| DataError::Unreadable { format, reason };
    let text = std::str::from_utf8(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes))
        .map_err(|err| unreadable(format!("the text is not UTF-8 ({err})")))?;
    let (shape, pointer) = if format == Format::Ndjson {
        let mut records = Vec::new();
        for (line_number, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let value: Json = serde_json::from_str(line)
                .map_err(|err| unreadable(format!("line {}: {err}", line_number + 1)))?;
            records.push(value);
            options
                .limits
                .check_shape(super::to_u64(records.len()), 1)?;
        }
        (Json::Array(records), String::new())
    } else {
        let root: Json = serde_json::from_str(text).map_err(|err| unreadable(err.to_string()))?;
        locate(root, json.pointer.as_deref())?
    };
    let (names, columns, kind) = tabulate(shape, &pointer, options)?;
    let mut diagnostics = Diagnostics::new();
    let (names, renamed) = unique_names(
        Some(&names.iter().map(|n| Some(n.as_str())).collect::<Vec<_>>()),
        names.len(),
    );
    let text_options = TextOptions {
        decimal_comma: false,
        date_format: json.date_format.clone(),
    };
    let table = text_table(
        name,
        &names,
        &columns,
        &text_options,
        options,
        &mut diagnostics,
    )?;
    let at = if pointer.is_empty() {
        "the root".to_owned()
    } else {
        format!("`{pointer}`")
    };
    diagnostics.push(Diagnostic::new(
        &codes::JSON_SHAPE,
        format!(
            "Read {kind} at {at}: {} fields, {} rows.",
            table.num_fields(),
            table.num_rows()
        ),
    ));
    if renamed > 0 {
        diagnostics.push(Diagnostic::new(
            &codes::COLUMN_NAMES_CHANGED,
            format!("{renamed} field name(s) were blank or repeated and were replaced."),
        ));
    }
    Ok((table, diagnostics))
}

/// Follows the pointer, or unwraps a root object with exactly one array member.
fn locate(root: Json, pointer: Option<&str>) -> Result<(Json, String), DataError> {
    let not_found = |pointer: &str| DataError::NotTabular {
        pointer: pointer.to_owned(),
        found: "nothing",
    };
    match pointer {
        Some(pointer) => {
            if !(pointer.is_empty() || pointer.starts_with('/')) {
                return Err(DataError::InvalidOption {
                    option: "pointer",
                    reason: "use an RFC 6901 JSON pointer such as /data/items",
                });
            }
            let mut value = root;
            for token in pointer.split('/').skip(1) {
                let token = token.replace("~1", "/").replace("~0", "~");
                value = match value {
                    Json::Object(members) => members
                        .into_iter()
                        .find(|(key, _)| *key == token)
                        .map(|(_, v)| v)
                        .ok_or_else(|| not_found(pointer))?,
                    Json::Array(items) => token
                        .parse::<usize>()
                        .ok()
                        .and_then(|index| items.into_iter().nth(index))
                        .ok_or_else(|| not_found(pointer))?,
                    _ => return Err(not_found(pointer)),
                };
            }
            Ok((value, pointer.to_owned()))
        }
        None => match root {
            Json::Object(members) if !is_columns(&members) => {
                let mut arrays = members
                    .into_iter()
                    .filter(|(_, v)| matches!(v, Json::Array(_)));
                match (arrays.next(), arrays.next()) {
                    (Some((key, value)), None) => {
                        let pointer = format!("/{}", key.replace('~', "~0").replace('/', "~1"));
                        Ok((value, pointer))
                    }
                    _ => Err(DataError::NotTabular {
                        pointer: String::new(),
                        found: "an object without one data array",
                    }),
                }
            }
            other => Ok((other, String::new())),
        },
    }
}

/// An object whose members are all arrays of one length: columns.
fn is_columns(members: &[(String, Json)]) -> bool {
    let mut lengths = members.iter().map(|(_, value)| match value {
        Json::Array(items) => Some(items.len()),
        _ => None,
    });
    match lengths.next() {
        Some(Some(first)) => members.len() > 1 && lengths.all(|len| len == Some(first)),
        _ => false,
    }
}

/// Field names, text columns and a description of the shape.
fn tabulate(
    shape: Json,
    pointer: &str,
    options: &ReadOptions,
) -> Result<(Vec<String>, Vec<RawColumn>, &'static str), DataError> {
    let not_tabular = |found| DataError::NotTabular {
        pointer: pointer.to_owned(),
        found,
    };
    match shape {
        Json::Object(members) if is_columns(&members) => {
            options.limits.check_shape(0, members.len())?;
            let mut names = Vec::with_capacity(members.len());
            let mut columns = Vec::with_capacity(members.len());
            for (key, value) in members {
                let mut column = RawColumn::default();
                if let Json::Array(items) = value {
                    options
                        .limits
                        .check_shape(super::to_u64(items.len()), names.len() + 1)?;
                    for item in &items {
                        column.push(&cell_text(item));
                    }
                }
                names.push(key);
                columns.push(column);
            }
            Ok((names, columns, "columns"))
        }
        Json::Array(items) => match items.first() {
            Some(Json::Object(_)) => records(&items, options).map(|(n, c)| (n, c, "records")),
            Some(Json::Array(_)) => rows(&items, options).map(|(n, c)| (n, c, "rows")),
            None => Ok((Vec::new(), Vec::new(), "records")),
            Some(_) => {
                options.limits.check_shape(super::to_u64(items.len()), 1)?;
                let mut column = RawColumn::default();
                for item in &items {
                    column.push(&cell_text(item));
                }
                Ok((vec!["value".to_owned()], vec![column], "values"))
            }
        },
        Json::Object(_) => Err(not_tabular("an object")),
        _ => Err(not_tabular("a single value")),
    }
}

/// Records: fields in order of first appearance; nested objects flattened with `.`.
fn records(
    items: &[Json],
    options: &ReadOptions,
) -> Result<(Vec<String>, Vec<RawColumn>), DataError> {
    let mut names: Vec<String> = Vec::new();
    // Lookups only; field order comes from `names`.
    let mut index_of: HashMap<String, usize> = HashMap::new();
    let mut columns: Vec<RawColumn> = Vec::new();
    let mut flat: Vec<(String, String)> = Vec::new();
    for (row, item) in items.iter().enumerate() {
        flat.clear();
        match item {
            Json::Object(members) => flatten("", members, &mut flat),
            other => flat.push(("value".to_owned(), cell_text(other))),
        }
        let mut seen = vec![false; columns.len()];
        for (key, text) in &flat {
            let index = if let Some(&index) = index_of.get(key) {
                index
            } else {
                index_of.insert(key.clone(), names.len());
                names.push(key.clone());
                let mut column = RawColumn::default();
                for _ in 0..row {
                    column.push("");
                }
                columns.push(column);
                seen.push(false);
                names.len() - 1
            };
            if let (Some(column), Some(mark)) = (columns.get_mut(index), seen.get_mut(index))
                && !*mark
            {
                column.push(text);
                *mark = true;
            }
        }
        for (column, mark) in columns.iter_mut().zip(&seen) {
            if !mark {
                column.push("");
            }
        }
        options
            .limits
            .check_shape(super::to_u64(row + 1), names.len())?;
    }
    Ok((names, columns))
}

/// Rows of arrays; the first row names the fields when all its values are strings.
fn rows(items: &[Json], options: &ReadOptions) -> Result<(Vec<String>, Vec<RawColumn>), DataError> {
    let width = items
        .iter()
        .map(|item| match item {
            Json::Array(values) => values.len(),
            _ => 1,
        })
        .max()
        .unwrap_or(0);
    let header = match items.first() {
        Some(Json::Array(first))
            if items.len() > 1 && first.iter().all(|v| matches!(v, Json::String(_))) =>
        {
            Some(first.iter().map(cell_text).collect::<Vec<_>>())
        }
        _ => None,
    };
    let names: Vec<String> = (0..width)
        .map(|i| {
            header
                .as_ref()
                .and_then(|h| h.get(i).cloned())
                .unwrap_or_default()
        })
        .collect();
    let mut columns: Vec<RawColumn> = (0..width).map(|_| RawColumn::default()).collect();
    for (row, item) in items.iter().enumerate().skip(usize::from(header.is_some())) {
        let values: &[Json] = match item {
            Json::Array(values) => values,
            other => std::slice::from_ref(other),
        };
        for (index, column) in columns.iter_mut().enumerate() {
            column.push(&values.get(index).map(cell_text).unwrap_or_default());
        }
        options.limits.check_shape(super::to_u64(row + 1), width)?;
    }
    Ok((names, columns))
}

fn flatten(prefix: &str, members: &[(String, Json)], out: &mut Vec<(String, String)>) {
    for (key, value) in members {
        let name = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        match value {
            Json::Object(inner) if !inner.is_empty() => flatten(&name, inner, out),
            other => out.push((name, cell_text(other))),
        }
    }
}

/// A value as the text type inference reads: numbers in their JSON form (which round-trips
/// exactly), `true`/`false`, empty for null, JSON text for arrays and objects.
fn cell_text(value: &Json) -> String {
    match value {
        Json::Null => String::new(),
        Json::Bool(b) => b.to_string(),
        Json::Number(n) => n.to_string(),
        Json::String(s) => s.clone(),
        nested => {
            let mut out = String::new();
            write_json(nested, &mut out);
            out
        }
    }
}

fn write_json(value: &Json, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => {
            let _ = write!(out, "{b}");
        }
        Json::Number(n) => {
            let _ = write!(out, "{n}");
        }
        Json::String(s) => out.push_str(&serde_json::Value::String(s.clone()).to_string()),
        Json::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_json(item, out);
            }
            out.push(']');
        }
        Json::Object(members) => {
            out.push('{');
            for (i, (key, item)) in members.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::Value::String(key.clone()).to_string());
                out.push(':');
                write_json(item, out);
            }
            out.push('}');
        }
    }
}

#[cfg(test)]
mod tests {
    use arrow_array::Array;
    use arrow_array::cast::AsArray;
    use arrow_array::types::{Date32Type, Float64Type, Int64Type};
    use arrow_schema::DataType;

    use super::*;
    use crate::source::read_bytes;

    fn read(
        text: &str,
        format: Format,
        pointer: Option<&str>,
    ) -> Result<(Table, Vec<String>), DataError> {
        let json = JsonOptions {
            pointer: pointer.map(str::to_owned),
            ..JsonOptions::default()
        };
        let options = ReadOptions::new().with_json(json);
        let loaded = read_bytes(text.as_bytes(), format, "api/items.json", &options)?;
        let codes = loaded
            .diagnostics
            .iter()
            .map(|d| d.code.as_str().to_owned())
            .collect();
        Ok((loaded.bundle.primary().unwrap().clone(), codes))
    }

    fn names(table: &Table) -> Vec<String> {
        table
            .schema()
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect()
    }

    #[test]
    fn records_keep_field_order_and_types() {
        let (table, codes) = read(
            r#"[{"t": "2024-01-01", "v": 1.5, "n": 3, "ok": true},
                {"t": "2024-01-02", "v": null, "n": 4, "ok": false}]"#,
            Format::Json,
            None,
        )
        .unwrap();
        assert_eq!(names(&table), ["t", "v", "n", "ok"]);
        let types: Vec<_> = table
            .schema()
            .fields()
            .iter()
            .map(|f| f.data_type().clone())
            .collect();
        assert_eq!(
            types,
            [
                DataType::Date32,
                DataType::Float64,
                DataType::Int64,
                DataType::Boolean
            ]
        );
        assert!(table.column("v").unwrap().is_null(1));
        assert_eq!(codes, ["SNR-DATA-020"]);
    }

    #[test]
    fn missing_keys_become_missing_values_and_nested_objects_flatten() {
        let (table, _) = read(
            r#"[{"id": "a", "pos": {"x": 1, "y": 2}}, {"id": "b", "tags": [1, 2]}]"#,
            Format::Json,
            None,
        )
        .unwrap();
        assert_eq!(names(&table), ["id", "pos.x", "pos.y", "tags"]);
        assert!(table.column("pos.x").unwrap().is_null(1));
        assert_eq!(
            table.column("tags").unwrap().as_string::<i32>().value(1),
            "[1,2]"
        );
    }

    #[test]
    fn a_single_data_array_is_found_without_a_pointer() {
        let (table, _) = read(
            r#"{"meta": {"n": 2}, "data": [{"a": 1}, {"a": 2}]}"#,
            Format::Json,
            None,
        )
        .unwrap();
        assert_eq!(table.num_rows(), 2);
        let err = read(
            r#"{"a": [1], "b": {"c": 2}, "d": [3, 4]}"#,
            Format::Json,
            None,
        )
        .unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-021");
    }

    #[test]
    fn pointers_and_column_objects() {
        let doc = r#"{"result": {"series": {"x": [1, 2, 3], "y": [0.5, 1.5, 2.5]}}}"#;
        let (table, _) = read(doc, Format::Json, Some("/result/series")).unwrap();
        assert_eq!(names(&table), ["x", "y"]);
        let y = table.column("y").unwrap();
        assert_eq!(
            y.as_primitive::<Float64Type>().values().to_vec(),
            [0.5, 1.5, 2.5]
        );
        let err = read(doc, Format::Json, Some("/result/missing")).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-021");
        let err = read(doc, Format::Json, Some("result")).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-006");
    }

    #[test]
    fn rows_with_a_header_and_plain_values() {
        let (table, _) = read(
            r#"[["day", "n"], ["2024-02-01", 3], ["2024-02-02", 5]]"#,
            Format::Json,
            None,
        )
        .unwrap();
        assert_eq!(names(&table), ["day", "n"]);
        assert_eq!(
            table
                .column("day")
                .unwrap()
                .as_primitive::<Date32Type>()
                .len(),
            2
        );
        let (table, _) = read("[1, 2, 3]", Format::Json, None).unwrap();
        assert_eq!(names(&table), ["value"]);
        assert_eq!(
            table
                .column("value")
                .unwrap()
                .as_primitive::<Int64Type>()
                .value(2),
            3
        );
    }

    #[test]
    fn ndjson_reads_one_record_per_line() {
        let text = "{\"a\": 1, \"b\": \"x\"}\n\n{\"a\": 2}\n{\"b\": \"y\", \"a\": 3}\n";
        let (table, _) = read(text, Format::Ndjson, None).unwrap();
        assert_eq!(names(&table), ["a", "b"]);
        assert_eq!(table.num_rows(), 3);
        let err = read("{\"a\": 1}\n{oops}\n", Format::Ndjson, None).unwrap_err();
        assert!(err.to_string().contains("line 2"), "{err}");
    }

    #[test]
    fn big_integers_and_exact_floats_survive() {
        let (table, _) =
            read(r#"[{"n": 9007199254740993, "f": 0.1}]"#, Format::Json, None).unwrap();
        assert_eq!(
            table
                .column("n")
                .unwrap()
                .as_primitive::<Int64Type>()
                .value(0),
            9_007_199_254_740_993
        );
        assert_eq!(
            table
                .column("f")
                .unwrap()
                .as_primitive::<Float64Type>()
                .value(0)
                .to_bits(),
            0.1_f64.to_bits()
        );
    }

    #[test]
    fn scalars_are_not_tables() {
        let err = read("42", Format::Json, None).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-021");
        let err = read("[1,", Format::Json, None).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-001");
    }
}

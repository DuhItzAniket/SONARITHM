// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Data fingerprints: SHA-256 over an encoding of the data's logical values (03 §2,
//! ADR-0013, [RFC-0008]). This is scheme [`FINGERPRINT_SCHEME`] (2).
//!
//! The input is fed to SHA-256 as a stream (RFC-0008 §2):
//!
//! 1. the tag `sonarithm-data table fingerprint v2` and a NUL byte;
//! 2. the schema: a u64 byte length, then RFC 8785 canonical JSON `{ "fields": [{ "meta",
//!    "name", "type" }, …], "meta" }`, where `type` is the logical type (§3) and each
//!    `meta` holds only the `snr.*` metadata keys (of the field, and of the schema);
//! 3. the row count, u64;
//! 4. each column in schema order, preceded by its byte length (u64): every row in order
//!    across all batches, as a tag byte (`0` null, `1` valid) and, for a valid row, the
//!    value bytes of §3.
//!
//! Integers are little-endian u64 throughout. What is left out is layout, and nothing a
//! compile path reads: batch boundaries, offsets, slicing, padding, the bytes under nulls,
//! string and list layouts (Utf8, LargeUtf8 and Utf8View are all `string`), dictionary key
//! widths, run ends, nullability, non-`snr.*` metadata, and the table's name, ID and
//! generation. Floats are hashed bit-exact (§4), because compile paths order them with
//! `f64::total_cmp`. A dictionary stays distinct from a plain column, because NUMISONA takes
//! a category order from its dictionary (§3).
//!
//! The digest does not depend on the Arrow version, and is the same on every platform
//! (the pinned corpus test runs on Linux, Windows and macOS in CI).
//!
//! Scheme 1, SHA-256 over the Arrow IPC bytes of a canonical copy (A04-T09), is kept as
//! [`table_fingerprint_v1`] for the RFC-0008 bench only: no record is written with it, and
//! a scheme-1 fingerprint is never recomputed, since it depends on the arrow-ipc version
//! that wrote it (RFC-0008 §6).
//!
//! [RFC-0008]: ../../docs/rfcs/0008-value-based-data-fingerprint.md

use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::{
    ArrowDictionaryKeyType, ArrowPrimitiveType, Date32Type, Date64Type, Decimal128Type,
    Decimal256Type, DurationMicrosecondType, DurationMillisecondType, DurationNanosecondType,
    DurationSecondType, Float16Type, Float32Type, Float64Type, Int8Type, Int16Type, Int32Type,
    Int64Type, Time32MillisecondType, Time32SecondType, Time64MicrosecondType,
    Time64NanosecondType, TimestampMicrosecondType, TimestampMillisecondType,
    TimestampNanosecondType, TimestampSecondType, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use arrow_array::{
    Array, ArrayRef, BinaryArray, BooleanArray, DictionaryArray, LargeBinaryArray,
    LargeStringArray, PrimitiveArray, RecordBatch, RecordBatchOptions, StringArray,
};
use arrow_ipc::writer::StreamWriter;
use arrow_schema::{DataType, Field, FieldRef, Schema, TimeUnit, UnionMode};
use sonarithm_core::{Sha256Digest, Sha256Hasher};

use crate::bundle::DataBundle;
use crate::error::DataError;
use crate::table::Table;

/// The fingerprint scheme this build computes (RFC-0008 §6). Records name it beside every
/// fingerprint; a record without it was written under scheme 1.
pub const FINGERPRINT_SCHEME: u32 = 2;

/// The tag that starts a table fingerprint's input.
const TABLE_TAG: &[u8] = b"sonarithm-data table fingerprint v2\0";

/// The tag that starts a bundle fingerprint's input (RFC-0008 §5).
const BUNDLE_TAG: &[u8] = b"sonarithm-data bundle fingerprint v2\0";

/// The fingerprint of one table's content (scheme 2).
///
/// # Errors
///
/// [`DataError::Unfingerprintable`] for a field whose type has no encoding (intervals,
/// 32- and 64-bit decimals, and the rarer Arrow types), and [`DataError::MalformedTable`]
/// if batches with different dictionaries cannot be concatenated.
pub fn table_fingerprint(table: &Table) -> Result<Sha256Digest, DataError> {
    let schema = table.schema();
    let types = schema
        .fields()
        .iter()
        .map(|field| {
            logical_type(field.data_type()).ok_or_else(|| DataError::Unfingerprintable {
                field: field.name().clone(),
                data_type: field.data_type().to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let header = schema_json(schema, &types);
    let mut out = Hashing::new();
    out.put(TABLE_TAG);
    out.put(&(header.len() as u64).to_le_bytes());
    out.put(header.as_bytes());
    out.put(&table.num_rows().to_le_bytes());
    for (index, field) in schema.fields().iter().enumerate() {
        let parts = column_parts(table, index, field)?;
        let mut counted = Counting(0);
        encode_column(&parts, &mut counted);
        out.put(&counted.0.to_le_bytes());
        encode_column(&parts, &mut out);
    }
    Ok(out.finish())
}

/// The fingerprint of a bundle: SHA-256 over a version tag and its tables' fingerprints,
/// in table order (RFC-0008 §5). Render manifests and Score headers record it (ADR-0013).
///
/// # Errors
///
/// As [`table_fingerprint`].
pub fn bundle_fingerprint(bundle: &DataBundle) -> Result<Sha256Digest, DataError> {
    let mut input = BUNDLE_TAG.to_vec();
    for table in bundle.tables() {
        input.extend_from_slice(table_fingerprint(table)?.as_bytes());
    }
    Ok(Sha256Digest::of(&input))
}

/// Where encoded bytes go: a counter (for a column's length) or the hasher.
trait Sink {
    fn put(&mut self, bytes: &[u8]);

    fn tag(&mut self, valid: bool) {
        self.put(&[u8::from(valid)]);
    }

    fn len(&mut self, len: usize) {
        self.put(&(len as u64).to_le_bytes());
    }
}

/// Counts bytes, so a column's length can precede it without buffering it.
struct Counting(u64);

impl Sink for Counting {
    fn put(&mut self, bytes: &[u8]) {
        self.0 += bytes.len() as u64;
    }
}

/// Feeds SHA-256 through a small buffer, so one-byte tags do not each reach the hasher.
struct Hashing {
    hasher: Sha256Hasher,
    buffer: Vec<u8>,
}

impl Hashing {
    const FLUSH: usize = 64 << 10;

    fn new() -> Self {
        Self {
            hasher: Sha256Hasher::new(),
            buffer: Vec::with_capacity(Self::FLUSH),
        }
    }

    fn finish(mut self) -> Sha256Digest {
        self.hasher.update(&self.buffer);
        self.hasher.finish()
    }
}

impl Sink for Hashing {
    fn put(&mut self, bytes: &[u8]) {
        if self.buffer.len() + bytes.len() > Self::FLUSH {
            self.hasher.update(&self.buffer);
            self.buffer.clear();
        }
        if bytes.len() > Self::FLUSH {
            self.hasher.update(bytes);
        } else {
            self.buffer.extend_from_slice(bytes);
        }
    }
}

/// The logical type of RFC-0008 §3, or `None` for a type with no encoding. Layouts that
/// hold the same values share one name: the string, binary and list layouts, key widths of
/// dictionaries, sparse and dense unions, and run-end encoding (the values' type).
fn logical_type(data_type: &DataType) -> Option<String> {
    let unit = |unit: &TimeUnit| match unit {
        TimeUnit::Second => "s",
        TimeUnit::Millisecond => "ms",
        TimeUnit::Microsecond => "us",
        TimeUnit::Nanosecond => "ns",
    };
    let child = |field: &FieldRef| logical_type(field.data_type());
    Some(match data_type {
        DataType::Null => "null".to_owned(),
        DataType::Boolean => "bool".to_owned(),
        DataType::Int8 => "int8".to_owned(),
        DataType::Int16 => "int16".to_owned(),
        DataType::Int32 => "int32".to_owned(),
        DataType::Int64 => "int64".to_owned(),
        DataType::UInt8 => "uint8".to_owned(),
        DataType::UInt16 => "uint16".to_owned(),
        DataType::UInt32 => "uint32".to_owned(),
        DataType::UInt64 => "uint64".to_owned(),
        DataType::Float16 => "float16".to_owned(),
        DataType::Float32 => "float32".to_owned(),
        DataType::Float64 => "float64".to_owned(),
        DataType::Decimal128(p, s) | DataType::Decimal256(p, s) => format!("decimal({p},{s})"),
        DataType::Date32 => "date32".to_owned(),
        DataType::Date64 => "date64".to_owned(),
        DataType::Time32(u) | DataType::Time64(u) => format!("time({})", unit(u)),
        DataType::Duration(u) => format!("duration({})", unit(u)),
        DataType::Timestamp(u, None) => format!("timestamp({})", unit(u)),
        DataType::Timestamp(u, Some(tz)) => format!("timestamp({},{tz})", unit(u)),
        DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View => "string".to_owned(),
        DataType::Binary | DataType::LargeBinary | DataType::BinaryView => "binary".to_owned(),
        DataType::FixedSizeBinary(n) => format!("fixed_binary({n})"),
        DataType::Dictionary(key, values) if key.is_dictionary_key_type() => {
            format!("dictionary<{}>", logical_type(values)?)
        }
        DataType::List(f)
        | DataType::LargeList(f)
        | DataType::ListView(f)
        | DataType::LargeListView(f) => format!("list<{}>", child(f)?),
        DataType::FixedSizeList(f, n) => format!("fixed_list<{},{n}>", child(f)?),
        DataType::Struct(fields) => {
            let children = fields
                .iter()
                .map(|f| Some(format!("{}:{}", quoted(f.name()), child(f)?)))
                .collect::<Option<Vec<_>>>()?;
            format!("struct{{{}}}", children.join(","))
        }
        DataType::Map(entries, sorted) => {
            let DataType::Struct(kv) = entries.data_type() else {
                return None;
            };
            let [key, value] = kv.iter().collect::<Vec<_>>()[..] else {
                return None;
            };
            let order = if *sorted { "sorted_map" } else { "map" };
            format!("{order}<{},{}>", child(key)?, child(value)?)
        }
        DataType::Union(fields, _) => {
            let children = fields
                .iter()
                .map(|(id, f)| Some(format!("{id} {}:{}", quoted(f.name()), child(f)?)))
                .collect::<Option<Vec<_>>>()?;
            format!("union{{{}}}", children.join(","))
        }
        DataType::RunEndEncoded(_, values) => child(values)?,
        _ => return None,
    })
}

/// A child name inside a logical type, as a JSON string, so no name can imitate the
/// type's punctuation.
fn quoted(name: &str) -> String {
    let mut out = String::new();
    json_string(&mut out, name);
    out
}

/// The schema part of the input: RFC 8785 canonical JSON. It holds only objects, arrays
/// and strings, so the canonical form needs no number formatting: keys sorted by UTF-16
/// code units, no whitespace, and strings escaped as RFC 8785 §3.2.2.2 says.
/// (`sonarithm-data` may depend only on `sonarithm-core`, so it cannot use
/// `sonarithm_spec::canonical`; the facade's tests check the two agree.)
fn schema_json(schema: &Schema, types: &[String]) -> String {
    let mut out = String::from("{\"fields\":[");
    for (index, (field, logical)) in schema.fields().iter().zip(types).enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str("{\"meta\":");
        snr_meta(&mut out, field.metadata().iter());
        out.push_str(",\"name\":");
        json_string(&mut out, field.name());
        out.push_str(",\"type\":");
        json_string(&mut out, logical);
        out.push('}');
    }
    out.push_str("],\"meta\":");
    snr_meta(&mut out, schema.metadata().iter());
    out.push('}');
    out
}

/// The `snr.*` entries of `metadata` as a canonical JSON object.
fn snr_meta<'a>(out: &mut String, metadata: impl Iterator<Item = (&'a String, &'a String)>) {
    let mut entries: Vec<(&String, &String)> = metadata
        .filter(|(key, _)| key.starts_with("snr."))
        .collect();
    entries.sort_by(|a, b| a.0.encode_utf16().cmp(b.0.encode_utf16()));
    out.push('{');
    for (index, (key, value)) in entries.into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        json_string(out, key);
        out.push(':');
        json_string(out, value);
    }
    out.push('}');
}

/// A JSON string as RFC 8785 writes it (ECMAScript `JSON.stringify`).
fn json_string(out: &mut String, text: &str) {
    use std::fmt::Write as _;
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// One column's arrays, one per batch. Batches of a dictionary column whose dictionaries
/// differ are concatenated first, as the compile paths see them through [`Table::column`],
/// so the column has one dictionary (RFC-0008 §3).
fn column_parts(table: &Table, index: usize, field: &Field) -> Result<Vec<ArrayRef>, DataError> {
    let parts: Vec<ArrayRef> = table
        .batches()
        .iter()
        .map(|batch| Arc::clone(batch.column(index)))
        .collect();
    if !matches!(field.data_type(), DataType::Dictionary(..)) || parts.len() < 2 {
        return Ok(parts);
    }
    let values = |array: &ArrayRef| {
        array
            .as_any_dictionary_opt()
            .map(|dictionary| dictionary.values().to_data())
    };
    let first = values(&parts[0]);
    if parts.iter().all(|part| values(part) == first) {
        return Ok(parts);
    }
    Ok(vec![table.column(field.name())?])
}

/// Every row of a column, in order across its parts.
fn encode_column(parts: &[ArrayRef], out: &mut dyn Sink) {
    let dictionary = parts.first().and_then(|part| part.as_any_dictionary_opt());
    if let Some(first) = dictionary {
        // The dictionary's values as a column (all of them, in order), then each row's key
        // as an index into them: key widths are layout, unused values and order are not.
        let values = first.values();
        out.len(values.len());
        encode_rows(values.as_ref(), out);
        for part in parts {
            if let Some(dictionary) = part.as_any_dictionary_opt() {
                let keys = dictionary.normalized_keys();
                for (row, key) in keys.into_iter().enumerate() {
                    let valid = dictionary.keys().is_valid(row);
                    out.tag(valid);
                    if valid {
                        out.put(&(key as u64).to_le_bytes());
                    }
                }
            }
        }
        return;
    }
    for part in parts {
        encode_rows(part.as_ref(), out);
    }
}

/// Every row of `array`: a tag byte, then a valid row's value bytes.
fn encode_rows(array: &dyn Array, out: &mut dyn Sink) {
    macro_rules! fixed {
        ($t:ty) => {{
            let typed = array.as_primitive::<$t>();
            for row in 0..typed.len() {
                let valid = typed.is_valid(row);
                out.tag(valid);
                if valid {
                    out.put(&typed.value(row).to_le_bytes());
                }
            }
        }};
    }
    match array.data_type() {
        DataType::Int8 => fixed!(Int8Type),
        DataType::Int16 => fixed!(Int16Type),
        DataType::Int32 => fixed!(Int32Type),
        DataType::Int64 => fixed!(Int64Type),
        DataType::UInt8 => fixed!(UInt8Type),
        DataType::UInt16 => fixed!(UInt16Type),
        DataType::UInt32 => fixed!(UInt32Type),
        DataType::UInt64 => fixed!(UInt64Type),
        DataType::Float32 => fixed!(Float32Type),
        DataType::Float64 => fixed!(Float64Type),
        DataType::Date32 => fixed!(Date32Type),
        DataType::Date64 => fixed!(Date64Type),
        DataType::Timestamp(TimeUnit::Second, _) => fixed!(TimestampSecondType),
        DataType::Timestamp(TimeUnit::Millisecond, _) => fixed!(TimestampMillisecondType),
        DataType::Timestamp(TimeUnit::Microsecond, _) => fixed!(TimestampMicrosecondType),
        DataType::Timestamp(TimeUnit::Nanosecond, _) => fixed!(TimestampNanosecondType),
        _ => {
            for row in 0..array.len() {
                encode_row(array, row, out);
            }
        }
    }
}

/// One row of `array`: its tag byte and, when valid, its value bytes (RFC-0008 §3).
fn encode_row(array: &dyn Array, row: usize, out: &mut dyn Sink) {
    match array.data_type() {
        DataType::Null => out.tag(false),
        // A run's value, whatever the run ends.
        DataType::RunEndEncoded(ends, _) => {
            let (values, physical) = match ends.data_type() {
                DataType::Int16 => run_value::<Int16Type>(array, row),
                DataType::Int32 => run_value::<Int32Type>(array, row),
                DataType::Int64 => run_value::<Int64Type>(array, row),
                _ => return out.tag(false),
            };
            match values {
                Some(values) => encode_row(values.as_ref(), physical, out),
                None => out.tag(false),
            }
        }
        // Inside a nested value a dictionary is resolved: its key selects a value row.
        DataType::Dictionary(..) => match array.as_any_dictionary_opt() {
            Some(dictionary) if dictionary.keys().is_valid(row) => {
                let key = dictionary.normalized_keys()[row];
                encode_row(dictionary.values().as_ref(), key, out);
            }
            _ => out.tag(false),
        },
        // A union slot is always present: the type ID, then the selected child's row.
        DataType::Union(_, mode) => {
            let union = array.as_union();
            let type_id = union.type_id(row);
            out.tag(true);
            out.put(&type_id.to_le_bytes());
            let at = match mode {
                UnionMode::Dense => union.value_offset(row),
                UnionMode::Sparse => row,
            };
            encode_row(union.child(type_id).as_ref(), at, out);
        }
        _ => {
            let valid = array.is_valid(row);
            out.tag(valid);
            if valid {
                encode_value(array, row, out);
            }
        }
    }
}

/// The values array and the physical index of a run-end encoded array's `row`.
fn run_value<R: arrow_array::types::RunEndIndexType>(
    array: &dyn Array,
    row: usize,
) -> (Option<&ArrayRef>, usize) {
    match array.as_run_opt::<R>() {
        Some(run) => (Some(run.values()), run.get_physical_index(row)),
        None => (None, 0),
    }
}

/// The value bytes of a valid row (RFC-0008 §3's table).
fn encode_value(array: &dyn Array, row: usize, out: &mut dyn Sink) {
    macro_rules! fixed {
        ($t:ty) => {
            out.put(&array.as_primitive::<$t>().value(row).to_le_bytes())
        };
    }
    let bytes = |out: &mut dyn Sink, value: &[u8]| {
        out.len(value.len());
        out.put(value);
    };
    let elements = |out: &mut dyn Sink, values: &ArrayRef, counted: bool| {
        if counted {
            out.len(values.len());
        }
        for index in 0..values.len() {
            encode_row(values.as_ref(), index, out);
        }
    };
    match array.data_type() {
        DataType::Boolean => out.put(&[u8::from(array.as_boolean().value(row))]),
        DataType::Int8 => fixed!(Int8Type),
        DataType::Int16 => fixed!(Int16Type),
        DataType::Int32 => fixed!(Int32Type),
        DataType::Int64 => fixed!(Int64Type),
        DataType::UInt8 => fixed!(UInt8Type),
        DataType::UInt16 => fixed!(UInt16Type),
        DataType::UInt32 => fixed!(UInt32Type),
        DataType::UInt64 => fixed!(UInt64Type),
        DataType::Float16 => fixed!(Float16Type),
        DataType::Float32 => fixed!(Float32Type),
        DataType::Float64 => fixed!(Float64Type),
        DataType::Decimal128(..) => fixed!(Decimal128Type),
        DataType::Decimal256(..) => fixed!(Decimal256Type),
        DataType::Date32 => fixed!(Date32Type),
        DataType::Date64 => fixed!(Date64Type),
        DataType::Time32(TimeUnit::Second) => fixed!(Time32SecondType),
        DataType::Time32(_) => fixed!(Time32MillisecondType),
        DataType::Time64(TimeUnit::Microsecond) => fixed!(Time64MicrosecondType),
        DataType::Time64(_) => fixed!(Time64NanosecondType),
        DataType::Duration(TimeUnit::Second) => fixed!(DurationSecondType),
        DataType::Duration(TimeUnit::Millisecond) => fixed!(DurationMillisecondType),
        DataType::Duration(TimeUnit::Microsecond) => fixed!(DurationMicrosecondType),
        DataType::Duration(TimeUnit::Nanosecond) => fixed!(DurationNanosecondType),
        DataType::Timestamp(TimeUnit::Second, _) => fixed!(TimestampSecondType),
        DataType::Timestamp(TimeUnit::Millisecond, _) => fixed!(TimestampMillisecondType),
        DataType::Timestamp(TimeUnit::Microsecond, _) => fixed!(TimestampMicrosecondType),
        DataType::Timestamp(TimeUnit::Nanosecond, _) => fixed!(TimestampNanosecondType),
        DataType::Utf8 => bytes(out, array.as_string::<i32>().value(row).as_bytes()),
        DataType::LargeUtf8 => bytes(out, array.as_string::<i64>().value(row).as_bytes()),
        DataType::Utf8View => bytes(out, array.as_string_view().value(row).as_bytes()),
        DataType::Binary => bytes(out, array.as_binary::<i32>().value(row)),
        DataType::LargeBinary => bytes(out, array.as_binary::<i64>().value(row)),
        DataType::BinaryView => bytes(out, array.as_binary_view().value(row)),
        DataType::FixedSizeBinary(_) => out.put(array.as_fixed_size_binary().value(row)),
        DataType::List(_) => elements(out, &array.as_list::<i32>().value(row), true),
        DataType::LargeList(_) => elements(out, &array.as_list::<i64>().value(row), true),
        DataType::ListView(_) => elements(out, &array.as_list_view::<i32>().value(row), true),
        DataType::LargeListView(_) => {
            elements(out, &array.as_list_view::<i64>().value(row), true);
        }
        DataType::FixedSizeList(..) => {
            elements(out, &array.as_fixed_size_list().value(row), false);
        }
        DataType::Struct(_) => {
            for column in array.as_struct().columns() {
                encode_row(column.as_ref(), row, out);
            }
        }
        DataType::Map(..) => {
            let entries: ArrayRef = Arc::new(array.as_map().value(row));
            elements(out, &entries, true);
        }
        // `logical_type` refused every other type before encoding started.
        _ => {}
    }
}

/// The scheme-1 fingerprint: SHA-256 over the Arrow IPC stream of a canonical copy of the
/// table (A04-T09). Kept for the RFC-0008 §8 bench, which compares the two schemes; no
/// record is written with it any more.
///
/// # Errors
///
/// [`DataError::Write`] if Arrow cannot encode the table.
#[doc(hidden)]
pub fn table_fingerprint_v1(table: &Table) -> Result<Sha256Digest, DataError> {
    let failed = |reason: String| DataError::Write {
        format: crate::bundle::Format::Arrow,
        reason,
    };
    let batch = table.to_batch()?;
    let columns: Vec<ArrayRef> = batch.columns().iter().map(canonical).collect();
    let options = RecordBatchOptions::new().with_row_count(Some(batch.num_rows()));
    let batch = RecordBatch::try_new_with_options(Arc::clone(table.schema()), columns, &options)
        .map_err(|err| failed(err.to_string()))?;
    let mut writer =
        StreamWriter::try_new(Vec::new(), table.schema()).map_err(|err| failed(err.to_string()))?;
    writer
        .write(&batch)
        .map_err(|err| failed(err.to_string()))?;
    writer.finish().map_err(|err| failed(err.to_string()))?;
    let bytes = writer.into_inner().map_err(|err| failed(err.to_string()))?;
    Ok(Sha256Digest::of(&bytes))
}

/// A canonical copy of an array for scheme 1: zeroed slots under nulls, offsets from zero.
fn canonical(array: &ArrayRef) -> ArrayRef {
    macro_rules! primitive {
        ($t:ty) => {
            rebuild_primitive::<$t>(array)
        };
    }
    match array.data_type() {
        DataType::Boolean => Arc::new(array.as_boolean().iter().collect::<BooleanArray>()),
        DataType::Int8 => primitive!(Int8Type),
        DataType::Int16 => primitive!(Int16Type),
        DataType::Int32 => primitive!(Int32Type),
        DataType::Int64 => primitive!(Int64Type),
        DataType::UInt8 => primitive!(UInt8Type),
        DataType::UInt16 => primitive!(UInt16Type),
        DataType::UInt32 => primitive!(UInt32Type),
        DataType::UInt64 => primitive!(UInt64Type),
        DataType::Float32 => primitive!(Float32Type),
        DataType::Float64 => primitive!(Float64Type),
        DataType::Date32 => primitive!(Date32Type),
        DataType::Date64 => primitive!(Date64Type),
        DataType::Time32(TimeUnit::Second) => primitive!(Time32SecondType),
        DataType::Time32(TimeUnit::Millisecond) => primitive!(Time32MillisecondType),
        DataType::Time64(TimeUnit::Microsecond) => primitive!(Time64MicrosecondType),
        DataType::Time64(TimeUnit::Nanosecond) => primitive!(Time64NanosecondType),
        DataType::Timestamp(TimeUnit::Second, _) => primitive!(TimestampSecondType),
        DataType::Timestamp(TimeUnit::Millisecond, _) => primitive!(TimestampMillisecondType),
        DataType::Timestamp(TimeUnit::Microsecond, _) => primitive!(TimestampMicrosecondType),
        DataType::Timestamp(TimeUnit::Nanosecond, _) => primitive!(TimestampNanosecondType),
        DataType::Duration(TimeUnit::Second) => primitive!(DurationSecondType),
        DataType::Duration(TimeUnit::Millisecond) => primitive!(DurationMillisecondType),
        DataType::Duration(TimeUnit::Microsecond) => primitive!(DurationMicrosecondType),
        DataType::Duration(TimeUnit::Nanosecond) => primitive!(DurationNanosecondType),
        DataType::Utf8 => Arc::new(array.as_string::<i32>().iter().collect::<StringArray>()),
        DataType::LargeUtf8 => Arc::new(
            array
                .as_string::<i64>()
                .iter()
                .collect::<LargeStringArray>(),
        ),
        DataType::Binary => Arc::new(array.as_binary::<i32>().iter().collect::<BinaryArray>()),
        DataType::LargeBinary => Arc::new(
            array
                .as_binary::<i64>()
                .iter()
                .collect::<LargeBinaryArray>(),
        ),
        DataType::Dictionary(key, _) => match key.as_ref() {
            DataType::Int8 => rebuild_dictionary::<Int8Type>(array),
            DataType::Int16 => rebuild_dictionary::<Int16Type>(array),
            DataType::Int32 => rebuild_dictionary::<Int32Type>(array),
            DataType::Int64 => rebuild_dictionary::<Int64Type>(array),
            DataType::UInt8 => rebuild_dictionary::<UInt8Type>(array),
            DataType::UInt16 => rebuild_dictionary::<UInt16Type>(array),
            DataType::UInt32 => rebuild_dictionary::<UInt32Type>(array),
            DataType::UInt64 => rebuild_dictionary::<UInt64Type>(array),
            _ => Arc::clone(array),
        },
        // Nested and rare types are written as they are.
        _ => Arc::clone(array),
    }
}

fn rebuild_primitive<T: ArrowPrimitiveType>(array: &ArrayRef) -> ArrayRef {
    let rebuilt: PrimitiveArray<T> = array.as_primitive::<T>().iter().collect();
    // Keep the data type's parameters (a timestamp's time zone).
    Arc::new(rebuilt.with_data_type(array.data_type().clone()))
}

fn rebuild_dictionary<K: ArrowDictionaryKeyType>(array: &ArrayRef) -> ArrayRef {
    let dictionary = array.as_dictionary::<K>();
    let keys: PrimitiveArray<K> = dictionary.keys().iter().collect();
    let values = canonical(dictionary.values());
    match DictionaryArray::<K>::try_new(keys, values) {
        Ok(rebuilt) => Arc::new(rebuilt),
        Err(_) => Arc::clone(array),
    }
}

#[cfg(test)]
mod tests;

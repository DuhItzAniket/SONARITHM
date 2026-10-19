// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Data fingerprints: SHA-256 over a canonical Arrow IPC serialization (03 §2, ADR-0013).
//!
//! The canonical form is one Arrow IPC stream (schema with its metadata, then a single
//! record batch, uncompressed) written from a canonical copy of the table:
//!
//! - batches are concatenated, so the fingerprint does not depend on how a reader chunked
//!   the rows;
//! - every array is rebuilt with zeroed slots under nulls and offsets starting at zero, so
//!   logically equal arrays hash equally whatever computation produced them (flat types:
//!   numbers, booleans, text, binary, temporal values and dictionaries of them; nested
//!   types are written as they are);
//! - schema and field metadata are sorted maps in arrow-rs 60, so their order is fixed.
//!
//! The digest is identical on every platform for the same table (tested on the corpus on
//! Linux, Windows and macOS in CI). It covers values, types, field names and `snr.*`
//! metadata, and therefore inferred semantics too; it does not cover the table name, ID or
//! generation. An Arrow upgrade that changes the IPC encoding changes digests; the pinned
//! corpus test catches that.

use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::{
    ArrowDictionaryKeyType, ArrowPrimitiveType, Date32Type, Date64Type, DurationMicrosecondType,
    DurationMillisecondType, DurationNanosecondType, DurationSecondType, Float32Type, Float64Type,
    Int8Type, Int16Type, Int32Type, Int64Type, Time32MillisecondType, Time32SecondType,
    Time64MicrosecondType, Time64NanosecondType, TimestampMicrosecondType,
    TimestampMillisecondType, TimestampNanosecondType, TimestampSecondType, UInt8Type, UInt16Type,
    UInt32Type, UInt64Type,
};
use arrow_array::{
    Array, ArrayRef, BinaryArray, BooleanArray, DictionaryArray, LargeBinaryArray,
    LargeStringArray, PrimitiveArray, RecordBatch, RecordBatchOptions, StringArray,
};
use arrow_ipc::writer::StreamWriter;
use arrow_schema::{DataType, TimeUnit};
use sonarithm_core::Sha256Digest;

use crate::bundle::DataBundle;
use crate::error::DataError;
use crate::table::Table;

/// The tag that starts a bundle fingerprint's input, versioning the scheme.
const BUNDLE_TAG: &[u8] = b"sonarithm-data bundle fingerprint v1\0";

/// The fingerprint of one table's content.
///
/// # Errors
///
/// [`DataError::Write`] if Arrow cannot encode the table.
pub fn table_fingerprint(table: &Table) -> Result<Sha256Digest, DataError> {
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

/// The fingerprint of a bundle: SHA-256 over a version tag and its tables' fingerprints,
/// in table order. Render manifests record it (ADR-0013).
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

/// A canonical copy of an array: zeroed slots under nulls, offsets from zero.
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
mod tests {
    use arrow_array::{Float64Array, Int64Array};
    use arrow_schema::{Field, Schema};
    use sonarithm_core::TableId;

    use super::*;
    use crate::meta::{FieldMeta, FieldType};

    fn table(values: ArrayRef, meta: &FieldMeta) -> Table {
        let schema = Arc::new(Schema::new(vec![meta.apply(Field::new(
            "v",
            values.data_type().clone(),
            true,
        ))]));
        let batch = RecordBatch::try_new(schema, vec![values]).unwrap();
        Table::from_batch(TableId(0), "t", batch)
    }

    #[test]
    fn batching_slicing_and_bytes_under_nulls_do_not_matter() {
        let meta = FieldMeta::new();
        let whole = table(
            Arc::new(Int64Array::from(vec![Some(1), None, Some(3)])),
            &meta,
        );
        // The same values, built from a slice of a longer array with garbage under the null.
        let longer = Int64Array::from(vec![Some(9), Some(1), Some(42), Some(3)]);
        let mask = BooleanArray::from(vec![false, false, true, false]);
        let with_garbage = arrow_select::nullif::nullif(&longer, &mask)
            .unwrap()
            .slice(1, 3);
        assert!(with_garbage.is_null(1));
        let sliced = table(with_garbage, &meta);
        assert_eq!(
            table_fingerprint(&whole).unwrap(),
            table_fingerprint(&sliced).unwrap()
        );
        // Two batches instead of one.
        let split = Table::new(
            TableId(5),
            "other name",
            Arc::clone(whole.schema()),
            vec![
                whole.batches()[0].slice(0, 1),
                whole.batches()[0].slice(1, 2),
            ],
        )
        .unwrap();
        assert_eq!(
            table_fingerprint(&whole).unwrap(),
            table_fingerprint(&split).unwrap()
        );
    }

    #[test]
    fn values_types_and_metadata_change_the_fingerprint() {
        let base = table(
            Arc::new(Float64Array::from(vec![1.0, 2.0])),
            &FieldMeta::new(),
        );
        let other_value = table(
            Arc::new(Float64Array::from(vec![1.0, 2.5])),
            &FieldMeta::new(),
        );
        let other_type = table(Arc::new(Int64Array::from(vec![1, 2])), &FieldMeta::new());
        let other_meta = table(
            Arc::new(Float64Array::from(vec![1.0, 2.0])),
            &FieldMeta::new().with_type(FieldType::Quantitative),
        );
        let base_fp = table_fingerprint(&base).unwrap();
        for other in [&other_value, &other_type, &other_meta] {
            assert_ne!(table_fingerprint(other).unwrap(), base_fp);
        }
        assert_eq!(
            table_fingerprint(&base).unwrap(),
            base_fp,
            "stable across calls"
        );
    }
}

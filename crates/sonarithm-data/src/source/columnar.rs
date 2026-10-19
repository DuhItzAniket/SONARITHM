// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Arrow IPC and Parquet sources and writers (10 §2).
//!
//! Both formats carry the Arrow schema, so field metadata (`snr.*`) and the provenance
//! columns (`__snr_prov` as a dictionary, `__snr_conf`) come back exactly as written.
//! Parquet keeps the Arrow schema in its `ARROW:schema` key-value metadata.

use std::io::Cursor;
use std::sync::Arc;

use arrow_array::RecordBatch;
use arrow_ipc::reader::{FileReader, StreamReader};
use arrow_ipc::writer::FileWriter;
use arrow_schema::SchemaRef;
use bytes::Bytes;
use parquet::arrow::ArrowWriter;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;

use super::{ReadOptions, to_u64};
use crate::bundle::Format;
use crate::error::DataError;
use crate::table::Table;

/// The magic bytes that open an Arrow IPC file (as opposed to a stream).
const IPC_FILE_MAGIC: &[u8] = b"ARROW1";

/// Rows per decoded Parquet batch.
const PARQUET_BATCH_ROWS: usize = 64 * 1024;

/// Reads Arrow IPC in the file (Feather v2) or the stream format.
pub(super) fn read_ipc(
    bytes: &[u8],
    name: &str,
    format: Format,
    options: &ReadOptions,
) -> Result<Table, DataError> {
    let unreadable = |source: arrow_schema::ArrowError| DataError::Unreadable {
        format,
        reason: source.to_string(),
    };
    let (schema, batches) = if bytes.starts_with(IPC_FILE_MAGIC) {
        let reader = FileReader::try_new(Cursor::new(bytes), None).map_err(unreadable)?;
        let schema = reader.schema();
        (
            schema.clone(),
            collect(&schema, reader, options, unreadable)?,
        )
    } else {
        let reader = StreamReader::try_new(Cursor::new(bytes), None).map_err(unreadable)?;
        let schema = reader.schema();
        (
            schema.clone(),
            collect(&schema, reader, options, unreadable)?,
        )
    };
    Table::new(options.table, name, schema, batches)
}

/// Reads Parquet. The row count is checked against the limits before any data is decoded.
pub(super) fn read_parquet(
    bytes: &[u8],
    name: &str,
    options: &ReadOptions,
) -> Result<Table, DataError> {
    let unreadable = |source: parquet::errors::ParquetError| DataError::Unreadable {
        format: Format::Parquet,
        reason: source.to_string(),
    };
    let builder = ParquetRecordBatchReaderBuilder::try_new(Bytes::copy_from_slice(bytes))
        .map_err(unreadable)?;
    let rows = u64::try_from(builder.metadata().file_metadata().num_rows()).unwrap_or(0);
    let schema = Arc::clone(builder.schema());
    options.limits.check_shape(rows, schema.fields().len())?;
    let reader = builder
        .with_batch_size(PARQUET_BATCH_ROWS)
        .build()
        .map_err(unreadable)?;
    let batches = collect(&schema, reader, options, |source| DataError::Unreadable {
        format: Format::Parquet,
        reason: source.to_string(),
    })?;
    Table::new(options.table, name, schema, batches)
}

/// Collects batches, checking the limits as rows arrive.
fn collect<E>(
    schema: &SchemaRef,
    reader: impl Iterator<Item = Result<RecordBatch, E>>,
    options: &ReadOptions,
    unreadable: impl Fn(E) -> DataError,
) -> Result<Vec<RecordBatch>, DataError> {
    let fields = schema.fields().len();
    options.limits.check_shape(0, fields)?;
    let mut rows = 0_u64;
    let mut batches = Vec::new();
    for batch in reader {
        let batch = batch.map_err(&unreadable)?;
        rows = rows.saturating_add(to_u64(batch.num_rows()));
        options.limits.check_shape(rows, fields)?;
        batches.push(batch);
    }
    Ok(batches)
}

/// Writes a table as an Arrow IPC file (Feather v2), uncompressed, with its field
/// metadata.
///
/// # Errors
///
/// [`DataError::Write`] if Arrow cannot encode the table.
pub fn write_ipc(table: &Table) -> Result<Vec<u8>, DataError> {
    let failed = |source: arrow_schema::ArrowError| DataError::Write {
        format: Format::Arrow,
        reason: source.to_string(),
    };
    let mut writer = FileWriter::try_new(Vec::new(), table.schema()).map_err(failed)?;
    for batch in table.batches() {
        writer.write(batch).map_err(failed)?;
    }
    writer.finish().map_err(failed)?;
    writer.into_inner().map_err(failed)
}

/// Writes a table as Parquet with Snappy compression (pure Rust, readable everywhere),
/// embedding the Arrow schema so field metadata survives.
///
/// # Errors
///
/// [`DataError::Write`] if the table cannot be encoded.
pub fn write_parquet(table: &Table) -> Result<Vec<u8>, DataError> {
    let failed = |source: parquet::errors::ParquetError| DataError::Write {
        format: Format::Parquet,
        reason: source.to_string(),
    };
    let properties = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();
    let mut writer = ArrowWriter::try_new(Vec::new(), Arc::clone(table.schema()), Some(properties))
        .map_err(failed)?;
    for batch in table.batches() {
        writer.write(batch).map_err(failed)?;
    }
    writer.into_inner().map_err(failed)
}

#[cfg(test)]
mod tests {
    use arrow_array::types::UInt8Type;
    use arrow_array::{DictionaryArray, Float32Array, Float64Array, StringArray};
    use arrow_schema::{DataType, Field, Schema};
    use proptest::prelude::*;
    use sonarithm_core::TableId;

    use super::*;
    use crate::meta::{Concept, FieldMeta, FieldType, Role, Unit};
    use crate::source::{Limits, read_bytes};

    fn temperature_meta() -> FieldMeta {
        FieldMeta::new()
            .with_type(FieldType::Quantitative)
            .with_unit(Unit::new("°C").unwrap().with_ucum("Cel"))
            .with_concept(Concept::new("temperature").unwrap())
            .with_role(Role::Y)
            .with_label("Temperature")
    }

    /// A table with field metadata, schema metadata and both provenance columns.
    fn table_with(meta: &FieldMeta) -> Table {
        let schema = Arc::new(
            Schema::new(vec![
                meta.apply(Field::new("temp", DataType::Float64, true)),
                FieldMeta::new()
                    .with_type(FieldType::Nominal)
                    .with_role(Role::Category)
                    .apply(Field::new("site", DataType::Utf8, true)),
                Field::new(
                    "__snr_prov",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new("__snr_conf", DataType::Float32, true),
            ])
            .with_metadata(arrow_schema::Metadata::from([("pandas", "{}")])),
        );
        let prov: DictionaryArray<UInt8Type> = vec!["exact", "cv_estimate", "user_edited"]
            .into_iter()
            .collect();
        let batch = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![
                Arc::new(Float64Array::from(vec![Some(21.5), None, Some(-3.0)])),
                Arc::new(StringArray::from(vec!["a", "b", "a"])),
                Arc::new(prov),
                Arc::new(Float32Array::from(vec![None, Some(0.8), Some(1.0)])),
            ],
        )
        .unwrap();
        Table::new(
            TableId(0),
            "t",
            schema,
            vec![batch.slice(0, 2), batch.slice(2, 1)],
        )
        .unwrap()
    }

    fn reread(bytes: &[u8], format: Format) -> Table {
        let options = ReadOptions::new().without_inference();
        let loaded = read_bytes(bytes, format, "dir/t.bin", &options).unwrap();
        assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
        loaded.bundle.primary().unwrap().clone()
    }

    fn assert_same(read: &Table, written: &Table) {
        assert_eq!(read.schema(), written.schema());
        assert_eq!(read.to_batch().unwrap(), written.to_batch().unwrap());
    }

    #[test]
    fn metadata_and_provenance_survive_arrow_ipc() {
        let table = table_with(&temperature_meta());
        let read = reread(&write_ipc(&table).unwrap(), Format::Arrow);
        assert_same(&read, &table);
        assert_eq!(read.field_meta("temp").unwrap(), temperature_meta());
        assert_eq!(read.name(), "t");
    }

    #[test]
    fn metadata_and_provenance_survive_parquet() {
        let table = table_with(&temperature_meta());
        let read = reread(&write_parquet(&table).unwrap(), Format::Parquet);
        assert_same(&read, &table);
        assert_eq!(read.field_meta("temp").unwrap(), temperature_meta());
    }

    #[test]
    fn the_ipc_stream_format_is_read_too() {
        let table = table_with(&temperature_meta());
        let mut writer =
            arrow_ipc::writer::StreamWriter::try_new(Vec::new(), table.schema()).unwrap();
        for batch in table.batches() {
            writer.write(batch).unwrap();
        }
        writer.finish().unwrap();
        let read = reread(&writer.into_inner().unwrap(), Format::Arrow);
        assert_same(&read, &table);
    }

    #[test]
    fn garbage_and_limits_fail_cleanly() {
        for format in [Format::Arrow, Format::Parquet] {
            let err = read_bytes(b"not a file", format, "x", &ReadOptions::default()).unwrap_err();
            assert_eq!(
                err.to_diagnostic().code.as_str(),
                "SNR-DATA-001",
                "{format}"
            );
        }
        let bytes = write_parquet(&table_with(&FieldMeta::new())).unwrap();
        let options = ReadOptions {
            limits: Limits {
                max_rows: 2,
                ..Limits::NONE
            },
            ..ReadOptions::default()
        };
        let err = read_bytes(&bytes, Format::Parquet, "x", &options).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-003");
    }

    #[test]
    fn unreadable_metadata_is_reported_not_fatal() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("v", DataType::Float64, true)
                .with_metadata(arrow_schema::Metadata::from([("snr.role", "z")])),
        ]));
        let batch = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![Arc::new(Float64Array::from(vec![1.0]))],
        )
        .unwrap();
        let bytes = write_ipc(&Table::new(TableId(0), "t", schema, vec![batch]).unwrap()).unwrap();
        let options = ReadOptions::new().without_inference();
        let loaded = read_bytes(&bytes, Format::Arrow, "t.arrow", &options).unwrap();
        let codes: Vec<_> = loaded.diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes, ["SNR-DATA-040"]);
    }

    fn any_meta() -> impl Strategy<Value = FieldMeta> {
        let ftype = prop::option::of(prop_oneof![
            Just(FieldType::Quantitative),
            Just(FieldType::Temporal),
            Just(FieldType::Ordinal),
            Just(FieldType::Nominal),
        ]);
        let role = prop::option::of(prop_oneof![Just(Role::X), Just(Role::Y), Just(Role::Lower)]);
        let unit = prop::option::of(("[^ ]{1,8}", prop::option::of("[A-Za-z]{1,4}")));
        let concept = prop::option::of("[a-z]{1,8}(\\.[a-z_]{1,8})?");
        let label = prop::option::of(".{0,16}");
        (ftype, role, unit, concept, label).prop_map(|(ftype, role, unit, concept, label)| {
            let mut meta = FieldMeta::new();
            meta.ftype = ftype;
            meta.role = role;
            meta.unit = unit.and_then(|(text, ucum)| {
                Unit::new(text).map(|unit| match ucum {
                    Some(code) => unit.with_ucum(code),
                    None => unit,
                })
            });
            meta.concept = concept.and_then(Concept::new);
            meta.label = label;
            meta
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        #[test]
        fn any_field_meta_survives_both_formats(meta in any_meta()) {
            let table = table_with(&meta);
            for (bytes, format) in [
                (write_ipc(&table).unwrap(), Format::Arrow),
                (write_parquet(&table).unwrap(), Format::Parquet),
            ] {
                let read = reread(&bytes, format);
                prop_assert_eq!(read.field_meta("temp").unwrap(), meta.clone());
                prop_assert_eq!(read.schema(), table.schema());
            }
        }
    }
}

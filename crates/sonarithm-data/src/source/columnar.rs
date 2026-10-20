// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Arrow IPC and Parquet sources and writers (10 §2).
//!
//! Both formats carry the Arrow schema, so field metadata (`snr.*`) and the provenance
//! columns (`__snr_prov` as a dictionary, `__snr_conf`) come back exactly as written.
//! Parquet keeps the Arrow schema in its `ARROW:schema` key-value metadata.

use std::io::Cursor;
use std::sync::Arc;

use arrow_array::{RecordBatch, make_array};
use arrow_data::ArrayData;
use arrow_ipc::reader::{FileReader, StreamReader};
use arrow_ipc::writer::FileWriter;
use arrow_schema::{ArrowError, DataType, Field, Fields, Schema, SchemaRef, UnionFields};
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
    let file = bytes.starts_with(IPC_FILE_MAGIC);
    check_ipc_framing(bytes, file, options)?;
    let (schema, batches) = super::contained(format, || {
        Ok(if file {
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
        })
    })?;
    columnar_table(format, options, name, &schema, batches)
}

/// Builds the table of an Arrow IPC or Parquet read, with empty time zones removed.
fn columnar_table(
    format: Format,
    options: &ReadOptions,
    name: &str,
    schema: &SchemaRef,
    batches: Vec<RecordBatch>,
) -> Result<Table, DataError> {
    let (schema, batches) = super::contained(format, || {
        let (schema, batches) =
            without_empty_time_zones(schema, batches).map_err(|source| DataError::Unreadable {
                format,
                reason: source.to_string(),
            })?;
        check_empty_columns(&schema).map_err(|reason| DataError::Unreadable { format, reason })?;
        Ok((schema, batches))
    })?;
    Table::new(options.table, name, schema, batches)
}

/// Checks that Arrow can build an empty column of every field's type: a table without
/// rows hands out empty columns ([`Table::column`]), and arrow-data panics on some types a
/// file can declare, such as a union without members (found by fuzzing, F-071).
fn check_empty_columns(schema: &Schema) -> Result<(), String> {
    for field in schema.fields() {
        let data_type = field.data_type();
        let built = std::panic::catch_unwind(|| arrow_array::new_empty_array(data_type));
        if built.is_err() {
            return Err(format!(
                "field `{}` has a type Arrow cannot hold: {data_type}",
                field.name()
            ));
        }
    }
    Ok(())
}

/// Rewrites timestamp types whose time zone is an empty string as having none, at any
/// depth. The Arrow format gives both the same meaning (time-zone naive) and arrow-ipc
/// writes an empty one as none, so without this a loaded table would not survive its own
/// IPC round trip (found by fuzzing, F-071).
fn without_empty_time_zones(
    schema: &SchemaRef,
    batches: Vec<RecordBatch>,
) -> Result<(SchemaRef, Vec<RecordBatch>), ArrowError> {
    let fields = schema
        .fields()
        .iter()
        .map(|field| naive_field(field))
        .collect::<Result<Vec<_>, _>>()?;
    let naive = Schema::new_with_metadata(fields, schema.metadata().clone());
    if naive == **schema {
        return Ok((Arc::clone(schema), batches));
    }
    let naive = Arc::new(naive);
    let batches = batches
        .into_iter()
        .map(|batch| {
            let columns = batch
                .columns()
                .iter()
                .map(|column| naive_data(column.to_data()).map(make_array))
                .collect::<Result<Vec<_>, _>>()?;
            RecordBatch::try_new(Arc::clone(&naive), columns)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((naive, batches))
}

fn naive_field(field: &Field) -> Result<Field, ArrowError> {
    Ok(field.clone().with_data_type(naive_type(field.data_type())?))
}

fn naive_type(data_type: &DataType) -> Result<DataType, ArrowError> {
    let field = |field: &Field| naive_field(field).map(Arc::new);
    Ok(match data_type {
        DataType::Timestamp(unit, Some(zone)) if zone.is_empty() => {
            DataType::Timestamp(*unit, None)
        }
        DataType::List(item) => DataType::List(field(item)?),
        DataType::LargeList(item) => DataType::LargeList(field(item)?),
        DataType::ListView(item) => DataType::ListView(field(item)?),
        DataType::LargeListView(item) => DataType::LargeListView(field(item)?),
        DataType::FixedSizeList(item, size) => DataType::FixedSizeList(field(item)?, *size),
        DataType::Struct(fields) => DataType::Struct(
            fields
                .iter()
                .map(|item| naive_field(item))
                .collect::<Result<Fields, _>>()?,
        ),
        DataType::Union(fields, mode) => DataType::Union(
            UnionFields::try_new(
                fields.iter().map(|(id, _)| id),
                fields
                    .iter()
                    .map(|(_, item)| naive_field(item))
                    .collect::<Result<Vec<_>, _>>()?,
            )?,
            *mode,
        ),
        DataType::Dictionary(key, value) => {
            DataType::Dictionary(key.clone(), Box::new(naive_type(value)?))
        }
        DataType::Map(entries, sorted) => DataType::Map(field(entries)?, *sorted),
        DataType::RunEndEncoded(ends, values) => {
            DataType::RunEndEncoded(Arc::clone(ends), field(values)?)
        }
        other => other.clone(),
    })
}

/// Rebuilds array data with [`naive_type`], children first; unchanged data is kept as is.
fn naive_data(data: ArrayData) -> Result<ArrayData, ArrowError> {
    let data_type = naive_type(data.data_type())?;
    if data_type == *data.data_type() {
        return Ok(data);
    }
    let children = data
        .child_data()
        .iter()
        .cloned()
        .map(naive_data)
        .collect::<Result<Vec<_>, _>>()?;
    data.into_builder()
        .data_type(data_type)
        .child_data(children)
        .build()
}

/// Checks the lengths Arrow IPC input declares before arrow-ipc allocates buffers from
/// them (both problems were found by fuzzing):
///
/// - every message and body lies inside the input (the footer and its blocks for the file
///   format, each message for the stream format), and every buffer inside its body;
/// - every compressed buffer declares an uncompressed size within the byte limit, and all of
///   them together stay within what the cell limit allows, since arrow-ipc reserves the
///   declared size before decompressing.
fn check_ipc_framing(bytes: &[u8], file: bool, options: &ReadOptions) -> Result<(), DataError> {
    let unreadable = |reason: &str| DataError::Unreadable {
        format: Format::Arrow,
        reason: reason.to_owned(),
    };
    let messages = if file {
        file_messages(bytes)
    } else {
        stream_messages(bytes)
    }
    .map_err(|r| unreadable(&r))?;
    // At most 16 bytes per allowed cell in total.
    let budget = options.limits.max_cells.saturating_mul(16);
    let mut declared_total = 0_u64;
    for (meta, body) in messages {
        let message = arrow_ipc::root_as_message(meta)
            .map_err(|err| unreadable(&format!("a message is invalid: {err}")))?;
        let batch = message
            .header_as_record_batch()
            .or_else(|| message.header_as_dictionary_batch().and_then(|d| d.data()));
        let Some(batch) = batch else {
            continue;
        };
        let compressed = batch.compression().is_some();
        for buffer in batch.buffers().into_iter().flatten() {
            let start = usize::try_from(buffer.offset())
                .map_err(|_| unreadable("a buffer offset is negative"))?;
            let length = usize::try_from(buffer.length())
                .map_err(|_| unreadable("a buffer length is negative"))?;
            if start.checked_add(length).is_none_or(|end| end > body.len()) {
                return Err(unreadable("a buffer lies outside its message body"));
            }
            if !compressed || length < 8 {
                continue;
            }
            let Some(prefix) = start.checked_add(8).and_then(|end| body.get(start..end)) else {
                return Err(unreadable("a buffer lies outside its message body"));
            };
            let declared = prefix.try_into().map_or(-1, i64::from_le_bytes);
            // -1 marks a buffer stored uncompressed.
            let Ok(declared) = u64::try_from(declared) else {
                continue;
            };
            declared_total = declared_total.saturating_add(declared);
            for (limit, max, found) in [
                (
                    crate::error::Limit::Bytes,
                    options.limits.max_bytes,
                    declared,
                ),
                (crate::error::Limit::Bytes, budget, declared_total),
            ] {
                if found > max {
                    return Err(DataError::LimitExceeded { limit, max, found });
                }
            }
        }
    }
    Ok(())
}

/// An IPC message: its metadata flatbuffer and its body.
type Message<'a> = (&'a [u8], &'a [u8]);

/// A little-endian `i32` at `at`.
fn word(bytes: &[u8], at: usize) -> Option<i32> {
    bytes
        .get(at..at.checked_add(4)?)
        .and_then(|b| <[u8; 4]>::try_from(b).ok())
        .map(i32::from_le_bytes)
}

/// The metadata flatbuffer of the message at `at` and the offset just after its length
/// prefix (with or without the 0xFFFFFFFF continuation marker).
fn message_at(bytes: &[u8], at: usize) -> Option<(usize, usize)> {
    let mut at = at;
    let mut length = word(bytes, at)?;
    at = at.checked_add(4)?;
    if length == -1 {
        length = word(bytes, at)?;
        at = at.checked_add(4)?;
    }
    Some((at, usize::try_from(length).ok()?))
}

/// The (metadata, body) of every block the file footer lists, checked to lie inside the
/// file.
fn file_messages(bytes: &[u8]) -> Result<Vec<Message<'_>>, String> {
    let tail = bytes.len().checked_sub(10).ok_or("the file is too short")?;
    let footer_len = usize::try_from(word(bytes, tail).ok_or("the file is too short")?)
        .map_err(|_| "the footer length is negative")?;
    let footer_start = tail
        .checked_sub(footer_len)
        .ok_or("the footer length points outside the file")?;
    let footer = arrow_ipc::root_as_footer(bytes.get(footer_start..tail).ok_or("bad footer")?)
        .map_err(|err| format!("the footer is invalid: {err}"))?;
    let mut messages = Vec::new();
    for block in footer
        .recordBatches()
        .into_iter()
        .chain(footer.dictionaries())
        .flatten()
    {
        let outside = || "a block points outside the file".to_owned();
        let offset = usize::try_from(block.offset()).map_err(|_| outside())?;
        let meta_total = usize::try_from(block.metaDataLength()).map_err(|_| outside())?;
        let body_len = usize::try_from(block.bodyLength()).map_err(|_| outside())?;
        let body_start = offset.checked_add(meta_total).ok_or_else(outside)?;
        let body = body_start
            .checked_add(body_len)
            .and_then(|end| bytes.get(body_start..end))
            .ok_or_else(outside)?;
        let (meta_start, meta_len) = message_at(bytes, offset).ok_or_else(outside)?;
        let meta = meta_start
            .checked_add(meta_len)
            .filter(|end| *end <= body_start)
            .and_then(|end| bytes.get(meta_start..end))
            .ok_or_else(outside)?;
        messages.push((meta, body));
    }
    Ok(messages)
}

/// The (metadata, body) of every message of a stream, checked to lie inside the input.
fn stream_messages(bytes: &[u8]) -> Result<Vec<Message<'_>>, String> {
    let mut messages = Vec::new();
    let mut at = 0_usize;
    while let Some((meta_start, meta_len)) = message_at(bytes, at) {
        if meta_len == 0 {
            break;
        }
        let meta_end = meta_start
            .checked_add(meta_len)
            .ok_or("a message runs past the end")?;
        let meta = bytes
            .get(meta_start..meta_end)
            .ok_or("a message runs past the end")?;
        let message = arrow_ipc::root_as_message(meta)
            .map_err(|err| format!("a message is invalid: {err}"))?;
        let body_len =
            usize::try_from(message.bodyLength()).map_err(|_| "a negative body length")?;
        let body_end = meta_end
            .checked_add(body_len)
            .ok_or("a message body runs past the end")?;
        let body = bytes
            .get(meta_end..body_end)
            .ok_or("a message body runs past the end")?;
        messages.push((meta, body));
        at = body_end;
    }
    Ok(messages)
}

/// Reads Parquet. The row count is checked against the limits before any data is decoded.
///
/// The footer and every page header are walked first ([`thrift`](super::thrift)): parquet
/// skips Thrift fields it does not know without bounding their declared sizes, and a few
/// fuzzed bytes kept it busy for seconds (#146).
pub(super) fn read_parquet(
    bytes: &[u8],
    name: &str,
    options: &ReadOptions,
) -> Result<Table, DataError> {
    let unreadable = |source: parquet::errors::ParquetError| DataError::Unreadable {
        format: Format::Parquet,
        reason: source.to_string(),
    };
    let malformed = |reason: String| DataError::Unreadable {
        format: Format::Parquet,
        reason,
    };
    let (schema, batches) = super::contained(Format::Parquet, || {
        super::thrift::check_footer(bytes).map_err(malformed)?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(Bytes::copy_from_slice(bytes))
            .map_err(unreadable)?;
        for row_group in builder.metadata().row_groups() {
            for column in row_group.columns() {
                let start = column
                    .dictionary_page_offset()
                    .unwrap_or_else(|| column.data_page_offset());
                super::thrift::check_column_chunk(bytes, start, column.compressed_size())
                    .map_err(malformed)?;
            }
        }
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
        Ok((schema, batches))
    })?;
    columnar_table(Format::Parquet, options, name, &schema, batches)
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
    use arrow_array::cast::AsArray;
    use arrow_array::{Float64Array, StringArray};
    use arrow_schema::{DataType, Field, Schema};
    use proptest::prelude::*;
    use sonarithm_core::{Confidence, Provenance, ProvenanceClass, TableId};

    use super::*;
    use crate::meta::{Concept, FieldMeta, FieldType, Role, Unit};
    use crate::provenance::{RowProvenance, with_provenance};
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
            ])
            .with_metadata(arrow_schema::Metadata::from([("pandas", "{}")])),
        );
        let batch = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![
                Arc::new(Float64Array::from(vec![Some(21.5), None, Some(-3.0)])),
                Arc::new(StringArray::from(vec!["a", "b", "a"])),
            ],
        )
        .unwrap();
        let data = Table::new(
            TableId(0),
            "t",
            schema,
            vec![batch.slice(0, 2), batch.slice(2, 1)],
        )
        .unwrap();
        let provenance = RowProvenance::from_rows(vec![
            Provenance::EXACT,
            Provenance {
                class: ProvenanceClass::CvEstimate,
                confidence: Confidence::new(0.8),
            },
            Provenance {
                class: ProvenanceClass::UserEdited,
                confidence: Confidence::new(1.0),
            },
        ]);
        with_provenance(&data, &provenance, true).unwrap()
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
    fn compressed_buffers_must_declare_sizes_within_the_limits() {
        let table = table_with(&FieldMeta::new());
        let options = arrow_ipc::writer::IpcWriteOptions::default()
            .try_with_compression(Some(arrow_ipc::CompressionType::LZ4_FRAME))
            .unwrap();
        let mut writer =
            FileWriter::try_new_with_options(Vec::new(), table.schema(), options).unwrap();
        for batch in table.batches() {
            writer.write(batch).unwrap();
        }
        writer.finish().unwrap();
        let bytes = writer.into_inner().unwrap();
        let read = read_bytes(
            &bytes,
            Format::Arrow,
            "x",
            &ReadOptions::new().without_inference(),
        )
        .unwrap();
        assert_eq!(
            read.bundle.primary().unwrap().num_rows(),
            3,
            "compressed files read"
        );
        let tiny = ReadOptions::new().with_limits(Limits {
            max_cells: 1,
            ..Limits::NONE
        });
        let err = read_bytes(&bytes, Format::Arrow, "x", &tiny).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-003", "{err}");
    }

    #[test]
    fn fuzzer_findings_stay_fixed() {
        // A Feather file whose LZ4 buffer declares an uncompressed size of 2^58 bytes.
        let bytes = include_bytes!("../../tests/corpus/fuzz/ipc-lz4-declared-size.arrow");
        let err = read_bytes(bytes, Format::Arrow, "x", &ReadOptions::default()).unwrap_err();
        assert!(
            matches!(
                err.to_diagnostic().code.as_str(),
                "SNR-DATA-001" | "SNR-DATA-003"
            ),
            "{err}"
        );
        // A Feather file whose validity bitmap is shorter than its column: arrow-buffer
        // panics, and the panic is contained as an error.
        let bytes = include_bytes!("../../tests/corpus/fuzz/ipc-short-validity-bitmap.arrow");
        let err = read_bytes(bytes, Format::Arrow, "x", &ReadOptions::default()).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-001", "{err}");
        // An Arrow file whose provenance column is a dictionary with no values: arrow's
        // `normalized_keys` panicked on it after decoding, outside the panic containment
        // (Linux fuzz run on the A04-T10 PR). Its keys are all null, so every row is exact.
        let bytes = include_bytes!("../../tests/corpus/fuzz/ipc-empty-provenance-dictionary.arrow");
        let loaded = read_bytes(bytes, Format::Arrow, "x", &ReadOptions::default()).unwrap();
        assert!(loaded.bundle.primary().is_some());
        // A Parquet page header whose statistics hold an unknown field, a list declaring
        // 268,434,391 UUIDs, in a 1.8 KB file. parquet 60 spent about 12 s skipping it (#146).
        // The header also runs past its column chunk, where the Thrift walk stops.
        let bytes = include_bytes!("../../tests/corpus/fuzz/parquet-slow-page-header.parquet");
        let err = read_bytes(bytes, Format::Parquet, "x", &ReadOptions::default()).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-001", "{err}");
        assert!(
            err.to_string().contains("page header is malformed"),
            "{err}"
        );
        // An Arrow file with a timestamp field whose time zone is an empty string: arrow-ipc
        // writes it back without one, so the loaded table failed its own IPC round trip.
        let bytes = include_bytes!("../../tests/corpus/fuzz/ipc-empty-time-zone.arrow");
        let table = read_bytes(bytes, Format::Arrow, "x", &ReadOptions::default())
            .unwrap()
            .bundle
            .primary()
            .unwrap()
            .clone();
        let written = write_ipc(&table).unwrap();
        let again = read_bytes(&written, Format::Arrow, "x", &ReadOptions::default()).unwrap();
        assert_eq!(again.bundle.primary().unwrap().schema(), table.schema());
        // An Arrow file without record batches whose provenance column is a dictionary of a
        // union without members: arrow-data panicked building its empty column after
        // decoding (nightly run on 2026-10-20).
        let bytes = include_bytes!("../../tests/corpus/fuzz/ipc-empty-union.arrow");
        let err = read_bytes(bytes, Format::Arrow, "x", &ReadOptions::default()).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-001", "{err}");
        assert!(err.to_string().contains("cannot hold"), "{err}");
    }

    #[test]
    fn types_without_an_empty_column_are_rejected() {
        use arrow_schema::UnionMode;

        let fine = Schema::new(vec![
            Field::new("v", DataType::Float64, true),
            Field::new(
                "d",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                true,
            ),
        ]);
        assert_eq!(check_empty_columns(&fine), Ok(()));
        let empty_union = DataType::Union(UnionFields::empty(), UnionMode::Sparse);
        let broken = Schema::new(vec![Field::new("u", empty_union, true)]);
        let err = check_empty_columns(&broken).unwrap_err();
        assert!(
            err.starts_with("field `u` has a type Arrow cannot hold"),
            "{err}"
        );
    }

    #[test]
    fn empty_time_zones_are_removed_at_any_depth() {
        use arrow_array::types::TimestampMicrosecondType;
        use arrow_array::{Array, ListArray, TimestampMicrosecondArray};
        use arrow_schema::TimeUnit;

        let empty = DataType::Timestamp(TimeUnit::Microsecond, Some("".into()));
        let naive = DataType::Timestamp(TimeUnit::Microsecond, None);
        let flat = TimestampMicrosecondArray::from(vec![Some(1), None]).with_timezone("");
        let list = ListArray::from_iter_primitive::<TimestampMicrosecondType, _, _>(vec![
            Some(vec![Some(2), Some(3)]),
            None,
        ]);
        let item = Arc::new(Field::new("item", empty.clone(), true));
        let values = list
            .values()
            .as_primitive::<TimestampMicrosecondType>()
            .clone()
            .with_timezone("");
        let list = ListArray::new(
            Arc::clone(&item),
            list.offsets().clone(),
            Arc::new(values),
            list.nulls().cloned(),
        );
        let schema = Arc::new(Schema::new(vec![
            Field::new("t", empty, true),
            Field::new("l", DataType::List(item), true),
        ]));
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(flat), Arc::new(list)])
            .unwrap();

        let (schema, batches) = without_empty_time_zones(&schema, vec![batch]).unwrap();
        assert_eq!(schema.field(0).data_type(), &naive);
        assert_eq!(
            schema.field(1).data_type(),
            &DataType::List(Arc::new(Field::new("item", naive.clone(), true)))
        );
        let column = batches[0]
            .column(0)
            .as_primitive::<TimestampMicrosecondType>();
        assert_eq!(column.data_type(), &naive);
        assert_eq!(column.value(0), 1);
        assert!(column.is_null(1));
        let list = batches[0].column(1).as_list::<i32>();
        assert_eq!(list.values().data_type(), &naive);
        assert!(list.is_null(1));
    }

    #[test]
    fn parquet_written_by_pyarrow_passes_the_thrift_walk() {
        // The fuzz seeds were written by pyarrow (Snappy and zstd); their first byte selects
        // the format for the fuzz target.
        for seed in [
            include_bytes!("../../fuzz/corpus/columnar_read/seed-parquet-snappy").as_slice(),
            include_bytes!("../../fuzz/corpus/columnar_read/seed-parquet-zstd").as_slice(),
        ] {
            let loaded = read_bytes(&seed[1..], Format::Parquet, "x", &ReadOptions::default());
            assert!(loaded.is_ok(), "{:?}", loaded.err());
        }
    }

    #[test]
    fn declared_lengths_past_the_end_are_rejected_before_allocation() {
        let table = table_with(&FieldMeta::new());
        let mut file = write_ipc(&table).unwrap();
        // Corrupt the footer length (the 4 bytes before the trailing magic).
        let at = file.len() - 10;
        file[at..at + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        let err = read_bytes(&file, Format::Arrow, "x", &ReadOptions::default()).unwrap_err();
        assert!(err.to_string().contains("footer"), "{err}");
        // A stream whose first message claims more bytes than there are.
        let mut stream = vec![0xff, 0xff, 0xff, 0xff];
        stream.extend_from_slice(&1_000_000_i32.to_le_bytes());
        stream.extend_from_slice(&[0; 16]);
        let err = read_bytes(&stream, Format::Arrow, "x", &ReadOptions::default()).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-001");
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

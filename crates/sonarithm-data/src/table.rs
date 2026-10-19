// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! [`Table`]: an Arrow schema plus record batches, immutable once built (03 §2).

use std::sync::Arc;

use arrow_array::{ArrayRef, RecordBatch};
use arrow_schema::SchemaRef;
use arrow_select::concat::{concat, concat_batches};
use sonarithm_core::{Generation, RowRange, RowRef, TableId};

use crate::error::DataError;
use crate::meta::FieldMeta;

/// One table of a [`DataBundle`](crate::DataBundle): an Arrow schema and its record
/// batches.
///
/// A table is immutable. An edit (a transform, a metadata change) builds a new table with
/// the same ID and the next [`Generation`], so a [`RowRef`] stays valid for the life of the
/// generation it names. Field semantics live in the schema's field metadata
/// ([`FieldMeta`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    id: TableId,
    name: String,
    generation: Generation,
    schema: SchemaRef,
    batches: Vec<RecordBatch>,
    num_rows: u64,
}

impl Table {
    /// A table at generation 0. Every batch is re-checked against `schema`: column count,
    /// types, nullability and lengths must match (field metadata comes from `schema`).
    ///
    /// # Errors
    ///
    /// [`DataError::MalformedTable`] if a batch does not match the schema.
    pub fn new(
        id: TableId,
        name: impl Into<String>,
        schema: SchemaRef,
        batches: Vec<RecordBatch>,
    ) -> Result<Self, DataError> {
        Self::build(id, name.into(), Generation(0), schema, batches)
    }

    /// A table at generation 0 holding one batch, with the batch's schema.
    #[must_use]
    pub fn from_batch(id: TableId, name: impl Into<String>, batch: RecordBatch) -> Self {
        let num_rows = row_count(&batch);
        Self {
            id,
            name: name.into(),
            generation: Generation(0),
            schema: batch.schema(),
            batches: vec![batch],
            num_rows,
        }
    }

    fn build(
        id: TableId,
        name: String,
        generation: Generation,
        schema: SchemaRef,
        batches: Vec<RecordBatch>,
    ) -> Result<Self, DataError> {
        let mut checked = Vec::with_capacity(batches.len());
        let mut num_rows = 0_u64;
        for batch in batches {
            let batch = RecordBatch::try_new(Arc::clone(&schema), batch.columns().to_vec())
                .map_err(|source| DataError::MalformedTable {
                    table: name.clone(),
                    source,
                })?;
            num_rows = num_rows.saturating_add(row_count(&batch));
            checked.push(batch);
        }
        Ok(Self {
            id,
            name,
            generation,
            schema,
            batches: checked,
            num_rows,
        })
    }

    /// The next generation of this table: same ID and name, new contents.
    ///
    /// # Errors
    ///
    /// [`DataError::MalformedTable`] if a batch does not match the schema.
    pub fn next_generation(
        &self,
        schema: SchemaRef,
        batches: Vec<RecordBatch>,
    ) -> Result<Self, DataError> {
        // A u64 generation counter cannot realistically overflow; saturating keeps this
        // panic-free regardless.
        let generation = Generation(self.generation.0.saturating_add(1));
        Self::build(self.id, self.name.clone(), generation, schema, batches)
    }

    /// The next generation with new metadata for one field; the data is shared, not
    /// copied.
    ///
    /// # Errors
    ///
    /// [`DataError::UnknownField`] if the table has no such field.
    pub fn with_field_meta(&self, field: &str, meta: &FieldMeta) -> Result<Self, DataError> {
        let index = self.field_index(field)?;
        let mut fields: Vec<_> = self.schema.fields().iter().cloned().collect();
        if let Some(slot) = fields.get_mut(index) {
            *slot = Arc::new(meta.apply(slot.as_ref().clone()));
        }
        let schema = Arc::new(
            arrow_schema::Schema::new(fields).with_metadata(self.schema.metadata().clone()),
        );
        self.next_generation(schema, self.batches.clone())
    }

    /// The table ID.
    #[must_use]
    pub const fn id(&self) -> TableId {
        self.id
    }

    /// The table name, such as the file stem or the sheet name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The generation of this table's contents.
    #[must_use]
    pub const fn generation(&self) -> Generation {
        self.generation
    }

    /// The Arrow schema, including `snr.*` field metadata.
    #[must_use]
    pub const fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    /// The record batches, in row order.
    #[must_use]
    pub fn batches(&self) -> &[RecordBatch] {
        &self.batches
    }

    /// The number of rows.
    #[must_use]
    pub const fn num_rows(&self) -> u64 {
        self.num_rows
    }

    /// The number of fields.
    #[must_use]
    pub fn num_fields(&self) -> usize {
        self.schema.fields().len()
    }

    /// The position of a field.
    ///
    /// # Errors
    ///
    /// [`DataError::UnknownField`] if the table has no such field.
    pub fn field_index(&self, field: &str) -> Result<usize, DataError> {
        self.schema
            .index_of(field)
            .map_err(|_| DataError::UnknownField {
                table: self.name.clone(),
                field: field.to_owned(),
            })
    }

    /// The semantics of a field. Unreadable metadata values are left out; sources report
    /// them as diagnostics when the data is loaded.
    ///
    /// # Errors
    ///
    /// [`DataError::UnknownField`] if the table has no such field.
    pub fn field_meta(&self, field: &str) -> Result<FieldMeta, DataError> {
        let index = self.field_index(field)?;
        Ok(FieldMeta::read(self.schema.field(index)).0)
    }

    /// One column as a single array (batches concatenated).
    ///
    /// # Errors
    ///
    /// [`DataError::UnknownField`] if the table has no such field.
    pub fn column(&self, field: &str) -> Result<ArrayRef, DataError> {
        let index = self.field_index(field)?;
        let parts: Vec<&dyn arrow_array::Array> = self
            .batches
            .iter()
            .map(|batch| batch.column(index).as_ref())
            .collect();
        match parts.as_slice() {
            [] => Ok(arrow_array::new_empty_array(
                self.schema.field(index).data_type(),
            )),
            [single] => Ok(single.slice(0, single.len())),
            _ => concat(&parts).map_err(|source| self.malformed(source)),
        }
    }

    /// The whole table as one record batch.
    ///
    /// # Errors
    ///
    /// [`DataError::MalformedTable`] only if Arrow cannot concatenate the batches, which
    /// the checks in [`Table::new`] rule out.
    pub fn to_batch(&self) -> Result<RecordBatch, DataError> {
        match self.batches.as_slice() {
            [single] => Ok(single.clone()),
            batches => {
                concat_batches(&self.schema, batches).map_err(|source| self.malformed(source))
            }
        }
    }

    /// A reference to one row of this generation, or `None` past the last row.
    #[must_use]
    pub fn row_ref(&self, row: u64) -> Option<RowRef> {
        (row < self.num_rows).then_some(RowRef {
            table: self.id,
            generation: self.generation,
            row,
        })
    }

    /// Every row of this generation.
    #[must_use]
    pub const fn rows(&self) -> RowRange {
        RowRange {
            table: self.id,
            generation: self.generation,
            start: 0,
            end: self.num_rows,
        }
    }

    fn malformed(&self, source: arrow_schema::ArrowError) -> DataError {
        DataError::MalformedTable {
            table: self.name.clone(),
            source,
        }
    }
}

/// A batch's row count as `u64` (usize is at most 64 bits on supported targets).
fn row_count(batch: &RecordBatch) -> u64 {
    u64::try_from(batch.num_rows()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use arrow_array::{Float64Array, Int64Array, StringArray};
    use arrow_schema::{DataType, Field, Schema};

    use super::*;
    use crate::meta::{FieldType, Role};

    fn schema() -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("t", DataType::Int64, false),
            Field::new("v", DataType::Float64, true),
        ]))
    }

    fn batch(t: &[i64], v: &[Option<f64>]) -> RecordBatch {
        RecordBatch::try_new(
            schema(),
            vec![
                Arc::new(Int64Array::from(t.to_vec())),
                Arc::new(Float64Array::from(v.to_vec())),
            ],
        )
        .unwrap()
    }

    fn table() -> Table {
        Table::new(
            TableId(3),
            "series",
            schema(),
            vec![
                batch(&[1, 2], &[Some(0.5), None]),
                batch(&[3], &[Some(2.0)]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn columns_span_batches() {
        let table = table();
        assert_eq!(table.num_rows(), 3);
        assert_eq!(table.num_fields(), 2);
        let t = table.column("t").unwrap();
        assert_eq!(t.as_ref(), &Int64Array::from(vec![1, 2, 3]));
        assert_eq!(table.to_batch().unwrap().num_rows(), 3);
    }

    #[test]
    fn batches_that_do_not_match_the_schema_are_rejected() {
        let other = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("s", DataType::Utf8, true)])),
            vec![Arc::new(StringArray::from(vec!["a"]))],
        )
        .unwrap();
        let err = Table::new(TableId(0), "bad", schema(), vec![other]).unwrap_err();
        assert!(matches!(err, DataError::MalformedTable { .. }), "{err}");
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-004");
    }

    #[test]
    fn edits_produce_a_new_generation_and_keep_old_row_refs_distinct() {
        let table = table();
        let meta = FieldMeta::new()
            .with_type(FieldType::Temporal)
            .with_role(Role::X);
        let edited = table.with_field_meta("t", &meta).unwrap();
        assert_eq!(edited.id(), table.id());
        assert_eq!(edited.generation(), Generation(1));
        assert_eq!(edited.field_meta("t").unwrap(), meta);
        assert_eq!(table.field_meta("t").unwrap(), FieldMeta::new());
        let old = table.row_ref(2).unwrap();
        assert!(table.rows().contains(&old));
        assert!(!edited.rows().contains(&old));
        assert_eq!(table.row_ref(3), None);
    }

    #[test]
    fn unknown_fields_are_errors_with_a_code() {
        let err = table().column("nope").unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-050");
        assert!(err.to_string().contains("`nope`"));
    }

    #[test]
    fn an_empty_table_has_empty_columns() {
        let table = Table::new(TableId(0), "empty", schema(), Vec::new()).unwrap();
        assert_eq!(table.num_rows(), 0);
        assert_eq!(table.column("v").unwrap().len(), 0);
    }
}

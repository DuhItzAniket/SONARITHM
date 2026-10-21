// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Rows collected between appends, turned into one Arrow batch.

use std::sync::Arc;

use arrow_array::{ArrayRef, BooleanArray, Float64Array, RecordBatch, StringArray};

use crate::StreamError;
use crate::decode::Cell;
use crate::schema::{FieldKind, MessageSchema};

/// Decoded rows waiting to be appended, column by column.
#[derive(Debug)]
pub struct RowBuffer {
    schema: MessageSchema,
    columns: Vec<Vec<Cell>>,
    rows: usize,
}

impl RowBuffer {
    /// An empty buffer for rows of `schema`.
    #[must_use]
    pub fn new(schema: MessageSchema) -> Self {
        let columns = vec![Vec::new(); schema.fields().len()];
        Self {
            schema,
            columns,
            rows: 0,
        }
    }

    /// Adds a row whose cells follow the schema's fields.
    ///
    /// # Errors
    ///
    /// [`StreamError::Decode`] if the row has another number of cells.
    pub fn push(&mut self, row: Vec<Cell>) -> Result<(), StreamError> {
        if row.len() != self.columns.len() {
            return Err(StreamError::Decode(format!(
                "a row has {} values, not {}",
                row.len(),
                self.columns.len()
            )));
        }
        for (column, cell) in self.columns.iter_mut().zip(row) {
            column.push(cell);
        }
        self.rows += 1;
        Ok(())
    }

    /// Rows waiting.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.rows
    }

    /// Whether no rows wait.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rows == 0
    }

    /// The schema.
    #[must_use]
    pub const fn schema(&self) -> &MessageSchema {
        &self.schema
    }

    /// The waiting rows as one batch; the buffer is empty afterwards.
    ///
    /// # Errors
    ///
    /// [`StreamError::Table`] if Arrow refuses the columns (not expected).
    pub fn take(&mut self) -> Result<RecordBatch, StreamError> {
        let arrays: Vec<ArrayRef> = self
            .schema
            .fields()
            .iter()
            .zip(&mut self.columns)
            .map(|((_, kind, _), cells)| {
                let cells = std::mem::take(cells);
                let array: ArrayRef = match kind {
                    FieldKind::Number => Arc::new(
                        cells
                            .into_iter()
                            .map(|c| match c {
                                Cell::Number(v) => Some(v),
                                _ => None,
                            })
                            .collect::<Float64Array>(),
                    ),
                    FieldKind::Text => Arc::new(
                        cells
                            .into_iter()
                            .map(|c| match c {
                                Cell::Text(v) => Some(v),
                                _ => None,
                            })
                            .collect::<StringArray>(),
                    ),
                    FieldKind::Boolean => Arc::new(
                        cells
                            .into_iter()
                            .map(|c| match c {
                                Cell::Bool(v) => Some(v),
                                _ => None,
                            })
                            .collect::<BooleanArray>(),
                    ),
                };
                array
            })
            .collect();
        self.rows = 0;
        RecordBatch::try_new(self.schema.arrow(), arrays)
            .map_err(|err| StreamError::Table(err.to_string()))
    }
}

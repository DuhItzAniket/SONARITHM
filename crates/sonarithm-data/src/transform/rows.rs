// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Row-wise transforms: `filter`, `sort` and `derive` (03 §6).
//!
//! `filter` and `sort` move rows with their provenance; `derive` leaves each row's
//! provenance unchanged, since its new value comes from that row (03 §4).

use arrow_array::{ArrayRef, UInt32Array};
use arrow_ord::ord::make_comparator;
use arrow_schema::SortOptions;
use rhai::Scope;
use sonarithm_core::{Diagnostic, Diagnostics, Generation, RowRange};

use super::expr::{self, Expr, Outcome, Value};
use super::{SortOrder, Working, computed_field};
use crate::codes;
use crate::error::DataError;

/// Rows that failed an expression, for one warning.
#[derive(Default)]
struct Failures {
    count: u64,
    first: Option<(usize, String)>,
}

impl Failures {
    fn note(&mut self, row: usize, reason: String) {
        self.count += 1;
        if self.first.is_none() {
            self.first = Some((row, reason));
        }
    }

    fn report(self, expr: &Expr, effect: &str, diagnostics: &mut Diagnostics) {
        if let Some((row, reason)) = self.first {
            let row = u64::try_from(row).unwrap_or(u64::MAX);
            let mut diagnostic = Diagnostic::new(
                &codes::EXPRESSION_FAILED,
                format!(
                    "The expression `{}` failed on {} row(s), first at row {} ({reason}); {effect}.",
                    expr.source(),
                    self.count,
                    row + 1
                ),
            );
            diagnostic = diagnostic.rows(RowRange {
                table: sonarithm_core::TableId::default(),
                generation: Generation::default(),
                start: row,
                end: row + 1,
            });
            diagnostics.push(diagnostic);
        }
    }
}

/// Keeps the rows where `source` is `true`; `false`, missing and failed rows are dropped.
pub(super) fn filter(
    working: &mut Working,
    source: &str,
    diagnostics: &mut Diagnostics,
) -> Result<String, DataError> {
    let expr = Expr::compile(source, &working.schema(), &working.columns)?;
    let mut scope = Scope::new();
    let mut keep: Vec<u32> = Vec::new();
    let mut failures = Failures::default();
    for row in 0..working.rows {
        match expr.eval(row, &mut scope)? {
            Outcome::Value(Value::Bool(true)) => keep.push(index(row)?),
            Outcome::Value(Value::Bool(false) | Value::Null) => {}
            Outcome::Value(other) => {
                return Err(DataError::InvalidExpression {
                    expr: source.to_owned(),
                    reason: format!("a filter must give true or false, but gave {other:?}"),
                });
            }
            Outcome::Failed(reason) => failures.note(row, reason),
        }
    }
    failures.report(&expr, "those rows were dropped", diagnostics);
    take(working, &keep)?;
    Ok(format!("`{source}`"))
}

/// Orders rows by one field, stably, with missing values last in either direction.
pub(super) fn sort(
    working: &mut Working,
    field: &str,
    order: SortOrder,
) -> Result<String, DataError> {
    let column = working
        .columns
        .get(working.index(field)?)
        .cloned()
        .unwrap_or_else(|| arrow_array::new_empty_array(&arrow_schema::DataType::Null));
    let options = SortOptions {
        descending: order == SortOrder::Descending,
        nulls_first: false,
    };
    let compare = make_comparator(column.as_ref(), column.as_ref(), options).map_err(|_| {
        DataError::FieldTypeMismatch {
            op: "sort",
            field: field.to_owned(),
            expected: "comparable values",
            found: column.data_type().to_string(),
        }
    })?;
    let mut rows: Vec<usize> = (0..working.rows).collect();
    // `sort_by` is stable: rows with equal keys keep their order.
    rows.sort_by(|&a, &b| compare(a, b));
    let rows: Vec<u32> = rows.into_iter().map(index).collect::<Result<_, _>>()?;
    take(working, &rows)?;
    Ok(format!(
        "by `{field}`, {}",
        if order == SortOrder::Descending {
            "descending"
        } else {
            "ascending"
        }
    ))
}

/// Adds or replaces the field `output` with the value of `source` on each row.
pub(super) fn derive(
    working: &mut Working,
    source: &str,
    output: &str,
    diagnostics: &mut Diagnostics,
) -> Result<String, DataError> {
    if output.is_empty() || output.starts_with(crate::infer::semantic::RESERVED_PREFIX) {
        return Err(DataError::InvalidTransform {
            op: "derive",
            reason: format!("`{output}` cannot be a field name"),
        });
    }
    let expr = Expr::compile(source, &working.schema(), &working.columns)?;
    let mut scope = Scope::new();
    let mut values = Vec::with_capacity(working.rows);
    let mut failures = Failures::default();
    for row in 0..working.rows {
        match expr.eval(row, &mut scope)? {
            Outcome::Value(value) => values.push(value),
            Outcome::Failed(reason) => {
                failures.note(row, reason);
                values.push(Value::Null);
            }
        }
    }
    failures.report(&expr, "their values are missing", diagnostics);
    let column: ArrayRef = expr::column_of(values, source)?;
    working.put(computed_field(output, &column), column);
    Ok(format!("`{output}` = `{source}`"))
}

/// Keeps the rows `rows` select, in that order, with their provenance.
fn take(working: &mut Working, rows: &[u32]) -> Result<(), DataError> {
    let indices = UInt32Array::from(rows.to_vec());
    let mut columns = Vec::with_capacity(working.columns.len());
    for column in &working.columns {
        columns.push(
            arrow_select::take::take(column.as_ref(), &indices, None).map_err(|source| {
                DataError::MalformedTable {
                    table: working.table_name.clone(),
                    source,
                }
            })?,
        );
    }
    let positions: Vec<usize> = rows.iter().map(|&r| r as usize).collect();
    working.provenance = working.provenance.take(&positions);
    working.columns = columns;
    working.rows = rows.len();
    Ok(())
}

/// A row position as a `u32` take index.
fn index(row: usize) -> Result<u32, DataError> {
    u32::try_from(row).map_err(|_| DataError::LimitExceeded {
        limit: crate::error::Limit::Rows,
        max: u64::from(u32::MAX),
        found: u64::try_from(row).unwrap_or(u64::MAX),
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::cast::AsArray;
    use arrow_array::types::{Float64Type, Int64Type};
    use arrow_array::{Array, Float64Array, Int64Array, RecordBatch, StringArray};
    use arrow_schema::{DataType, Field, Schema};
    use proptest::prelude::*;
    use sonarithm_core::{Confidence, Generation, Provenance, ProvenanceClass, TableId};

    use crate::provenance::{RowProvenance, with_provenance};
    use crate::table::Table;
    use crate::transform::{SortOrder, Transform, apply};

    fn table(t: &[i64], v: &[Option<f64>], provenance: Option<RowProvenance>) -> Table {
        let schema = Arc::new(Schema::new(vec![
            Field::new("t", DataType::Int64, true),
            Field::new("v", DataType::Float64, true),
            Field::new("s", DataType::Utf8, true),
        ]));
        let s: StringArray = t.iter().map(|i| Some(format!("r{i}"))).collect();
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int64Array::from(t.to_vec())),
                Arc::new(Float64Array::from(v.to_vec())),
                Arc::new(s),
            ],
        )
        .unwrap();
        let table = Table::from_batch(TableId(0), "t", batch);
        match provenance {
            Some(p) => with_provenance(&table, &p, true).unwrap(),
            None => table,
        }
    }

    fn run(table: &Table, transform: &Transform) -> (Table, Vec<String>) {
        let mut diagnostics = sonarithm_core::Diagnostics::new();
        let out = apply(table, transform, &mut diagnostics).unwrap();
        (
            out,
            diagnostics
                .iter()
                .map(|d| d.code.as_str().to_owned())
                .collect(),
        )
    }

    fn ints(table: &Table, field: &str) -> Vec<Option<i64>> {
        table
            .column(field)
            .unwrap()
            .as_primitive::<Int64Type>()
            .iter()
            .collect()
    }

    #[test]
    fn filter_keeps_true_rows_and_their_provenance() {
        let estimate = Provenance {
            class: ProvenanceClass::CvEstimate,
            confidence: Confidence::new(0.5),
        };
        let p = RowProvenance::from_rows(vec![Provenance::EXACT, estimate, Provenance::EXACT]);
        let t = table(&[1, 2, 3], &[Some(1.0), None, Some(3.0)], Some(p));
        let (out, codes) = run(
            &t,
            &Transform::Filter {
                expr: "t >= 2".into(),
            },
        );
        assert_eq!(ints(&out, "t"), [Some(2), Some(3)]);
        assert_eq!(RowProvenance::read(&out).unwrap().get(0), estimate);
        assert_eq!(out.generation(), Generation(1));
        assert_eq!(out.id(), t.id());
        assert_eq!(codes, ["SNR-DATA-056"]);
        // Missing values make comparisons false.
        let (out, _) = run(
            &t,
            &Transform::Filter {
                expr: "v > 0.0".into(),
            },
        );
        assert_eq!(ints(&out, "t"), [Some(1), Some(3)]);
    }

    #[test]
    fn filters_must_be_boolean() {
        let t = table(&[1], &[Some(1.0)], None);
        let err = apply(
            &t,
            &Transform::Filter {
                expr: "t + 1".into(),
            },
            &mut sonarithm_core::Diagnostics::default(),
        )
        .unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-052");
    }

    #[test]
    fn sort_is_stable_with_missing_values_last() {
        let t = table(
            &[1, 2, 3, 4],
            &[Some(2.0), None, Some(1.0), Some(2.0)],
            None,
        );
        let (up, _) = run(
            &t,
            &Transform::Sort {
                field: "v".into(),
                order: SortOrder::Ascending,
            },
        );
        assert_eq!(ints(&up, "t"), [Some(3), Some(1), Some(4), Some(2)]);
        let (down, _) = run(
            &t,
            &Transform::Sort {
                field: "v".into(),
                order: SortOrder::Descending,
            },
        );
        assert_eq!(ints(&down, "t"), [Some(1), Some(4), Some(3), Some(2)]);
        let err = apply(
            &t,
            &Transform::Sort {
                field: "nope".into(),
                order: SortOrder::Ascending,
            },
            &mut sonarithm_core::Diagnostics::default(),
        )
        .unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-050");
    }

    #[test]
    fn derive_adds_a_typed_field_and_reports_failed_rows() {
        let t = table(&[1, 2, 0], &[Some(1.5), None, Some(3.0)], None);
        let (out, _) = run(
            &t,
            &Transform::Derive {
                expr: "v * 2.0".into(),
                output: "v2".into(),
            },
        );
        let v2: Vec<_> = out
            .column("v2")
            .unwrap()
            .as_primitive::<Float64Type>()
            .iter()
            .collect();
        assert_eq!(v2, [Some(3.0), None, Some(6.0)]);
        assert_eq!(
            out.field_meta("v2").unwrap().ftype,
            Some(crate::meta::FieldType::Quantitative)
        );
        let (out, codes) = run(
            &t,
            &Transform::Derive {
                expr: "10 / t".into(),
                output: "inv".into(),
            },
        );
        assert_eq!(ints(&out, "inv"), [Some(10), Some(5), None]);
        assert!(codes.contains(&"SNR-DATA-053".to_owned()), "{codes:?}");
        // Replacing a field keeps its position.
        let (out, _) = run(
            &t,
            &Transform::Derive {
                expr: "t + 1".into(),
                output: "t".into(),
            },
        );
        assert_eq!(out.schema().field(0).name(), "t");
        assert_eq!(ints(&out, "t"), [Some(2), Some(3), Some(1)]);
    }

    fn any_rows() -> impl Strategy<Value = Vec<(i64, Option<f64>)>> {
        prop::collection::vec((-50_i64..50, prop::option::of(-10.0_f64..10.0)), 0..40)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        #[test]
        fn filter_composes_like_a_conjunction(rows in any_rows(), a in -50_i64..50, b in -50_i64..50) {
            let (t, v): (Vec<i64>, Vec<Option<f64>>) = rows.into_iter().unzip();
            let input = table(&t, &v, None);
            let (once, _) = run(&input, &Transform::Filter { expr: format!("t > {a} && t < {b}") });
            let (first, _) = run(&input, &Transform::Filter { expr: format!("t > {a}") });
            let (twice, _) = run(&first, &Transform::Filter { expr: format!("t < {b}") });
            prop_assert_eq!(ints(&once, "t"), ints(&twice, "t"));
            let (all, _) = run(&input, &Transform::Filter { expr: "true".into() });
            prop_assert_eq!(all.to_batch().unwrap(), input.to_batch().unwrap());
        }

        #[test]
        fn sort_orders_a_permutation_and_is_idempotent(rows in any_rows()) {
            let (t, v): (Vec<i64>, Vec<Option<f64>>) = rows.into_iter().unzip();
            let input = table(&t, &v, None);
            let sort = Transform::Sort { field: "t".into(), order: SortOrder::Ascending };
            let (once, _) = run(&input, &sort);
            let (twice, _) = run(&once, &sort);
            let sorted = ints(&once, "t");
            let mut expected: Vec<Option<i64>> = t.iter().copied().map(Some).collect();
            expected.sort();
            prop_assert_eq!(&sorted, &expected);
            prop_assert_eq!(ints(&twice, "t"), sorted);
            prop_assert_eq!(once.num_rows(), input.num_rows());
        }

        #[test]
        fn derive_applies_the_expression_to_each_row(rows in any_rows()) {
            prop_assume!(!rows.is_empty());
            let (t, v): (Vec<i64>, Vec<Option<f64>>) = rows.into_iter().unzip();
            let input = table(&t, &v, None);
            let (out, _) = run(&input, &Transform::Derive { expr: "t * 3 - 1".into(), output: "d".into() });
            let expected: Vec<Option<i64>> = t.iter().map(|x| Some(x * 3 - 1)).collect();
            prop_assert_eq!(ints(&out, "d"), expected);
        }
    }

    #[test]
    fn the_column_helper_handles_empty_tables() {
        let t = table(&[], &[], None);
        let (out, _) = run(
            &t,
            &Transform::Derive {
                expr: "t + 1".into(),
                output: "d".into(),
            },
        );
        assert_eq!(out.column("d").unwrap().len(), 0);
    }
}

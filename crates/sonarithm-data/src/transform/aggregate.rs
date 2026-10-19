// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `aggregate` and `bin` (03 §6).
//!
//! `aggregate` groups rows (in order of first appearance) and records `derived` provenance
//! combined with every input row of the group, so an estimated input makes the result an
//! estimate (03 §4). `bin` adds each row's bin edges as two fields, Vega-Lite style, and
//! keeps each row's provenance.

use std::collections::HashMap;
use std::sync::Arc;

use arrow_array::{Array, ArrayRef, Float64Array, Int64Array, UInt32Array};
use arrow_ord::ord::make_comparator;
use arrow_row::{RowConverter, SortField};
use arrow_schema::{DataType, Field, SortOptions};
use serde::{Deserialize, Serialize};

use super::{Working, computed_field};
use crate::error::DataError;
use crate::infer::semantic::numbers;
use crate::meta::FieldMeta;
use crate::provenance::{RowProvenance, derived};

/// An aggregate function (RFC-0001 §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum AggregateFn {
    /// The sum of the non-missing values.
    Sum,
    /// The arithmetic mean of the non-missing values.
    Mean,
    /// The median of the non-missing values (the mean of the two middle values for an
    /// even count).
    Median,
    /// The smallest value; any comparable field.
    Min,
    /// The largest value; any comparable field.
    Max,
    /// The number of non-missing values of `field`, or of rows without a field.
    Count,
}

/// One output of an `aggregate` (RFC-0001 §4: `{ op, field, as }`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregateOp {
    /// The function.
    pub op: AggregateFn,
    /// The input field; optional for `count`.
    #[serde(default)]
    pub field: Option<String>,
    /// The output field.
    #[serde(rename = "as")]
    pub output: String,
}

/// How `bin` places edges (RFC-0001 §4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BinMethod {
    /// `count` bins of equal width over the value range.
    #[default]
    EqualWidth,
    /// `count` bins holding about the same number of values.
    Quantile,
    /// The explicit `edges`.
    Edges,
}

/// Groups rows by `groupby` and computes `ops` per group.
pub(super) fn aggregate(
    working: &mut Working,
    groupby: &[String],
    ops: &[AggregateOp],
) -> Result<String, DataError> {
    let invalid = |reason: String| DataError::InvalidTransform {
        op: "aggregate",
        reason,
    };
    if ops.is_empty() {
        return Err(invalid("`ops` must name at least one function".to_owned()));
    }
    let mut outputs: Vec<&str> = groupby.iter().map(String::as_str).collect();
    for op in ops {
        if outputs.contains(&op.output.as_str()) || op.output.is_empty() {
            return Err(invalid(format!(
                "the output name `{}` is empty or used twice",
                op.output
            )));
        }
        outputs.push(&op.output);
    }
    let keys: Vec<usize> = groupby
        .iter()
        .map(|g| working.index(g))
        .collect::<Result<_, _>>()?;
    let groups = group_rows(working, &keys)?;

    let firsts = UInt32Array::from(
        groups
            .iter()
            .map(|rows| rows.first().copied().unwrap_or(0))
            .collect::<Vec<u32>>(),
    );
    let mut fields = Vec::new();
    let mut columns = Vec::new();
    for &key in &keys {
        let (Some(field), Some(column)) = (working.fields.get(key), working.columns.get(key))
        else {
            continue;
        };
        fields.push(field.clone());
        columns.push(take(column, &firsts, &working.table_name)?);
    }
    for op in ops {
        let (field, column) = aggregate_column(working, op, &groups)?;
        fields.push(field);
        columns.push(column);
    }
    working.provenance = RowProvenance::from_rows(
        groups
            .iter()
            .map(|rows| derived(working.provenance.combine(rows.iter().map(|&r| r as usize))))
            .collect(),
    );
    working.rows = groups.len();
    working.fields = fields;
    working.columns = columns;
    let by = if groupby.is_empty() {
        "all rows".to_owned()
    } else {
        format!("by {}", groupby.join(", "))
    };
    Ok(format!("{} group(s) {by}", groups.len()))
}

/// Rows of each group, groups in order of first appearance.
fn group_rows(working: &Working, keys: &[usize]) -> Result<Vec<Vec<u32>>, DataError> {
    let all: Vec<u32> = (0..working.rows)
        .map(|r| u32::try_from(r).unwrap_or(u32::MAX))
        .collect();
    if keys.is_empty() {
        return Ok(if all.is_empty() {
            Vec::new()
        } else {
            vec![all]
        });
    }
    let columns: Vec<ArrayRef> = keys
        .iter()
        .filter_map(|&k| working.columns.get(k).cloned())
        .collect();
    let malformed = |source| DataError::MalformedTable {
        table: working.table_name.clone(),
        source,
    };
    let converter = RowConverter::new(
        columns
            .iter()
            .map(|c| SortField::new(c.data_type().clone()))
            .collect(),
    )
    .map_err(malformed)?;
    let rows = converter.convert_columns(&columns).map_err(malformed)?;
    // A map for lookups only; group order comes from `groups`.
    let mut index: HashMap<&[u8], usize> = HashMap::new();
    let mut groups: Vec<Vec<u32>> = Vec::new();
    for (row, key) in all.iter().zip(rows.iter()) {
        let next = groups.len();
        let group = *index.entry(key.data()).or_insert(next);
        if group == next {
            groups.push(Vec::new());
        }
        if let Some(rows) = groups.get_mut(group) {
            rows.push(*row);
        }
    }
    Ok(groups)
}

fn aggregate_column(
    working: &Working,
    op: &AggregateOp,
    groups: &[Vec<u32>],
) -> Result<(Field, ArrayRef), DataError> {
    let source = match &op.field {
        Some(name) => Some(working.index(name)?),
        None if op.op == AggregateFn::Count => None,
        None => {
            return Err(DataError::InvalidTransform {
                op: "aggregate",
                reason: format!("`{}` needs a `field`", op.output),
            });
        }
    };
    let input = source.and_then(|i| working.columns.get(i));
    let input_field = source.and_then(|i| working.fields.get(i));
    let column: ArrayRef = match (op.op, input) {
        (AggregateFn::Count, None) => Arc::new(Int64Array::from_iter_values(
            groups
                .iter()
                .map(|rows| i64::try_from(rows.len()).unwrap_or(i64::MAX)),
        )),
        (AggregateFn::Count, Some(column)) => {
            Arc::new(Int64Array::from_iter_values(groups.iter().map(|rows| {
                let valid = rows
                    .iter()
                    .filter(|&&r| column.is_valid(r as usize))
                    .count();
                i64::try_from(valid).unwrap_or(i64::MAX)
            })))
        }
        (AggregateFn::Min | AggregateFn::Max, Some(column)) => {
            let options = SortOptions {
                descending: op.op == AggregateFn::Max,
                nulls_first: false,
            };
            let compare = make_comparator(column.as_ref(), column.as_ref(), options)
                .map_err(|_| mismatch(op, column, "comparable values"))?;
            let picks: UInt32Array = groups
                .iter()
                .map(|rows| {
                    rows.iter()
                        .copied()
                        .filter(|&r| column.is_valid(r as usize))
                        .min_by(|&a, &b| compare(a as usize, b as usize).then(a.cmp(&b)))
                })
                .collect();
            take(column, &picks, &working.table_name)?
        }
        (_, Some(column)) => {
            let values = numbers(column)
                .filter(|_| column.data_type().is_numeric())
                .ok_or_else(|| mismatch(op, column, "numbers"))?;
            Arc::new(
                groups
                    .iter()
                    .map(|rows| reduce(op.op, &values, rows))
                    .collect::<Float64Array>(),
            )
        }
        (_, None) => unreachable_count(),
    };
    let mut field = computed_field(&op.output, &column);
    if op.op != AggregateFn::Count
        && let Some(input_field) = input_field
    {
        let (source_meta, _) = FieldMeta::read(input_field);
        let (mut meta, _) = FieldMeta::read(&field);
        meta.unit = source_meta.unit;
        meta.concept = source_meta.concept;
        field = meta.apply(field);
    }
    Ok((field, column))
}

/// Without a field only `count` gets this far (checked in `aggregate_column`); an empty
/// column keeps this panic-free.
fn unreachable_count() -> ArrayRef {
    arrow_array::new_empty_array(&DataType::Float64)
}

fn mismatch(op: &AggregateOp, column: &ArrayRef, expected: &'static str) -> DataError {
    DataError::FieldTypeMismatch {
        op: "aggregate",
        field: op.field.clone().unwrap_or_default(),
        expected,
        found: column.data_type().to_string(),
    }
}

/// Sum, mean or median of a group's non-missing values; missing when there are none.
fn reduce(function: AggregateFn, values: &Float64Array, rows: &[u32]) -> Option<f64> {
    let mut present: Vec<f64> = rows
        .iter()
        .filter(|&&r| values.is_valid(r as usize))
        .map(|&r| values.value(r as usize))
        .collect();
    if present.is_empty() {
        return None;
    }
    match function {
        AggregateFn::Sum => Some(neumaier_sum(&present)),
        AggregateFn::Mean => {
            #[expect(clippy::cast_precision_loss, reason = "group sizes are below 2^53")]
            let n = present.len() as f64;
            Some(neumaier_sum(&present) / n)
        }
        AggregateFn::Median => {
            present.sort_by(f64::total_cmp);
            let mid = present.len() / 2;
            if present.len() % 2 == 1 {
                present.get(mid).copied()
            } else {
                match (present.get(mid - 1), present.get(mid)) {
                    (Some(a), Some(b)) => Some(a + (b - a) / 2.0),
                    _ => None,
                }
            }
        }
        _ => None,
    }
}

/// Compensated (Neumaier) summation in row order: deterministic and accurate.
fn neumaier_sum(values: &[f64]) -> f64 {
    let mut sum = 0.0_f64;
    let mut compensation = 0.0_f64;
    for &v in values {
        let t = sum + v;
        if sum.abs() >= v.abs() {
            compensation += (sum - t) + v;
        } else {
            compensation += (v - t) + sum;
        }
        sum = t;
    }
    sum + compensation
}

/// Adds the bin edges of `field` as `{output}` (start) and `{output}_end`.
pub(super) fn bin(
    working: &mut Working,
    field: &str,
    method: BinMethod,
    count: Option<u32>,
    edges: Option<&[f64]>,
    output: &str,
) -> Result<String, DataError> {
    let invalid = |reason: &str| DataError::InvalidTransform {
        op: "bin",
        reason: reason.to_owned(),
    };
    let index = working.index(field)?;
    let column = working
        .columns
        .get(index)
        .cloned()
        .ok_or_else(|| invalid("missing column"))?;
    let values = numbers(&column)
        .filter(|_| column.data_type().is_numeric())
        .ok_or_else(|| DataError::FieldTypeMismatch {
            op: "bin",
            field: field.to_owned(),
            expected: "numbers",
            found: column.data_type().to_string(),
        })?;
    let mut finite: Vec<f64> = values.iter().flatten().filter(|v| v.is_finite()).collect();
    finite.sort_by(f64::total_cmp);
    let edges: Vec<f64> = match method {
        BinMethod::EqualWidth | BinMethod::Quantile => {
            let count = count
                .filter(|c| *c > 0)
                .ok_or_else(|| invalid("`count` must be at least 1"))?;
            if method == BinMethod::EqualWidth {
                equal_width_edges(&finite, count)
            } else {
                quantile_edges(&finite, count)
            }
        }
        BinMethod::Edges => {
            let edges = edges.ok_or_else(|| invalid("`edges` is required for the edges method"))?;
            if edges.len() < 2
                || edges.iter().any(|e| !e.is_finite())
                || edges.windows(2).any(|w| w[0] >= w[1])
            {
                return Err(invalid(
                    "`edges` must be at least two finite numbers in increasing order",
                ));
            }
            edges.to_vec()
        }
    };
    let mut starts = Vec::with_capacity(working.rows);
    let mut ends = Vec::with_capacity(working.rows);
    for value in &values {
        let bin = value.and_then(|v| locate(&edges, v));
        starts.push(bin.and_then(|i| edges.get(i).copied()));
        ends.push(bin.and_then(|i| edges.get(i + 1).or_else(|| edges.get(i)).copied()));
    }
    let (source_meta, _) = working
        .fields
        .get(index)
        .map(FieldMeta::read)
        .unwrap_or_default();
    let mut put = |name: String, data: Vec<Option<f64>>| {
        let column: ArrayRef = Arc::new(Float64Array::from(data));
        let base = computed_field(&name, &column);
        let (mut meta, _) = FieldMeta::read(&base);
        meta.unit.clone_from(&source_meta.unit);
        working.put(meta.apply(base), column);
    };
    put(output.to_owned(), starts);
    put(format!("{output}_end"), ends);
    Ok(format!(
        "`{field}` into {} bin(s)",
        edges.len().saturating_sub(1).max(1)
    ))
}

/// `count` equal-width bins over [min, max]; one degenerate bin when they are equal.
fn equal_width_edges(sorted: &[f64], count: u32) -> Vec<f64> {
    let (Some(&lo), Some(&hi)) = (sorted.first(), sorted.last()) else {
        return Vec::new();
    };
    #[expect(
        clippy::float_cmp,
        reason = "exact equality of data values is intended"
    )]
    if lo == hi {
        return vec![lo];
    }
    let width = (hi - lo) / f64::from(count);
    let mut edges: Vec<f64> = (0..count).map(|k| lo + f64::from(k) * width).collect();
    edges.push(hi);
    edges
}

/// Edges at the quantiles k/count (linear interpolation, type 7), repeated edges merged.
fn quantile_edges(sorted: &[f64], count: u32) -> Vec<f64> {
    if sorted.is_empty() {
        return Vec::new();
    }
    #[expect(clippy::cast_precision_loss, reason = "value counts are below 2^53")]
    let last = (sorted.len() - 1) as f64;
    let mut edges: Vec<f64> = (0..=count)
        .filter_map(|k| {
            let position = last * f64::from(k) / f64::from(count);
            let below = position.floor();
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "0 <= position <= len - 1"
            )]
            let (lower, upper) = (below as usize, position.ceil() as usize);
            let (low, high) = (sorted.get(lower)?, sorted.get(upper)?);
            Some(low + (position - below) * (high - low))
        })
        .collect();
    edges.dedup();
    edges
}

/// The bin of `v`: `edges[i] <= v < edges[i + 1]`, the last bin closed on the right; a
/// single edge is one degenerate bin. `None` outside the edges.
fn locate(edges: &[f64], v: f64) -> Option<usize> {
    match edges {
        [] => None,
        #[expect(clippy::float_cmp, reason = "a degenerate bin holds exactly its edge")]
        [only] => (v == *only).then_some(0),
        [first, .., last] => {
            if !(v >= *first && v <= *last) {
                return None;
            }
            let after = edges.partition_point(|e| *e <= v);
            Some(after.saturating_sub(1).min(edges.len() - 2))
        }
    }
}

fn take(column: &ArrayRef, indices: &UInt32Array, table: &str) -> Result<ArrayRef, DataError> {
    arrow_select::take::take(column.as_ref(), indices, None).map_err(|source| {
        DataError::MalformedTable {
            table: table.to_owned(),
            source,
        }
    })
}

/// The values of a `Float64` column, for tests.
#[cfg(test)]
pub(super) fn floats(column: &ArrayRef) -> Vec<Option<f64>> {
    use arrow_array::cast::AsArray;
    column
        .as_primitive::<arrow_array::types::Float64Type>()
        .iter()
        .collect()
}

#[cfg(test)]
#[allow(clippy::float_cmp, reason = "integer-valued sums are exact")]
mod tests {
    use arrow_array::cast::AsArray;
    use arrow_array::types::Int64Type;
    use arrow_array::{RecordBatch, StringArray};
    use arrow_schema::Schema;
    use proptest::prelude::*;
    use sonarithm_core::{Confidence, Diagnostics, Provenance, ProvenanceClass, TableId};

    use super::*;
    use crate::provenance::with_provenance;
    use crate::table::Table;
    use crate::transform::{Transform, apply};

    fn table(groups: &[&str], values: &[Option<f64>], provenance: Option<RowProvenance>) -> Table {
        let schema = Arc::new(Schema::new(vec![
            Field::new("g", DataType::Utf8, true),
            Field::new("v", DataType::Float64, true),
        ]));
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(groups.to_vec())),
                Arc::new(Float64Array::from(values.to_vec())),
            ],
        )
        .unwrap();
        let t = Table::from_batch(TableId(0), "t", batch);
        provenance.map_or_else(|| t.clone(), |p| with_provenance(&t, &p, true).unwrap())
    }

    fn op(op: AggregateFn, field: Option<&str>, output: &str) -> AggregateOp {
        AggregateOp {
            op,
            field: field.map(str::to_owned),
            output: output.to_owned(),
        }
    }

    fn run(t: &Table, transform: &Transform) -> Table {
        apply(t, transform, &mut Diagnostics::new()).unwrap()
    }

    #[test]
    fn groups_keep_first_appearance_order_and_every_function_works() {
        let t = table(
            &["b", "a", "b", "a", "b"],
            &[Some(1.0), Some(10.0), Some(3.0), None, Some(2.0)],
            None,
        );
        let out = run(
            &t,
            &Transform::Aggregate {
                groupby: vec!["g".into()],
                ops: vec![
                    op(AggregateFn::Sum, Some("v"), "sum"),
                    op(AggregateFn::Mean, Some("v"), "mean"),
                    op(AggregateFn::Median, Some("v"), "median"),
                    op(AggregateFn::Min, Some("v"), "min"),
                    op(AggregateFn::Max, Some("v"), "max"),
                    op(AggregateFn::Count, Some("v"), "n"),
                    op(AggregateFn::Count, None, "rows"),
                ],
            },
        );
        let g: Vec<_> = out
            .column("g")
            .unwrap()
            .as_string::<i32>()
            .iter()
            .flatten()
            .map(str::to_owned)
            .collect();
        assert_eq!(g, ["b", "a"]);
        assert_eq!(floats(&out.column("sum").unwrap()), [Some(6.0), Some(10.0)]);
        assert_eq!(
            floats(&out.column("mean").unwrap()),
            [Some(2.0), Some(10.0)]
        );
        assert_eq!(
            floats(&out.column("median").unwrap()),
            [Some(2.0), Some(10.0)]
        );
        assert_eq!(floats(&out.column("min").unwrap()), [Some(1.0), Some(10.0)]);
        assert_eq!(floats(&out.column("max").unwrap()), [Some(3.0), Some(10.0)]);
        let n: Vec<_> = out
            .column("n")
            .unwrap()
            .as_primitive::<Int64Type>()
            .iter()
            .collect();
        assert_eq!(n, [Some(3), Some(1)]);
        let rows: Vec<_> = out
            .column("rows")
            .unwrap()
            .as_primitive::<Int64Type>()
            .iter()
            .collect();
        assert_eq!(rows, [Some(3), Some(2)]);
        // Exact inputs aggregate to derived values.
        assert_eq!(
            RowProvenance::read(&out).unwrap().get(0).class,
            ProvenanceClass::Derived
        );
    }

    #[test]
    fn an_estimated_input_makes_the_group_an_estimate() {
        let estimate = Provenance {
            class: ProvenanceClass::MlEstimate,
            confidence: Confidence::new(0.4),
        };
        let p = RowProvenance::from_rows(vec![Provenance::EXACT, estimate, Provenance::EXACT]);
        let t = table(
            &["a", "a", "b"],
            &[Some(1.0), Some(2.0), Some(3.0)],
            Some(p),
        );
        let out = run(
            &t,
            &Transform::Aggregate {
                groupby: vec!["g".into()],
                ops: vec![op(AggregateFn::Mean, Some("v"), "m")],
            },
        );
        let p = RowProvenance::read(&out).unwrap();
        assert_eq!(p.get(0), estimate);
        assert_eq!(p.get(1).class, ProvenanceClass::Derived);
    }

    #[test]
    fn aggregate_parameters_are_checked() {
        let t = table(&["a"], &[Some(1.0)], None);
        for ops in [
            vec![],
            vec![op(AggregateFn::Sum, None, "s")],
            vec![op(AggregateFn::Sum, Some("v"), "g")],
        ] {
            let err = apply(
                &t,
                &Transform::Aggregate {
                    groupby: vec!["g".into()],
                    ops,
                },
                &mut Diagnostics::new(),
            )
            .unwrap_err();
            assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-055");
        }
        let err = apply(
            &t,
            &Transform::Aggregate {
                groupby: vec![],
                ops: vec![op(AggregateFn::Sum, Some("g"), "s")],
            },
            &mut Diagnostics::new(),
        )
        .unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-051");
    }

    fn bin_transform(method: BinMethod, count: Option<u32>, edges: Option<Vec<f64>>) -> Transform {
        Transform::Bin {
            field: "v".into(),
            method,
            count,
            edges,
            output: "b".into(),
        }
    }

    #[test]
    fn bins_by_equal_width_quantile_and_edges() {
        let t = table(
            &["a"; 5],
            &[Some(0.0), Some(1.0), Some(2.5), Some(10.0), None],
            None,
        );
        let out = run(&t, &bin_transform(BinMethod::EqualWidth, Some(4), None));
        assert_eq!(
            floats(&out.column("b").unwrap()),
            [Some(0.0), Some(0.0), Some(2.5), Some(7.5), None]
        );
        assert_eq!(
            floats(&out.column("b_end").unwrap()),
            [Some(2.5), Some(2.5), Some(5.0), Some(10.0), None]
        );
        let out = run(
            &t,
            &bin_transform(BinMethod::Edges, None, Some(vec![0.0, 1.0, 5.0])),
        );
        assert_eq!(
            floats(&out.column("b").unwrap()),
            [Some(0.0), Some(1.0), Some(1.0), None, None]
        );
        let out = run(&t, &bin_transform(BinMethod::Quantile, Some(2), None));
        assert_eq!(
            floats(&out.column("b").unwrap()),
            [Some(0.0), Some(0.0), Some(1.75), Some(1.75), None]
        );
        let err = apply(
            &t,
            &bin_transform(BinMethod::Edges, None, Some(vec![1.0, 0.0])),
            &mut Diagnostics::new(),
        )
        .unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-055");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        #[test]
        fn group_sums_and_counts_add_up(rows in prop::collection::vec((0_u8..4, prop::option::of(-1000_i32..1000)), 0..60)) {
            let groups: Vec<String> = rows.iter().map(|(g, _)| format!("g{g}")).collect();
            let values: Vec<Option<f64>> = rows.iter().map(|(_, v)| v.map(f64::from)).collect();
            let refs: Vec<&str> = groups.iter().map(String::as_str).collect();
            let t = table(&refs, &values, None);
            let out = run(&t, &Transform::Aggregate {
                groupby: vec!["g".into()],
                ops: vec![op(AggregateFn::Sum, Some("v"), "s"), op(AggregateFn::Count, Some("v"), "n")],
            });
            let total: f64 = values.iter().flatten().sum();
            let sums: f64 = floats(&out.column("s").unwrap()).into_iter().flatten().sum();
            prop_assert_eq!(sums, total, "integer-valued sums are exact");
            let n: i64 = out.column("n").unwrap().as_primitive::<Int64Type>().iter().flatten().sum();
            prop_assert_eq!(n, i64::try_from(values.iter().flatten().count()).unwrap());
            let distinct: std::collections::BTreeSet<&str> = refs.iter().copied().collect();
            prop_assert_eq!(out.num_rows(), distinct.len() as u64);
        }

        #[test]
        fn every_value_lies_in_its_bin(values in prop::collection::vec(prop::option::of(-100.0_f64..100.0), 1..60), count in 1_u32..12) {
            let t = table(&vec!["a"; values.len()], &values, None);
            for method in [BinMethod::EqualWidth, BinMethod::Quantile] {
                let out = run(&t, &bin_transform(method, Some(count), None));
                let starts = floats(&out.column("b").unwrap());
                let ends = floats(&out.column("b_end").unwrap());
                for ((v, s), e) in values.iter().zip(&starts).zip(&ends) {
                    match (v, s, e) {
                        (Some(v), Some(s), Some(e)) => prop_assert!(s <= v && v <= e, "{s} <= {v} <= {e}"),
                        (None, None, None) => {}
                        other => prop_assert!(false, "{other:?}"),
                    }
                }
                let distinct: std::collections::BTreeSet<u64> = starts.iter().flatten().map(|s| s.to_bits()).collect();
                prop_assert!(distinct.len() <= count as usize);
            }
        }
    }
}

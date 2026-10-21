// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Binary views of datasets (B04-T03, 03 §7, ADR-0017 §6): M4 and LTTB tiles for plots and
//! pages of rows for the data table. All downsampling happens here, in Rust (09 §3).

use sonarithm_data::view::{PageSort, PageValues, axis_values, table_page};
use sonarithm_data::{DataError, Table, TileCache, XRange, lttb};

use crate::ipc::binary::{Column, ROW_COLUMN, Values, View, ViewKind};
use crate::ipc::{IpcError, TableSort, ViewMethod, ViewRange};

/// The tiles kept per app.
pub(crate) const TILE_CACHE: usize = 64;

fn data_error(err: &DataError) -> IpcError {
    IpcError::from_diagnostic(&err.to_diagnostic())
}

fn encode(view: &View) -> Result<Vec<u8>, IpcError> {
    view.encode().map_err(|err| {
        IpcError::from_diagnostic(
            &DataError::InvalidView {
                reason: err.to_string(),
            }
            .to_diagnostic(),
        )
    })
}

fn to_i64(rows: &[u64]) -> Vec<i64> {
    rows.iter()
        .map(|r| i64::try_from(*r).unwrap_or(i64::MAX))
        .collect()
}

/// A downsampled tile of `y` over `x`: the columns `row`, `x` and `y`.
pub(crate) fn tile(
    table: &Table,
    tiles: &mut TileCache,
    x: &str,
    y: &str,
    range: Option<ViewRange>,
    width: u32,
    method: ViewMethod,
) -> Result<Vec<u8>, IpcError> {
    let xs = axis_values(table, x).map_err(|err| data_error(&err))?;
    let ys = axis_values(table, y).map_err(|err| data_error(&err))?;
    let (rows, kind) = match method {
        ViewMethod::M4 => {
            let range = match range {
                Some(r) => XRange::new(r.start, r.end),
                None => full_range(&xs),
            }
            .map_err(|err| data_error(&err))?;
            let rows = tiles
                .tile(table, x, y, range, width)
                .map_err(|err| data_error(&err))?;
            (rows.to_vec(), ViewKind::M4Tile)
        }
        ViewMethod::Lttb => {
            if width < 3 {
                return Err(data_error(&DataError::InvalidView {
                    reason: "an overview needs at least 3 points".to_owned(),
                }));
            }
            (lttb(&xs, &ys, width as usize), ViewKind::LttbTile)
        }
    };
    let pick = |values: &arrow_array::Float64Array| -> Vec<f64> {
        rows.iter()
            .map(|r| values.value(usize::try_from(*r).unwrap_or(0)))
            .collect()
    };
    let view = View::new(kind, table.generation().0, rows.len() as u64)
        .with_column(Column::new("row", Values::I64(to_i64(&rows))))
        .and_then(|v| v.with_column(Column::new("x", Values::F64(pick(&xs)))))
        .and_then(|v| v.with_column(Column::new("y", Values::F64(pick(&ys)))))
        .map_err(|err| {
            data_error(&DataError::InvalidView {
                reason: err.to_string(),
            })
        })?;
    encode(&view)
}

/// All of x: from its smallest to just past its largest value, so the largest row lands in
/// the last pixel column.
fn full_range(xs: &arrow_array::Float64Array) -> Result<XRange, DataError> {
    use arrow_array::Array as _;
    let (lo, hi) = (0..xs.len())
        .filter(|&i| xs.is_valid(i) && xs.value(i).is_finite())
        .map(|i| xs.value(i))
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| {
            (a.min(v), b.max(v))
        });
    if lo > hi {
        return Err(DataError::InvalidView {
            reason: "x has no values".to_owned(),
        });
    }
    let pad = if hi > lo { (hi - lo) * 1e-9 } else { 1.0 };
    XRange::new(lo, hi + pad)
}

/// A page of rows: `__snr_row`, then every field.
pub(crate) fn page(
    table: &Table,
    offset: u64,
    limit: u32,
    sort: Option<&TableSort>,
) -> Result<Vec<u8>, IpcError> {
    let sort = sort.map(|s| PageSort::new(s.field.clone(), s.descending));
    let page = table_page(table, offset, limit, sort.as_ref()).map_err(|err| data_error(&err))?;
    let mut view = View::new(
        ViewKind::TablePage,
        table.generation().0,
        page.rows.len() as u64,
    )
    .with_column(Column::new(ROW_COLUMN, Values::I64(to_i64(&page.rows))))
    .map_err(|err| {
        data_error(&DataError::InvalidView {
            reason: err.to_string(),
        })
    })?;
    for (name, values) in page.columns {
        let column = column(name, values);
        view = view.with_column(column).map_err(|err| {
            data_error(&DataError::InvalidView {
                reason: err.to_string(),
            })
        })?;
    }
    encode(&view)
}

/// A page column as a view column, with a validity mask when a value is missing.
fn column(name: String, values: PageValues) -> Column {
    fn split<T: Default>(cells: Vec<Option<T>>) -> (Vec<T>, Option<Vec<bool>>) {
        let valid: Vec<bool> = cells.iter().map(Option::is_some).collect();
        let values = cells.into_iter().map(Option::unwrap_or_default).collect();
        let mask = valid.iter().any(|v| !v).then_some(valid);
        (values, mask)
    }
    let (values, mask) = match values {
        PageValues::F64(cells) => {
            let (v, m) = split(cells);
            (Values::F64(v), m)
        }
        PageValues::I64(cells) => {
            let (v, m) = split(cells);
            (Values::I64(v), m)
        }
        PageValues::Text(cells) => {
            let (v, m) = split(cells);
            (Values::Utf8(v), m)
        }
        PageValues::Bool(cells) => {
            let (v, m) = split(cells);
            (Values::Bool(v), m)
        }
        PageValues::Provenance(codes) => (Values::Provenance(codes), None),
        PageValues::Confidence(cells) => {
            let (v, m) = split(cells);
            (Values::Confidence(v), m)
        }
        _ => (Values::Utf8(Vec::new()), None),
    };
    let column = Column::new(name, values);
    match mask {
        Some(mask) => column.with_validity(mask),
        None => column,
    }
}

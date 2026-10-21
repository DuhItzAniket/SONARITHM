// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Binary views (ADR-0017 §6): bulk data for the UI as one little-endian buffer the UI
//! reads with typed-array views, without copying or JSON (F-041, PB-09).
//!
//! ```text
//! [header, 32 bytes][column directory, 16 bytes per column][column names, padded to 8]
//! [column 0: validity bitmap (if any), padded to 8][column 0 data, padded to 8] …
//! ```
//!
//! | Header offset | Size | Field |
//! |---|---|---|
//! | 0 | 4 | [`MAGIC`] `SNRV` |
//! | 4 | 2 | [`HEADER_VERSION`] |
//! | 6 | 2 | [`ViewKind`] |
//! | 8 | 4 | Column count |
//! | 12 | 4 | Reserved, 0 |
//! | 16 | 8 | Row count |
//! | 24 | 8 | Data generation |
//!
//! A directory entry holds the [`DataType`] (u16), flags (u16, [`FLAG_VALIDITY`]), the
//! name's length in bytes (u32) and the offset of the column's data from the start of the
//! buffer (u64). Names follow the directory in column order. A validity bitmap (bit `i` of
//! byte `i / 8`, least significant first, set when row `i` has a value) sits just before
//! its column's data. UTF-8 columns hold `rows + 1` u32 offsets, padded to 8, then the
//! bytes. Booleans are one byte per row.

use std::fmt;

/// The first four bytes of every view.
pub const MAGIC: [u8; 4] = *b"SNRV";
/// The header layout version.
pub const HEADER_VERSION: u16 = 1;
/// The header's size in bytes.
pub const HEADER_BYTES: usize = 32;
/// A directory entry's size in bytes.
pub const COLUMN_ENTRY_BYTES: usize = 16;
/// Every section starts at a multiple of this many bytes, so `Float64Array` views work.
pub const ALIGN: usize = 8;
/// The number of provenance classes (03 §4); a class code is below it.
pub const PROVENANCE_CLASSES: usize = 8;

/// Directory flag: the column has a validity bitmap.
pub const FLAG_VALIDITY: u16 = 1;

/// Byte offsets of the header fields and directory entry fields.
pub mod offset {
    /// Header: the magic bytes.
    pub const MAGIC: usize = 0;
    /// Header: the header version (u16).
    pub const VERSION: usize = 4;
    /// Header: the view kind (u16).
    pub const KIND: usize = 6;
    /// Header: the column count (u32).
    pub const COLUMNS: usize = 8;
    /// Header: reserved (u32).
    pub const RESERVED: usize = 12;
    /// Header: the row count (u64).
    pub const ROWS: usize = 16;
    /// Header: the data generation (u64).
    pub const GENERATION: usize = 24;
    /// Directory entry: the data type (u16).
    pub const COLUMN_TYPE: usize = 0;
    /// Directory entry: the flags (u16).
    pub const COLUMN_FLAGS: usize = 2;
    /// Directory entry: the name's length in bytes (u32).
    pub const COLUMN_NAME_LEN: usize = 4;
    /// Directory entry: the data offset (u64).
    pub const COLUMN_DATA: usize = 8;
}

/// What a view holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
#[non_exhaustive]
pub enum ViewKind {
    /// An M4-downsampled tile of a series (B04).
    M4Tile = 1,
    /// An LTTB-downsampled tile of a series (B04).
    LttbTile = 2,
    /// A page of table rows (B04).
    TablePage = 3,
}

impl ViewKind {
    /// Every kind, in code order.
    pub const ALL: [Self; 3] = [Self::M4Tile, Self::LttbTile, Self::TablePage];

    /// The kind with code `code`.
    #[must_use]
    pub fn from_code(code: u16) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| *kind as u16 == code)
    }

    /// The name used in TypeScript, such as `m4Tile`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::M4Tile => "m4Tile",
            Self::LttbTile => "lttbTile",
            Self::TablePage => "tablePage",
        }
    }
}

/// A column's element type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
#[non_exhaustive]
pub enum DataType {
    /// 64-bit floats.
    F64 = 1,
    /// 64-bit signed integers.
    I64 = 2,
    /// UTF-8 strings.
    Utf8 = 3,
    /// Booleans, one byte each.
    Bool = 4,
    /// Provenance classes, one byte each: the 03 §4 codes (`sonarithm_data::provenance::CLASS_CODES`).
    Provenance = 5,
    /// Confidences, 32-bit floats in `[0, 1]`; null (not applicable) through the validity bitmap.
    Confidence = 6,
}

impl DataType {
    /// Every type, in code order.
    pub const ALL: [Self; 6] = [
        Self::F64,
        Self::I64,
        Self::Utf8,
        Self::Bool,
        Self::Provenance,
        Self::Confidence,
    ];

    /// The type with code `code`.
    #[must_use]
    pub fn from_code(code: u16) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| *kind as u16 == code)
    }

    /// The name used in TypeScript, such as `f64`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::F64 => "f64",
            Self::I64 => "i64",
            Self::Utf8 => "utf8",
            Self::Bool => "bool",
            Self::Provenance => "provenance",
            Self::Confidence => "confidence",
        }
    }
}

/// A column's values.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Values {
    /// 64-bit floats.
    F64(Vec<f64>),
    /// 64-bit signed integers.
    I64(Vec<i64>),
    /// Strings.
    Utf8(Vec<String>),
    /// Booleans.
    Bool(Vec<bool>),
    /// Provenance class codes (03 §4).
    Provenance(Vec<u8>),
    /// Confidences.
    Confidence(Vec<f32>),
}

impl Values {
    fn len(&self) -> usize {
        match self {
            Self::F64(v) => v.len(),
            Self::I64(v) => v.len(),
            Self::Utf8(v) => v.len(),
            Self::Bool(v) => v.len(),
            Self::Provenance(v) => v.len(),
            Self::Confidence(v) => v.len(),
        }
    }

    fn data_type(&self) -> DataType {
        match self {
            Self::F64(_) => DataType::F64,
            Self::I64(_) => DataType::I64,
            Self::Utf8(_) => DataType::Utf8,
            Self::Bool(_) => DataType::Bool,
            Self::Provenance(_) => DataType::Provenance,
            Self::Confidence(_) => DataType::Confidence,
        }
    }
}

/// One column of a view.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Column {
    /// The column's name.
    pub name: String,
    /// The values.
    pub values: Values,
    /// Which rows have a value; `None` when all do.
    pub validity: Option<Vec<bool>>,
}

impl Column {
    /// A column where every row has a value.
    #[must_use]
    pub fn new(name: impl Into<String>, values: Values) -> Self {
        Self {
            name: name.into(),
            values,
            validity: None,
        }
    }

    /// The same column with a validity mask.
    #[must_use]
    pub fn with_validity(mut self, validity: Vec<bool>) -> Self {
        self.validity = Some(validity);
        self
    }
}

/// A decoded or to-be-encoded view.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct View {
    /// What it holds.
    pub kind: ViewKind,
    /// The data generation it was built from.
    pub generation: u64,
    /// The row count.
    pub rows: u64,
    /// The columns.
    pub columns: Vec<Column>,
}

/// Why a view cannot be encoded or decoded.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum BinaryError {
    /// Columns differ in length, or a validity mask differs from its column.
    #[error("column {column} has {found} rows, not {expected}")]
    Rows {
        /// The column.
        column: String,
        /// The view's row count.
        expected: u64,
        /// The column's.
        found: u64,
    },
    /// A UTF-8 column holds more than 4 GiB of text.
    #[error("column {0} holds more text than a page may (4 GiB)")]
    TooLarge(String),
    /// The buffer is not a view this version reads.
    #[error("not a binary view this version reads: {0}")]
    Malformed(&'static str),
}

const fn pad(len: usize) -> usize {
    len.next_multiple_of(ALIGN)
}

fn to_u64(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

impl View {
    /// An empty view of `kind` with `rows` rows.
    #[must_use]
    pub fn new(kind: ViewKind, generation: u64, rows: u64) -> Self {
        Self {
            kind,
            generation,
            rows,
            columns: Vec::new(),
        }
    }

    /// The same view with one more column.
    ///
    /// # Errors
    ///
    /// [`BinaryError::Rows`] if the column's length (or its mask's) is not the row count.
    pub fn with_column(mut self, column: Column) -> Result<Self, BinaryError> {
        let found = to_u64(column.values.len());
        let mask = column.validity.as_ref().map_or(found, |v| to_u64(v.len()));
        if found != self.rows || mask != self.rows {
            return Err(BinaryError::Rows {
                column: column.name,
                expected: self.rows,
                found: if found == self.rows { mask } else { found },
            });
        }
        self.columns.push(column);
        Ok(self)
    }

    /// The view as bytes.
    ///
    /// # Errors
    ///
    /// [`BinaryError::TooLarge`] for a UTF-8 column over 4 GiB.
    pub fn encode(&self) -> Result<Vec<u8>, BinaryError> {
        let names: usize = self.columns.iter().map(|c| c.name.len()).sum();
        let mut out = vec![0_u8; HEADER_BYTES + COLUMN_ENTRY_BYTES * self.columns.len()];
        out[offset::MAGIC..offset::MAGIC + 4].copy_from_slice(&MAGIC);
        put_u16(&mut out, offset::VERSION, HEADER_VERSION);
        put_u16(&mut out, offset::KIND, self.kind as u16);
        put_u32(
            &mut out,
            offset::COLUMNS,
            u32::try_from(self.columns.len()).unwrap_or(u32::MAX),
        );
        put_u64(&mut out, offset::ROWS, self.rows);
        put_u64(&mut out, offset::GENERATION, self.generation);
        for column in &self.columns {
            out.extend_from_slice(column.name.as_bytes());
        }
        out.resize(pad(out.len()), 0);
        debug_assert!(out.len() >= HEADER_BYTES + names);
        for (index, column) in self.columns.iter().enumerate() {
            let entry = HEADER_BYTES + COLUMN_ENTRY_BYTES * index;
            let mut flags = 0;
            if let Some(validity) = &column.validity {
                flags |= FLAG_VALIDITY;
                let mut bits = vec![0_u8; validity.len().div_ceil(8)];
                for (row, valid) in validity.iter().enumerate() {
                    if *valid {
                        bits[row / 8] |= 1 << (row % 8);
                    }
                }
                out.extend_from_slice(&bits);
                out.resize(pad(out.len()), 0);
            }
            let data_at = out.len();
            write_values(&mut out, column)?;
            out.resize(pad(out.len()), 0);
            put_u16(
                &mut out,
                entry + offset::COLUMN_TYPE,
                column.values.data_type() as u16,
            );
            put_u16(&mut out, entry + offset::COLUMN_FLAGS, flags);
            put_u32(
                &mut out,
                entry + offset::COLUMN_NAME_LEN,
                u32::try_from(column.name.len()).unwrap_or(u32::MAX),
            );
            put_u64(&mut out, entry + offset::COLUMN_DATA, to_u64(data_at));
        }
        Ok(out)
    }

    /// Reads a view.
    ///
    /// # Errors
    ///
    /// [`BinaryError::Malformed`] if the bytes are not a view this version reads.
    pub fn decode(bytes: &[u8]) -> Result<Self, BinaryError> {
        let bad = BinaryError::Malformed;
        if bytes.get(..4) != Some(&MAGIC[..]) {
            return Err(bad("wrong magic bytes"));
        }
        if get_u16(bytes, offset::VERSION)? != HEADER_VERSION {
            return Err(bad("unknown header version"));
        }
        let kind =
            ViewKind::from_code(get_u16(bytes, offset::KIND)?).ok_or(bad("unknown view kind"))?;
        let count = usize::try_from(get_u32(bytes, offset::COLUMNS)?)
            .map_err(|_| bad("too many columns"))?;
        let rows = get_u64(bytes, offset::ROWS)?;
        let generation = get_u64(bytes, offset::GENERATION)?;
        let n = usize::try_from(rows).map_err(|_| bad("too many rows"))?;
        let mut name_at = HEADER_BYTES
            .checked_add(
                count
                    .checked_mul(COLUMN_ENTRY_BYTES)
                    .ok_or(bad("too many columns"))?,
            )
            .ok_or(bad("too many columns"))?;
        let mut view = Self::new(kind, generation, rows);
        for index in 0..count {
            let entry = HEADER_BYTES + COLUMN_ENTRY_BYTES * index;
            let data_type = DataType::from_code(get_u16(bytes, entry + offset::COLUMN_TYPE)?)
                .ok_or(bad("unknown data type"))?;
            let flags = get_u16(bytes, entry + offset::COLUMN_FLAGS)?;
            let name_len = usize::try_from(get_u32(bytes, entry + offset::COLUMN_NAME_LEN)?)
                .map_err(|_| bad("name too long"))?;
            let data_at = usize::try_from(get_u64(bytes, entry + offset::COLUMN_DATA)?)
                .map_err(|_| bad("data offset too large"))?;
            let name = slice(bytes, name_at, name_len)?;
            name_at += name_len;
            let name = std::str::from_utf8(name)
                .map_err(|_| bad("a column name is not UTF-8"))?
                .to_owned();
            let validity = if flags & FLAG_VALIDITY == 0 {
                None
            } else {
                let len = n.div_ceil(8);
                let at = data_at
                    .checked_sub(pad(len))
                    .ok_or(bad("validity out of range"))?;
                let bits = slice(bytes, at, len)?;
                Some(
                    (0..n)
                        .map(|row| bits[row / 8] & (1 << (row % 8)) != 0)
                        .collect(),
                )
            };
            let values = read_values(bytes, data_type, data_at, n)?;
            view.columns.push(Column {
                name,
                values,
                validity,
            });
        }
        Ok(view)
    }
}

fn write_values(out: &mut Vec<u8>, column: &Column) -> Result<(), BinaryError> {
    match &column.values {
        Values::F64(values) => values
            .iter()
            .for_each(|v| out.extend_from_slice(&v.to_le_bytes())),
        Values::I64(values) => values
            .iter()
            .for_each(|v| out.extend_from_slice(&v.to_le_bytes())),
        Values::Bool(values) => values.iter().for_each(|v| out.push(u8::from(*v))),
        Values::Provenance(values) => out.extend_from_slice(values),
        Values::Confidence(values) => values
            .iter()
            .for_each(|v| out.extend_from_slice(&v.to_le_bytes())),
        Values::Utf8(values) => {
            let too_large = || BinaryError::TooLarge(column.name.clone());
            let mut end = 0_u32;
            out.extend_from_slice(&end.to_le_bytes());
            for value in values {
                let len = u32::try_from(value.len()).map_err(|_| too_large())?;
                end = end.checked_add(len).ok_or_else(too_large)?;
                out.extend_from_slice(&end.to_le_bytes());
            }
            out.resize(pad(out.len()), 0);
            for value in values {
                out.extend_from_slice(value.as_bytes());
            }
        }
    }
    Ok(())
}

fn read_values(
    bytes: &[u8],
    data_type: DataType,
    at: usize,
    n: usize,
) -> Result<Values, BinaryError> {
    let bad = BinaryError::Malformed;
    let words = |width: usize| -> Result<&[u8], BinaryError> {
        slice(
            bytes,
            at,
            n.checked_mul(width).ok_or(bad("column too large"))?,
        )
    };
    Ok(match data_type {
        DataType::F64 => Values::F64(
            words(8)?
                .as_chunks::<8>()
                .0
                .iter()
                .map(|c| f64::from_le_bytes(*c))
                .collect(),
        ),
        DataType::I64 => Values::I64(
            words(8)?
                .as_chunks::<8>()
                .0
                .iter()
                .map(|c| i64::from_le_bytes(*c))
                .collect(),
        ),
        DataType::Bool => Values::Bool(words(1)?.iter().map(|b| *b != 0).collect()),
        DataType::Provenance => {
            let codes = words(1)?;
            if codes
                .iter()
                .any(|code| usize::from(*code) >= PROVENANCE_CLASSES)
            {
                return Err(bad("unknown provenance class"));
            }
            Values::Provenance(codes.to_vec())
        }
        DataType::Confidence => Values::Confidence(
            words(4)?
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| f32::from_le_bytes(*c))
                .collect(),
        ),
        DataType::Utf8 => {
            let offsets_len = n.checked_add(1).ok_or(bad("column too large"))? * 4;
            let offsets: Vec<usize> = slice(bytes, at, offsets_len)?
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| u32::from_le_bytes(*c) as usize)
                .collect();
            let text_at = at + pad(offsets_len);
            let mut values = Vec::with_capacity(n);
            for pair in offsets.windows(2) {
                let (start, end) = (pair[0], pair[1]);
                let len = end.checked_sub(start).ok_or(bad("offsets go backwards"))?;
                let text = slice(bytes, text_at + start, len)?;
                values.push(
                    std::str::from_utf8(text)
                        .map_err(|_| bad("a value is not UTF-8"))?
                        .to_owned(),
                );
            }
            Values::Utf8(values)
        }
    })
}

fn slice(bytes: &[u8], at: usize, len: usize) -> Result<&[u8], BinaryError> {
    at.checked_add(len)
        .and_then(|end| bytes.get(at..end))
        .ok_or(BinaryError::Malformed("the buffer ends early"))
}

fn get_u16(bytes: &[u8], at: usize) -> Result<u16, BinaryError> {
    Ok(u16::from_le_bytes(
        slice(bytes, at, 2)?.try_into().unwrap_or_default(),
    ))
}

fn get_u32(bytes: &[u8], at: usize) -> Result<u32, BinaryError> {
    Ok(u32::from_le_bytes(
        slice(bytes, at, 4)?.try_into().unwrap_or_default(),
    ))
}

fn get_u64(bytes: &[u8], at: usize) -> Result<u64, BinaryError> {
    Ok(u64::from_le_bytes(
        slice(bytes, at, 8)?.try_into().unwrap_or_default(),
    ))
}

fn put_u16(out: &mut [u8], at: usize, value: u16) {
    out[at..at + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut [u8], at: usize, value: u32) {
    out[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut [u8], at: usize, value: u64) {
    out[at..at + 8].copy_from_slice(&value.to_le_bytes());
}

/// A small view with every data type, a validity mask and multi-byte text: the shared
/// fixture of the Rust and TypeScript round-trip tests.
#[doc(hidden)]
#[must_use]
pub fn sample() -> View {
    let mut view = View::new(ViewKind::TablePage, 7, 3);
    view.columns = vec![
        Column::new("x", Values::F64(vec![1.5, -2.0, 3.25])),
        Column::new("n", Values::I64(vec![1, 2, i64::MIN])).with_validity(vec![true, false, true]),
        Column::new(
            "label",
            Values::Utf8(vec!["a".into(), String::new(), "€uro".into()]),
        ),
        Column::new("flag", Values::Bool(vec![true, false, true])),
        Column::new("__snr_prov", Values::Provenance(vec![0, 2, 6])),
        Column::new("__snr_conf", Values::Confidence(vec![0.0, 0.5, 0.0]))
            .with_validity(vec![false, true, true]),
    ];
    view
}

impl From<BinaryError> for crate::ipc::IpcError {
    /// `SNR-APP-050` (ADR-0017 §6).
    fn from(err: BinaryError) -> Self {
        Self::new(&crate::codes::BINARY_VIEW, err.to_string())
    }
}

impl fmt::Display for ViewKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn header_fields_sit_at_their_documented_offsets() {
        let bytes = sample().encode().unwrap();
        assert_eq!(&bytes[..4], b"SNRV");
        assert_eq!(get_u16(&bytes, offset::VERSION).unwrap(), 1);
        assert_eq!(get_u16(&bytes, offset::KIND).unwrap(), 3);
        assert_eq!(get_u32(&bytes, offset::COLUMNS).unwrap(), 6);
        assert_eq!(get_u32(&bytes, offset::RESERVED).unwrap(), 0);
        assert_eq!(get_u64(&bytes, offset::ROWS).unwrap(), 3);
        assert_eq!(get_u64(&bytes, offset::GENERATION).unwrap(), 7);
        assert_eq!(bytes.len() % ALIGN, 0);
        for index in 0..6 {
            let entry = HEADER_BYTES + COLUMN_ENTRY_BYTES * index;
            let data = get_u64(&bytes, entry + offset::COLUMN_DATA).unwrap();
            assert_eq!(data % 8, 0, "column {index} data is 8-byte aligned");
        }
    }

    #[test]
    fn a_view_round_trips() {
        let view = sample();
        assert_eq!(View::decode(&view.encode().unwrap()).unwrap(), view);
    }

    #[test]
    fn ragged_columns_and_bad_buffers_are_refused() {
        let ragged =
            View::new(ViewKind::M4Tile, 0, 2).with_column(Column::new("x", Values::F64(vec![1.0])));
        assert!(matches!(ragged, Err(BinaryError::Rows { .. })));
        let bytes = sample().encode().unwrap();
        assert!(View::decode(&bytes[..40]).is_err());
        assert!(View::decode(b"JSON").is_err());
        // Strict: an unknown data type, version or class is refused, never guessed.
        let mut unknown_type = bytes.clone();
        unknown_type[HEADER_BYTES] = 99;
        assert!(View::decode(&unknown_type).is_err());
        let mut newer = bytes.clone();
        newer[offset::VERSION] = 2;
        assert!(View::decode(&newer).is_err());
        let error: crate::ipc::IpcError = View::decode(&newer).unwrap_err().into();
        assert_eq!(error.code, "SNR-APP-050");
        let bad_class = View::new(ViewKind::TablePage, 0, 1)
            .with_column(Column::new("p", Values::Provenance(vec![8])))
            .unwrap()
            .encode()
            .unwrap();
        assert!(View::decode(&bad_class).is_err());
    }

    proptest! {
        /// Any view decodes to itself, and decoding never panics on truncated input.
        #[test]
        fn views_round_trip(
            xs in proptest::collection::vec(any::<f64>().prop_filter("not NaN", |x| !x.is_nan()), 0..40),
            seed in any::<u64>(),
        ) {
            let n = xs.len();
            let ints: Vec<i64> = (0..n).map(|i| i64::try_from(i).unwrap_or(0).wrapping_mul(i64::from_ne_bytes(seed.to_ne_bytes()))).collect();
            let texts: Vec<String> = (0..n).map(|i| "é".repeat(i % 5)).collect();
            let mask: Vec<bool> = (0..n).map(|i| (seed >> (i % 64)) & 1 == 1).collect();
            let view = View::new(ViewKind::LttbTile, seed, n as u64)
                .with_column(Column::new("xs", Values::F64(xs)).with_validity(mask)).unwrap()
                .with_column(Column::new("ints", Values::I64(ints))).unwrap()
                .with_column(Column::new("texts", Values::Utf8(texts))).unwrap();
            let bytes = view.encode().unwrap();
            prop_assert_eq!(View::decode(&bytes).unwrap(), view);
            for cut in [0, 7, 31, bytes.len() / 2] {
                let _ = View::decode(&bytes[..cut.min(bytes.len())]);
            }
        }
    }
}

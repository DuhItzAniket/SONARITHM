// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A bounded walk over the Thrift structures of a Parquet file, run before parquet decodes
//! them (#146, F-071).
//!
//! parquet 60 skips Thrift fields it does not know without checking their declared sizes
//! against the input. Elements that read nothing (booleans in a collection) or whose reads
//! stop silently at the end of input (skips on the page-header reader) let a collection
//! that declares hundreds of millions of elements keep it busy for seconds: in a fuzzed
//! 1.8 KB file, a page header holding a list of 268 million UUIDs took about 12 s. This
//! walk reads the compact protocol itself, counts a boolean in a collection as the byte it
//! is on the wire, and rejects a collection that declares more elements than the bytes
//! left could hold, so its work stays linear in the input size. Page headers are read
//! only inside their column chunk, the byte range parquet reads page by page.
//!
//! parquet reads a page header's fields by their IDs, whatever wire type the bytes give:
//! a field 6 (`index_page_header`, a struct) sent as a boolean made it read the next bytes
//! as a struct, find a huge collection there and spend seconds on a 2 KB file, while this
//! walk had skipped the same bytes as one boolean (F-169). So the fields parquet.thrift
//! gives a page header and the structs inside it must carry their declared wire type here;
//! a mismatch is malformed.

use super::to_u64;

/// Thrift compact protocol type codes (thrift-compact-protocol.md), plus parquet's UUID.
const STOP: u8 = 0;
const BOOL_TRUE: u8 = 1;
const BOOL_FALSE: u8 = 2;
const BYTE: u8 = 3;
const I16: u8 = 4;
const I32: u8 = 5;
const I64: u8 = 6;
const DOUBLE: u8 = 7;
const BINARY: u8 = 8;
const LIST: u8 = 9;
const SET: u8 = 10;
const MAP: u8 = 11;
const STRUCT: u8 = 12;
const UUID: u8 = 13;

/// The nesting limit parquet applies when it skips fields.
const MAX_DEPTH: u8 = 64;

/// `PageHeader.compressed_page_size` in parquet.thrift.
const COMPRESSED_PAGE_SIZE: i16 = 3;

/// A field of a parquet.thrift struct whose wire type the walk checks: its ID, its type
/// code, and for a struct the fields of that struct.
struct Field {
    id: i16,
    kind: u8,
    fields: &'static [Field],
}

const fn field(id: i16, kind: u8) -> Field {
    Field {
        id,
        kind,
        fields: &[],
    }
}

const fn struct_field(id: i16, fields: &'static [Field]) -> Field {
    Field {
        id,
        kind: STRUCT,
        fields,
    }
}

/// `Statistics`.
const STATISTICS: &[Field] = &[
    field(1, BINARY),
    field(2, BINARY),
    field(3, I64),
    field(4, I64),
    field(5, BINARY),
    field(6, BINARY),
    field(7, BOOL_TRUE),
    field(8, BOOL_TRUE),
];

/// `DataPageHeader`.
const DATA_PAGE_HEADER: &[Field] = &[
    field(1, I32),
    field(2, I32),
    field(3, I32),
    field(4, I32),
    struct_field(5, STATISTICS),
];

/// `DictionaryPageHeader`.
const DICTIONARY_PAGE_HEADER: &[Field] = &[field(1, I32), field(2, I32), field(3, BOOL_TRUE)];

/// `DataPageHeaderV2`.
const DATA_PAGE_HEADER_V2: &[Field] = &[
    field(1, I32),
    field(2, I32),
    field(3, I32),
    field(4, I32),
    field(5, I32),
    field(6, I32),
    field(7, BOOL_TRUE),
    struct_field(8, STATISTICS),
];

/// `PageHeader`; `IndexPageHeader` (field 6) has no fields.
const PAGE_HEADER: &[Field] = &[
    field(1, I32),
    field(2, I32),
    field(COMPRESSED_PAGE_SIZE, I32),
    field(4, I32),
    struct_field(5, DATA_PAGE_HEADER),
    struct_field(6, &[]),
    struct_field(7, DICTIONARY_PAGE_HEADER),
    struct_field(8, DATA_PAGE_HEADER_V2),
];

/// Whether wire type `kind` is the declared type `expected` (either boolean code for a
/// boolean).
fn same_type(kind: u8, expected: u8) -> bool {
    match expected {
        BOOL_TRUE | BOOL_FALSE => matches!(kind, BOOL_TRUE | BOOL_FALSE),
        _ => kind == expected,
    }
}

/// The magic bytes that end a Parquet file with a plaintext footer.
const PARQUET_MAGIC: &[u8] = b"PAR1";

/// Walks the footer (`FileMetaData`) of a plaintext Parquet file. Anything that is not a
/// plaintext Parquet file passes, so that parquet reports it in its own words.
pub(super) fn check_footer(file: &[u8]) -> Result<(), String> {
    let Some(end) = file.len().checked_sub(8) else {
        return Ok(());
    };
    if file.get(end + 4..) != Some(PARQUET_MAGIC) {
        return Ok(());
    }
    let Some(length) = file
        .get(end..end + 4)
        .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
        .and_then(|bytes| usize::try_from(u32::from_le_bytes(bytes)).ok())
    else {
        return Ok(());
    };
    let Some(footer) = end
        .checked_sub(length)
        .and_then(|start| file.get(start..end))
    else {
        return Ok(());
    };
    Input::new(footer)
        .structure(MAX_DEPTH, None)
        .map(drop)
        .map_err(|reason| format!("the footer is malformed: {reason}"))
}

/// Walks every page header of the column chunk at `start..start + length`, the byte range
/// parquet reads page by page until it is used up.
#[cfg(test)]
pub(super) fn check_column_chunk(file: &[u8], start: i64, length: i64) -> Result<(), String> {
    let (Ok(start), Ok(length)) = (usize::try_from(start), usize::try_from(length)) else {
        return Err("a column chunk has a negative offset or length".to_owned());
    };
    let chunk = start
        .checked_add(length)
        .and_then(|end| file.get(start..end))
        .ok_or_else(|| "a column chunk lies outside the file".to_owned())?;
    let mut input = Input::new(chunk);
    while input.remaining() > 0 {
        let size = input
            .typed(MAX_DEPTH, Some(COMPRESSED_PAGE_SIZE), PAGE_HEADER)
            .map_err(|reason| format!("a page header is malformed: {reason}"))?
            .ok_or_else(|| "a page header has no compressed size".to_owned())?;
        let size = u64::try_from(size).map_err(|_| "a page has a negative size".to_owned())?;
        input
            .skip(size)
            .map_err(|_| "a page runs past the end of its column chunk".to_owned())?;
    }
    Ok(())
}
/// One page header at the start of `window`: its length in bytes and the page's
/// compressed size. `Ok(None)` when the window ends inside the header, so the caller reads
/// a larger window.
pub(super) fn page_header(window: &[u8]) -> Result<Option<(usize, u64)>, String> {
    let mut input = Input::new(window);
    match input.typed(MAX_DEPTH, Some(COMPRESSED_PAGE_SIZE), PAGE_HEADER) {
        Ok(size) => {
            let size = size.ok_or_else(|| "a page header has no compressed size".to_owned())?;
            let size = u64::try_from(size).map_err(|_| "a page has a negative size".to_owned())?;
            Ok(Some((input.at, size)))
        }
        Err(reason) if reason == ends_early() => Ok(None),
        Err(reason) => Err(format!("a page header is malformed: {reason}")),
    }
}

/// Thrift compact protocol input with a cursor; every read is bounds-checked.
struct Input<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Input<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.at)
    }

    fn byte(&mut self) -> Result<u8, String> {
        let byte = *self.bytes.get(self.at).ok_or_else(ends_early)?;
        self.at += 1;
        Ok(byte)
    }

    fn skip(&mut self, count: u64) -> Result<(), String> {
        match usize::try_from(count) {
            Ok(count) if count <= self.remaining() => {
                self.at += count;
                Ok(())
            }
            _ => Err(ends_early()),
        }
    }

    /// An unsigned LEB128 varint of at most 10 bytes.
    fn varint(&mut self) -> Result<u64, String> {
        let mut value = 0_u64;
        for shift in (0..64).step_by(7) {
            let byte = self.byte()?;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err("a varint is longer than 10 bytes".to_owned())
    }

    /// A zig-zag encoded signed varint.
    fn zigzag(&mut self) -> Result<i64, String> {
        let raw = self.varint()?;
        let magnitude = i64::try_from(raw >> 1).unwrap_or(i64::MAX);
        Ok(if raw & 1 == 0 {
            magnitude
        } else {
            -magnitude - 1
        })
    }

    /// Walks a struct up to its stop byte. When `wanted` names an `i32` field, returns its
    /// last value.
    fn structure(&mut self, depth: u8, wanted: Option<i16>) -> Result<Option<i64>, String> {
        self.typed(depth, wanted, &[])
    }

    /// [`Input::structure`] for a struct whose `fields` must carry their declared wire
    /// types; fields it does not list are skipped by their wire type, as parquet skips
    /// fields it does not know.
    fn typed(
        &mut self,
        depth: u8,
        wanted: Option<i16>,
        fields: &'static [Field],
    ) -> Result<Option<i64>, String> {
        if depth == 0 {
            return Err(too_deep());
        }
        let mut id = 0_i16;
        let mut found = None;
        loop {
            let header = self.byte()?;
            if header == STOP {
                return Ok(found);
            }
            let kind = header & 0x0f;
            id = match header >> 4 {
                0 => i16::try_from(self.zigzag()?)
                    .map_err(|_| "a field id is out of range".to_owned())?,
                delta => id
                    .checked_add(i16::from(delta))
                    .ok_or_else(|| "a field id is out of range".to_owned())?,
            };
            let declared = fields.iter().find(|f| f.id == id);
            if let Some(declared) = declared
                && !same_type(kind, declared.kind)
            {
                return Err(format!(
                    "field {id} has wire type {kind}, not its declared type {}",
                    declared.kind
                ));
            }
            if Some(id) == wanted && kind == I32 {
                found = Some(self.zigzag()?);
            } else if let Some(declared) = declared.filter(|d| d.kind == STRUCT) {
                self.typed(depth - 1, None, declared.fields)?;
            } else {
                self.value(kind, depth - 1, false)?;
            }
        }
    }

    /// Skips one value. A boolean is a struct field's type code alone, but one byte as a
    /// collection element.
    fn value(&mut self, kind: u8, depth: u8, in_collection: bool) -> Result<(), String> {
        if depth == 0 {
            return Err(too_deep());
        }
        match kind {
            BOOL_TRUE | BOOL_FALSE if in_collection => self.skip(1),
            BOOL_TRUE | BOOL_FALSE => Ok(()),
            BYTE => self.skip(1),
            I16 | I32 | I64 => self.varint().map(drop),
            DOUBLE => self.skip(8),
            BINARY => {
                let length = self.varint()?;
                self.skip(length)
            }
            LIST | SET => {
                let header = self.byte()?;
                let element = header & 0x0f;
                let count = match header >> 4 {
                    15 => self.varint()?,
                    short => u64::from(short),
                };
                self.fits(count, 1)?;
                for _ in 0..count {
                    self.value(element, depth - 1, true)?;
                }
                Ok(())
            }
            MAP => {
                let count = self.varint()?;
                if count == 0 {
                    return Ok(());
                }
                let kinds = self.byte()?;
                self.fits(count, 2)?;
                for _ in 0..count {
                    self.value(kinds >> 4, depth - 1, true)?;
                    self.value(kinds & 0x0f, depth - 1, true)?;
                }
                Ok(())
            }
            STRUCT => self.structure(depth, None).map(drop),
            UUID => self.skip(16),
            other => Err(format!("unknown Thrift type {other}")),
        }
    }

    /// Checks that `count` elements of at least `bytes_each` bytes fit in the input left.
    fn fits(&self, count: u64, bytes_each: u64) -> Result<(), String> {
        let left = to_u64(self.remaining());
        if count.saturating_mul(bytes_each) > left {
            return Err(format!(
                "a collection declares {count} elements but only {left} bytes are left"
            ));
        }
        Ok(())
    }
}

fn ends_early() -> String {
    "the Thrift data ends early".to_owned()
}

fn too_deep() -> String {
    format!("Thrift nesting is deeper than {MAX_DEPTH} levels")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A struct holding one list field (id 1) with the given element type and count.
    fn struct_with_list(element: u8, count: u64, payload: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0x10 | LIST, 0xf0 | element];
        let mut rest = count;
        loop {
            let low = u8::try_from(rest & 0x7f).unwrap();
            rest >>= 7;
            if rest == 0 {
                bytes.push(low);
                break;
            }
            bytes.push(low | 0x80);
        }
        bytes.extend_from_slice(payload);
        bytes.push(STOP);
        bytes
    }

    #[test]
    fn a_list_of_uuids_larger_than_the_input_is_rejected() {
        // The fuzzed page header of #146 declared 268,434,391 UUIDs.
        let bytes = struct_with_list(UUID, 268_434_391, &[0; 32]);
        let err = Input::new(&bytes).structure(MAX_DEPTH, None).unwrap_err();
        assert!(err.contains("declares 268434391 elements"), "{err}");
    }

    #[test]
    fn booleans_in_a_list_take_a_byte_each() {
        let ok = struct_with_list(BOOL_TRUE, 3, &[1, 0, 1]);
        assert_eq!(Input::new(&ok).structure(MAX_DEPTH, None), Ok(None));
        let huge = struct_with_list(BOOL_FALSE, u64::from(u32::MAX), &[1]);
        assert!(Input::new(&huge).structure(MAX_DEPTH, None).is_err());
    }

    #[test]
    fn a_map_declaring_more_pairs_than_bytes_is_rejected() {
        // Field 1 is a map of 1 000 000 bool -> bool pairs with two bytes of payload.
        let bytes = [0x10 | MAP, 0xc0, 0x84, 0x3d, 0x11, 1, 1, STOP];
        let err = Input::new(&bytes).structure(MAX_DEPTH, None).unwrap_err();
        assert!(err.contains("1000000 elements"), "{err}");
    }

    #[test]
    fn nesting_deeper_than_parquet_allows_is_rejected() {
        // 70 nested structs, each the first field (id 1) of its parent.
        let mut bytes = vec![0x10 | STRUCT; 70];
        bytes.extend(std::iter::repeat_n(STOP, 71));
        let err = Input::new(&bytes).structure(MAX_DEPTH, None).unwrap_err();
        assert!(err.contains("deeper than 64"), "{err}");
    }

    #[test]
    fn the_wanted_field_is_read_and_others_are_skipped() {
        // Field 1: i32 5; field 2: binary "ab"; field 3: i32 -7 (zig-zag 13); field 5 (long
        // form id): double.
        let mut bytes = vec![0x10 | I32, 10, 0x10 | BINARY, 2, b'a', b'b', 0x10 | I32, 13];
        bytes.extend([DOUBLE, 10]);
        bytes.extend([0; 8]);
        bytes.push(STOP);
        assert_eq!(
            Input::new(&bytes).structure(MAX_DEPTH, Some(3)),
            Ok(Some(-7))
        );
        assert_eq!(
            Input::new(&bytes[..9]).structure(MAX_DEPTH, Some(3)),
            Err(ends_early())
        );
    }

    #[test]
    fn pages_must_tile_their_column_chunk() {
        // One page header: compressed_page_size (field 3) = 2, then two data bytes.
        let page = [0x30 | I32, 4, STOP, 0xaa, 0xbb];
        assert_eq!(check_column_chunk(&page, 0, 5), Ok(()));
        let err = check_column_chunk(&page, 0, 4).unwrap_err();
        assert!(err.contains("runs past"), "{err}");
        assert!(check_column_chunk(&page, 2, 9).is_err());
        assert!(check_column_chunk(&page, -1, 2).is_err());
        let no_size = [0x10 | I32, 4, STOP];
        assert!(check_column_chunk(&no_size, 0, 3).is_err());
    }

    #[test]
    fn page_header_fields_must_carry_their_declared_wire_types() {
        // F-169: field 6 (`index_page_header`, a struct) sent as a boolean. parquet would
        // read the following bytes as that struct; the walk refuses the mismatch.
        let page = [0x30 | I32, 4, 0x30 | BOOL_TRUE, STOP, 0xaa, 0xbb];
        let err = check_column_chunk(&page, 0, 6).unwrap_err();
        assert!(err.contains("field 6 has wire type 1"), "{err}");
        // A boolean nested in the statistics struct may carry either boolean code, and a
        // field the schema does not list (id 20, long form) is skipped by its wire type.
        let page = [
            0x10 | I32,
            0, // type: DATA_PAGE
            0x10 | I32,
            2, // uncompressed_page_size: 1
            0x10 | I32,
            4,             // compressed_page_size: 2
            0x20 | STRUCT, // data_page_header
            0x10 | I32,
            2,                 // num_values: 1
            0x40 | STRUCT,     // statistics
            0x70 | BOOL_FALSE, // is_max_value_exact: false
            STOP,
            STOP,
            I32,
            0x28,
            0, // field 20: i32 0
            STOP,
            0xaa,
            0xbb,
        ];
        assert_eq!(check_column_chunk(&page, 0, 19), Ok(()));
    }

    #[test]
    fn a_footer_with_a_huge_boolean_list_is_rejected() {
        // parquet skips booleans in a collection as zero bytes, so this footer would loop
        // about four billion times there.
        let footer = struct_with_list(BOOL_TRUE, u64::from(u32::MAX), &[]);
        let mut file = PARQUET_MAGIC.to_vec();
        file.extend(&footer);
        file.extend(u32::try_from(footer.len()).unwrap().to_le_bytes());
        file.extend(PARQUET_MAGIC);
        let err = check_footer(&file).unwrap_err();
        assert!(err.starts_with("the footer is malformed"), "{err}");
    }

    #[test]
    fn files_that_are_not_plaintext_parquet_are_left_to_parquet() {
        assert_eq!(check_footer(b""), Ok(()));
        assert_eq!(check_footer(b"PAR1\x00\x00\x00\x00PARE"), Ok(()));
        // A footer length past the start of the file.
        assert_eq!(check_footer(b"PAR1\xff\x00\x00\x00PAR1"), Ok(()));
        // An empty struct as the footer.
        assert_eq!(check_footer(b"PAR1\x00\x01\x00\x00\x00PAR1"), Ok(()));
    }
}

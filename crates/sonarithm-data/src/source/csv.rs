// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! CSV and TSV sources (10 §2): encoding, delimiter and header sniffing, then per-column
//! type inference ([`crate::infer::text`]).
//!
//! Every guess is reported: the dialect (SNR-DATA-010), a converted encoding (011), ragged
//! rows (012), renamed columns (013), replaced bytes (014) and skipped preamble lines
//! (015).

use std::borrow::Cow;
use std::sync::Arc;

use arrow_schema::Schema;
use encoding_rs::Encoding;
use serde::Deserialize;
use sonarithm_core::{Diagnostic, Diagnostics};

use super::columns::{RawColumn, detect_header, row_range, text_table, unique_names};
use super::{ReadOptions, to_u64};
use crate::bundle::Format;
use crate::codes;
use crate::error::DataError;
use crate::infer::text::TextOptions;
use crate::table::Table;

/// Delimiters tried when sniffing, in order of preference for ties.
const DELIMITERS: [u8; 4] = *b",;\t|";

/// Bytes of input used to sniff the dialect.
const SNIFF_BYTES: usize = 64 * 1024;

/// Records used to sniff the dialect and the header.
const SNIFF_RECORDS: usize = 100;

/// Leading lines that may be skipped as a preamble (a title above the header).
const MAX_PREAMBLE: usize = 10;

/// Input bytes given to the encoding detector.
const DETECT_BYTES: usize = 1 << 20;

/// Options for CSV and TSV sources, as written in a spec's `data.source.options`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct CsvOptions {
    /// The field delimiter, one ASCII character; sniffed when `None` (TSV: tab).
    pub delimiter: Option<char>,
    /// Whether the first row names the fields; detected when `None`.
    pub header: Option<bool>,
    /// The text encoding as a WHATWG label, such as `windows-1252`; detected when `None`.
    pub encoding: Option<String>,
    /// Read `,` as the decimal separator (10 §2: locale-aware decimals are opt-in).
    pub decimal_comma: bool,
    /// A `strftime`-style format for dates that are not ISO-8601, such as `%d.%m.%Y`.
    pub date_format: Option<String>,
}

impl CsvOptions {
    /// The default options: everything sniffed, `.` as the decimal point.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The same options with a declared delimiter.
    #[must_use]
    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = Some(delimiter);
        self
    }

    /// The same options with the header row declared instead of detected.
    #[must_use]
    pub fn with_header(mut self, header: bool) -> Self {
        self.header = Some(header);
        self
    }

    /// The same options with a declared encoding (a WHATWG label).
    #[must_use]
    pub fn with_encoding(mut self, label: impl Into<String>) -> Self {
        self.encoding = Some(label.into());
        self
    }

    /// The same options reading `,` as the decimal separator.
    #[must_use]
    pub fn with_decimal_comma(mut self) -> Self {
        self.decimal_comma = true;
        self
    }

    /// The same options with a `strftime`-style date format.
    #[must_use]
    pub fn with_date_format(mut self, format: impl Into<String>) -> Self {
        self.date_format = Some(format.into());
        self
    }
}

/// Reads CSV or TSV into a table and the diagnostics that describe what was assumed.
pub(super) fn read_csv(
    bytes: &[u8],
    name: &str,
    format: Format,
    options: &ReadOptions,
) -> Result<(Table, Diagnostics), DataError> {
    let csv = &options.csv;
    let mut diagnostics = Diagnostics::new();
    let (text, notes) = decode(bytes, csv.encoding.as_deref())?;
    diagnostics.extend(notes);
    let delimiter = delimiter(csv, format, &text)?;
    let text_options = TextOptions {
        decimal_comma: csv.decimal_comma,
        date_format: csv.date_format.clone(),
    };
    let Some(mut parsed) = parse(&text, delimiter, format, options)? else {
        let schema = Arc::new(Schema::empty());
        return Ok((
            Table::new(options.table, name, schema, Vec::new())?,
            diagnostics,
        ));
    };
    let first_record = std::mem::take(&mut parsed.first);
    let first: Vec<&str> = first_record.iter().collect();
    let header = csv
        .header
        .unwrap_or_else(|| detect_header(&first, &parsed.columns, &text_options));
    if !header {
        parsed.prepend(&first);
    }
    let header_names: Option<Vec<Option<&str>>> =
        header.then(|| first.iter().map(|name| Some(*name)).collect());
    let (names, renamed) = unique_names(header_names.as_deref(), first.len());
    let shape = Shape {
        format,
        delimiter,
        header,
        width: names.len(),
        rows: parsed.rows,
    };
    let table = text_table(
        name,
        &names,
        &parsed.columns,
        &text_options,
        options,
        &mut diagnostics,
    )?;
    diagnostics.extend(dialect_diagnostics(&shape, &parsed, renamed, options));
    Ok((table, diagnostics))
}

/// The delimiter: the option, tab for TSV, or a sniffed one.
fn delimiter(csv: &CsvOptions, format: Format, text: &str) -> Result<u8, DataError> {
    match (csv.delimiter, format) {
        (Some(c), _) => u8::try_from(c)
            .ok()
            .filter(|b| b.is_ascii() && !matches!(b, b'"' | b'\n' | b'\r'))
            .ok_or(DataError::InvalidOption {
                option: "delimiter",
                reason: "use one ASCII character other than a quote or a line break",
            }),
        (None, Format::Tsv) => Ok(b'\t'),
        (None, _) => Ok(sniff_delimiter(text)),
    }
}

/// Records split into columns.
struct Parsed {
    /// The first record after the preamble: the header or the first data row.
    first: csv::StringRecord,
    columns: Vec<RawColumn>,
    rows: u64,
    ragged: Ragged,
    preamble: usize,
}

impl Parsed {
    /// Puts the first record in front of the data (it was not a header).
    fn prepend(&mut self, first: &[&str]) {
        for (column, value) in self.columns.iter_mut().zip(first) {
            column.prepend(value);
        }
        self.rows += 1;
        if let Some(row) = &mut self.ragged.first {
            *row += 1;
        }
    }
}

/// Splits the text into columns, skipping a preamble and checking the limits as rows
/// arrive. `None` for input without records.
fn parse(
    text: &str,
    delimiter: u8,
    format: Format,
    options: &ReadOptions,
) -> Result<Option<Parsed>, DataError> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());
    let unreadable = |source: csv::Error| DataError::Unreadable {
        format,
        reason: source.to_string(),
    };
    let mut head: Vec<csv::StringRecord> = Vec::new();
    for record in reader.records().take(SNIFF_RECORDS) {
        head.push(record.map_err(unreadable)?);
    }
    let preamble = preamble_len(&head);
    let mut records = head.into_iter().skip(preamble);
    let Some(first) = records.next() else {
        return Ok(None);
    };
    let width = first.len();
    options.limits.check_shape(0, width)?;
    let mut parsed = Parsed {
        first,
        columns: (0..width).map(|_| RawColumn::default()).collect(),
        rows: 0,
        ragged: Ragged::default(),
        preamble,
    };
    let push = |record: &csv::StringRecord, parsed: &mut Parsed| {
        if record.len() != width {
            parsed.ragged.count += 1;
            parsed.ragged.first.get_or_insert(parsed.rows);
        }
        for (index, column) in parsed.columns.iter_mut().enumerate() {
            column.push(record.get(index).unwrap_or_default());
        }
        parsed.rows += 1;
        options.limits.check_shape(parsed.rows, width)
    };
    for record in records {
        push(&record, &mut parsed)?;
    }
    let mut record = csv::StringRecord::new();
    while reader.read_record(&mut record).map_err(unreadable)? {
        push(&record, &mut parsed)?;
    }
    Ok(Some(parsed))
}

/// What was read, for the dialect diagnostic.
struct Shape {
    format: Format,
    delimiter: u8,
    header: bool,
    width: usize,
    rows: u64,
}

/// The dialect, preamble, renamed-column and ragged-row diagnostics.
fn dialect_diagnostics(
    shape: &Shape,
    parsed: &Parsed,
    renamed: usize,
    options: &ReadOptions,
) -> Vec<Diagnostic> {
    let mut diagnostics = vec![Diagnostic::new(
        &codes::CSV_DIALECT,
        format!(
            "Read as {}: delimiter {}, {}, {} fields, {} rows.",
            if shape.format == Format::Tsv {
                "TSV"
            } else {
                "CSV"
            },
            describe_delimiter(shape.delimiter),
            if shape.header {
                "first row is the header"
            } else {
                "no header row"
            },
            shape.width,
            shape.rows,
        ),
    )];
    if parsed.preamble > 0 {
        diagnostics.push(Diagnostic::new(
            &codes::PREAMBLE_SKIPPED,
            format!(
                "Skipped {} line(s) above the header that did not fit the table. Remove them, \
                 or set `header` if this is wrong.",
                parsed.preamble
            ),
        ));
    }
    if renamed > 0 {
        diagnostics.push(Diagnostic::new(
            &codes::COLUMN_NAMES_CHANGED,
            format!(
                "{renamed} field name(s) were blank or repeated and were replaced \
                 (`column_N`, `name_2`)."
            ),
        ));
    }
    if let Some(first_row) = parsed.ragged.first {
        diagnostics.push(
            Diagnostic::new(
                &codes::RAGGED_ROWS,
                format!(
                    "{} row(s) do not have {} fields, first at row {}; short rows were padded \
                     with missing values and extra fields were dropped. Check the delimiter \
                     and quoting.",
                    parsed.ragged.count,
                    shape.width,
                    first_row + 1
                ),
            )
            .rows(row_range(options, first_row)),
        );
    }
    diagnostics
}

/// Rows whose field count differs from the header.
#[derive(Default)]
struct Ragged {
    count: u64,
    first: Option<u64>,
}

/// Decodes the input to UTF-8: a byte-order mark wins, then the declared label, then valid
/// UTF-8, then a statistical guess (chardetng).
fn decode<'a>(
    bytes: &'a [u8],
    label: Option<&str>,
) -> Result<(Cow<'a, str>, Vec<Diagnostic>), DataError> {
    let mut notes = Vec::new();
    let declared = match label {
        Some(label) => Some(Encoding::for_label(label.as_bytes()).ok_or(
            DataError::InvalidOption {
                option: "encoding",
                reason: "use a WHATWG encoding label such as utf-8 or windows-1252",
            },
        )?),
        None => None,
    };
    let (encoding, body) = match (Encoding::for_bom(bytes), declared) {
        (Some((encoding, bom)), _) => (encoding, bytes.get(bom..).unwrap_or_default()),
        (None, Some(encoding)) => (encoding, bytes),
        (None, None) => {
            if let Ok(text) = std::str::from_utf8(bytes) {
                return Ok((Cow::Borrowed(text), notes));
            }
            let mut detector =
                chardetng::EncodingDetector::new(chardetng::Iso2022JpDetection::Deny);
            detector.feed(
                bytes.get(..DETECT_BYTES).unwrap_or(bytes),
                bytes.len() <= DETECT_BYTES,
            );
            (detector.guess(None, chardetng::Utf8Detection::Allow), bytes)
        }
    };
    if encoding == encoding_rs::UTF_8
        && let Ok(text) = std::str::from_utf8(body)
    {
        return Ok((Cow::Borrowed(text), notes));
    }
    let (text, had_errors) = encoding.decode_without_bom_handling(body);
    if encoding != encoding_rs::UTF_8 {
        notes.push(Diagnostic::new(
            &codes::ENCODING_CONVERTED,
            format!(
                "The text was read as {} and converted to UTF-8. Set `encoding` if the letters look wrong.",
                encoding.name()
            ),
        ));
    }
    if had_errors {
        notes.push(Diagnostic::new(
            &codes::INVALID_BYTES,
            format!(
                "Some bytes are not valid {} and were replaced with U+FFFD. Check the file's encoding.",
                encoding.name()
            ),
        ));
    }
    Ok((text, notes))
}

/// Picks the delimiter that splits the sample into the most consistent number of fields
/// (more than one), preferring `,` then `;`, tab and `|` on ties.
fn sniff_delimiter(text: &str) -> u8 {
    let sample = match text.get(..SNIFF_BYTES) {
        Some(sample) => sample.rfind('\n').map_or(sample, |end| &sample[..end]),
        None => text,
    };
    let mut best = (b',', (false, 0_u64, 0_usize));
    for delimiter in DELIMITERS {
        let counts: Vec<usize> = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(false)
            .flexible(true)
            .from_reader(sample.as_bytes())
            .records()
            .take(SNIFF_RECORDS)
            .filter_map(Result::ok)
            .map(|record| record.len())
            .collect();
        let (mode, frequency) = mode(&counts);
        let consistency = to_u64(frequency) * 1000 / to_u64(counts.len().max(1));
        let score = (mode > 1, consistency, mode);
        if score > best.1 {
            best = (delimiter, score);
        }
    }
    best.0
}

/// The most common value and how often it occurs; the smallest value wins ties.
fn mode(counts: &[usize]) -> (usize, usize) {
    let mut sorted = counts.to_vec();
    sorted.sort_unstable();
    let mut best = (0, 0);
    for run in sorted.chunk_by(|a, b| a == b) {
        if run.len() > best.1 {
            best = (run[0], run.len());
        }
    }
    best
}

/// The number of leading records to skip: records before the first one whose field count is
/// the body's most common count, when that count is larger.
fn preamble_len(head: &[csv::StringRecord]) -> usize {
    let counts: Vec<usize> = head.iter().map(csv::StringRecord::len).collect();
    let (mode, _) = mode(&counts);
    if mode <= 1 {
        return 0;
    }
    let skip = counts.iter().take_while(|&&count| count < mode).count();
    if skip <= MAX_PREAMBLE && skip < counts.len() {
        skip
    } else {
        0
    }
}

fn describe_delimiter(delimiter: u8) -> String {
    match delimiter {
        b'\t' => "tab".to_owned(),
        b' ' => "space".to_owned(),
        other => format!("`{}`", char::from(other)),
    }
}

#[cfg(test)]
mod tests {
    use arrow_array::Array;
    use arrow_array::cast::AsArray;
    use arrow_array::types::{Float64Type, Int64Type};
    use arrow_schema::DataType;

    use super::*;
    use crate::source::read_bytes;

    fn read(text: &str) -> (Table, Vec<String>) {
        read_with(text.as_bytes(), CsvOptions::default())
    }

    fn read_with(bytes: &[u8], csv: CsvOptions) -> (Table, Vec<String>) {
        let options = ReadOptions {
            csv,
            ..ReadOptions::default()
        };
        let loaded = read_bytes(bytes, Format::Csv, "data/t.csv", &options).unwrap();
        let codes = loaded
            .diagnostics
            .iter()
            .map(|d| d.code.as_str().to_owned())
            .filter(|code| {
                !matches!(
                    code.as_str(),
                    "SNR-DATA-030" | "SNR-DATA-031" | "SNR-DATA-032"
                )
            })
            .collect();
        (loaded.bundle.primary().unwrap().clone(), codes)
    }

    fn types(table: &Table) -> Vec<DataType> {
        table
            .schema()
            .fields()
            .iter()
            .map(|f| f.data_type().clone())
            .collect()
    }

    #[test]
    fn a_plain_csv_gets_typed_columns() {
        let (table, codes) = read("t,v,name\n1,0.5,a\n2,1.5,b\n");
        assert_eq!(
            types(&table),
            [DataType::Int64, DataType::Float64, DataType::Utf8]
        );
        assert_eq!(table.num_rows(), 2);
        assert_eq!(codes, ["SNR-DATA-010"]);
        assert_eq!(table.name(), "t");
    }

    #[test]
    fn semicolon_delimiters_with_decimal_commas() {
        let csv = CsvOptions {
            decimal_comma: true,
            ..CsvOptions::default()
        };
        let (table, _) = read_with("a;b\n1,5;2\n2,5;3\n".as_bytes(), csv);
        let a = table.column("a").unwrap();
        assert_eq!(
            a.as_primitive::<Float64Type>().values().to_vec(),
            [1.5, 2.5]
        );
    }

    #[test]
    fn headerless_numeric_files_are_detected() {
        let (table, _) = read("1,2\n3,4\n");
        let names: Vec<_> = table
            .schema()
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect();
        assert_eq!(names, ["column_1", "column_2"]);
        assert_eq!(table.num_rows(), 2);
        let first = table.column("column_1").unwrap();
        assert_eq!(first.as_primitive::<Int64Type>().value(0), 1);
    }

    #[test]
    fn ragged_rows_preambles_and_renamed_columns_are_reported() {
        let (table, codes) = read("Monthly report\nx,,x\n1,2,3\n4,5\n6,7,8,9\n");
        let names: Vec<_> = table
            .schema()
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect();
        assert_eq!(names, ["x", "column_2", "x_2"]);
        assert_eq!(table.num_rows(), 3);
        assert!(table.column("x_2").unwrap().is_null(1));
        for code in [
            "SNR-DATA-010",
            "SNR-DATA-012",
            "SNR-DATA-013",
            "SNR-DATA-015",
        ] {
            assert!(codes.iter().any(|c| c == code), "{code} in {codes:?}");
        }
    }

    #[test]
    fn legacy_encodings_and_byte_order_marks() {
        // "café" in windows-1252.
        let bytes = b"name,n\ncaf\xe9,1\nthe,2\n";
        let (table, codes) = read_with(bytes, CsvOptions::default());
        assert_eq!(
            table.column("name").unwrap().as_string::<i32>().value(0),
            "café"
        );
        assert!(codes.iter().any(|c| c == "SNR-DATA-011"), "{codes:?}");

        let (table, codes) = read_with(b"\xef\xbb\xbfa,b\n1,2\n", CsvOptions::default());
        assert_eq!(table.schema().field(0).name(), "a");
        assert_eq!(codes, ["SNR-DATA-010"]);
    }

    #[test]
    fn units_come_from_values_and_names() {
        let (table, codes) = read("temp (°C),price\n21.5,$3.50\n22.0,$4.00\n");
        assert_eq!(
            table.field_meta("temp (°C)").unwrap().unit.unwrap().text,
            "°C"
        );
        assert_eq!(table.field_meta("price").unwrap().unit.unwrap().text, "$");
        assert_eq!(codes.iter().filter(|c| *c == "SNR-DATA-037").count(), 2);
    }

    #[test]
    fn options_are_checked() {
        let options = ReadOptions {
            csv: CsvOptions {
                delimiter: Some('"'),
                ..CsvOptions::default()
            },
            ..ReadOptions::default()
        };
        let err = read_bytes(b"a\n", Format::Csv, "t.csv", &options).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-006");
        let options: CsvOptions =
            serde_json::from_str(r#"{"delimiter": ";", "decimalComma": true}"#).unwrap();
        assert_eq!(options.delimiter, Some(';'));
        assert!(serde_json::from_str::<CsvOptions>(r#"{"delim": ";"}"#).is_err());
    }

    #[test]
    fn empty_input_gives_an_empty_table() {
        let (table, _) = read("");
        assert_eq!((table.num_rows(), table.num_fields()), (0, 0));
    }

    #[test]
    fn tsv_uses_tabs() {
        let options = ReadOptions::default();
        let loaded = read_bytes(b"a\tb\n1\tx,y\n", Format::Tsv, "t.tsv", &options).unwrap();
        let table = loaded.bundle.primary().unwrap();
        assert_eq!(
            table.column("b").unwrap().as_string::<i32>().value(0),
            "x,y"
        );
    }
}

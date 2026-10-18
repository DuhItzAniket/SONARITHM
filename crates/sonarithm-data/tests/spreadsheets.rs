// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A04-T04 acceptance: the test workbook suite. Workbooks are built here as minimal XLSX
//! and ODS packages (no binary fixtures to keep in the repository), covering shared and
//! inline strings, dates through number formats, cached formula values, error cells,
//! booleans, several sheets, sheet and range selection, and headerless sheets.

#![cfg(feature = "xlsx")]
#![allow(
    clippy::unwrap_used,
    clippy::format_push_string,
    clippy::similar_names,
    clippy::format_collect,
    reason = "fixture builders in a test crate: a failed write is a test failure, and the XML is assembled from short literal pieces"
)]

use std::io::{Cursor, Write as _};

use arrow_array::Array;
use arrow_array::cast::AsArray;
use arrow_array::types::{Date32Type, Float64Type, Int64Type, TimestampMicrosecondType};
use arrow_schema::DataType;
use sonarithm_data::{DataError, Format, ReadOptions, SheetOptions, SheetRef, Table, read_bytes};

/// One spreadsheet cell for the XLSX builder.
enum Cell {
    Num(f64),
    Str(&'static str),
    Shared(usize),
    Bool(bool),
    /// An Excel serial date shown with the built-in date format 14.
    Date(f64),
    /// A formula and its cached value.
    Formula(&'static str, f64),
    Error(&'static str),
    Empty,
}

fn zip(parts: &[(&str, String)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let stored =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, content) in parts {
        writer.start_file(*name, stored).unwrap();
        writer.write_all(content.as_bytes()).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn column_name(index: usize) -> String {
    let mut name = String::new();
    let mut n = index + 1;
    while n > 0 {
        let rem = (n - 1) % 26;
        name.insert(0, char::from(b'A' + u8::try_from(rem).unwrap()));
        n = (n - 1) / 26;
    }
    name
}

fn sheet_xml(rows: &[Vec<Cell>]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>"#,
    );
    for (r, row) in rows.iter().enumerate() {
        xml.push_str(&format!(r#"<row r="{}">"#, r + 1));
        for (c, cell) in row.iter().enumerate() {
            let at = format!("{}{}", column_name(c), r + 1);
            xml.push_str(&match cell {
                Cell::Num(v) => format!(r#"<c r="{at}"><v>{v}</v></c>"#),
                Cell::Str(s) => format!(r#"<c r="{at}" t="inlineStr"><is><t>{s}</t></is></c>"#),
                Cell::Shared(i) => format!(r#"<c r="{at}" t="s"><v>{i}</v></c>"#),
                Cell::Bool(b) => format!(r#"<c r="{at}" t="b"><v>{}</v></c>"#, u8::from(*b)),
                Cell::Date(v) => format!(r#"<c r="{at}" s="1"><v>{v}</v></c>"#),
                Cell::Formula(f, v) => format!(r#"<c r="{at}"><f>{f}</f><v>{v}</v></c>"#),
                Cell::Error(e) => format!(r#"<c r="{at}" t="e"><v>{e}</v></c>"#),
                Cell::Empty => String::new(),
            });
        }
        xml.push_str("</row>");
    }
    xml.push_str("</sheetData></worksheet>");
    xml
}

/// A minimal XLSX package with the given sheets and shared strings.
fn xlsx(sheets: &[(&str, Vec<Vec<Cell>>)], shared: &[&str]) -> Vec<u8> {
    let main = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
    let rel = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let mut types = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/><Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>"#,
    );
    let mut workbook = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><workbook xmlns="{main}" xmlns:r="{rel}"><sheets>"#
    );
    let mut rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );
    let mut parts: Vec<(String, String)> = Vec::new();
    for (i, (name, rows)) in sheets.iter().enumerate() {
        let n = i + 1;
        types.push_str(&format!(
            r#"<Override PartName="/xl/worksheets/sheet{n}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#
        ));
        workbook.push_str(&format!(
            r#"<sheet name="{name}" sheetId="{n}" r:id="rId{n}"/>"#
        ));
        rels.push_str(&format!(
            r#"<Relationship Id="rId{n}" Type="{rel}/worksheet" Target="worksheets/sheet{n}.xml"/>"#
        ));
        parts.push((format!("xl/worksheets/sheet{n}.xml"), sheet_xml(rows)));
    }
    types.push_str("</Types>");
    workbook.push_str("</sheets></workbook>");
    rels.push_str(&format!(
        r#"<Relationship Id="rIdS" Type="{rel}/styles" Target="styles.xml"/><Relationship Id="rIdT" Type="{rel}/sharedStrings" Target="sharedStrings.xml"/></Relationships>"#
    ));
    let strings: String = shared
        .iter()
        .map(|s| format!("<si><t>{s}</t></si>"))
        .collect();
    let mut all = vec![
        ("[Content_Types].xml".to_owned(), types),
        (
            "_rels/.rels".to_owned(),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="{rel}/officeDocument" Target="xl/workbook.xml"/></Relationships>"#
            ),
        ),
        ("xl/workbook.xml".to_owned(), workbook),
        ("xl/_rels/workbook.xml.rels".to_owned(), rels),
        (
            "xl/styles.xml".to_owned(),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?><styleSheet xmlns="{main}"><cellXfs count="2"><xf numFmtId="0"/><xf numFmtId="14" applyNumberFormat="1"/></cellXfs></styleSheet>"#
            ),
        ),
        (
            "xl/sharedStrings.xml".to_owned(),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?><sst xmlns="{main}" count="{0}" uniqueCount="{0}">{strings}</sst>"#,
                shared.len()
            ),
        ),
    ];
    all.extend(parts);
    let refs: Vec<(&str, String)> = all.iter().map(|(n, c)| (n.as_str(), c.clone())).collect();
    zip(&refs)
}

/// A minimal ODS package with one sheet; cells are (value-type, value attribute, text).
fn ods(rows: &[Vec<(&str, &str, &str)>]) -> Vec<u8> {
    let mut body = String::new();
    for row in rows {
        body.push_str("<table:table-row>");
        for (kind, value, text) in row {
            let attribute = match *kind {
                "float" => format!(r#" office:value="{value}""#),
                "date" => format!(r#" office:date-value="{value}""#),
                "boolean" => format!(r#" office:boolean-value="{value}""#),
                _ => String::new(),
            };
            body.push_str(&format!(
                r#"<table:table-cell office:value-type="{kind}"{attribute}><text:p>{text}</text:p></table:table-cell>"#
            ));
        }
        body.push_str("</table:table-row>");
    }
    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" office:version="1.2"><office:body><office:spreadsheet><table:table table:name="Readings">{body}</table:table></office:spreadsheet></office:body></office:document-content>"#
    );
    let manifest = r#"<?xml version="1.0" encoding="UTF-8"?><manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2"><manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.spreadsheet"/><manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/></manifest:manifest>"#;
    zip(&[
        (
            "mimetype",
            "application/vnd.oasis.opendocument.spreadsheet".to_owned(),
        ),
        ("META-INF/manifest.xml", manifest.to_owned()),
        ("content.xml", content),
    ])
}

fn read(
    bytes: &[u8],
    format: Format,
    sheet: SheetOptions,
) -> Result<(Table, Vec<String>), DataError> {
    let loaded = read_bytes(
        bytes,
        format,
        "book.xlsx",
        &ReadOptions::new().with_sheet(sheet),
    )?;
    let codes = loaded
        .diagnostics
        .iter()
        .map(|d| d.code.as_str().to_owned())
        .collect();
    Ok((loaded.bundle.primary().unwrap().clone(), codes))
}

fn names(table: &Table) -> Vec<String> {
    table
        .schema()
        .fields()
        .iter()
        .map(|f| f.name().clone())
        .collect()
}

fn types(table: &Table) -> Vec<DataType> {
    table
        .schema()
        .fields()
        .iter()
        .map(|f| f.data_type().clone())
        .collect()
}

/// The main test sheet: every cell kind in a header-plus-data layout.
fn readings() -> Vec<Vec<Cell>> {
    vec![
        vec![
            Cell::Shared(0),
            Cell::Str("temp"),
            Cell::Str("count"),
            Cell::Str("ok"),
            Cell::Str("double"),
            Cell::Str("site"),
        ],
        vec![
            Cell::Date(45292.0),
            Cell::Num(21.5),
            Cell::Num(3.0),
            Cell::Bool(true),
            Cell::Formula("C2*2", 6.0),
            Cell::Shared(1),
        ],
        vec![
            Cell::Date(45293.0),
            Cell::Error("#DIV/0!"),
            Cell::Num(4.0),
            Cell::Bool(false),
            Cell::Formula("C3*2", 8.0),
            Cell::Shared(2),
        ],
        vec![
            Cell::Date(45294.0),
            Cell::Num(-1.25),
            Cell::Empty,
            Cell::Bool(true),
            Cell::Formula("C4*2", 0.0),
            Cell::Shared(1),
        ],
    ]
}

#[test]
fn an_xlsx_sheet_gets_typed_fields() {
    let book = xlsx(&[("Readings", readings())], &["day", "north", "south"]);
    let (table, codes) = read(&book, Format::Xlsx, SheetOptions::default()).unwrap();
    assert_eq!(
        names(&table),
        ["day", "temp", "count", "ok", "double", "site"]
    );
    assert_eq!(
        types(&table),
        [
            DataType::Date32,
            DataType::Float64,
            DataType::Int64,
            DataType::Boolean,
            DataType::Int64,
            DataType::Utf8
        ]
    );
    assert_eq!(table.name(), "Readings");
    // Serial 45292 is 2024-01-01.
    assert_eq!(
        table
            .column("day")
            .unwrap()
            .as_primitive::<Date32Type>()
            .value(0),
        19_723
    );
    assert!(
        table.column("temp").unwrap().is_null(1),
        "the #DIV/0! cell is missing"
    );
    assert!(table.column("count").unwrap().is_null(2));
    let double = table.column("double").unwrap();
    assert_eq!(
        double.as_primitive::<Int64Type>().values().to_vec(),
        [6, 8, 0],
        "cached formula values"
    );
    assert_eq!(
        table.column("site").unwrap().as_string::<i32>().value(1),
        "south"
    );
    assert!(codes.contains(&"SNR-DATA-022".to_owned()), "{codes:?}");
    assert!(codes.contains(&"SNR-DATA-024".to_owned()), "{codes:?}");
}

#[test]
fn sheets_are_chosen_by_name_or_position_and_empty_sheets_are_skipped() {
    let other = vec![
        vec![Cell::Str("x"), Cell::Str("y")],
        vec![Cell::Num(1.0), Cell::Num(0.5)],
    ];
    let book = xlsx(
        &[
            ("Empty", vec![]),
            ("Readings", readings()),
            ("Other", other),
        ],
        &["day", "north", "south"],
    );
    let (table, _) = read(&book, Format::Xlsx, SheetOptions::default()).unwrap();
    assert_eq!(table.name(), "Readings", "the first sheet with data");
    let by_name = SheetOptions::new().with_sheet(SheetRef::Name("Other".into()));
    assert_eq!(
        names(&read(&book, Format::Xlsx, by_name).unwrap().0),
        ["x", "y"]
    );
    let by_index = SheetOptions::new().with_sheet(SheetRef::Index(2));
    assert_eq!(
        read(&book, Format::Xlsx, by_index).unwrap().0.name(),
        "Other"
    );
    let missing = SheetOptions::new().with_sheet(SheetRef::Name("Nope".into()));
    let err = read(&book, Format::Xlsx, missing).unwrap_err();
    assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-023");
}

#[test]
fn a_range_selects_a_block_of_cells() {
    let book = xlsx(&[("Readings", readings())], &["day", "north", "south"]);
    let block = SheetOptions::new().with_range("B1:C3");
    let (table, _) = read(&book, Format::Xlsx, block).unwrap();
    assert_eq!(names(&table), ["temp", "count"]);
    assert_eq!(table.num_rows(), 2);
    let bad = SheetOptions::new().with_range("C3:B1");
    let err = read(&book, Format::Xlsx, bad).unwrap_err();
    assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-006");
}

#[test]
fn headerless_sheets_and_date_times() {
    let rows = vec![
        vec![Cell::Date(45292.5), Cell::Num(1.5)],
        vec![Cell::Date(45292.75), Cell::Num(2.5)],
    ];
    let book = xlsx(&[("Data", rows)], &[]);
    let (table, _) = read(&book, Format::Xlsx, SheetOptions::default()).unwrap();
    assert_eq!(names(&table), ["column_1", "column_2"]);
    assert_eq!(table.num_rows(), 2);
    let stamps = table.column("column_1").unwrap();
    let stamps = stamps.as_primitive::<TimestampMicrosecondType>();
    assert_eq!(stamps.value(1) - stamps.value(0), 6 * 3600 * 1_000_000);
}

#[test]
fn an_ods_sheet_gets_typed_fields() {
    let book = ods(&[
        vec![
            ("string", "", "day"),
            ("string", "", "level"),
            ("string", "", "on"),
        ],
        vec![
            ("date", "2024-01-01", "01/01/24"),
            ("float", "1.5", "1.5"),
            ("boolean", "true", "TRUE"),
        ],
        vec![
            ("date", "2024-01-02", "02/01/24"),
            ("float", "2.25", "2.25"),
            ("boolean", "false", "FALSE"),
        ],
    ]);
    let (table, codes) = read(&book, Format::Ods, SheetOptions::default()).unwrap();
    assert_eq!(names(&table), ["day", "level", "on"]);
    assert_eq!(
        types(&table),
        [DataType::Date32, DataType::Float64, DataType::Boolean]
    );
    assert_eq!(
        table
            .column("level")
            .unwrap()
            .as_primitive::<Float64Type>()
            .value(1),
        2.25
    );
    assert_eq!(table.name(), "Readings");
    assert!(codes.contains(&"SNR-DATA-022".to_owned()), "{codes:?}");
}

#[test]
fn files_that_are_not_workbooks_fail_cleanly() {
    for bytes in [
        &b"not a workbook"[..],
        &zip(&[("hello.txt", "hi".to_owned())])[..],
    ] {
        let err = read(bytes, Format::Xlsx, SheetOptions::default()).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-001", "{err}");
    }
}

#[test]
fn sheet_options_deserialize_from_a_spec() {
    let options: SheetOptions = serde_json::from_str(r#"{"sheet": 1, "range": "A1:B9"}"#).unwrap();
    assert_eq!(options.sheet, Some(SheetRef::Index(1)));
    let options: SheetOptions =
        serde_json::from_str(r#"{"sheet": "Data", "header": false}"#).unwrap();
    assert_eq!(options.sheet, Some(SheetRef::Name("Data".into())));
    assert_eq!(options.header, Some(false));
}

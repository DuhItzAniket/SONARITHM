// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Office chart XML: tier 1, exact (10 §3, notes/06 §6).
//!
//! An XLSX, PPTX or DOCX chart part (`*/charts/chartN.xml`, in `DrawingML`) keeps, for every
//! series, a name (`c:tx`), categories (`c:cat`, or `c:xVal` for scatter and bubble charts)
//! and values (`c:val`, or `c:yVal`). Each is a reference with a formula and a cache of
//! the last values shown (`c:numRef`/`c:numCache`, `c:strRef`/`c:strCache`,
//! `c:multiLvlStrRef`) or a literal (`c:numLit`, `c:strLit`).
//!
//! The **workbook is preferred** when it can be read: the file itself for an XLSX, or the
//! workbook embedded in a PPTX or DOCX chart (its `c:externalData` relationship). A formula
//! such as `Sheet1!$B$2:$B$13` is resolved through calamine, after every part of that
//! workbook has been inflated once under the budgets, so calamine never sees a bomb. When
//! the workbook is missing or a formula cannot be resolved, the cache is read. Values
//! from either are `exact`. Nothing is followed outside the file: an external data link,
//! a formula into another workbook and macros are ignored (12 §6).
//!
//! When a file holds several charts, the first (by part number) is read and the others are
//! reported (`SNR-ENG-340`): a spec names one chart source.

use std::collections::BTreeMap;
use std::io::Cursor;

use roxmltree::{Document, Node};
use sonarithm_core::Diagnostic;
use sonarithm_core::diagnostic::Severity;

use crate::budget::{Budget, Budgets, Work, len64, over};
use crate::codes;
use crate::container::Package;
use crate::model::{ChartData, ChartType, Point, Series, XValue};
use crate::xml;

/// Excel serial day of 1970-01-01 in the 1900 date system.
const EPOCH_1900: f64 = 25_569.0;
/// Excel serial day of 1970-01-01 in the 1904 date system.
const EPOCH_1904: f64 = 24_107.0;
/// Milliseconds in a day.
const DAY_MS: f64 = 86_400_000.0;

/// The relationship type of an embedded package (the chart's workbook).
const PACKAGE_REL: &str = "/package";

/// Extracts the first chart of an Office package.
pub(crate) fn extract(
    bytes: &[u8],
    is_xlsx: bool,
    budgets: &Budgets,
    work: &mut Work,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<ChartData, Box<Diagnostic>> {
    let mut package = Package::open(bytes, budgets)?;
    let parts = chart_parts(&package.names());
    let Some(first) = parts.first() else {
        return Err(Box::new(Diagnostic::new(
            &codes::NO_CHART,
            "The document has no charts.",
        )));
    };
    if parts.len() > 1 {
        diagnostics.push(Diagnostic::new(
            &codes::OTHER_CHARTS,
            format!(
                "The file holds {} charts; the first, `{first}`, was read.",
                parts.len()
            ),
        ));
    }
    let xml_bytes = package.read(first, work)?.unwrap_or_default();
    let document = xml::parse(&xml_bytes, "chart part", budgets, work)?;
    let chart = read_chart(&document, budgets)?;
    let workbook = if is_xlsx {
        Workbook::open(bytes, budgets, work, diagnostics)
    } else {
        embedded_workbook(&mut package, first, &document, budgets, work, diagnostics)
    };
    resolve(chart, workbook.as_ref(), budgets, diagnostics)
}

/// Reads one chart part on its own, from its caches (the `office_chart` fuzz target).
pub(crate) fn chart_part(
    xml_bytes: &[u8],
    budgets: &Budgets,
    work: &mut Work,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<ChartData, Box<Diagnostic>> {
    let document = xml::parse(xml_bytes, "chart part", budgets, work)?;
    let chart = read_chart(&document, budgets)?;
    resolve(chart, None, budgets, diagnostics)
}

/// The chart parts of an Office package (`xl/charts/chart1.xml`, `ppt/charts/…`,
/// `word/charts/…`), in part-number order.
pub(crate) fn chart_parts(names: &[String]) -> Vec<String> {
    let mut parts: Vec<(u64, String)> = names
        .iter()
        .filter_map(|name| {
            let number = ["xl/charts/chart", "ppt/charts/chart", "word/charts/chart"]
                .iter()
                .find_map(|prefix| name.strip_prefix(prefix))
                .and_then(|rest| rest.strip_suffix(".xml"))
                .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))?;
            Some((number.parse().unwrap_or(u64::MAX), name.clone()))
        })
        .collect();
    parts.sort();
    parts.into_iter().map(|(_, name)| name).collect()
}

// ---------------------------------------------------------------------------------------
// Reading the chart part

/// A reference or literal: an optional formula and the cached or literal values.
#[derive(Clone, Debug, Default, PartialEq)]
struct DataRef {
    formula: Option<String>,
    cache: Cache,
}

#[derive(Clone, Debug, Default, PartialEq)]
enum Cache {
    #[default]
    None,
    /// Numbers by point index; `date` when the format code is a date format.
    Numbers {
        values: Vec<Option<f64>>,
        date: bool,
    },
    /// Labels by point index; multi-level labels joined outer to inner with " / ".
    Text(Vec<Option<String>>),
}

#[derive(Clone, Debug)]
struct RawSeries {
    kind: ChartType,
    name: Option<DataRef>,
    x: Option<DataRef>,
    y: Option<DataRef>,
}

#[derive(Clone, Debug)]
struct RawChart {
    chart_type: Option<ChartType>,
    title: Option<String>,
    x_title: Option<String>,
    y_title: Option<String>,
    date1904: bool,
    series: Vec<RawSeries>,
}

fn read_chart(document: &Document<'_>, budgets: &Budgets) -> Result<RawChart, Box<Diagnostic>> {
    let space = document.root_element();
    let date1904 = xml::child(space, "date1904").is_some_and(|n| flag(n, true));
    let malformed = || {
        Box::new(Diagnostic::new(
            &codes::MALFORMED,
            "The chart part has no plot area.",
        ))
    };
    let chart = xml::child(space, "chart").ok_or_else(malformed)?;
    let plot = xml::child(chart, "plotArea").ok_or_else(malformed)?;
    let mut series = Vec::new();
    let mut chart_type = None;
    for group in plot.children().filter(Node::is_element) {
        let Some(kind) = group_kind(group) else {
            continue;
        };
        chart_type.get_or_insert(kind);
        let scatter = matches!(group.tag_name().name(), "scatterChart" | "bubbleChart");
        for ser in xml::children(group, "ser") {
            let (x, y) = if scatter {
                ("xVal", "yVal")
            } else {
                ("cat", "val")
            };
            series.push(RawSeries {
                kind,
                name: xml::child(ser, "tx")
                    .map(|tx| data_ref(tx, budgets))
                    .transpose()?,
                x: xml::child(ser, x)
                    .map(|n| data_ref(n, budgets))
                    .transpose()?,
                y: xml::child(ser, y)
                    .map(|n| data_ref(n, budgets))
                    .transpose()?,
            });
        }
    }
    // Numeric categories on a date axis are dates, whatever the cache's format code says
    // (writers often cache them as `General`).
    if xml::child(plot, "dateAx").is_some() {
        for raw in &mut series {
            if let Some(DataRef {
                cache: Cache::Numbers { date, .. },
                ..
            }) = &mut raw.x
            {
                *date = true;
            }
        }
    }
    let (x_title, y_title) = axis_titles(plot, chart_type == Some(ChartType::Scatter));
    Ok(RawChart {
        chart_type,
        title: xml::child(chart, "title").and_then(title_text),
        x_title,
        y_title,
        date1904,
        series,
    })
}

/// The chart type of a plot-area group, or `None` for an element that is not one.
fn group_kind(group: Node<'_, '_>) -> Option<ChartType> {
    Some(match group.tag_name().name() {
        "lineChart" | "line3DChart" => ChartType::Line,
        "areaChart" | "area3DChart" => ChartType::Area,
        "barChart" | "bar3DChart" => {
            let grouping = xml::child(group, "grouping").and_then(|g| g.attribute("val"));
            if matches!(grouping, Some("stacked" | "percentStacked")) {
                ChartType::StackedBar
            } else {
                ChartType::Bar
            }
        }
        "scatterChart" => ChartType::Scatter,
        "pieChart" | "pie3DChart" | "doughnutChart" | "ofPieChart" => ChartType::Pie,
        "radarChart" | "bubbleChart" | "stockChart" | "surfaceChart" | "surface3DChart" => {
            ChartType::Other
        }
        _ => return None,
    })
}

/// A `c:tx`, `c:cat`, `c:val`, `c:xVal` or `c:yVal` element.
fn data_ref(node: Node<'_, '_>, budgets: &Budgets) -> Result<DataRef, Box<Diagnostic>> {
    for child in node.children().filter(Node::is_element) {
        let formula = xml::child(child, "f")
            .and_then(|f| f.text())
            .map(str::to_owned);
        let cache = match child.tag_name().name() {
            "numRef" => xml::child(child, "numCache")
                .map(|c| numbers(c, budgets))
                .transpose()?,
            "numLit" => Some(numbers(child, budgets)?),
            "strRef" => xml::child(child, "strCache")
                .map(|c| texts(c, budgets))
                .transpose()?,
            "strLit" => Some(texts(child, budgets)?),
            "multiLvlStrRef" => xml::child(child, "multiLvlStrCache")
                .map(|c| multi_level(c, budgets))
                .transpose()?,
            // A series name written inline: `<c:tx><c:v>Sales</c:v></c:tx>`.
            "v" => Some(Cache::Text(vec![child.text().map(str::to_owned)])),
            _ => continue,
        };
        return Ok(DataRef {
            formula,
            cache: cache.unwrap_or_default(),
        });
    }
    Ok(DataRef::default())
}

/// The point count of a cache, checked against the point budget before any allocation.
fn point_count(cache: Node<'_, '_>, budgets: &Budgets) -> Result<usize, Box<Diagnostic>> {
    let declared: u64 = xml::child(cache, "ptCount")
        .and_then(|n| n.attribute("val"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let highest = xml::children(cache, "pt")
        .filter_map(|pt| pt.attribute("idx")?.parse::<u64>().ok())
        .max()
        .map_or(0, |i| i.saturating_add(1));
    let count = declared.max(highest);
    if count > budgets.max_points {
        return Err(over(Budget::Points, count, budgets.max_points));
    }
    Ok(usize::try_from(count).unwrap_or(usize::MAX))
}

/// Points by index; a point without `idx` takes the next index.
fn points<'a, 'input>(cache: Node<'a, 'input>) -> impl Iterator<Item = (usize, Node<'a, 'input>)> {
    let mut next = 0_usize;
    xml::children(cache, "pt").map(move |pt| {
        let index = pt
            .attribute("idx")
            .and_then(|i| i.parse().ok())
            .unwrap_or(next);
        next = index.saturating_add(1);
        (index, pt)
    })
}

fn numbers(cache: Node<'_, '_>, budgets: &Budgets) -> Result<Cache, Box<Diagnostic>> {
    let mut values = vec![None; point_count(cache, budgets)?];
    for (index, pt) in points(cache) {
        let value = xml::child(pt, "v")
            .and_then(|v| v.text())
            .and_then(|t| t.trim().parse::<f64>().ok())
            .filter(|v| v.is_finite());
        if let Some(slot) = values.get_mut(index) {
            *slot = value;
        }
    }
    let format = xml::child(cache, "formatCode").and_then(|f| f.text());
    Ok(Cache::Numbers {
        values,
        date: format.is_some_and(is_date_format),
    })
}

fn texts(cache: Node<'_, '_>, budgets: &Budgets) -> Result<Cache, Box<Diagnostic>> {
    let mut values = vec![None; point_count(cache, budgets)?];
    for (index, pt) in points(cache) {
        if let Some(slot) = values.get_mut(index) {
            *slot = xml::child(pt, "v")
                .and_then(|v| v.text())
                .map(str::to_owned);
        }
    }
    Ok(Cache::Text(values))
}

/// Multi-level categories: `c:lvl` elements from the innermost level outwards.
fn multi_level(cache: Node<'_, '_>, budgets: &Budgets) -> Result<Cache, Box<Diagnostic>> {
    let count = point_count(cache, budgets)?;
    let mut labels: Vec<Vec<String>> = vec![Vec::new(); count];
    for level in xml::children(cache, "lvl") {
        // An outer level labels the first point of each group; later points repeat it.
        let mut at: BTreeMap<usize, String> = BTreeMap::new();
        for (index, pt) in points(level) {
            if let Some(text) = xml::child(pt, "v").and_then(|v| v.text()) {
                at.insert(index, text.to_owned());
            }
        }
        let mut current = String::new();
        for (index, label) in labels.iter_mut().enumerate() {
            if let Some(text) = at.get(&index) {
                text.clone_into(&mut current);
            }
            if !current.is_empty() {
                label.insert(0, current.clone());
            }
        }
    }
    Ok(Cache::Text(
        labels
            .into_iter()
            .map(|parts| (!parts.is_empty()).then(|| parts.join(" / ")))
            .collect(),
    ))
}

/// Whether an Excel number format shows a date: it has a year, day or month-name token
/// outside quoted text and `[…]` sections.
fn is_date_format(format: &str) -> bool {
    let mut plain = String::new();
    let mut quoted = false;
    let mut bracket = false;
    for c in format.chars() {
        match c {
            '"' => quoted = !quoted,
            '[' if !quoted => bracket = true,
            ']' if !quoted => bracket = false,
            _ if !quoted && !bracket => plain.push(c.to_ascii_lowercase()),
            _ => {}
        }
    }
    plain != "general" && (plain.contains('y') || plain.contains('d') || plain.contains("mmm"))
}

fn flag(node: Node<'_, '_>, default: bool) -> bool {
    node.attribute("val")
        .map_or(default, |v| matches!(v, "1" | "true"))
}

/// The text of a `c:title`: rich text runs, or a referenced string.
fn title_text(title: Node<'_, '_>) -> Option<String> {
    let text: String = title
        .descendants()
        .filter(|n| n.is_element() && matches!(n.tag_name().name(), "t" | "v"))
        .filter_map(|n| n.text())
        .collect();
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// The x- and y-axis titles. The x axis is the category or date axis, or for a scatter
/// chart the value axis at the bottom or top.
fn axis_titles(plot: Node<'_, '_>, scatter: bool) -> (Option<String>, Option<String>) {
    let mut x = None;
    let mut y = None;
    for axis in plot.children().filter(Node::is_element) {
        let name = axis.tag_name().name();
        if !matches!(name, "catAx" | "dateAx" | "valAx") {
            continue;
        }
        let position = xml::child(axis, "axPos").and_then(|p| p.attribute("val"));
        let is_x =
            matches!(name, "catAx" | "dateAx") || (scatter && matches!(position, Some("b" | "t")));
        let title = xml::child(axis, "title").and_then(title_text);
        let slot = if is_x { &mut x } else { &mut y };
        if slot.is_none() {
            *slot = title;
        }
    }
    (x, y)
}

// ---------------------------------------------------------------------------------------
// The workbook

/// A workbook read through calamine, its sheets keyed by name.
struct Workbook {
    sheets: BTreeMap<String, calamine::Range<calamine::Data>>,
}

impl Workbook {
    /// Reads `bytes` as a workbook, after inflating every part once under the budgets.
    /// A workbook that cannot be read is reported (`SNR-ENG-343`) and the cache is used.
    fn open(
        bytes: &[u8],
        budgets: &Budgets,
        work: &mut Work,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<Self> {
        let unreadable = |diagnostics: &mut Vec<Diagnostic>, reason: &str| {
            diagnostics.push(Diagnostic::new(
                &codes::WORKBOOK_UNREADABLE,
                format!("The chart's workbook could not be read ({reason}); the values cached in the chart were used."),
            ));
        };
        if let Err(diagnostic) = inflate_all(bytes, budgets, work) {
            unreadable(diagnostics, &diagnostic.message);
            return None;
        }
        let read = std::panic::catch_unwind(|| -> Result<Self, String> {
            use calamine::Reader as _;
            let mut book: calamine::Xlsx<_> =
                calamine::Xlsx::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
            let mut sheets = BTreeMap::new();
            for name in book.sheet_names() {
                let range = book.worksheet_range(&name).map_err(|e| e.to_string())?;
                sheets.insert(name, range);
            }
            Ok(Self { sheets })
        });
        match read {
            Ok(Ok(book)) => Some(book),
            Ok(Err(reason)) => {
                unreadable(diagnostics, &reason);
                None
            }
            Err(_) => {
                unreadable(diagnostics, "the workbook reader failed");
                None
            }
        }
    }

    /// The cells of a formula such as `Sheet1!$B$2:$B$13`, in row-major order, or `None`
    /// when the formula is not a plain range on one of this workbook's sheets.
    fn cells(&self, formula: &str) -> Option<Vec<&calamine::Data>> {
        let (sheet, (r1, c1), (r2, c2)) = parse_range(formula)?;
        let range = self.sheets.get(&sheet)?;
        let mut cells = Vec::new();
        for row in r1.min(r2)..=r1.max(r2) {
            for col in c1.min(c2)..=c1.max(c2) {
                cells.push(
                    range
                        .get_value((row, col))
                        .unwrap_or(&calamine::Data::Empty),
                );
            }
        }
        Some(cells)
    }
}

/// Inflates every part of a package under the budgets, so a hostile workbook fails here
/// rather than inside the workbook reader.
fn inflate_all(bytes: &[u8], budgets: &Budgets, work: &mut Work) -> Result<(), Box<Diagnostic>> {
    let mut package = Package::open(bytes, budgets)?;
    for name in package.names() {
        package.read(&name, work)?;
    }
    Ok(())
}

/// The workbook embedded in a PPTX or DOCX chart, through the chart's `c:externalData`
/// relationship. An external target is never followed.
fn embedded_workbook(
    package: &mut Package<'_>,
    part: &str,
    document: &Document<'_>,
    budgets: &Budgets,
    work: &mut Work,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Workbook> {
    let id = xml::child(document.root_element(), "externalData")?
        .attributes()
        .find(|a| a.name() == "id")?
        .value()
        .to_owned();
    let (dir, file) = part.rsplit_once('/')?;
    let rels_name = format!("{dir}/_rels/{file}.rels");
    let rels_bytes = package.read(&rels_name, work).ok()??;
    let rels = xml::parse(&rels_bytes, "relationships part", budgets, work).ok()?;
    let rel = rels
        .root_element()
        .children()
        .find(|r| r.is_element() && r.attribute("Id") == Some(id.as_str()))?;
    if rel.attribute("TargetMode") == Some("External") {
        diagnostics.push(Diagnostic::new(
            &codes::EXTERNAL_LINK,
            "The chart's data links to a file outside the document; the link was not \
             followed and the values cached in the chart were used.",
        ));
        return None;
    }
    if !rel
        .attribute("Type")
        .is_some_and(|t| t.ends_with(PACKAGE_REL))
    {
        return None;
    }
    let target = resolve_target(dir, rel.attribute("Target")?)?;
    let embedded = package.read(&target, work).ok()??;
    if !embedded.starts_with(b"PK\x03\x04") {
        diagnostics.push(Diagnostic::new(
            &codes::WORKBOOK_UNREADABLE,
            "The chart's embedded workbook is not an XLSX package; the values cached in \
             the chart were used.",
        ));
        return None;
    }
    Workbook::open(&embedded, budgets, work, diagnostics)
}

/// A relationship target relative to the source part's directory, or `None` if it
/// leaves the package.
fn resolve_target(dir: &str, target: &str) -> Option<String> {
    let mut parts: Vec<&str> = if target.starts_with('/') {
        Vec::new()
    } else {
        dir.split('/').filter(|p| !p.is_empty()).collect()
    };
    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

/// A 0-based (row, column) cell position.
type Cell = (u32, u32);

/// `Sheet1!$B$2:$B$13`, `'My sheet'!B2:B13` or `Sheet1!$B$1`: the sheet and the 0-based
/// (row, column) corners. Unions, defined names and other workbooks give `None`.
fn parse_range(formula: &str) -> Option<(String, Cell, Cell)> {
    let formula = formula.trim();
    if formula.contains(['[', ',', '(']) {
        return None;
    }
    let (sheet, cells) = formula.rsplit_once('!')?;
    let sheet = match sheet.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
        Some(quoted) => quoted.replace("''", "'"),
        None => sheet.to_owned(),
    };
    let (start, end) = cells.split_once(':').unwrap_or((cells, cells));
    Some((sheet, cell(start)?, cell(end)?))
}

/// `$B$2` → (1, 1).
fn cell(reference: &str) -> Option<Cell> {
    let reference = reference.replace('$', "");
    let split = reference.find(|c: char| c.is_ascii_digit())?;
    let (letters, digits) = reference.split_at(split);
    if letters.is_empty() || letters.len() > 3 {
        return None;
    }
    let mut col = 0_u32;
    for c in letters.chars() {
        let c = c.to_ascii_uppercase();
        if !c.is_ascii_uppercase() {
            return None;
        }
        col = col * 26 + (u32::from(c) - u32::from('A') + 1);
    }
    let row: u32 = digits.parse().ok()?;
    Some((row.checked_sub(1)?, col - 1))
}

// ---------------------------------------------------------------------------------------
// Resolving series

/// Values of one reference, from the workbook when it resolves the formula, else from the
/// cache.
#[derive(Clone, Debug, PartialEq)]
enum Values {
    Numbers(Vec<Option<f64>>),
    Times(Vec<Option<i64>>),
    Text(Vec<Option<String>>),
}

struct Resolver<'a> {
    workbook: Option<&'a Workbook>,
    epoch: f64,
    used_workbook: bool,
    stale: bool,
}

impl Resolver<'_> {
    fn values(&mut self, data: &DataRef) -> Option<Values> {
        let cached = self.cached(&data.cache);
        let from_book = self
            .workbook
            .zip(data.formula.as_deref())
            .and_then(|(book, formula)| book.cells(formula))
            .map(|cells| self.cell_values(&cells, cached.as_ref()));
        match from_book {
            Some(values) => {
                self.used_workbook = true;
                if cached.as_ref().is_some_and(|c| !same(c, &values)) {
                    self.stale = true;
                }
                Some(values)
            }
            None => cached,
        }
    }

    fn cached(&self, cache: &Cache) -> Option<Values> {
        match cache {
            Cache::None => None,
            Cache::Numbers { values, date: true } => Some(Values::Times(
                values.iter().map(|v| v.map(|v| self.ms(v))).collect(),
            )),
            Cache::Numbers {
                values,
                date: false,
            } => Some(Values::Numbers(values.clone())),
            Cache::Text(values) => Some(Values::Text(values.clone())),
        }
    }

    /// Cells as values, keeping the cache's kind where the cells are ambiguous.
    fn cell_values(&self, cells: &[&calamine::Data], cached: Option<&Values>) -> Values {
        use calamine::Data;
        let dates = cells.iter().any(|c| matches!(c, Data::DateTime(_)))
            || matches!(cached, Some(Values::Times(_)));
        let text = cells.iter().any(|c| matches!(c, Data::String(_)))
            || matches!(cached, Some(Values::Text(_)));
        let number = |cell: &Data| -> Option<f64> {
            match cell {
                Data::Int(i) => {
                    #[expect(clippy::cast_precision_loss, reason = "spreadsheet integers")]
                    let value = *i as f64;
                    Some(value)
                }
                Data::Float(f) => Some(*f),
                Data::DateTime(d) => Some(d.as_f64()),
                Data::String(s) => s.trim().parse().ok(),
                Data::Bool(b) => Some(f64::from(u8::from(*b))),
                _ => None,
            }
            .filter(|v| v.is_finite())
        };
        if text {
            Values::Text(
                cells
                    .iter()
                    .map(|c| match c {
                        Data::Empty | Data::Error(_) => None,
                        Data::String(s) => Some(s.clone()),
                        other => number(other).map(|v| v.to_string()),
                    })
                    .collect(),
            )
        } else if dates {
            Values::Times(
                cells
                    .iter()
                    .map(|c| number(c).map(|v| self.ms(v)))
                    .collect(),
            )
        } else {
            Values::Numbers(cells.iter().map(|c| number(c)).collect())
        }
    }

    /// An Excel serial date as milliseconds since the Unix epoch.
    fn ms(&self, serial: f64) -> i64 {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "dates within ±285,000 years"
        )]
        let ms = ((serial - self.epoch) * DAY_MS).round() as i64;
        ms
    }
}

/// Whether workbook and cache agree: same length and the same values, compared as numbers
/// when both read as numbers (writers cache numeric text as numbers, and round to 15 or 17
/// significant digits, so numbers agree within a part in 10^12).
fn same(a: &Values, b: &Values) -> bool {
    match (numeric(a), numeric(b)) {
        (Some(a), Some(b)) => {
            a.len() == b.len()
                && a.iter().zip(&b).all(|(x, y)| match (x, y) {
                    (Some(x), Some(y)) => (x - y).abs() <= x.abs().max(y.abs()) * 1e-12,
                    (None, None) => true,
                    _ => false,
                })
        }
        _ => a == b,
    }
}

/// The values as numbers, or `None` when some text is not a number.
fn numeric(values: &Values) -> Option<Vec<Option<f64>>> {
    match values {
        Values::Numbers(v) => Some(v.clone()),
        Values::Times(v) => Some(
            v.iter()
                .map(|t| {
                    #[expect(clippy::cast_precision_loss, reason = "dates are exact in f64")]
                    let value = t.map(|t| t as f64);
                    value
                })
                .collect(),
        ),
        Values::Text(v) => v
            .iter()
            .map(|t| match t {
                Some(t) => t.trim().parse::<f64>().ok().map(Some),
                None => Some(None),
            })
            .collect(),
    }
}

fn resolve(
    chart: RawChart,
    workbook: Option<&Workbook>,
    budgets: &Budgets,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<ChartData, Box<Diagnostic>> {
    let mut resolver = Resolver {
        workbook,
        epoch: if chart.date1904 {
            EPOCH_1904
        } else {
            EPOCH_1900
        },
        used_workbook: false,
        stale: false,
    };
    let mut series = Vec::new();
    let mut without_values = Vec::new();
    let mut total = 0_u64;
    for (index, raw) in chart.series.iter().enumerate() {
        let name = raw
            .name
            .as_ref()
            .and_then(|n| resolver.values(n))
            .and_then(|v| match v {
                Values::Text(t) => t.into_iter().flatten().next(),
                Values::Numbers(n) => n.into_iter().flatten().next().map(|v| v.to_string()),
                Values::Times(_) => None,
            })
            .unwrap_or_else(|| format!("Series {}", index + 1));
        let Some(y) = raw.y.as_ref().and_then(|y| resolver.values(y)) else {
            without_values.push(name);
            continue;
        };
        let ys: Vec<Option<f64>> = match y {
            Values::Numbers(v) => v,
            Values::Times(v) => v
                .into_iter()
                .map(|t| {
                    #[expect(clippy::cast_precision_loss, reason = "dates are exact in f64")]
                    let value = t.map(|t| t as f64);
                    value
                })
                .collect(),
            Values::Text(v) => v
                .into_iter()
                .map(|t| t.and_then(|t| t.trim().parse().ok()))
                .collect(),
        };
        let xs = raw.x.as_ref().and_then(|x| resolver.values(x));
        total = total.saturating_add(len64(ys.len()));
        if total > budgets.max_points {
            return Err(over(Budget::Points, total, budgets.max_points));
        }
        let points = ys
            .into_iter()
            .enumerate()
            .map(|(i, y)| Point {
                x: x_at(xs.as_ref(), i),
                y,
            })
            .collect();
        let mut s = Series::exact(name, points);
        s.chart_type = Some(raw.kind);
        series.push(s);
    }
    if !without_values.is_empty() {
        let names = without_values.join("`, `");
        let diagnostic = Diagnostic::new(
            &codes::NO_CACHE,
            format!(
                "The series `{names}` have neither cached values nor a readable workbook \
                 range; open the file in its program and save it again."
            ),
        );
        if series.is_empty() {
            return Err(Box::new(diagnostic));
        }
        diagnostics.push(diagnostic.with_severity(Severity::Warning));
    }
    if resolver.used_workbook {
        diagnostics.push(Diagnostic::new(
            &codes::VALUES_SOURCE,
            "Values were read from the chart's workbook, the data the chart was built from.",
        ));
    }
    if resolver.stale {
        diagnostics.push(Diagnostic::new(
            &codes::STALE_CACHE,
            "The values cached in the chart differ from its workbook; the workbook's were \
             used. The chart may show older numbers until the file is opened and saved.",
        ));
    }
    let chart_type = if series.iter().all(|s| s.chart_type == chart.chart_type) {
        chart.chart_type
    } else {
        chart.chart_type.map(|_| ChartType::Other)
    };
    Ok(ChartData {
        chart_type,
        title: chart.title,
        x_title: chart.x_title,
        y_title: chart.y_title,
        series,
    })
}

/// The x position of point `i`: its category, number or time, or its 1-based index when
/// the series has no categories.
fn x_at(xs: Option<&Values>, i: usize) -> XValue {
    #[expect(clippy::cast_precision_loss, reason = "point indices far below 2^52")]
    let index = (i + 1) as f64;
    match xs {
        Some(Values::Numbers(v)) => v
            .get(i)
            .copied()
            .flatten()
            .map_or(XValue::Number(index), XValue::Number),
        Some(Values::Times(v)) => v
            .get(i)
            .copied()
            .flatten()
            .map_or(XValue::Number(index), XValue::Time),
        Some(Values::Text(v)) => XValue::Category(v.get(i).cloned().flatten().unwrap_or_default()),
        None => XValue::Number(index),
    }
}

#[cfg(test)]
mod tests;

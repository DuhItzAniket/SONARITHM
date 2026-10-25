// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! PLOTSONA's diagnostic codes, in its block of the shared `SNR-ENG` area:
//! `SNR-ENG-300` to `SNR-ENG-399` (coding-standards §4.2).
//!
//! 300–319 detection, budgets and reading; 320–339 axis calibration; 340–359 exact
//! extractors (Office charts, chart JSON); 360–379 vector extractors (SVG, PDF); 380–399
//! templates and importers. A released code is never reused.

use sonarithm_core::codes::{Area, CodeInfo, CodeStatus, CodeTable, validate};
use sonarithm_core::diagnostic::Severity;

/// The file is not a chart format PLOTSONA reads.
pub const UNKNOWN_FORMAT: CodeInfo = CodeInfo {
    code: "SNR-ENG-300",
    severity: Severity::Error,
    summary: "The file is not a chart format PLOTSONA reads",
    help: "PLOTSONA reads XLSX, PPTX and DOCX charts, Vega, Vega-Lite, Plotly and Highcharts \
           JSON (also inside a local HTML file), SVG and PDF; PNG and JPEG need the raster \
           tier. Save the chart in one of these formats, or load its data as a file source.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The input file is larger than the size budget.
pub const INPUT_TOO_LARGE: CodeInfo = CodeInfo {
    code: "SNR-ENG-301",
    severity: Severity::Error,
    summary: "The chart file is larger than the size limit",
    help: "Chart files are untrusted input with a size budget (12 §6). Export a smaller \
           chart, or raise the limit for a file you trust.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A container exceeded an entry, decompression or compression-ratio budget.
pub const CONTAINER_BUDGET: CodeInfo = CodeInfo {
    code: "SNR-ENG-302",
    severity: Severity::Error,
    summary: "The container exceeds a decompression limit",
    help: "The file has too many parts, decompresses to too much data, or compresses far \
           better than real documents do, as zip bombs do. PLOTSONA stopped before \
           decompressing it.",
    rules: &[],
    status: CodeStatus::Active,
};

/// An XML document declares a DTD, which PLOTSONA refuses.
pub const DTD_REFUSED: CodeInfo = CodeInfo {
    code: "SNR-ENG-303",
    severity: Severity::Error,
    summary: "The XML declares a DTD, which is refused",
    help: "Document type declarations can define entities that expand without end or point \
           at other files. Office charts and SVG exports do not need them; export the chart \
           again without a DOCTYPE.",
    rules: &[],
    status: CodeStatus::Active,
};

/// XML elements or JSON values nest deeper than the depth budget.
pub const NESTING_TOO_DEEP: CodeInfo = CodeInfo {
    code: "SNR-ENG-304",
    severity: Severity::Error,
    summary: "The file nests deeper than the limit",
    help: "Real charts nest a few dozen levels at most. The file may be damaged or crafted.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The extraction used up its work budget.
pub const WORK_BUDGET: CodeInfo = CodeInfo {
    code: "SNR-ENG-305",
    severity: Severity::Error,
    summary: "Extraction stopped at the work limit",
    help: "Extraction counts its steps instead of reading a clock, so the same file always \
           stops at the same point. Simplify the chart, or raise the limit for a file you \
           trust.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A document has more XML nodes or yields more points than the content budget.
pub const CONTENT_BUDGET: CodeInfo = CodeInfo {
    code: "SNR-ENG-306",
    severity: Severity::Error,
    summary: "The chart has more content than the limit",
    help: "The document has more XML nodes, or the chart more data points, than one \
           extraction allows. Load the chart's data as a file source instead.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The file is damaged: its container, XML or JSON cannot be parsed.
pub const MALFORMED: CodeInfo = CodeInfo {
    code: "SNR-ENG-307",
    severity: Severity::Error,
    summary: "The chart file is damaged",
    help: "The file's container, XML or JSON could not be parsed. Open it in the program \
           that made it and save it again.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The file needs an extraction tier this build does not have yet.
pub const TIER_NOT_AVAILABLE: CodeInfo = CodeInfo {
    code: "SNR-ENG-308",
    severity: Severity::Error,
    summary: "This kind of chart needs an extractor this build does not have",
    help: "Raster images (PNG, JPEG) need the raster digitizer (C02) and model-based \
           extraction needs the ML tier (D04). Use the chart's source file if you have it.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The container holds no chart, or the chart holds no data.
pub const NO_CHART: CodeInfo = CodeInfo {
    code: "SNR-ENG-309",
    severity: Severity::Error,
    summary: "No chart data was found",
    help: "The file was read, but it holds no chart with data PLOTSONA can extract.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The spec's `extractor` does not fit the file.
pub const EXTRACTOR_MISMATCH: CodeInfo = CodeInfo {
    code: "SNR-ENG-310",
    severity: Severity::Error,
    summary: "The chosen extractor does not fit the file",
    help: "Set `extractor` to `auto`, or to the extractor of the file's format.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The file is encrypted or password-protected.
pub const ENCRYPTED: CodeInfo = CodeInfo {
    code: "SNR-ENG-311",
    severity: Severity::Error,
    summary: "The chart file is encrypted",
    help: "PLOTSONA does not decrypt files. Save an unprotected copy.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The chart file could not be read from disk.
pub const FILE_UNREADABLE: CodeInfo = CodeInfo {
    code: "SNR-ENG-312",
    severity: Severity::Error,
    summary: "The chart file cannot be read",
    help: "Check that the path is relative to the spec's directory and the file exists.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Every PLOTSONA code, in ascending order.
pub const CODES: CodeTable = CodeTable {
    area: Area::Eng,
    owner: "sonarithm-plotsona",
    codes: &[
        UNKNOWN_FORMAT,
        INPUT_TOO_LARGE,
        CONTAINER_BUDGET,
        DTD_REFUSED,
        NESTING_TOO_DEEP,
        WORK_BUDGET,
        CONTENT_BUDGET,
        MALFORMED,
        TIER_NOT_AVAILABLE,
        NO_CHART,
        EXTRACTOR_MISMATCH,
        ENCRYPTED,
        FILE_UNREADABLE,
    ],
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid PLOTSONA code table");

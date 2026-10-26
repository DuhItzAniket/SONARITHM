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

/// An axis has too few usable anchors to calibrate.
pub const CALIBRATION_ANCHORS: CodeInfo = CodeInfo {
    code: "SNR-ENG-320",
    severity: Severity::Error,
    summary: "The axis has too few anchors to calibrate",
    help: "Calibration needs at least two ticks or anchors with different positions and            values. Enter two anchors by hand in `calibration` if the labels cannot be read.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A logarithmic axis has a value that is zero or negative.
pub const LOG_NOT_POSITIVE: CodeInfo = CodeInfo {
    code: "SNR-ENG-321",
    severity: Severity::Error,
    summary: "A logarithmic axis has a value that is not above zero",
    help: "Log axes place only positive values. Check the axis type and the tick labels.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Tick values do not change monotonically along the axis.
pub const TICKS_NOT_MONOTONIC: CodeInfo = CodeInfo {
    code: "SNR-ENG-322",
    severity: Severity::Error,
    summary: "The axis ticks are not monotonic",
    help: "Tick values must rise (or fall) steadily along the axis (10 §3.1). Labels may            have been read from the wrong ticks; enter the anchors by hand.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The axis was read with one scale but its ticks fit the other far better.
pub const SCALE_MISMATCH: CodeInfo = CodeInfo {
    code: "SNR-ENG-323",
    severity: Severity::Warning,
    summary: "The axis scale does not match its tick spacing",
    help: "A log axis read as linear, or the reverse, distorts every value (10 §3.1). Set            the axis kind in `calibration`.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Points lie outside the plot area.
pub const POINTS_OUTSIDE_PLOT: CodeInfo = CodeInfo {
    code: "SNR-ENG-324",
    severity: Severity::Warning,
    summary: "Some points lie outside the plot area",
    help: "Their values are extrapolated beyond the ticks and may belong to a legend or an            annotation rather than the data (10 §3.1).",
    rules: &[],
    status: CodeStatus::Active,
};

/// The calibration residual is larger than the accuracy target.
pub const CALIBRATION_RESIDUAL: CodeInfo = CodeInfo {
    code: "SNR-ENG-325",
    severity: Severity::Warning,
    summary: "The axis calibration is imprecise",
    help: "The ticks sit more than 0.5 % of the axis away from the fitted scale, so values            read through it carry that error. Their confidence is lowered to match.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Some tick labels could not be read as values.
pub const TICK_LABEL_UNREAD: CodeInfo = CodeInfo {
    code: "SNR-ENG-326",
    severity: Severity::Warning,
    summary: "Some tick labels could not be read",
    help: "Unread labels are skipped, never guessed; the axis is calibrated from the rest.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The file holds several charts; only the first was read.
pub const OTHER_CHARTS: CodeInfo = CodeInfo {
    code: "SNR-ENG-340",
    severity: Severity::Warning,
    summary: "Only the first of several charts was read",
    help: "A chart source reads one chart. Save the chart you want in a file of its own.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A series has neither cached values nor a readable workbook range.
pub const NO_CACHE: CodeInfo = CodeInfo {
    code: "SNR-ENG-341",
    severity: Severity::Error,
    summary: "A chart series has no stored values",
    help: "Office charts keep a cache of the values they show. Open the file in the            program that made it and save it again to rebuild the cache.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The chart's data links outside the file; the link was not followed.
pub const EXTERNAL_LINK: CodeInfo = CodeInfo {
    code: "SNR-ENG-342",
    severity: Severity::Warning,
    summary: "An external data link was not followed",
    help: "PLOTSONA never opens files or addresses a document points to (12 §6). The values            cached in the chart are used.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The chart's workbook could not be read; its cache was used.
pub const WORKBOOK_UNREADABLE: CodeInfo = CodeInfo {
    code: "SNR-ENG-343",
    severity: Severity::Warning,
    summary: "The chart's workbook could not be read",
    help: "The values cached in the chart are used instead; they are what the chart shows.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The values came from the chart's workbook.
pub const VALUES_SOURCE: CodeInfo = CodeInfo {
    code: "SNR-ENG-344",
    severity: Severity::Info,
    summary: "Values were read from the chart's workbook",
    help: "The workbook is the data the chart was built from, so it is preferred to the            chart's cache.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The chart's cache differs from its workbook.
pub const STALE_CACHE: CodeInfo = CodeInfo {
    code: "SNR-ENG-345",
    severity: Severity::Warning,
    summary: "The chart's cache differs from its workbook",
    help: "The workbook's values were used. What the chart shows may be older until the            file is opened and saved in its program.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Data outside the file (a URL) was not fetched.
pub const REMOTE_DATA: CodeInfo = CodeInfo {
    code: "SNR-ENG-346",
    severity: Severity::Error,
    summary: "Remote data was not fetched",
    help: "PLOTSONA reads only the data inside a chart file and never connects anywhere \
           (ADR-0012, OD-25). Download the data and load it as a file source, or put it in \
           the chart's JSON.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Data that only a script would produce was not computed.
pub const SCRIPT_DATA: CodeInfo = CodeInfo {
    code: "SNR-ENG-347",
    severity: Severity::Error,
    summary: "Data built by a script was not run",
    help: "Chart data made by a function, a signal or a generator needs its script, and \
           PLOTSONA runs no scripts from files (OD-25). Export the chart's data or JSON.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A data transform or encoding operation was not applied.
pub const TRANSFORM_NOT_APPLIED: CodeInfo = CodeInfo {
    code: "SNR-ENG-348",
    severity: Severity::Warning,
    summary: "A data transform was not applied",
    help: "Transforms that need an expression language (filters, formulas, time units, \
           bins) are not run, so the values are the stored data before them. Check them \
           against the chart.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Some values could not be read.
pub const VALUE_UNREADABLE: CodeInfo = CodeInfo {
    code: "SNR-ENG-349",
    severity: Severity::Warning,
    summary: "Some values could not be read",
    help: "Values in a form PLOTSONA does not read are left out as gaps, never guessed.",
    rules: &[],
    status: CodeStatus::Active,
};

/// An SVG refers to content outside the file; it was not followed.
pub const SVG_EXTERNAL_REF: CodeInfo = CodeInfo {
    code: "SNR-ENG-360",
    severity: Severity::Warning,
    summary: "References outside the SVG were not followed",
    help: "Images, styles or uses that point outside the file are never loaded (12 §6). \
           Only what the SVG draws itself is read.",
    rules: &[],
    status: CodeStatus::Active,
};

/// An SVG holds scripts; they were ignored.
pub const SVG_SCRIPT_IGNORED: CodeInfo = CodeInfo {
    code: "SNR-ENG-361",
    severity: Severity::Warning,
    summary: "Scripts in the SVG were ignored",
    help: "PLOTSONA never runs scripts from files (OD-25). A chart that a script draws \
           after loading has nothing to read; export it after it is drawn.",
    rules: &[],
    status: CodeStatus::Active,
};

/// No axes with readable tick labels were found.
pub const AXIS_NOT_FOUND: CodeInfo = CodeInfo {
    code: "SNR-ENG-362",
    severity: Severity::Error,
    summary: "No axes with tick labels were found",
    help: "Vector charts are read through their axes' tick labels (10 §3.1). If the labels \
           are drawn as outlines rather than text, enter the axes' anchors in \
           `calibration`.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Pie slices give shares, not values.
pub const PIE_SHARES: CodeInfo = CodeInfo {
    code: "SNR-ENG-363",
    severity: Severity::Info,
    summary: "Pie values are shares in percent",
    help: "A pie's geometry holds each slice's share of the whole, not the numbers behind \
           it. The shares are exact to the drawing's precision.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A PDF has no vector drawing, only images.
pub const PDF_NO_VECTOR: CodeInfo = CodeInfo {
    code: "SNR-ENG-364",
    severity: Severity::Error,
    summary: "The PDF has no vector chart",
    help: "The pages hold only images, so the chart must be read by the raster tier (C02),            as an estimate. Use the chart's source file or a vector export if you have one.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A chart-type template has no design rule behind it yet (C01-T09).
pub const EXPERIMENTAL_TEMPLATE: CodeInfo = CodeInfo {
    code: "SNR-ENG-380",
    severity: Severity::Warning,
    summary: "Experimental chart template",
    help: "No design rule covers this chart type's sound design yet (pie and stacked bar), so            it may change when a study supports one. Write `tracks` to choose the design            yourself.",
    rules: &["SDR-03"],
    status: CodeStatus::Active,
};

/// An importer left a construct out (C01-T10).
pub const IMPORT_UNMAPPED: CodeInfo = CodeInfo {
    code: "SNR-ENG-381",
    severity: Severity::Warning,
    summary: "A chart construct was not imported",
    help: "SONARITHM has no equivalent for it; the note says what to write by hand. The            pointer is in the imported document.",
    rules: &["SDR-05"],
    status: CodeStatus::Active,
};

/// An importer mapped a construct approximately (C01-T10).
pub const IMPORT_APPROXIMATED: CodeInfo = CodeInfo {
    code: "SNR-ENG-382",
    severity: Severity::Info,
    summary: "A chart construct was imported approximately",
    help: "The note says how the sound differs from the picture.",
    rules: &["SDR-05"],
    status: CodeStatus::Active,
};

/// A document is not a chart the importer reads (C01-T10).
pub const IMPORT_UNREADABLE: CodeInfo = CodeInfo {
    code: "SNR-ENG-383",
    severity: Severity::Error,
    summary: "The document cannot be imported",
    help: "Check that it is a Vega-Lite or MAIDR document of a supported chart type, or            write the spec's tracks by hand.",
    rules: &[],
    status: CodeStatus::Active,
};

/// An imported chart has no data SONARITHM can read (C01-T10).
pub const IMPORT_NO_DATA: CodeInfo = CodeInfo {
    code: "SNR-ENG-384",
    severity: Severity::Error,
    summary: "The imported chart has no readable data",
    help: "Give the chart inline values or a local data file; SONARITHM never fetches data            from the network (ADR-0012).",
    rules: &[],
    status: CodeStatus::Active,
};

/// A chart was extracted; the message is the summary sentence of 10 §5 (C01-T12).
pub const CHART_EXTRACTED: CodeInfo = CodeInfo {
    code: "SNR-ENG-385",
    severity: Severity::Info,
    summary: "Chart extracted",
    help: "The message counts the points by how they were obtained: exact, calibrated from            the drawing, or estimated. Estimated values are announced as estimates.",
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
        CALIBRATION_ANCHORS,
        LOG_NOT_POSITIVE,
        TICKS_NOT_MONOTONIC,
        SCALE_MISMATCH,
        POINTS_OUTSIDE_PLOT,
        CALIBRATION_RESIDUAL,
        TICK_LABEL_UNREAD,
        OTHER_CHARTS,
        NO_CACHE,
        EXTERNAL_LINK,
        WORKBOOK_UNREADABLE,
        VALUES_SOURCE,
        STALE_CACHE,
        REMOTE_DATA,
        SCRIPT_DATA,
        TRANSFORM_NOT_APPLIED,
        VALUE_UNREADABLE,
        SVG_EXTERNAL_REF,
        SVG_SCRIPT_IGNORED,
        AXIS_NOT_FOUND,
        PIE_SHARES,
        PDF_NO_VECTOR,
        EXPERIMENTAL_TEMPLATE,
        IMPORT_UNMAPPED,
        IMPORT_APPROXIMATED,
        IMPORT_UNREADABLE,
        IMPORT_NO_DATA,
        CHART_EXTRACTED,
    ],
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid PLOTSONA code table");

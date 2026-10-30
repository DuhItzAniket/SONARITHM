// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! PLOTSONA, SONARITHM's chart ingestion engine (07 PLOTSONA; C01, C02).
//!
//! PLOTSONA reads charts **exact first** (10 §1): it takes the data a chart was built from
//! when the file carries it (Office chart caches, chart JSON), calibrated vector geometry
//! when it does not (SVG, vector PDF), and estimates from pixels only as a last resort
//! (C02). Every value it returns carries its provenance class and confidence (03 §4,
//! AGENTS.md rule 7), and every file is untrusted input read under [`Budgets`] (12 §6).
//! It never runs a script from a file and never fetches anything (OD-25, ADR-0012).
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`detect`](mod@detect) | Container sniffing from the bytes: XLSX, PPTX, DOCX, chart JSON, HTML, SVG, PDF, PNG, JPEG; the tier of each |
//! | [`budget`] | Size, decompression, ratio, depth, node, point and work budgets |
//! | [`calibrate`] | Axis calibration for the vector and raster tiers: linear, log and date fits from anchors or tick labels, residuals, the sanity checks of 10 §3.1, category axes |
//! | [`model`] | The extracted chart (series, points, provenance) and its long table |
//! | `chartjson` | Tier 1: Vega-Lite, Vega, Plotly and Highcharts JSON, also from a local HTML page's scripts, read statically (`exact`) |
//! | `office` | Tier 1: XLSX, PPTX and DOCX chart parts, from the chart's workbook or its caches (`exact`) |
//! | `svg` | Tier 2: SVG geometry calibrated through its axes' tick labels and lines (`vector`, with the residual) |
//! | `pdf` | Tier 2: vector PDF pages (own object reader, content-stream interpreter) read like SVG (`vector`) |
//! | [`report`] | The extraction report: container, tier, series, provenance mix, diagnostics |
//! | engine | The `chart` source: a spec's `{"kind": "chart", "path": …}` |
//!
//! Like every engine it depends only on core, spec and data (02 §3), and it builds for
//! `wasm32-unknown-unknown`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg(test)]
mod accuracy;
pub mod budget;
pub mod calibrate;
mod chartjson;
pub mod codes;
mod container;
#[cfg(test)]
mod corpus;
pub mod corrections;
pub mod detect;
pub mod import;
pub mod model;
mod office;
mod pdf;
pub mod raster;
pub mod report;
mod svg;
pub mod templates;
mod vector;
mod xml;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use sonarithm_core::engine::{
    CompileContext, CompileError, Engine, EngineManifest, Registrar, RegistryError, Source,
    SourceKind, SpecFragment, TemplateProvider,
};
use sonarithm_core::{Diagnostic, EngineId, EngineVersion, TableId};
use sonarithm_data::bundle::SourceKind as DataSourceKind;
use sonarithm_data::{BundleId, DataBundle, SourceDescriptor};
use sonarithm_spec::data::Extractor;
use sonarithm_spec::v02::Calibration as SpecCalibration;

pub use budget::{Budgets, Work};
pub use corrections::{CorrectionRecord, CorrectionSet};
pub use detect::{Container, Tier, detect};
pub use model::{ChartData, ChartType, Point, Series, XValue};
pub use report::{ClassCounts, ExtractionReport, SeriesReport};

use crate::budget::{Budget, len64, over};

/// The engine ID.
pub const ENGINE: EngineId = EngineId::from_static("plotsona");

/// The table ID of the extracted chart in the bundle.
pub const CHART_TABLE: TableId = TableId(0);

/// How to extract a chart.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ExtractOptions {
    /// The input budgets.
    pub budgets: Budgets,
    /// The extractor the spec asks for; `auto` picks one from the container.
    pub extractor: Extractor,
    /// The bundle and table IDs of the result; a host that keeps several datasets gives
    /// each its own (default `BundleId(0)` and [`CHART_TABLE`]).
    pub ids: (BundleId, TableId),
    /// The axis calibration the user entered (`data.source.calibration`), which the raster
    /// tier needs.
    pub calibration: Option<SpecCalibration>,
    /// Corrections to apply after extraction (C02-T06): inline edits or a correction file.
    pub corrections: Option<CorrectionSet>,
    /// The series names the chart's legend lists, when the user gives them; the raster tier
    /// checks its series count against them (10 §3.1). Not yet in the spec.
    pub legend: Option<Vec<String>>,
}

impl ExtractOptions {
    /// Default budgets and the `auto` extractor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            budgets: Budgets::DEFAULT,
            extractor: Extractor::Auto,
            ids: (BundleId(0), CHART_TABLE),
            calibration: None,
            corrections: None,
            legend: None,
        }
    }

    /// The same options with other bundle and table IDs.
    #[must_use]
    pub const fn with_ids(mut self, bundle: BundleId, table: TableId) -> Self {
        self.ids = (bundle, table);
        self
    }

    /// The same options with other budgets.
    #[must_use]
    pub const fn with_budgets(mut self, budgets: Budgets) -> Self {
        self.budgets = budgets;
        self
    }

    /// The same options with a named extractor.
    #[must_use]
    pub const fn with_extractor(mut self, extractor: Extractor) -> Self {
        self.extractor = extractor;
        self
    }

    /// The same options with corrections to apply after extraction.
    #[must_use]
    pub fn with_corrections(mut self, corrections: CorrectionSet) -> Self {
        self.corrections = Some(corrections);
        self
    }

    /// The same options with the series names of the chart's legend.
    #[must_use]
    pub fn with_legend(mut self, legend: Vec<String>) -> Self {
        self.legend = Some(legend);
        self
    }

    /// The same options with an axis calibration.
    #[must_use]
    pub fn with_calibration(mut self, calibration: SpecCalibration) -> Self {
        self.calibration = Some(calibration);
        self
    }
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// An extracted chart: its data, the bundle the compiler reads and the report.
#[derive(Clone, Debug)]
pub struct Extraction {
    /// The chart as extracted.
    pub chart: ChartData,
    /// The chart's long table in a bundle (see [`model`]).
    pub bundle: DataBundle,
    /// What was read and how.
    pub report: ExtractionReport,
}

/// Extracts the chart in `bytes`. `locator` names the file in reports; only its last
/// component is kept (P8).
///
/// # Errors
///
/// A diagnostic with a registered `SNR-ENG-3xx` code when the file is not a chart PLOTSONA
/// reads, exceeds a budget, is damaged or holds no data.
pub fn extract(
    bytes: &[u8],
    locator: &str,
    options: &ExtractOptions,
) -> Result<Extraction, Box<Diagnostic>> {
    let budgets = &options.budgets;
    let mut work = Work::new(budgets.max_work);
    let sniffed = detect::sniff(bytes, budgets, &mut work)?;
    let container = sniffed.container;
    if !container.fits(options.extractor) {
        return Err(Box::new(Diagnostic::new(
            &codes::EXTRACTOR_MISMATCH,
            format!(
                "The file is a {}, which the `{}` extractor does not read.",
                container.name(),
                extractor_name(options.extractor)
            ),
        )));
    }
    if options.extractor == Extractor::Ml {
        return Err(not_available(container));
    }
    let mut diagnostics = Vec::new();
    let (mut chart, extractor) = read_chart(&sniffed, bytes, options, &mut work, &mut diagnostics)?;
    let history = options.corrections.as_ref().map_or_else(Vec::new, |set| {
        corrections::apply(&mut chart, set, bytes, extractor, &mut diagnostics)
    });
    let mut extraction = finish(
        locator,
        container,
        extractor,
        chart,
        diagnostics,
        budgets,
        options.ids,
    )?;
    extraction.report.corrections = history;
    Ok(extraction)
}

/// Runs the extractor of the sniffed container and names it.
///
/// Before a tier's extractor exists, the container is still parsed under the budgets, so a
/// damaged or hostile file fails with its own code rather than as "not available".
fn read_chart(
    sniffed: &detect::Sniffed,
    bytes: &[u8],
    options: &ExtractOptions,
    work: &mut Work,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(ChartData, &'static str), Box<Diagnostic>> {
    let container = sniffed.container;
    let budgets = &options.budgets;
    match container {
        Container::Xlsx | Container::Pptx | Container::Docx => {
            let is_xlsx = container == Container::Xlsx;
            let chart = office::extract(bytes, is_xlsx, budgets, work, diagnostics)?;
            Ok((chart, "sonarithm-plotsona/office"))
        }
        Container::VegaLite | Container::Vega | Container::Plotly | Container::Highcharts => {
            let json = sniffed
                .json
                .as_ref()
                .ok_or_else(|| not_available(container))?;
            let mut cx = chartjson::Cx::new(budgets, work, diagnostics);
            let chart = chartjson::extract(container, json, &mut cx)?;
            Ok((chart, json_extractor(container)))
        }
        Container::Html => {
            let mut cx = chartjson::Cx::new(budgets, work, diagnostics);
            let (found, json) = chartjson::html::find_chart(bytes, &mut cx)?;
            let chart = chartjson::extract(found, &json, &mut cx)?;
            Ok((chart, json_extractor(found)))
        }
        Container::Svg => {
            let chart = svg::extract(bytes, budgets, work, diagnostics)?;
            Ok((chart, "sonarithm-plotsona/svg"))
        }
        Container::Pdf => {
            let chart = pdf::extract(bytes, budgets, work, diagnostics)?;
            Ok((chart, "sonarithm-plotsona/pdf-vector"))
        }
        Container::Png | Container::Jpeg => {
            // Decoded under the budgets first, so a hostile image fails as one even without
            // a calibration.
            let format = if container == Container::Png {
                image::ImageFormat::Png
            } else {
                image::ImageFormat::Jpeg
            };
            let raster = raster::decode(bytes, format, budgets, work)?;
            let Some(calibration) = &options.calibration else {
                return Err(Box::new(Diagnostic::new(
                    &codes::RASTER_NEEDS_CALIBRATION,
                    format!(
                        "A {} chart is read through an axis calibration: add `data.source.calibration` with two anchors on each axis.",
                        container.name()
                    ),
                )));
            };
            let chart = raster::extract::read_raster(
                &raster,
                calibration,
                options.legend.as_deref(),
                work,
                diagnostics,
            )?;
            Ok((chart, "sonarithm-plotsona/raster"))
        }
    }
}

/// The extractor name of a chart-JSON library.
fn json_extractor(container: Container) -> &'static str {
    match container {
        Container::Vega => "sonarithm-plotsona/vega",
        Container::Plotly => "sonarithm-plotsona/plotly",
        Container::Highcharts => "sonarithm-plotsona/highcharts",
        _ => "sonarithm-plotsona/vega-lite",
    }
}

/// Checks the result and builds the bundle and report.
fn finish(
    locator: &str,
    container: Container,
    extractor: &str,
    chart: ChartData,
    diagnostics: Vec<Diagnostic>,
    budgets: &Budgets,
    ids: (BundleId, TableId),
) -> Result<Extraction, Box<Diagnostic>> {
    let points = len64(chart.points());
    if points > budgets.max_points {
        return Err(over(Budget::Points, points, budgets.max_points));
    }
    if points == 0 {
        return Err(Box::new(Diagnostic::new(
            &codes::NO_CHART,
            format!("The {} holds no chart data.", container.name()),
        )));
    }
    let report = ExtractionReport::new(locator, container, extractor, &chart, diagnostics);
    let (bundle_id, table_id) = ids;
    let table = chart
        .to_table(table_id, "chart")
        .map_err(|err| Box::new(err.to_diagnostic()))?;
    let mut source = SourceDescriptor::new(DataSourceKind::Chart, locator, "chart");
    extractor.clone_into(&mut source.extractor);
    env!("CARGO_PKG_VERSION").clone_into(&mut source.extractor_version);
    Ok(Extraction {
        bundle: DataBundle::single(bundle_id, source, table),
        chart,
        report,
    })
}

fn not_available(container: Container) -> Box<Diagnostic> {
    Box::new(Diagnostic::new(
        &codes::TIER_NOT_AVAILABLE,
        format!(
            "This build cannot extract data from a {} yet.",
            container.name()
        ),
    ))
}

fn extractor_name(extractor: Extractor) -> &'static str {
    match extractor {
        Extractor::Auto => "auto",
        Extractor::Office => "office",
        Extractor::Svg => "svg",
        Extractor::Vega => "vega",
        Extractor::Plotly => "plotly",
        Extractor::PdfVector => "pdf-vector",
        Extractor::Raster => "raster",
        Extractor::Ml => "ml",
        _ => "unknown",
    }
}

/// The PLOTSONA engine. It registers the `chart` source, which reads chart files relative
/// to a base directory (the spec's directory, P8).
#[derive(Clone, Debug, Default)]
pub struct Plotsona {
    base: Option<PathBuf>,
    budgets: Budgets,
}

impl Plotsona {
    /// The engine without file access: a `chart` source fails with `SNR-ENG-312`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The engine reading chart files relative to `base`.
    #[must_use]
    pub fn with_base_dir(base: impl Into<PathBuf>) -> Self {
        Self {
            base: Some(base.into()),
            budgets: Budgets::DEFAULT,
        }
    }

    /// The same engine with other input budgets.
    #[must_use]
    pub fn with_budgets(mut self, budgets: Budgets) -> Self {
        self.budgets = budgets;
        self
    }

    /// The engine's manifest.
    #[must_use]
    pub fn manifest() -> EngineManifest {
        let mut manifest = EngineManifest::new(
            ENGINE,
            "PLOTSONA",
            EngineVersion(env!("CARGO_PKG_VERSION").to_owned()),
        );
        manifest.spec_keys = vec!["data.source.kind=chart".to_owned()];
        manifest
    }
}

impl Engine<DataBundle> for Plotsona {
    fn manifest(&self) -> EngineManifest {
        Self::manifest()
    }

    fn register(&self, registrar: &mut Registrar<'_, DataBundle>) -> Result<(), RegistryError> {
        registrar.source(
            SourceKind::Chart,
            Arc::new(ChartSource {
                base: self.base.clone(),
                budgets: self.budgets,
            }),
        )?;
        let templates: Arc<dyn TemplateProvider<DataBundle>> = Arc::new(templates::ChartTemplates);
        for template in &templates::TEMPLATES {
            registrar.template(template.id, Arc::clone(&templates))?;
        }
        Ok(())
    }
}

/// The `chart` source.
struct ChartSource {
    base: Option<PathBuf>,
    budgets: Budgets,
}

impl Source<DataBundle> for ChartSource {
    fn ingest(
        &self,
        source: &SpecFragment,
        cx: &mut CompileContext,
    ) -> Result<DataBundle, CompileError> {
        let parsed: sonarithm_spec::Source = source.parse()?;
        let sonarithm_spec::Source::Chart {
            path,
            extractor,
            calibration,
            corrections,
            ..
        } = parsed
        else {
            return Err(CompileError::Engine {
                engine: ENGINE,
                message: "the chart source was given another kind of source".to_owned(),
            });
        };
        let at = source.pointer().key("path");
        let bytes = self
            .read(&path)
            .map_err(|d| CompileError::from(d.at(at.clone())))?;
        let mut options = ExtractOptions::new()
            .with_budgets(self.budgets)
            .with_extractor(extractor.unwrap_or(Extractor::Auto));
        if let Some(calibration) = &calibration {
            let typed = sonarithm_spec::v02::read(calibration).map_err(|err| {
                let diagnostic = Diagnostic::new(
                    &codes::CALIBRATION_ANCHORS,
                    format!("The calibration cannot be read: {err}."),
                );
                CompileError::from(diagnostic.at(source.pointer().key("calibration")))
            })?;
            options = options.with_calibration(typed);
        }
        if let Some(corrections) = &corrections {
            let at = source.pointer().key("corrections");
            let set = self
                .corrections(corrections)
                .map_err(|d| CompileError::from(d.at(at)))?;
            options = options.with_corrections(set);
        }
        let extraction =
            extract(&bytes, &path, &options).map_err(|d| CompileError::from(d.at(at.clone())))?;
        for diagnostic in extraction.report.diagnostics {
            cx.report(diagnostic.at(at.clone()));
        }
        Ok(extraction.bundle)
    }
}

impl ChartSource {
    /// The corrections of a chart source: inline edits, or the correction file
    /// `corrections.path` names (relative to the base directory, RFC-0005 §7).
    fn corrections(
        &self,
        value: &sonarithm_spec::JsonValue,
    ) -> Result<CorrectionSet, Box<Diagnostic>> {
        let unreadable = |reason: String| {
            Box::new(Diagnostic::new(
                &codes::FILE_UNREADABLE,
                format!("The corrections cannot be read: {reason}."),
            ))
        };
        let json = sonarithm_spec::v02::to_json(value);
        let Some(path) = json.get("path").and_then(serde_json::Value::as_str) else {
            let inline =
                sonarithm_spec::v02::read(value).map_err(|err| unreadable(err.to_string()))?;
            return Ok(CorrectionSet::Inline(inline));
        };
        let safe = !path.is_empty()
            && !path.contains(':')
            && !path.starts_with(['/', '\\'])
            && !path.split(['/', '\\']).any(|part| part == "..");
        if !safe {
            return Err(unreadable(format!(
                "`{path}` is not a path relative to the spec"
            )));
        }
        let bytes = self.read(path)?;
        let (file, _) = sonarithm_spec::correction_file::read(&bytes).map_err(|errors| {
            let first = errors
                .iter()
                .next()
                .map_or_else(String::new, |d| d.message.clone());
            unreadable(first)
        })?;
        Ok(CorrectionSet::File(Box::new(file)))
    }

    /// Reads the chart file, checking its size before reading it.
    fn read(&self, path: &str) -> Result<Vec<u8>, Box<Diagnostic>> {
        let unreadable = |reason: String| {
            Box::new(Diagnostic::new(
                &codes::FILE_UNREADABLE,
                format!("The chart `{path}` cannot be read: {reason}."),
            ))
        };
        let Some(base) = &self.base else {
            return Err(unreadable(
                "this host reads no chart files from disk".to_owned(),
            ));
        };
        let full = base.join(Path::new(path));
        let size = std::fs::metadata(&full)
            .map_err(|err| unreadable(err.to_string()))?
            .len();
        self.budgets.check_input(size)?;
        std::fs::read(&full).map_err(|err| unreadable(err.to_string()))
    }
}

/// Entry points for the fuzz targets, which reach one parser without its container. Not a
/// stable API.
#[doc(hidden)]
pub mod fuzzing {
    use sonarithm_core::Diagnostic;

    use crate::{Budgets, ChartData, Container, Work, chartjson, detect, office, pdf, svg};

    /// Reads `xml` as an Office chart part from its caches, as the Office extractor does.
    ///
    /// # Errors
    ///
    /// The extractor's diagnostic.
    pub fn office_chart_part(xml: &[u8], budgets: &Budgets) -> Result<ChartData, Box<Diagnostic>> {
        let mut work = Work::new(budgets.max_work);
        office::chart_part(xml, budgets, &mut work, &mut Vec::new())
    }

    /// Reads `json` as a chart of one library, chosen by `library` modulo 4 (Vega-Lite,
    /// Vega, Plotly, Highcharts), whatever its keys say, so every reader sees every input.
    ///
    /// # Errors
    ///
    /// The extractor's diagnostic.
    pub fn chart_json(
        library: u8,
        json: &[u8],
        budgets: &Budgets,
    ) -> Result<ChartData, Box<Diagnostic>> {
        let mut work = Work::new(budgets.max_work);
        let value = detect::parse_json(json, budgets, &mut work)?;
        let container = match library % 4 {
            0 => Container::VegaLite,
            1 => Container::Vega,
            2 => Container::Plotly,
            _ => Container::Highcharts,
        };
        let mut diagnostics = Vec::new();
        let mut cx = chartjson::Cx::new(budgets, &mut work, &mut diagnostics);
        chartjson::extract(container, &value, &mut cx)
    }

    /// Decodes `bytes` as a PNG (`png`) or JPEG, finds its axes and reads its marks through a
    /// calibration spanning the image: the raster tier.
    ///
    /// # Errors
    ///
    /// The raster tier's diagnostic.
    pub fn raster_image(
        png: bool,
        bytes: &[u8],
        budgets: &Budgets,
    ) -> Result<ChartData, Box<Diagnostic>> {
        let mut work = Work::new(budgets.max_work);
        let format = if png {
            image::ImageFormat::Png
        } else {
            image::ImageFormat::Jpeg
        };
        let raster = crate::raster::decode(bytes, format, budgets, &mut work)?;
        crate::raster::find_axes(&raster, &mut work)?;
        let (w, h) = (f64::from(raster.width()), f64::from(raster.height()));
        let calibration = serde_json::json!({ "axes": {
            "x": { "kind": "linear", "anchors": [{ "at": [0.0, 0.0], "value": 0.0 }, { "at": [w, 0.0], "value": 1.0 }] },
            "y": { "kind": "linear", "anchors": [{ "at": [0.0, h], "value": 0.0 }, { "at": [0.0, 0.0], "value": 1.0 }] },
        }});
        let calibration = serde_json::from_value(calibration).map_err(|err| {
            Box::new(Diagnostic::new(
                &crate::codes::CALIBRATION_ANCHORS,
                err.to_string(),
            ))
        })?;
        crate::raster::extract::read_raster(&raster, &calibration, None, &mut work, &mut Vec::new())
    }

    /// Reads `bytes` as an SVG chart: scene, axes, calibration and marks.
    ///
    /// # Errors
    ///
    /// The extractor's diagnostic.
    pub fn svg_chart(bytes: &[u8], budgets: &Budgets) -> Result<ChartData, Box<Diagnostic>> {
        let mut work = Work::new(budgets.max_work);
        svg::extract(bytes, budgets, &mut work, &mut Vec::new())
    }

    /// Reads `bytes` as a PDF: objects, page content, axes, calibration and marks.
    ///
    /// # Errors
    ///
    /// The extractor's diagnostic.
    pub fn pdf_chart(bytes: &[u8], budgets: &Budgets) -> Result<ChartData, Box<Diagnostic>> {
        let mut work = Work::new(budgets.max_work);
        pdf::extract(bytes, budgets, &mut work, &mut Vec::new())
    }

    /// Reads `html` as a page whose scripts hold a chart.
    ///
    /// # Errors
    ///
    /// The extractor's diagnostic.
    pub fn html_page(html: &[u8], budgets: &Budgets) -> Result<ChartData, Box<Diagnostic>> {
        let mut work = Work::new(budgets.max_work);
        let mut diagnostics = Vec::new();
        let mut cx = chartjson::Cx::new(budgets, &mut work, &mut diagnostics);
        let (container, json) = chartjson::html::find_chart(html, &mut cx)?;
        chartjson::extract(container, &json, &mut cx)
    }
}

#[cfg(test)]
mod tests;

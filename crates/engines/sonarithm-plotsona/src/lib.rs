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
//! | [`model`] | The extracted chart (series, points, provenance) and its long table |
//! | [`report`] | The extraction report: container, tier, series, provenance mix, diagnostics |
//! | engine | The `chart` source: a spec's `{"kind": "chart", "path": …}` |
//!
//! Like every engine it depends only on core, spec and data (02 §3), and it builds for
//! `wasm32-unknown-unknown`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod budget;
pub mod codes;
mod container;
pub mod detect;
pub mod model;
pub mod report;
mod xml;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use sonarithm_core::engine::{
    CompileContext, CompileError, Engine, EngineManifest, Registrar, RegistryError, Source,
    SourceKind, SpecFragment,
};
use sonarithm_core::{Diagnostic, EngineId, EngineVersion, TableId};
use sonarithm_data::bundle::SourceKind as DataSourceKind;
use sonarithm_data::{BundleId, DataBundle, SourceDescriptor};
use sonarithm_spec::data::Extractor;

pub use budget::{Budgets, Work};
pub use detect::{Container, Tier, detect};
pub use model::{ChartData, ChartType, Point, Series, XValue};
pub use report::{ExtractionReport, SeriesReport};

use crate::budget::{Budget, len64, over};
use crate::container::Package;

/// The engine ID.
pub const ENGINE: EngineId = EngineId::from_static("plotsona");

/// The table ID of the extracted chart in the bundle.
pub const CHART_TABLE: TableId = TableId(0);

/// How to extract a chart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExtractOptions {
    /// The input budgets.
    pub budgets: Budgets,
    /// The extractor the spec asks for; `auto` picks one from the container.
    pub extractor: Extractor,
}

impl ExtractOptions {
    /// Default budgets and the `auto` extractor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            budgets: Budgets::DEFAULT,
            extractor: Extractor::Auto,
        }
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
    if matches!(options.extractor, Extractor::Raster | Extractor::Ml) {
        return Err(not_available(container));
    }
    let mut diagnostics = Vec::new();
    let (chart, extractor) = read_chart(&sniffed, bytes, budgets, &mut work, &mut diagnostics)?;
    finish(locator, container, extractor, chart, diagnostics, budgets)
}

/// Runs the extractor of the sniffed container and names it.
///
/// Before a tier's extractor exists, the container is still opened and parsed under the
/// budgets, so a damaged or hostile file fails with its own code rather than as "not
/// available".
fn read_chart(
    sniffed: &detect::Sniffed,
    bytes: &[u8],
    budgets: &Budgets,
    work: &mut Work,
    _diagnostics: &mut Vec<Diagnostic>,
) -> Result<(ChartData, &'static str), Box<Diagnostic>> {
    let container = sniffed.container;
    match container {
        Container::Xlsx | Container::Pptx | Container::Docx => {
            let mut package = Package::open(bytes, budgets)?;
            let parts = chart_parts(&package.names());
            if parts.is_empty() {
                return Err(Box::new(Diagnostic::new(
                    &codes::NO_CHART,
                    format!("The {} has no charts.", container.name()),
                )));
            }
            for part in parts {
                let xml = package.read(&part, work)?.unwrap_or_default();
                xml::parse(&xml, "chart part", budgets, work)?;
            }
        }
        Container::Svg => {
            xml::parse(bytes, "SVG image", budgets, work)?;
        }
        _ => {}
    }
    Err(not_available(container))
}

/// The chart parts of an Office package (`xl/charts/chart1.xml`, `ppt/charts/…`,
/// `word/charts/…`), in name order.
fn chart_parts(names: &[String]) -> Vec<String> {
    names
        .iter()
        .filter(|name| {
            ["xl/charts/chart", "ppt/charts/chart", "word/charts/chart"]
                .iter()
                .find_map(|prefix| name.strip_prefix(prefix))
                .and_then(|rest| rest.strip_suffix(".xml"))
                .is_some_and(|number| {
                    !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit())
                })
        })
        .cloned()
        .collect()
}

/// Checks the result and builds the bundle and report.
fn finish(
    locator: &str,
    container: Container,
    extractor: &str,
    chart: ChartData,
    diagnostics: Vec<Diagnostic>,
    budgets: &Budgets,
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
    let table = chart
        .to_table(CHART_TABLE, "chart")
        .map_err(|err| Box::new(err.to_diagnostic()))?;
    let mut source = SourceDescriptor::new(DataSourceKind::Chart, locator, "chart");
    extractor.clone_into(&mut source.extractor);
    env!("CARGO_PKG_VERSION").clone_into(&mut source.extractor_version);
    Ok(Extraction {
        bundle: DataBundle::single(BundleId(0), source, table),
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
        )
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
            path, extractor, ..
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
        let options = ExtractOptions::new()
            .with_budgets(self.budgets)
            .with_extractor(extractor.unwrap_or(Extractor::Auto));
        let extraction =
            extract(&bytes, &path, &options).map_err(|d| CompileError::from(d.at(at.clone())))?;
        for diagnostic in extraction.report.diagnostics {
            cx.report(diagnostic.at(at.clone()));
        }
        Ok(extraction.bundle)
    }
}

impl ChartSource {
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

#[cfg(test)]
mod tests;

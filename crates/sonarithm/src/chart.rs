// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Charts as data (C01-T12): extract a chart file's data with PLOTSONA, read its report and
//! the summary sentence of 10 §5, and get a spec that plays it through its chart-type
//! template.
//!
//! ```no_run
//! use sonarithm::{Sonarithm, read_chart};
//!
//! let chart = read_chart("figure.svg")?;
//! println!("{}", chart.summary()); // "36 points: 0 exact, 36 calibrated, 0 estimated, …"
//! let spec = chart.spec().expect("a line chart has a template");
//! let compiled = Sonarithm::new().compile(&spec)?;
//! # Ok::<(), sonarithm::Error>(())
//! ```
//!
//! Extraction reads files statically and never fetches anything (OD-25, ADR-0012).

use std::path::Path;

use sonarithm_core::Diagnostic;
use sonarithm_data::DataBundle;
use sonarithm_i18n::{Args, I18nError, Localizer, Number, NumberOptions};
use sonarithm_plotsona::templates::{spec_for, template_for};
use sonarithm_plotsona::{ChartData, ExtractOptions, Extraction, ExtractionReport, extract};

use crate::error::Error;
use crate::spec::Spec;

/// An extracted chart: its data, its report and how to play it.
#[derive(Clone, Debug)]
pub struct Chart {
    extraction: Extraction,
    name: String,
    base_dir: Option<std::path::PathBuf>,
}

/// Reads the chart in a file: Office (XLSX, PPTX, DOCX), Vega, Vega-Lite, Plotly or
/// Highcharts JSON, an HTML page holding one, SVG or vector PDF. The file is recognised by
/// its contents, not its name.
///
/// # Errors
///
/// [`Error::Read`] when the file cannot be read; [`Error::Chart`] with PLOTSONA's
/// diagnostic when it is not a chart PLOTSONA reads, exceeds a budget or holds no data.
#[cfg(feature = "fs")]
pub fn read_chart(path: impl AsRef<Path>) -> Result<Chart, Error> {
    let path = path.as_ref();
    let name = crate::error::file_name(path);
    let bytes = std::fs::read(path).map_err(|source| Error::Read {
        name: name.clone(),
        source,
    })?;
    let mut chart = read_chart_bytes(&bytes, &name, &ExtractOptions::new())?;
    chart.base_dir = path.parent().map(Path::to_path_buf);
    Ok(chart)
}

/// [`read_chart`] for bytes already in memory; `name` names the file in reports (its last
/// path component only, P8).
///
/// # Errors
///
/// [`Error::Chart`], as [`read_chart`].
pub fn read_chart_bytes(
    bytes: &[u8],
    name: &str,
    options: &ExtractOptions,
) -> Result<Chart, Error> {
    let name = crate::error::file_name(Path::new(name));
    let extraction = extract(bytes, &name, options).map_err(Error::Chart)?;
    Ok(Chart {
        extraction,
        name,
        base_dir: None,
    })
}

impl Chart {
    /// What was read and how: container, tier, series with their class, confidence and
    /// residual, points per class, and extraction diagnostics.
    #[must_use]
    pub fn report(&self) -> &ExtractionReport {
        &self.extraction.report
    }

    /// The chart as extracted: series, points, chart type and titles.
    #[must_use]
    pub fn data(&self) -> &ChartData {
        &self.extraction.chart
    }

    /// The chart's long table (`series`, `x`, `y` and provenance columns) in a bundle.
    #[must_use]
    pub fn bundle(&self) -> &DataBundle {
        &self.extraction.bundle
    }

    /// The bundle, taking it out of the chart.
    #[must_use]
    pub fn into_bundle(self) -> DataBundle {
        self.extraction.bundle
    }

    /// The summary sentence of 10 §5 in English, such as "146 points: 0 exact, 0
    /// calibrated, 146 estimated, median confidence 0.82".
    #[must_use]
    pub fn summary(&self) -> String {
        Localizer::new(sonarithm_i18n::SOURCE_LOCALE)
            .and_then(|l| summary_sentence(self.report(), &l))
            .unwrap_or_else(|_| {
                let c = self.report().counts();
                format!(
                    "{} points: {} exact, {} calibrated, {} estimated",
                    c.points, c.exact, c.calibrated, c.estimated
                )
            })
    }

    /// The summary sentence in `locale` (or the pseudo-locale `en-XA`).
    ///
    /// # Errors
    ///
    /// [`Error::Render`] with the catalogue's error when the locale has no catalogue.
    pub fn summary_in(&self, locale: &str) -> Result<String, Error> {
        Localizer::new(locale)
            .and_then(|l| summary_sentence(self.report(), &l))
            .map_err(|err| Error::Render(err.to_string()))
    }

    /// The chart-type template the chart's type calls for (C01-T09), if any.
    #[must_use]
    pub fn template(&self) -> Option<&'static str> {
        template_for(self.extraction.chart.chart_type)
    }

    /// A v0.2 spec that reads this chart file and plays it through [`Chart::template`],
    /// named after the chart's title; `None` for a chart type with no template. A chart
    /// read with [`read_chart`] is found again in its own directory.
    #[must_use]
    pub fn spec(&self) -> Option<Spec> {
        let value = spec_for(
            self.extraction.chart.chart_type,
            &self.name,
            self.extraction.chart.title.as_deref(),
        )?;
        let spec = Spec::from_value(&value).with_name(format!("{}.spec.json", self.name));
        Some(match &self.base_dir {
            Some(dir) => spec.with_base_dir(dir.clone()),
            None => spec,
        })
    }
}

/// The summary sentence of `report` from the catalogue (`chart-extraction-summary`, or
/// `chart-extraction-summary-confidence` when some values have a confidence).
///
/// # Errors
///
/// The catalogue's error, which a complete catalogue never gives.
pub fn summary_sentence(
    report: &ExtractionReport,
    localizer: &Localizer,
) -> Result<String, I18nError> {
    let counts = report.counts();
    #[expect(
        clippy::cast_precision_loss,
        reason = "point counts are bounded by the point budget, far below 2^52"
    )]
    let count = |n: usize| n as f64;
    let mut args = Args::new()
        .number("points", count(counts.points))
        .number("exact", count(counts.exact))
        .number("calibrated", count(counts.calibrated))
        .number("estimated", count(counts.estimated));
    let id = match counts.median_confidence {
        Some(confidence) => {
            args = args.number_with(
                "confidence",
                Number::with_options(confidence, NumberOptions::fixed(2)),
            );
            "chart-extraction-summary-confidence"
        }
        None => "chart-extraction-summary",
    };
    localizer.format(id, &args)
}

/// The summary as an `Info` diagnostic (`SNR-ENG-385`), for hosts that report a chart's
/// extraction with their other diagnostics.
#[must_use]
pub fn summary_diagnostic(chart: &Chart) -> Diagnostic {
    Diagnostic::new(
        &sonarithm_plotsona::codes::CHART_EXTRACTED,
        format!("{}: {}.", chart.name, chart.summary()),
    )
}

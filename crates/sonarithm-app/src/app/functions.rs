// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Functions in the app core (C03-T08; ADR-0017 "Functions").
//!
//! `function_check` reads an expression without sampling it, so an expression field can show
//! diagnostics while the user types. `function_open` samples a function and opens it as a
//! dataset, so the table, the plot, the cursor and the cues treat it like any data: a spec
//! reads it as `{"kind": "dataset", "id": "dN"}`, and the critical-points analyzer finds the
//! function in the table's own metadata. `audify_calculate` is the SDR-31 calculator over an
//! open dataset, on the same grid as the `audify` encoder.

use std::collections::BTreeMap;

use sonarithm::FunctionSpec;
use sonarithm_core::TableId;
use sonarithm_data::BundleId;

use super::{App, Open, summarize, unknown_dataset};
use crate::codes;
use crate::ipc::{CriticalPointInfo, DiagnosticInfo, FunctionInfo, IpcError, Reply, Request};

/// The longest expression the protocol accepts, in bytes: four times the parser's 1024
/// characters, so any expression the parser would read fits, and a pasted megabyte is
/// refused before it is read (`SNR-APP-088`).
pub(crate) const MAX_EXPR_BYTES: usize = 4096;

/// The most variables the protocol accepts: four times the parser's 16, refused before
/// parsing (`SNR-APP-088`).
pub(crate) const MAX_VARIABLES: usize = 64;

/// Refuses input too large to read at all (`SNR-APP-088`).
fn check_size(expr: &str, variables: &BTreeMap<String, f64>) -> Result<(), IpcError> {
    if expr.len() > MAX_EXPR_BYTES {
        return Err(IpcError::new(
            &codes::FUNCTION_INPUT_TOO_LARGE,
            format!(
                "The expression is {} bytes long, more than the {MAX_EXPR_BYTES} the app reads.",
                expr.len()
            ),
        ));
    }
    if variables.len() > MAX_VARIABLES {
        return Err(IpcError::new(
            &codes::FUNCTION_INPUT_TOO_LARGE,
            format!(
                "The function names {} variables, more than the {MAX_VARIABLES} the app reads.",
                variables.len()
            ),
        ));
    }
    Ok(())
}

/// The arguments of `function_open`.
struct FunctionRequest {
    expr: String,
    domain: [f64; 2],
    samples: Option<u32>,
    step: Option<f64>,
    variables: BTreeMap<String, f64>,
    name: Option<String>,
}

impl App {
    /// The function commands (ADR-0017 "Functions").
    pub(super) fn functions(&mut self, request: Request) -> Result<Reply, IpcError> {
        match request {
            Request::FunctionCheck { expr, variables } => Self::function_check(&expr, &variables),
            Request::FunctionOpen {
                expr,
                domain,
                samples,
                step,
                variables,
                name,
            } => self.edit(|app| {
                app.function_open(FunctionRequest {
                    expr,
                    domain,
                    samples,
                    step,
                    variables,
                    name,
                })
            }),
            Request::AudifyCalculate {
                dataset,
                time,
                k,
                top_hz,
            } => self.audify_calculate(&dataset, time.as_deref(), k, top_hz),
            other => Err(IpcError::new(
                &codes::BAD_REQUEST,
                format!("{} is not a function command.", other.name()),
            )),
        }
    }

    /// `function_check`: the canonical form, or the diagnostic and where reading stopped.
    fn function_check(expr: &str, variables: &BTreeMap<String, f64>) -> Result<Reply, IpcError> {
        check_size(expr, variables)?;
        let check = sonarithm::check_expression(expr, variables);
        let diagnostics: Vec<DiagnosticInfo> =
            check.diagnostic.iter().map(DiagnosticInfo::from).collect();
        Ok(Reply::FunctionCheck {
            ok: check.canonical.is_some(),
            canonical: check.canonical,
            span: check.span.map(|(at, len)| {
                [
                    u32::try_from(at).unwrap_or(u32::MAX),
                    u32::try_from(len).unwrap_or(u32::MAX),
                ]
            }),
            diagnostics,
        })
    }

    /// `function_open`: samples the function and opens it as the next dataset.
    fn function_open(&mut self, request: FunctionRequest) -> Result<Reply, IpcError> {
        check_size(&request.expr, &request.variables)?;
        let mut source = FunctionSpec::new(request.expr, request.domain);
        source.samples = request.samples;
        source.step = request.step;
        source.variables = request.variables;
        let number = self.next_dataset;
        let ids = u32::try_from(number).unwrap_or(u32::MAX);
        let function = sonarithm::read_function_with_ids(&source, BundleId(number), TableId(ids))
            .map_err(|err| {
            err.diagnostics().iter().next().map_or_else(
                || IpcError::new(&codes::BAD_REQUEST, err.to_string()),
                IpcError::from_diagnostic,
            )
        })?;
        self.next_dataset += 1;
        let id = format!("d{number}");
        let name = request
            .name
            .unwrap_or_else(|| format!("f(x) = {}", function.expr()));
        let summary = summarize(&id, &name, function.bundle(), &function.diagnostics());
        let info = FunctionInfo {
            canonical: function.expr().to_owned(),
            summary: function.summary(),
            critical_points: function
                .critical_points()
                .into_iter()
                .map(|point| CriticalPointInfo {
                    kind: point.kind.to_owned(),
                    x: point.x,
                    end: point.end,
                    y: point.y,
                    row: point.row,
                    text: point.text,
                })
                .collect(),
        };
        self.datasets.insert(
            number,
            Open {
                summary: summary.clone(),
                bundle: function.into_bundle(),
                origin: None,
                chart: None,
                function: Some(source),
            },
        );
        Ok(Reply::Function {
            dataset: summary,
            function: info,
        })
    }

    /// `audify_calculate`: the SDR-31 calculator over an open dataset.
    fn audify_calculate(
        &self,
        dataset: &str,
        time: Option<&str>,
        k: Option<f64>,
        top_hz: Option<f64>,
    ) -> Result<Reply, IpcError> {
        let open = self
            .datasets
            .values()
            .find(|open| open.summary.id == dataset)
            .ok_or_else(|| unknown_dataset(dataset))?;
        let table = open
            .bundle
            .primary()
            .ok_or_else(|| unknown_dataset(dataset))?;
        let readout = sonarithm::audify_readout(table, time, k, top_hz)
            .map_err(|diagnostic| IpcError::from_diagnostic(&diagnostic))?;
        let speedup = readout.speedup;
        Ok(Reply::AudifyReadout {
            k: speedup.k,
            fs_hz: speedup.fs,
            samples: u64::try_from(readout.samples).unwrap_or(u64::MAX),
            series: readout.series,
            duration_s: speedup.duration.0,
            band_hz: speedup.band,
            audible_hz: speedup.audible,
            codes: readout
                .codes
                .iter()
                .map(|code| (*code).to_owned())
                .collect(),
        })
    }
}

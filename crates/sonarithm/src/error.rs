// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The facade's error type.

use sonarithm_core::{Diagnostic, Diagnostics};
use sonarithm_data::DataError;
use sonarithm_numisona::templates::TemplateError;

/// Why a facade call failed.
///
/// Spec and compile problems carry every diagnostic found so far ([`Error::diagnostics`]);
/// they are the same diagnostics the compiler reports, with codes, rule IDs and JSON
/// pointers. Messages name files by their last path component only, never a directory
/// (P8, ADR-0012).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The spec is invalid or the compile failed; every diagnostic so far, warnings
    /// included.
    #[error("{}", summary(.0))]
    Spec(Diagnostics),
    /// The compile was cancelled.
    #[error("the compile was cancelled")]
    Cancelled,
    /// A data source could not be read.
    #[error(transparent)]
    Data(#[from] DataError),
    /// A spec file could not be read.
    #[error("`{name}` cannot be read: {source}")]
    Read {
        /// The file name, without directories.
        name: String,
        /// The I/O error.
        #[source]
        source: std::io::Error,
    },
    /// `quick` cannot map the data with its template: a field is missing or of the wrong
    /// kind.
    #[error(transparent)]
    Template(#[from] TemplateError),
    /// `quick` was asked to compile without data.
    #[error("quick needs data: call `data_file` or `data` first")]
    NoData,
    /// Rendering failed: an unsupported output path, or a file that cannot be written.
    #[error("the render failed: {0}")]
    Render(String),
    /// The Score could not be written in its canonical form (a non-finite number).
    #[error("the Score has no canonical form: {0}")]
    Canonical(String),
}

impl Error {
    /// The diagnostics behind this error: every diagnostic of a spec or compile failure,
    /// the data layer's diagnostic for a data error, and none otherwise.
    #[must_use]
    pub fn diagnostics(&self) -> Diagnostics {
        match self {
            Self::Spec(diagnostics) => diagnostics.clone(),
            Self::Data(err) => {
                let mut one = Diagnostics::new();
                one.push(err.to_diagnostic());
                one
            }
            _ => Diagnostics::new(),
        }
    }
}

/// "the spec has 2 errors: <first error>".
fn summary(diagnostics: &Diagnostics) -> String {
    let errors: Vec<&Diagnostic> = diagnostics.iter().filter(|d| d.is_error()).collect();
    match errors.as_slice() {
        [] => "the spec could not be compiled".to_owned(),
        [only] => format!("the spec has 1 error: {only}"),
        [first, ..] => format!("the spec has {} errors; the first: {first}", errors.len()),
    }
}

/// The last path component of `path`, for messages (P8).
pub(crate) fn file_name(path: &std::path::Path) -> String {
    path.file_name().map_or_else(
        || "the file".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    )
}

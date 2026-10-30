// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Input budgets for untrusted chart files (12 §6, 10 §2).
//!
//! Every chart file is untrusted: a crafted XLSX can be a zip bomb, an SVG can declare
//! entities that expand without end, a JSON or XML document can nest deeper than any stack.
//! PLOTSONA checks each budget before it does the work the budget guards, and an exceeded
//! budget stops the extraction with a registered code instead of exhausting memory.
//!
//! The "time" budget is a **work budget**: a count of steps (bytes inflated, nodes and
//! values visited), not a clock. A clock read would make extraction depend on the machine
//! and break determinism (ADR-0013), so the same file always fails, or succeeds, at the
//! same step.

use core::fmt;

use sonarithm_core::Diagnostic;

use crate::codes;

/// Size, work, depth and compression budgets for one extraction.
///
/// [`Budgets::DEFAULT`] fits charts far larger than any real one; hosts with less memory,
/// and the fuzz targets, lower them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Budgets {
    /// The largest input file, in bytes.
    pub max_bytes: u64,
    /// The most entries a zip container (XLSX, PPTX, DOCX) may list.
    pub max_entries: u64,
    /// The largest single decompressed part, in bytes.
    pub max_part_bytes: u64,
    /// The most bytes all decompressed parts may add up to.
    pub max_inflated_bytes: u64,
    /// The highest compression ratio a part larger than [`Budgets::RATIO_FLOOR`] may
    /// have. Office XML compresses about 10:1; a zip bomb compresses 1000:1 and more.
    pub max_ratio: u64,
    /// The deepest nesting of XML elements or JSON values.
    pub max_depth: u32,
    /// The most XML nodes in one document.
    pub max_nodes: u32,
    /// The most data points one chart may yield, over all series.
    pub max_points: u64,
    /// The most work steps one extraction may take (see the module documentation).
    pub max_work: u64,
    /// The most pixels a raster image may have (width × height), checked from its header
    /// before any pixel is decoded (C02-T02).
    pub max_pixels: u64,
}

impl Budgets {
    /// Budgets for desktop use: 256 MiB files, 1 GiB inflated, 50 million points.
    pub const DEFAULT: Self = Self {
        max_bytes: 256 << 20,
        max_entries: 20_000,
        max_part_bytes: 512 << 20,
        max_inflated_bytes: 1 << 30,
        max_ratio: 200,
        max_depth: 256,
        max_nodes: 20_000_000,
        max_points: 50_000_000,
        max_work: 4_000_000_000,
        // 50 megapixels: a 7,000 × 7,000 scan, far above a chart screenshot.
        max_pixels: 50_000_000,
    };

    /// Parts at or below this size are never refused for their compression ratio, since
    /// small runs of repeated XML compress far better than whole documents.
    pub const RATIO_FLOOR: u64 = 1 << 20;

    /// Budgets with every limit lifted except the depth, which protects the stack. For
    /// trusted files only.
    pub const UNLIMITED: Self = Self {
        max_bytes: u64::MAX,
        max_entries: u64::MAX,
        max_part_bytes: u64::MAX,
        max_inflated_bytes: u64::MAX,
        max_ratio: u64::MAX,
        max_depth: 1024,
        max_nodes: u32::MAX,
        max_points: u64::MAX,
        max_work: u64::MAX,
        max_pixels: u64::MAX,
    };

    /// Checks the size of the whole input.
    ///
    /// # Errors
    ///
    /// `SNR-ENG-301` if `bytes` is above [`Budgets::max_bytes`].
    pub fn check_input(&self, bytes: u64) -> Result<(), Box<Diagnostic>> {
        if bytes > self.max_bytes {
            return Err(over(Budget::InputBytes, bytes, self.max_bytes));
        }
        Ok(())
    }

    /// Checks a nesting depth.
    ///
    /// # Errors
    ///
    /// `SNR-ENG-304` if `depth` is above [`Budgets::max_depth`].
    pub fn check_depth(&self, depth: usize) -> Result<(), Box<Diagnostic>> {
        let max = usize::try_from(self.max_depth).unwrap_or(usize::MAX);
        if depth > max {
            return Err(Box::new(Diagnostic::new(
                &codes::NESTING_TOO_DEEP,
                format!(
                    "The file nests {depth} levels deep; the limit is {}.",
                    self.max_depth
                ),
            )));
        }
        Ok(())
    }
}

impl Default for Budgets {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Which budget an input exceeded, for messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Budget {
    InputBytes,
    Entries,
    PartBytes,
    InflatedBytes,
    Ratio,
    Nodes,
    Points,
    Pixels,
}

impl fmt::Display for Budget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InputBytes => "file size in bytes",
            Self::Entries => "number of entries in the container",
            Self::PartBytes => "decompressed size of one part in bytes",
            Self::InflatedBytes => "total decompressed size in bytes",
            Self::Ratio => "compression ratio",
            Self::Nodes => "number of XML nodes",
            Self::Points => "number of data points",
            Self::Pixels => "number of pixels",
        })
    }
}

/// The diagnostic for an exceeded budget: `SNR-ENG-301` for the file size, `SNR-ENG-302`
/// for the container and decompression budgets, `SNR-ENG-306` for the content budgets.
pub(crate) fn over(budget: Budget, value: u64, max: u64) -> Box<Diagnostic> {
    let code = match budget {
        Budget::InputBytes => &codes::INPUT_TOO_LARGE,
        Budget::Entries | Budget::PartBytes | Budget::InflatedBytes | Budget::Ratio => {
            &codes::CONTAINER_BUDGET
        }
        Budget::Nodes | Budget::Points | Budget::Pixels => &codes::CONTENT_BUDGET,
    };
    Box::new(Diagnostic::new(
        code,
        format!("The {budget} is {value}, above the limit of {max}."),
    ))
}

/// The work budget of one extraction: a step counter shared by every stage.
#[derive(Clone, Debug)]
pub struct Work {
    left: u64,
    max: u64,
}

impl Work {
    /// A counter with `max` steps.
    #[must_use]
    pub const fn new(max: u64) -> Self {
        Self { left: max, max }
    }

    /// Spends `steps` steps.
    ///
    /// # Errors
    ///
    /// `SNR-ENG-305` once the budget is spent.
    pub fn spend(&mut self, steps: u64) -> Result<(), Box<Diagnostic>> {
        if let Some(left) = self.left.checked_sub(steps) {
            self.left = left;
            Ok(())
        } else {
            self.left = 0;
            Err(Box::new(Diagnostic::new(
                &codes::WORK_BUDGET,
                format!(
                    "Extraction stopped after {} work steps, the limit for one file.",
                    self.max
                ),
            )))
        }
    }

    /// The steps spent so far.
    #[must_use]
    pub const fn spent(&self) -> u64 {
        self.max.saturating_sub(self.left)
    }
}

/// Converts a length to `u64` for budget checks (lossless on every supported target).
pub(crate) fn len64(len: usize) -> u64 {
    u64::try_from(len).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_stops_at_the_limit_and_names_its_code() {
        let mut work = Work::new(10);
        assert!(work.spend(4).is_ok());
        assert!(work.spend(6).is_ok());
        let err = work.spend(1).expect_err("spent");
        assert_eq!(err.code.as_str(), "SNR-ENG-305");
        assert_eq!(work.spent(), 10);
    }

    #[test]
    fn input_and_depth_checks() {
        let mut budgets = Budgets::DEFAULT;
        budgets.max_bytes = 100;
        budgets.max_depth = 3;
        assert!(budgets.check_input(100).is_ok());
        assert_eq!(
            budgets.check_input(101).expect_err("too big").code.as_str(),
            "SNR-ENG-301"
        );
        assert!(budgets.check_depth(3).is_ok());
        assert_eq!(
            budgets.check_depth(4).expect_err("too deep").code.as_str(),
            "SNR-ENG-304"
        );
    }
}

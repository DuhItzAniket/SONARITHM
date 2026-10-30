// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Axis calibration: from drawing positions to data values (10 §3.1, notes/06 §8).
//!
//! The SVG, PDF and raster tiers see marks as positions on a page. An axis is calibrated
//! from **anchors**, pairs of a position and the value it stands for, which come from tick
//! labels and tick marks or from the user. A [`Calibration`] is a least-squares line from
//! the scale's transform of the values (identity for linear and date axes, `log10` for log
//! axes) to positions; its **residual** is how far the anchors sit from that line, as a
//! share of the axis length, so a calibration from clean vector ticks has a residual near
//! zero and one from jittered pixels a larger one.
//!
//! The sanity checks of 10 §3.1 run before a calibration is used: ticks must be monotonic,
//! the scale must match the label spacing (a log axis read as linear, or the reverse, is
//! caught), points must lie within the plot, and the residual must be small. **A failed
//! check is a diagnostic, never a silent fix**: an error refuses the calibration, a warning
//! goes into the extraction report.
//!
//! Dates are milliseconds since 1970-01-01T00:00:00 UTC on a linear scale. Category axes
//! map a position to the nearest label instead ([`CategoryAxis`]). Every transcendental
//! function goes through `libm`, so values are identical on every platform (ADR-0013).

pub mod ticks;

use serde::Serialize;
use sonarithm_core::{Confidence, Diagnostic};

use crate::codes;

/// The residual above which a calibration is reported (`SNR-ENG-325`): 0.5 % of the axis
/// length, the C01 accuracy target for exact and vector extraction.
pub const RESIDUAL_LIMIT: f64 = 0.005;

/// How much smaller the other scale's residual must be before a scale is reported as
/// mismatched (`SNR-ENG-323`).
const MISMATCH_FACTOR: f64 = 10.0;

/// Residuals at or below this are "fits exactly" for the mismatch check, so two exact fits
/// never count as a mismatch.
const EXACT_RESIDUAL: f64 = 1e-6;

/// The kind of a continuous axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ScaleKind {
    /// Equal value steps at equal distances.
    Linear,
    /// Equal value ratios at equal distances (any base: the fit does not depend on it).
    Log,
    /// Times, in milliseconds since the Unix epoch, on a linear scale.
    Date,
}

impl ScaleKind {
    /// The scale's transform of a value, or `None` where it is undefined (log of a value
    /// that is not positive, or a value that is not finite).
    #[must_use]
    pub fn transform(self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        match self {
            Self::Linear | Self::Date => Some(value),
            Self::Log => (value > 0.0).then(|| libm::log10(value)),
        }
    }

    /// The inverse of [`ScaleKind::transform`].
    #[must_use]
    pub fn invert(self, transformed: f64) -> f64 {
        match self {
            Self::Linear | Self::Date => transformed,
            Self::Log => libm::pow(10.0, transformed),
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Log => "logarithmic",
            Self::Date => "date",
        }
    }
}

/// A position on the drawing and the value it stands for.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Anchor {
    /// The position along the axis, in drawing units (SVG user units, PDF points, pixels).
    pub position: f64,
    /// The data value (milliseconds since the epoch on a date axis).
    pub value: f64,
}

impl Anchor {
    /// An anchor.
    #[must_use]
    pub const fn new(position: f64, value: f64) -> Self {
        Self { position, value }
    }
}

/// A fitted continuous axis.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Calibration {
    kind: ScaleKind,
    /// Positions per transformed value unit.
    slope: f64,
    /// The position of transformed value 0.
    intercept: f64,
    residual: f64,
    anchors: usize,
    /// The lowest and highest anchor positions.
    extent: (f64, f64),
}

impl Calibration {
    /// Fits `kind` through `anchors` by least squares.
    ///
    /// # Errors
    ///
    /// `SNR-ENG-320` with fewer than two anchors at distinct positions and values (or a
    /// non-finite one), `SNR-ENG-321` for a log axis with a value that is not positive.
    pub fn fit(kind: ScaleKind, anchors: &[Anchor]) -> Result<Self, Box<Diagnostic>> {
        let mut points = Vec::with_capacity(anchors.len());
        for anchor in anchors {
            if !anchor.position.is_finite() || !anchor.value.is_finite() {
                return Err(too_few("an anchor is not a finite number"));
            }
            let Some(t) = kind.transform(anchor.value) else {
                return Err(Box::new(Diagnostic::new(
                    &codes::LOG_NOT_POSITIVE,
                    format!(
                        "A logarithmic axis cannot place the value {}; its values must be \
                         above zero.",
                        anchor.value
                    ),
                )));
            };
            points.push((t, anchor.position));
        }
        let n = points.len();
        #[expect(clippy::cast_precision_loss, reason = "anchor counts are tiny")]
        let count = n as f64;
        if n < 2 {
            return Err(too_few("it has fewer than two anchors"));
        }
        // Centred sums keep date values (about 1.7e12 ms) well conditioned.
        let mean_t = points.iter().map(|p| p.0).sum::<f64>() / count;
        let mean_p = points.iter().map(|p| p.1).sum::<f64>() / count;
        let (mut stt, mut stp) = (0.0, 0.0);
        for (t, p) in &points {
            stt += (t - mean_t) * (t - mean_t);
            stp += (t - mean_t) * (p - mean_p);
        }
        let (low, high) = points
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
                (lo.min(p.1), hi.max(p.1))
            });
        if stt <= 0.0 || high <= low || stp == 0.0 {
            return Err(too_few(
                "its anchors do not span two distinct values and positions",
            ));
        }
        let slope = stp / stt;
        let intercept = mean_p - slope * mean_t;
        let span = high - low;
        let squares: f64 = points
            .iter()
            .map(|(t, p)| {
                let off = (slope * t + intercept - p) / span;
                off * off
            })
            .sum();
        Ok(Self {
            kind,
            slope,
            intercept,
            residual: libm::sqrt(squares / count),
            anchors: n,
            extent: (low, high),
        })
    }

    /// Fits a linear axis, or a log axis when the anchors fit one far better (at least
    /// three positive anchors, a residual ten times smaller and above exact).
    ///
    /// # Errors
    ///
    /// As [`Calibration::fit`] for the linear fit.
    pub fn fit_auto(anchors: &[Anchor]) -> Result<Self, Box<Diagnostic>> {
        let linear = Self::fit(ScaleKind::Linear, anchors)?;
        if anchors.len() >= 3
            && let Ok(log) = Self::fit(ScaleKind::Log, anchors)
            && better(log.residual, linear.residual)
        {
            return Ok(log);
        }
        Ok(linear)
    }

    /// The scale kind.
    #[must_use]
    pub const fn kind(&self) -> ScaleKind {
        self.kind
    }

    /// The root-mean-square distance of the anchors from the fit, as a share of the span
    /// of their positions.
    #[must_use]
    pub const fn residual(&self) -> f64 {
        self.residual
    }

    /// The number of anchors fitted.
    #[must_use]
    pub const fn anchors(&self) -> usize {
        self.anchors
    }

    /// The value at a drawing position.
    #[must_use]
    pub fn value_at(&self, position: f64) -> f64 {
        self.kind.invert((position - self.intercept) / self.slope)
    }

    /// The drawing position of a value, or `None` where the scale cannot place it.
    #[must_use]
    pub fn position_of(&self, value: f64) -> Option<f64> {
        self.kind
            .transform(value)
            .map(|t| self.slope * t + self.intercept)
    }

    /// The confidence of values read through this calibration: 1 for an exact fit,
    /// falling by 0.1 for each 1 % of residual and reaching 0 at 10 %.
    ///
    /// A convention, not a calibrated probability: no study has calibrated confidence
    /// scores for chart extraction (notes/06 §8, Gaps). TODO(verify) against the C01 corpus.
    #[must_use]
    pub fn confidence(&self) -> Confidence {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a confidence in [0, 1] fits f32"
        )]
        let value = (1.0 - (self.residual * 10.0).min(1.0)).max(0.0) as f32;
        Confidence::new(value).unwrap_or(Confidence::CERTAIN)
    }

    /// The sanity checks of 10 §3.1 for a fitted axis: its residual, and whether the other
    /// scale fits the anchors far better.
    #[must_use]
    pub fn check(&self, anchors: &[Anchor], axis: &str) -> Vec<Diagnostic> {
        let mut found = Vec::new();
        if self.residual > RESIDUAL_LIMIT {
            found.push(Diagnostic::new(
                &codes::CALIBRATION_RESIDUAL,
                format!(
                    "The {axis} axis calibration is off by {:.2} % of the axis on average; \
                     values read through it are less exact.",
                    self.residual * 100.0
                ),
            ));
        }
        if let Some(diagnostic) = check_scale(self.kind, anchors, axis) {
            found.push(diagnostic);
        }
        found
    }

    /// Diagnoses positions outside the plot: further than `tolerance` (a share of the
    /// axis length) beyond `plot`, the lowest and highest positions of the plot area.
    #[must_use]
    pub fn check_bounds(
        &self,
        positions: &[f64],
        plot: (f64, f64),
        tolerance: f64,
        axis: &str,
    ) -> Option<Diagnostic> {
        let (low, high) = (plot.0.min(plot.1), plot.0.max(plot.1));
        let slack = (high - low) * tolerance;
        let outside = positions
            .iter()
            .filter(|p| **p < low - slack || **p > high + slack)
            .count();
        (outside > 0).then(|| {
            Diagnostic::new(
                &codes::POINTS_OUTSIDE_PLOT,
                format!(
                    "{outside} point(s) lie outside the plot area along the {axis} axis; \
                     their values are extrapolated from the calibration."
                ),
            )
        })
    }
}

/// Whether `candidate` fits far better than `current`.
fn better(candidate: f64, current: f64) -> bool {
    current > EXACT_RESIDUAL && candidate * MISMATCH_FACTOR < current
}

/// Checks that tick values change monotonically with their positions.
///
/// # Errors
///
/// `SNR-ENG-322` when two ticks share a position, or values rise and fall along the axis.
pub fn check_monotonic(anchors: &[Anchor], axis: &str) -> Result<(), Box<Diagnostic>> {
    let mut sorted: Vec<Anchor> = anchors.to_vec();
    sorted.sort_by(|a, b| a.position.total_cmp(&b.position));
    let mut rising: Option<bool> = None;
    for pair in sorted.windows(2) {
        let [a, b] = pair else { continue };
        let step = b.value - a.value;
        let up = step > 0.0;
        let monotonic = b.position > a.position && step != 0.0 && rising.is_none_or(|r| r == up);
        if !monotonic {
            return Err(Box::new(Diagnostic::new(
                &codes::TICKS_NOT_MONOTONIC,
                format!(
                    "The {axis} axis ticks do not change in one direction (at {} and {});                      the axis cannot be calibrated from them.",
                    a.value, b.value
                ),
            )));
        }
        rising = Some(up);
    }
    Ok(())
}

/// Diagnoses a scale that fits its anchors far worse than the other one: a log axis read
/// as linear or a linear axis read as log. Needs three anchors; date axes are not checked.
#[must_use]
pub fn check_scale(kind: ScaleKind, anchors: &[Anchor], axis: &str) -> Option<Diagnostic> {
    if anchors.len() < 3 {
        return None;
    }
    let other = match kind {
        ScaleKind::Linear => ScaleKind::Log,
        ScaleKind::Log => ScaleKind::Linear,
        ScaleKind::Date => return None,
    };
    let current = Calibration::fit(kind, anchors).ok()?;
    let alternative = Calibration::fit(other, anchors).ok()?;
    better(alternative.residual, current.residual).then(|| {
        Diagnostic::new(
            &codes::SCALE_MISMATCH,
            format!(
                "The {axis} axis was read as {} but its ticks are spaced like a {} axis; \
                 check the axis type.",
                kind.name(),
                other.name()
            ),
        )
    })
}

fn too_few(reason: &str) -> Box<Diagnostic> {
    Box::new(Diagnostic::new(
        &codes::CALIBRATION_ANCHORS,
        format!("The axis cannot be calibrated: {reason}."),
    ))
}

/// A category axis: labels at positions, read by the nearest label.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CategoryAxis {
    labels: Vec<(f64, String)>,
}

impl CategoryAxis {
    /// The axis with `labels` at their positions, sorted by position.
    #[must_use]
    pub fn new(mut labels: Vec<(f64, String)>) -> Self {
        labels.retain(|(p, _)| p.is_finite());
        labels.sort_by(|a, b| a.0.total_cmp(&b.0));
        Self { labels }
    }

    /// The labels and their positions, in position order.
    #[cfg(test)]
    pub(crate) fn entries(&self) -> &[(f64, String)] {
        &self.labels
    }

    /// The labels in position order.
    pub fn labels(&self) -> impl Iterator<Item = &str> {
        self.labels.iter().map(|(_, label)| label.as_str())
    }

    /// The label nearest to `position`, if it lies within half the distance to the
    /// neighbouring labels (or, with one label, anywhere).
    #[must_use]
    pub fn label_at(&self, position: f64) -> Option<&str> {
        let index = self
            .labels
            .iter()
            .enumerate()
            .min_by(|a, b| {
                (a.1.0 - position)
                    .abs()
                    .total_cmp(&(b.1.0 - position).abs())
            })
            .map(|(i, _)| i)?;
        let (at, label) = self.labels.get(index)?;
        let gap = |other: Option<&(f64, String)>| other.map(|(p, _)| (p - at).abs());
        let half_band = [
            gap(index.checked_sub(1).and_then(|i| self.labels.get(i))),
            gap(self.labels.get(index + 1)),
        ]
        .into_iter()
        .flatten()
        .fold(f64::INFINITY, f64::min)
            / 2.0;
        ((position - at).abs() <= half_band).then_some(label.as_str())
    }
}

#[cfg(test)]
mod tests;

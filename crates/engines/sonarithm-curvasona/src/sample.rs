// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The sample grid of a function's domain: `samples` evenly spaced points from `min` to
//! `max` inclusive (default 1000, RFC-0001 §4), or points `step` apart from `min`.
//!
//! The grid is checked against [`Budgets::max_samples`] before any point is made. Adaptive
//! sampling near high curvature and discontinuities refines this grid (C03-T02).

use sonarithm_core::Diagnostic;

use crate::{Budgets, codes};

/// The default number of samples (RFC-0001 §4).
pub const DEFAULT_SAMPLES: u32 = 1000;

/// An evenly spaced grid over a domain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    min: f64,
    max: f64,
    len: usize,
    /// The spacing for a `step` grid; `None` spreads `len` points from `min` to `max`.
    step: Option<f64>,
}

impl Grid {
    /// The grid for `domain` with `samples` or `step` (at most one is given; validation
    /// checks that, and that `min < max`).
    ///
    /// # Errors
    ///
    /// `SNR-ENG-404` when the grid has more points than [`Budgets::max_samples`], or the
    /// domain or step is not usable.
    pub fn new(
        domain: [f64; 2],
        samples: Option<u32>,
        step: Option<f64>,
        budgets: &Budgets,
    ) -> Result<Self, Box<Diagnostic>> {
        let [min, max] = domain;
        let too_many = |count: String| {
            Box::new(Diagnostic::new(
                &codes::TOO_MANY_SAMPLES,
                format!(
                    "The domain gives {count} samples, more than the limit of {}.",
                    budgets.max_samples
                ),
            ))
        };
        if !(min.is_finite() && max.is_finite() && min < max && (max - min).is_finite()) {
            return Err(Box::new(Diagnostic::new(
                &codes::TOO_MANY_SAMPLES,
                "The domain must be two finite numbers, the first below the second.",
            )));
        }
        if let Some(step) = step {
            if !(step.is_finite() && step > 0.0) {
                return Err(too_many("unboundedly many".to_owned()));
            }
            // A small allowance so that a step dividing the domain exactly reaches `max`.
            let intervals = ((max - min) / step * (1.0 + 1e-12)).floor();
            #[expect(
                clippy::cast_precision_loss,
                reason = "the sample limit is far below 2^52"
            )]
            let limit = budgets.max_samples as f64;
            if intervals.is_nan() || intervals + 1.0 > limit {
                return Err(too_many(format!("{}", intervals + 1.0)));
            }
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "checked above: a non-negative whole number below the sample limit"
            )]
            let len = intervals as usize + 1;
            return Ok(Self {
                min,
                max,
                len,
                step: Some(step),
            });
        }
        let len = usize::try_from(samples.unwrap_or(DEFAULT_SAMPLES)).unwrap_or(usize::MAX);
        if len > budgets.max_samples {
            return Err(too_many(len.to_string()));
        }
        Ok(Self {
            min,
            max,
            len: len.max(2),
            step: None,
        })
    }

    /// The number of points.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the grid has no points (never, for a valid domain).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The points in increasing order. A `samples` grid ends exactly at `max`.
    pub fn points(&self) -> impl Iterator<Item = f64> + '_ {
        (0..self.len).map(move |i| self.point(i))
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "indices are below the sample limit, far below 2^52"
    )]
    fn point(&self, i: usize) -> f64 {
        match self.step {
            Some(step) => self.min + i as f64 * step,
            None if i + 1 == self.len => self.max,
            None => self.min + (self.max - self.min) * (i as f64 / (self.len - 1) as f64),
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn samples_span_the_domain_inclusively() {
        let grid = Grid::new([-20.0, 20.0], Some(2000), None, &Budgets::DEFAULT).unwrap();
        let points: Vec<f64> = grid.points().collect();
        assert_eq!(points.len(), 2000);
        assert!((points[0] - -20.0).abs() < f64::EPSILON);
        assert!((points[1999] - 20.0).abs() < f64::EPSILON);
        assert_eq!(
            Grid::new([0.0, 1.0], None, None, &Budgets::DEFAULT)
                .unwrap()
                .len(),
            1000
        );
    }

    #[test]
    fn a_step_that_divides_the_domain_reaches_its_end() {
        let grid = Grid::new([0.0, 1.0], None, Some(0.1), &Budgets::DEFAULT).unwrap();
        let points: Vec<f64> = grid.points().collect();
        assert_eq!(points.len(), 11);
        assert!((points[10] - 1.0).abs() < 1e-12);
        let ragged = Grid::new([0.0, 1.0], None, Some(0.3), &Budgets::DEFAULT).unwrap();
        assert_eq!(ragged.len(), 4);
    }

    #[test]
    fn snr_eng_404_the_sample_budget_holds() {
        for (samples, step) in [(Some(100_001), None), (None, Some(1e-9))] {
            let error = Grid::new([0.0, 1.0], samples, step, &Budgets::DEFAULT).unwrap_err();
            assert_eq!(error.code.as_str(), "SNR-ENG-404");
        }
        let huge = Grid::new([-f64::MAX, f64::MAX], None, Some(1.0), &Budgets::DEFAULT);
        assert_eq!(huge.unwrap_err().code.as_str(), "SNR-ENG-404");
    }

    proptest! {
        /// Grids are increasing, inside the domain, within budget and the same every time.
        #[test]
        fn grids_are_ordered_bounded_and_deterministic(
            min in -1e3..1e3_f64,
            width in 1e-3..1e6_f64,
            samples in proptest::option::of(2_u32..5000),
            step_fraction in 1e-4..1.0_f64,
            by_step in any::<bool>(),
        ) {
            let max = min + width;
            let step = by_step.then_some(width * step_fraction);
            let samples = if by_step { None } else { samples };
            let Ok(grid) = Grid::new([min, max], samples, step, &Budgets::DEFAULT) else {
                return Ok(());
            };
            prop_assert!(grid.len() <= Budgets::DEFAULT.max_samples);
            let points: Vec<f64> = grid.points().collect();
            prop_assert_eq!(&points, &grid.points().collect::<Vec<_>>());
            for pair in points.windows(2) {
                prop_assert!(pair[0] < pair[1]);
            }
            prop_assert!(points.iter().all(|&x| x >= min && x <= max + width * 1e-9));
        }
    }
}

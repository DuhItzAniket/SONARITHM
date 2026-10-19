// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Scales: from data values to positions in [0, 1] (04 §5.3, RFC-0001 §6).
//!
//! A scale places a datum at a position `u` in [0, 1]. The channel then maps `u` into its
//! range; polarity flips `u` between the two steps.
//!
//! | Scale | Position of a value `v` | Domain `[d0, d1]` |
//! |---|---|---|
//! | `linear` | `(v − d0) / (d1 − d0)` | finite, `d0 ≤ d1` |
//! | `log` | the same on `ln v` | positive |
//! | `symlog` | the same on `sign(v) · ln(1 + abs(v) / c)`, with `c` = [`SYMLOG_CONSTANT`] | finite |
//! | `sqrt`, `pow` | the same on `sign(v) · abs(v)^k`, with `k` = 0.5 for `sqrt` | finite |
//! | `quantile` | the mid-rank of `v` among the data, divided by `n − 1` | the data itself |
//! | `band` | the centre of the band of `v`'s category, `(i + 0.5) / k` | the categories, in the order given |
//! | `ordinal` | evenly spaced steps that include both ends, `i / (k − 1)` | the categories, in the order given |
//!
//! `domain` is `auto` (the extent of the finite values), `zero-based` (the extent widened to
//! include 0) or explicit. `nice` rounds a domain outward to round numbers, using the rule
//! of d3-scale's `nice` (about ten intervals of 1, 2 or 5 times a power of ten; powers of
//! ten for `log`). A value outside the domain gets the nearest end of [0, 1] and is reported
//! as off-scale, so the encoder can clamp it and cue it (03 §5). A degenerate domain
//! (`d0 = d1`, as when every value is equal) places its value at the centre, 0.5.
//!
//! These are mapping rules, not research defaults: the scale type is chosen by the spec,
//! and its default (`linear`, `domain: "auto"`, `nice: false`) is fixed by RFC-0001 §6.

#[cfg(test)]
mod tests;

use std::cmp::Ordering;
use std::collections::BTreeMap;

use sonarithm_core::math;

/// The constant `c` of the symmetric log transform, in data units: the transform is linear
/// within about `c` of zero and logarithmic beyond. 1 is d3-scale's default; the spec has
/// no key for it in v0.1.
pub const SYMLOG_CONSTANT: f64 = 1.0;

/// The number of intervals `nice` aims for (d3-scale's default).
const NICE_COUNT: f64 = 10.0;

/// At most this many rounding passes in `nice`; d3-scale stops after the same number.
const NICE_PASSES: usize = 10;

/// The scale types of 04 §5.3 that map values to positions. `psychophysical` (SDR-11) is a
/// pitch mapping built on [`Transform::Log`]; it lives with the pitch channel.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum ScaleKind {
    /// `linear`.
    Linear,
    /// `log`, natural base (the base does not change positions).
    Log,
    /// `symlog`, for data with zeros and both signs.
    Symlog,
    /// `sqrt`: `pow` with exponent 0.5.
    Sqrt,
    /// `pow` with an exponent greater than 0.
    Pow(f64),
    /// `quantile`: positions by rank.
    Quantile,
    /// `band`: one band per category.
    Band,
    /// `ordinal`: one step per category.
    Ordinal,
}

impl ScaleKind {
    /// The transform of a continuous scale, or `None` for `quantile`, `band` and `ordinal`.
    #[must_use]
    pub const fn transform(self) -> Option<Transform> {
        match self {
            Self::Linear => Some(Transform::Linear),
            Self::Log => Some(Transform::Log),
            Self::Symlog => Some(Transform::Symlog),
            Self::Sqrt => Some(Transform::SQRT),
            Self::Pow(exponent) => Some(Transform::Pow(exponent)),
            Self::Quantile | Self::Band | Self::Ordinal => None,
        }
    }

    /// Whether the scale maps categories (`band`, `ordinal`) rather than numbers.
    #[must_use]
    pub const fn is_discrete(self) -> bool {
        matches!(self, Self::Band | Self::Ordinal)
    }
}

/// The transform a continuous scale applies before normalizing.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Transform {
    /// The identity.
    Linear,
    /// The natural logarithm; the domain must be positive.
    Log,
    /// `sign(v) · ln(1 + abs(v) / c)` with `c` = [`SYMLOG_CONSTANT`].
    Symlog,
    /// `sign(v) · abs(v)^k`, with `k` > 0.
    Pow(f64),
}

impl Transform {
    /// The square root, `pow` with exponent 0.5.
    pub const SQRT: Self = Self::Pow(0.5);

    fn apply(self, v: f64) -> f64 {
        match self {
            Self::Linear => v,
            Self::Log => math::ln(v),
            Self::Symlog => v.signum() * math::ln(1.0 + v.abs() / SYMLOG_CONSTANT),
            Self::Pow(k) => v.signum() * math::pow(v.abs(), k),
        }
    }

    fn invert(self, t: f64) -> f64 {
        match self {
            Self::Linear => t,
            Self::Log => math::exp(t),
            Self::Symlog => t.signum() * (math::exp(t.abs()) - 1.0) * SYMLOG_CONSTANT,
            Self::Pow(k) => t.signum() * math::pow(t.abs(), 1.0 / k),
        }
    }

    fn check(self) -> Result<(), ScaleError> {
        match self {
            Self::Pow(k) if !(k.is_finite() && k > 0.0) => Err(ScaleError::InvalidExponent(k)),
            _ => Ok(()),
        }
    }
}

/// How a continuous scale finds its domain (RFC-0001 §6).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Domain {
    /// The extent of the finite values (the default).
    #[default]
    Auto,
    /// The extent of the finite values, widened to include 0.
    ZeroBased,
    /// An explicit `[min, max]` with `min < max`.
    Explicit([f64; 2]),
}

/// Why a scale cannot be built.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum ScaleError {
    /// There is no finite value to take the domain from.
    #[error("the field has no finite values to fit the scale to")]
    NoValues,
    /// A log scale's domain starts at zero or below.
    #[error(
        "a log scale needs positive values, but its domain starts at {min}; use a symlog scale for data with zeros or negative values"
    )]
    NonPositiveLogDomain {
        /// The domain minimum.
        min: f64,
    },
    /// An explicit domain is not finite and increasing.
    #[error("the domain [{min}, {max}] must be two finite numbers, low to high")]
    InvalidDomain {
        /// The first bound.
        min: f64,
        /// The second bound.
        max: f64,
    },
    /// A `pow` exponent is not a finite number greater than 0.
    #[error("the exponent {0} must be a finite number greater than 0")]
    InvalidExponent(f64),
    /// A band or ordinal scale has no categories.
    #[error("the field has no categories to fit the scale to")]
    NoCategories,
}

/// Where a datum lies relative to its scale's domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OffScale {
    /// Inside the domain, ends included.
    Within,
    /// Below the domain (or not positive, for a log scale).
    Below,
    /// Above the domain.
    Above,
}

/// A datum's position on a scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    /// The position in [0, 1].
    pub u: f64,
    /// Whether the datum was outside the domain and clamped to an end.
    pub off_scale: OffScale,
}

impl Position {
    /// A position inside the domain.
    #[must_use]
    pub const fn within(u: f64) -> Self {
        Self {
            u,
            off_scale: OffScale::Within,
        }
    }
}

/// A continuous scale: `linear`, `log`, `symlog`, `sqrt` or `pow`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContinuousScale {
    transform: Transform,
    domain: [f64; 2],
    transformed: [f64; 2],
}

impl ContinuousScale {
    /// A scale over an explicit domain.
    ///
    /// # Errors
    ///
    /// [`ScaleError::InvalidDomain`] unless the domain is finite and `d0 ≤ d1`;
    /// [`ScaleError::NonPositiveLogDomain`] for a log scale whose domain is not positive;
    /// [`ScaleError::InvalidExponent`] for a bad `pow` exponent.
    pub fn new(transform: Transform, domain: [f64; 2]) -> Result<Self, ScaleError> {
        transform.check()?;
        let [d0, d1] = domain;
        if !(d0.is_finite() && d1.is_finite() && d0 <= d1) {
            return Err(ScaleError::InvalidDomain { min: d0, max: d1 });
        }
        if transform == Transform::Log && d0 <= 0.0 {
            return Err(ScaleError::NonPositiveLogDomain { min: d0 });
        }
        Ok(Self {
            transform,
            domain,
            transformed: [transform.apply(d0), transform.apply(d1)],
        })
    }

    /// A scale whose domain comes from `domain` and the data `values`, rounded outward when
    /// `nice` is set. Non-finite values are ignored when the domain is taken from the data.
    ///
    /// # Errors
    ///
    /// [`ScaleError::NoValues`] when the data has no finite value, and the errors of
    /// [`ContinuousScale::new`].
    pub fn fit(
        transform: Transform,
        domain: Domain,
        nice: bool,
        values: &[f64],
    ) -> Result<Self, ScaleError> {
        let bounds = match domain {
            Domain::Explicit([min, max]) => {
                if min.partial_cmp(&max) != Some(Ordering::Less) {
                    return Err(ScaleError::InvalidDomain { min, max });
                }
                [min, max]
            }
            Domain::Auto => extent(values)?,
            Domain::ZeroBased => {
                let [min, max] = extent(values)?;
                [min.min(0.0), max.max(0.0)]
            }
        };
        let bounds = if nice {
            nice_domain(transform, bounds)
        } else {
            bounds
        };
        Self::new(transform, bounds)
    }

    /// The domain `[d0, d1]`.
    #[must_use]
    pub const fn domain(&self) -> [f64; 2] {
        self.domain
    }

    /// The transform.
    #[must_use]
    pub const fn transform(&self) -> Transform {
        self.transform
    }

    /// Whether the domain is a single value; its value is then placed at 0.5.
    #[must_use]
    pub fn is_degenerate(&self) -> bool {
        self.transformed[0] >= self.transformed[1]
    }

    /// The position of `value`, or `None` for NaN (a missing value, 03 §5). Infinities are
    /// off-scale and clamped to the nearest end.
    #[must_use]
    pub fn position(&self, value: f64) -> Option<Position> {
        if value.is_nan() {
            return None;
        }
        let [d0, d1] = self.domain;
        let off_scale = if value < d0 {
            OffScale::Below
        } else if value > d1 {
            OffScale::Above
        } else {
            OffScale::Within
        };
        let u = match off_scale {
            OffScale::Below => 0.0,
            OffScale::Above => 1.0,
            OffScale::Within if self.is_degenerate() => 0.5,
            OffScale::Within => {
                let [t0, t1] = self.transformed;
                ((self.transform.apply(value) - t0) / (t1 - t0)).clamp(0.0, 1.0)
            }
        };
        Some(Position { u, off_scale })
    }

    /// The domain value at position `u` (clamped to [0, 1]); the inverse of
    /// [`ContinuousScale::position`] inside the domain, used by legends and announcements.
    #[must_use]
    pub fn invert(&self, u: f64) -> f64 {
        let [t0, t1] = self.transformed;
        if self.is_degenerate() {
            return self.domain[0];
        }
        let value = self.transform.invert(t0 + u.clamp(0.0, 1.0) * (t1 - t0));
        value.clamp(self.domain[0], self.domain[1])
    }
}

/// The extent of the finite values.
fn extent(values: &[f64]) -> Result<[f64; 2], ScaleError> {
    values
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold(None, |extent: Option<[f64; 2]>, v| {
            Some(extent.map_or([v, v], |[lo, hi]| [lo.min(v), hi.max(v)]))
        })
        .ok_or(ScaleError::NoValues)
}

/// Rounds a domain outward (d3-scale's `nice`): powers of ten for `log`, otherwise about
/// [`NICE_COUNT`] intervals of 1, 2 or 5 times a power of ten.
fn nice_domain(transform: Transform, [d0, d1]: [f64; 2]) -> [f64; 2] {
    match transform {
        Transform::Log if d0 > 0.0 && d0 < d1 => [
            math::pow(10.0, math::log10(d0).floor()),
            math::pow(10.0, math::log10(d1).ceil()),
        ],
        Transform::Log => [d0, d1],
        _ => nice_linear([d0, d1]),
    }
}

fn nice_linear([mut low, mut high]: [f64; 2]) -> [f64; 2] {
    if low.partial_cmp(&high) != Some(Ordering::Less) {
        return [low, high];
    }
    let mut previous = None;
    for _ in 0..NICE_PASSES {
        let increment = tick_increment(low, high);
        if previous == Some(increment) || increment == 0.0 || !increment.is_finite() {
            break;
        }
        if increment > 0.0 {
            low = (low / increment).floor() * increment;
            high = (high / increment).ceil() * increment;
        } else {
            // A negative increment `-n` stands for a step of `1 / n`, which keeps the
            // rounding exact for steps below 1.
            low = (low * increment).ceil() / increment;
            high = (high * increment).floor() / increment;
        }
        previous = Some(increment);
    }
    [low, high]
}

/// The tick step for about [`NICE_COUNT`] intervals over `[low, high]`: 1, 2 or 5 times a
/// power of ten, or for steps below 1 the negated reciprocal `-1 / step`.
fn tick_increment(low: f64, high: f64) -> f64 {
    let raw = (high - low) / NICE_COUNT;
    let power = math::log10(raw).floor();
    let error = raw / math::pow(10.0, power);
    let factor = if error >= 50_f64.sqrt() {
        10.0
    } else if error >= 10_f64.sqrt() {
        5.0
    } else if error >= 2_f64.sqrt() {
        2.0
    } else {
        1.0
    };
    if power >= 0.0 {
        factor * math::pow(10.0, power)
    } else {
        -math::pow(10.0, -power) / factor
    }
}

/// A `quantile` scale: a value's position is its mid-rank among the data, divided by
/// `n − 1`, so the data spreads evenly over [0, 1] whatever its distribution. Tied values
/// share the mean of their ranks; a value between two data values gets a rank between
/// theirs.
#[derive(Clone, Debug, PartialEq)]
pub struct QuantileScale {
    sorted: Vec<f64>,
}

impl QuantileScale {
    /// A scale over the finite `values`.
    ///
    /// # Errors
    ///
    /// [`ScaleError::NoValues`] when there is no finite value.
    pub fn fit(values: &[f64]) -> Result<Self, ScaleError> {
        let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
        if sorted.is_empty() {
            return Err(ScaleError::NoValues);
        }
        sorted.sort_by(f64::total_cmp);
        Ok(Self { sorted })
    }

    /// The smallest and largest data value.
    #[must_use]
    pub fn domain(&self) -> [f64; 2] {
        let first = self.sorted.first().copied().unwrap_or(0.0);
        let last = self.sorted.last().copied().unwrap_or(first);
        [first, last]
    }

    /// The position of `value`, or `None` for NaN.
    #[must_use]
    pub fn position(&self, value: f64) -> Option<Position> {
        if value.is_nan() {
            return None;
        }
        let [min, max] = self.domain();
        let off_scale = if value < min {
            OffScale::Below
        } else if value > max {
            OffScale::Above
        } else {
            OffScale::Within
        };
        let n = self.sorted.len();
        if n == 1 {
            let u = match off_scale {
                OffScale::Below => 0.0,
                OffScale::Within => 0.5,
                OffScale::Above => 1.0,
            };
            return Some(Position { u, off_scale });
        }
        let below = self.sorted.partition_point(|x| *x < value);
        let not_above = self.sorted.partition_point(|x| *x <= value);
        #[expect(
            clippy::cast_precision_loss,
            reason = "row counts stay far below 2^52, where usize to f64 is exact"
        )]
        let (rank, last) = ((below + not_above) as f64 / 2.0 - 0.5, (n - 1) as f64);
        Some(Position {
            u: (rank / last).clamp(0.0, 1.0),
            off_scale,
        })
    }
}

/// A category of a `band` or `ordinal` scale: a number or a text label.
///
/// Categories are totally ordered: numbers before text, numbers by IEEE 754 `totalOrder`,
/// text by byte order. The order is used by [`sorted_categories`] and for lookups; a scale's
/// own order is the one it was built with.
#[derive(Clone, Debug)]
pub enum Category {
    /// A numeric category, such as an ordinal level.
    Number(f64),
    /// A text label.
    Text(String),
}

impl Ord for Category {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.total_cmp(b),
            (Self::Number(_), Self::Text(_)) => Ordering::Less,
            (Self::Text(_), Self::Number(_)) => Ordering::Greater,
            (Self::Text(a), Self::Text(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for Category {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Category {}

/// The distinct categories of `values`, in the order they first appear.
pub fn categories_in_order_of_appearance(
    values: impl IntoIterator<Item = Category>,
) -> Vec<Category> {
    let mut seen = std::collections::BTreeSet::new();
    values
        .into_iter()
        .filter(|category| seen.insert(category.clone()))
        .collect()
}

/// The distinct categories of `values`, sorted (see [`Category`]).
pub fn sorted_categories(values: impl IntoIterator<Item = Category>) -> Vec<Category> {
    values
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Which discrete scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscreteKind {
    /// `band`: category `i` of `k` at the centre of its band, `(i + 0.5) / k`.
    Band,
    /// `ordinal`: category `i` of `k` at `i / (k − 1)`, both ends included; one category
    /// sits at 0.5.
    Ordinal,
}

/// A `band` or `ordinal` scale over categories in a given order.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteScale {
    kind: DiscreteKind,
    categories: Vec<Category>,
    index: BTreeMap<Category, usize>,
}

impl DiscreteScale {
    /// A scale over `categories` in the order given. A repeated category keeps its first
    /// place.
    ///
    /// # Errors
    ///
    /// [`ScaleError::NoCategories`] when there is none.
    pub fn new(
        kind: DiscreteKind,
        categories: impl IntoIterator<Item = Category>,
    ) -> Result<Self, ScaleError> {
        let categories = categories_in_order_of_appearance(categories);
        if categories.is_empty() {
            return Err(ScaleError::NoCategories);
        }
        let index = categories
            .iter()
            .enumerate()
            .map(|(i, category)| (category.clone(), i))
            .collect();
        Ok(Self {
            kind,
            categories,
            index,
        })
    }

    /// The kind.
    #[must_use]
    pub const fn kind(&self) -> DiscreteKind {
        self.kind
    }

    /// The categories in scale order.
    #[must_use]
    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    /// The position of `category`, or `None` if the scale does not have it.
    #[must_use]
    pub fn position(&self, category: &Category) -> Option<Position> {
        self.index
            .get(category)
            .map(|&i| Position::within(self.position_of_index(i)))
    }

    /// The position of the category at `index` in scale order.
    #[must_use]
    pub fn position_of_index(&self, index: usize) -> f64 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "category counts stay far below 2^52, where usize to f64 is exact"
        )]
        let (i, k) = (index as f64, self.categories.len() as f64);
        match self.kind {
            DiscreteKind::Band => ((i + 0.5) / k).clamp(0.0, 1.0),
            DiscreteKind::Ordinal if self.categories.len() == 1 => 0.5,
            DiscreteKind::Ordinal => (i / (k - 1.0)).clamp(0.0, 1.0),
        }
    }
}

/// A fitted scale of any type.
#[derive(Clone, Debug, PartialEq)]
pub enum Scale {
    /// `linear`, `log`, `symlog`, `sqrt` or `pow`.
    Continuous(ContinuousScale),
    /// `quantile`.
    Quantile(QuantileScale),
    /// `band` or `ordinal`.
    Discrete(DiscreteScale),
}

impl Scale {
    /// A numeric scale of `kind` fitted to `values`. Band and ordinal scales use the sorted
    /// distinct values as their categories.
    ///
    /// # Errors
    ///
    /// The [`ScaleError`] of the scale type.
    pub fn fit_numeric(
        kind: ScaleKind,
        domain: Domain,
        nice: bool,
        values: &[f64],
    ) -> Result<Self, ScaleError> {
        match kind {
            ScaleKind::Quantile => QuantileScale::fit(values).map(Self::Quantile),
            ScaleKind::Band | ScaleKind::Ordinal => {
                let kind = if kind == ScaleKind::Band {
                    DiscreteKind::Band
                } else {
                    DiscreteKind::Ordinal
                };
                let categories = sorted_categories(
                    values
                        .iter()
                        .filter(|v| !v.is_nan())
                        .map(|&v| Category::Number(v)),
                );
                DiscreteScale::new(kind, categories).map(Self::Discrete)
            }
            continuous => {
                let transform = continuous.transform().unwrap_or(Transform::Linear);
                ContinuousScale::fit(transform, domain, nice, values).map(Self::Continuous)
            }
        }
    }

    /// The position of a numeric `value`, or `None` for NaN and for a number a discrete
    /// scale does not have.
    #[must_use]
    pub fn position(&self, value: f64) -> Option<Position> {
        match self {
            Self::Continuous(scale) => scale.position(value),
            Self::Quantile(scale) => scale.position(value),
            Self::Discrete(_) if value.is_nan() => None,
            Self::Discrete(scale) => scale.position(&Category::Number(value)),
        }
    }
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reading marks back into values.
//!
//! | Mark | Recognised by | Value |
//! |---|---|---|
//! | Pie slice | A filled, curved, closed outline whose vertices lie on one circle about its local origin, or at it | Its angle as a share of the circle, in percent |
//! | Line | An open, stroked path of two or more points, not a straight axis-parallel segment | Each vertex through both axes |
//! | Area | A filled closed outline that is not a rectangle or a symbol | Its upper edge, from the first vertex until the outline turns back |
//! | Bar | A filled axis-parallel rectangle inside the plot | Its far edge through the value axis, less its near edge (stacked segments) |
//! | Marker | A circle, or a small curved outline drawn about its origin | Its centre through both axes |
//!
//! Only marks inside the plot (the span of the axes' ticks, widened by a tenth) are read;
//! a legend's swatches lie outside and name the series of their colour.

use std::collections::BTreeMap;

use sonarithm_core::{Confidence, Diagnostic, Provenance, ProvenanceClass};

use super::axes::{self, Axes, Axis, Dir};
use super::{Pt, Rgb, Scene, Shape, Text};
use crate::budget::{Budgets, Work};
use crate::codes;
use crate::model::{ChartData, ChartType, Point, Series, XValue};

/// The plot area: the axes' tick spans widened by a tenth on each side.
#[derive(Clone, Copy, Debug)]
struct Plot {
    x: (f64, f64),
    y: (f64, f64),
}

impl Plot {
    fn contains(&self, (x, y): Pt) -> bool {
        x >= self.x.0 && x <= self.x.1 && y >= self.y.0 && y <= self.y.1
    }

    fn diagonal(&self) -> f64 {
        (self.x.1 - self.x.0).hypot(self.y.1 - self.y.0)
    }
}

fn widen((lo, hi): (f64, f64)) -> (f64, f64) {
    let pad = ((hi - lo).abs() * 0.02).max(2.0);
    (lo.min(hi) - pad, lo.max(hi) + pad)
}

/// The plot area: the tick spans, grown to the frame, grid and domain lines that charts
/// draw around their plots (never a legend's), then widened by 2 %.
fn plot_area(scene: &Scene, xa: &Axis, ya: &Axis) -> Plot {
    let (mut x, mut y) = (sorted(xa.span()), sorted(ya.span()));
    if xa.dir() == Dir::Vertical {
        std::mem::swap(&mut x, &mut y);
    }
    // The tightest rectangle around the ticks, if not much larger than their span, is the
    // plot frame or background (PDF has no class names to say so).
    let ticks = (x, y);
    let frame = scene
        .shapes
        .iter()
        .filter(|shape| shape.closed && shape.points.len() == 4)
        .map(Shape::bbox)
        .filter(|(x0, y0, x1, y1)| {
            let covers =
                *x0 <= ticks.0.0 && *x1 >= ticks.0.1 && *y0 <= ticks.1.0 && *y1 >= ticks.1.1;
            let snug = (x1 - x0) <= (ticks.0.1 - ticks.0.0) * 1.5
                && (y1 - y0) <= (ticks.1.1 - ticks.1.0) * 1.5;
            covers && snug
        })
        .min_by(|a, b| ((a.2 - a.0) * (a.3 - a.1)).total_cmp(&((b.2 - b.0) * (b.3 - b.1))));
    if let Some((x0, y0, x1, y1)) = frame {
        x = (x.0.min(x0), x.1.max(x1));
        y = (y.0.min(y0), y.1.max(y1));
    }
    for shape in &scene.shapes {
        let c = shape.classes.to_ascii_lowercase();
        let frame = ["background", " bg", "domain", "grid"]
            .iter()
            .any(|k| c.contains(k))
            && !c.contains("legend")
            && !c.contains("title");
        if frame {
            let (x0, y0, x1, y1) = shape.bbox();
            if x1 > x0 || y1 > y0 {
                x = (x.0.min(x0), x.1.max(x1));
                y = (y.0.min(y0), y.1.max(y1));
            }
        }
    }
    Plot {
        x: widen(x),
        y: widen(y),
    }
}

fn sorted((a, b): (f64, f64)) -> (f64, f64) {
    (a.min(b), a.max(b))
}

/// An axis-parallel rectangle as (x0, y0, x1, y1).
type Rect = (f64, f64, f64, f64);

/// Reads the chart a scene draws.
#[expect(clippy::too_many_lines, reason = "one pass over the scene's marks")]
pub(crate) fn read(
    scene: &Scene,
    budgets: &Budgets,
    work: &mut Work,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<ChartData, Box<Diagnostic>> {
    work.spend(crate::budget::len64(scene.shapes.len() + scene.texts.len()))?;
    let title = chart_title(scene);
    if let Some(chart) = pie(scene, title.clone(), diagnostics) {
        return Ok(chart);
    }
    let found = axes::find(scene, diagnostics);
    let (Some(xa), Some(ya)) = (&found.x, &found.y) else {
        return Err(Box::new(Diagnostic::new(
            &codes::AXIS_NOT_FOUND,
            "No pair of axes with tick labels was found, so the marks cannot be read as \
             values. Give the axes' anchors in `calibration`.",
        )));
    };
    let plot = plot_area(scene, xa, ya);
    let legend = legend(scene, &found, &plot);
    let provenance = Provenance {
        class: ProvenanceClass::Vector,
        confidence: Some(min_confidence(xa.confidence(), ya.confidence())),
    };
    let residual = match (xa.residual(), ya.residual()) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    };
    let mut groups: Vec<(Rgb, ChartType, Vec<Point>)> = Vec::new();
    let mut add = |color: Rgb, kind: ChartType, points: Vec<Point>| match groups
        .iter_mut()
        .find(|(c, k, _)| *c == color && *k == kind)
    {
        Some((_, _, existing)) => existing.extend(points),
        None => groups.push((color, kind, points)),
    };
    let mut bars: Vec<(Rgb, Rect)> = Vec::new();
    let mut positions = Vec::new();
    for shape in &scene.shapes {
        if !plot.contains(shape.center()) || is_furniture(shape) {
            continue;
        }
        if let Some(rect) = rectangle(shape) {
            // A background spans most of the plot's width and height; a bar does not.
            let (x0, y0, x1, y1) = rect;
            let background = (x1 - x0) >= (plot.x.1 - plot.x.0) * 0.8
                && (y1 - y0) >= (plot.y.1 - plot.y.0) * 0.5;
            if let Some(color) = shape.fill
                && !background
            {
                bars.push((color, rect));
            }
            continue;
        }
        if is_marker(shape, &plot) {
            let center = marker_center(shape);
            positions.push(center);
            if let Some(color) = shape.fill.or(shape.stroke) {
                add(color, ChartType::Scatter, vec![point(center, xa, ya)]);
            }
            continue;
        }
        if !shape.closed && shape.points.len() >= 2 && !is_straight(shape) {
            if let Some(color) = shape.stroke {
                positions.extend(&shape.points);
                add(
                    color,
                    ChartType::Line,
                    shape.points.iter().map(|p| point(*p, xa, ya)).collect(),
                );
            }
            continue;
        }
        if shape.closed
            && shape.points.len() > 4
            && !shape.curved
            && let Some(color) = shape.fill
        {
            let edge = upper_edge(&shape.points);
            positions.extend(&edge);
            add(
                color,
                ChartType::Area,
                edge.iter().map(|p| point(*p, xa, ya)).collect(),
            );
        }
    }
    let stacked = bar_groups(&bars, xa, ya, &mut add);
    for axis in [xa, ya] {
        if let Axis::Continuous {
            calibration, dir, ..
        } = axis
        {
            let along: Vec<f64> = positions.iter().map(|p| dir.of(*p)).collect();
            let frame = if *dir == Dir::Horizontal {
                plot.x
            } else {
                plot.y
            };
            if let Some(d) = calibration.check_bounds(
                &along,
                frame,
                0.0,
                if *dir == Dir::Horizontal { "x" } else { "y" },
            ) {
                diagnostics.push(d);
            }
        }
    }
    // An area drawn with an outline line of the same colour: keep the line's points only.
    let line_colors: Vec<Rgb> = groups
        .iter()
        .filter(|g| g.1 == ChartType::Line)
        .map(|g| g.0)
        .collect();
    groups.retain(|(c, k, _)| !(*k == ChartType::Area && line_colors.contains(c)));
    let has_area = groups.iter().any(|g| g.1 == ChartType::Area);
    if has_area {
        for g in &mut groups {
            if g.1 == ChartType::Line && line_colors.contains(&g.0) {
                g.1 = ChartType::Area;
            }
        }
    }
    let mut series: Vec<(Option<usize>, Series)> = Vec::new();
    let mut unnamed = 0;
    for (color, kind, mut points) in groups {
        if kind == ChartType::Scatter {
            points.sort_by(|a, b| x_order(&a.x, &b.x));
        }
        let rank = legend.iter().position(|(c, _)| *c == color);
        let name = rank.and_then(|r| legend.get(r)).map_or_else(
            || {
                unnamed += 1;
                format!("Series {unnamed}")
            },
            |(_, n)| n.clone(),
        );
        let budget_points = crate::budget::len64(points.len());
        if budget_points > budgets.max_points {
            return Err(crate::budget::over(
                crate::budget::Budget::Points,
                budget_points,
                budgets.max_points,
            ));
        }
        let mut s = Series::exact(name, points);
        s.provenance = provenance;
        s.residual = residual;
        s.chart_type = Some(if kind == ChartType::Bar && stacked {
            ChartType::StackedBar
        } else {
            kind
        });
        series.push((rank, s));
    }
    series.sort_by_key(|(rank, _)| rank.unwrap_or(usize::MAX));
    let series: Vec<Series> = series.into_iter().map(|(_, s)| s).collect();
    let chart_type = [
        ChartType::Line,
        ChartType::Area,
        ChartType::StackedBar,
        ChartType::Bar,
        ChartType::Scatter,
    ]
    .into_iter()
    .find(|k| series.iter().any(|s| s.chart_type == Some(*k)));
    if series.is_empty() {
        return Err(Box::new(Diagnostic::new(
            &codes::NO_CHART,
            "The chart's axes were found, but no lines, bars, areas or markers inside them.",
        )));
    }
    let (x_title, y_title) = axis_titles(scene);
    Ok(ChartData {
        chart_type,
        title,
        x_title,
        y_title,
        series,
    })
}

fn min_confidence(a: Confidence, b: Confidence) -> Confidence {
    if f32::from(a) <= f32::from(b) { a } else { b }
}

fn x_order(a: &XValue, b: &XValue) -> std::cmp::Ordering {
    match (a, b) {
        (XValue::Number(x), XValue::Number(y)) => x.total_cmp(y),
        (XValue::Time(x), XValue::Time(y)) => x.cmp(y),
        _ => std::cmp::Ordering::Equal,
    }
}

/// Backgrounds, frames and grid furniture by class.
fn is_furniture(shape: &Shape) -> bool {
    let c = shape.classes.to_ascii_lowercase();
    [
        "background",
        " bg",
        "grid",
        "domain",
        "legend",
        "axis",
        "zeroline",
    ]
    .iter()
    .any(|k| c.contains(k))
}

/// Whether a two-point shape (or a polyline) is one straight axis-parallel segment.
fn is_straight(shape: &Shape) -> bool {
    let (x0, y0, x1, y1) = shape.bbox();
    shape.points.len() <= 3 && ((x1 - x0).abs() < 1e-6 || (y1 - y0).abs() < 1e-6)
}

/// An axis-parallel rectangle as (x0, y0, x1, y1).
fn rectangle(shape: &Shape) -> Option<(f64, f64, f64, f64)> {
    if !shape.closed || shape.curved {
        return None;
    }
    let mut corners: Vec<Pt> = shape.points.clone();
    if corners.len() == 5 && corners.first() == corners.last() {
        corners.pop();
    }
    if corners.len() != 4 {
        return None;
    }
    let (x0, y0, x1, y1) = shape.bbox();
    let on_corner = |(x, y): &Pt| {
        ((x - x0).abs() < 1e-6 || (x - x1).abs() < 1e-6)
            && ((y - y0).abs() < 1e-6 || (y - y1).abs() < 1e-6)
    };
    (corners.iter().all(on_corner) && x1 > x0 && y1 > y0).then_some((x0, y0, x1, y1))
}

/// A circle, or a small curved outline: a scatter marker.
fn is_marker(shape: &Shape, plot: &Plot) -> bool {
    let (x0, y0, x1, y1) = shape.bbox();
    let size = (x1 - x0).max(y1 - y0);
    // Symbols are curved and small; some renderers leave their outline open.
    shape.curved && size < plot.diagonal() * 0.03
}

/// A symbol's centre: its local origin when it is drawn about it, else its box's centre.
fn marker_center(shape: &Shape) -> Pt {
    let (x0, y0, x1, y1) = shape.bbox();
    let size = (x1 - x0).max(y1 - y0);
    if shape.local_extent <= size && {
        let (ox, oy) = shape.origin;
        ox >= x0 && ox <= x1 && oy >= y0 && oy <= y1
    } {
        shape.origin
    } else {
        shape.center()
    }
}

/// An area outline's upper edge: its vertices from the first until x turns back.
fn upper_edge(points: &[Pt]) -> Vec<Pt> {
    let mut edge: Vec<Pt> = Vec::new();
    for p in points {
        if let Some(last) = edge.last()
            && p.0 < last.0 - 1e-6
        {
            break;
        }
        edge.push(*p);
    }
    // Drop the closing drop to the baseline: a last vertex straight below the one before.
    if edge.len() >= 2
        && let [.., a, b] = edge.as_slice()
        && (a.0 - b.0).abs() < 1e-6
    {
        edge.pop();
    }
    edge
}

fn point(p: Pt, xa: &Axis, ya: &Axis) -> Point {
    Point {
        x: x_value(p, xa),
        y: y_value(p, ya),
    }
}

fn x_value(p: Pt, axis: &Axis) -> XValue {
    match axis {
        Axis::Continuous {
            calibration, dir, ..
        } => {
            let v = calibration.value_at(dir.of(p));
            if calibration.kind() == crate::calibrate::ScaleKind::Date {
                #[expect(clippy::cast_possible_truncation, reason = "times in ms")]
                let ms = v.round() as i64;
                XValue::Time(ms)
            } else {
                XValue::Number(v)
            }
        }
        Axis::Category { axis, dir, .. } => {
            XValue::Category(axis.label_at(dir.of(p)).unwrap_or_default().to_owned())
        }
    }
}

fn y_value(p: Pt, axis: &Axis) -> Option<f64> {
    match axis {
        Axis::Continuous {
            calibration, dir, ..
        } => Some(calibration.value_at(dir.of(p))),
        Axis::Category { .. } => None,
    }
}

/// Bars become points: the category from the bar's centre on the category axis, the value
/// from its edges on the value axis. Returns whether bars of different colours stack.
fn bar_groups(
    bars: &[(Rgb, Rect)],
    xa: &Axis,
    ya: &Axis,
    add: &mut impl FnMut(Rgb, ChartType, Vec<Point>),
) -> bool {
    // Vertical bars stand on a category (or numeric) x axis with a continuous y axis;
    // horizontal bars lie along a continuous x axis from a category y axis.
    let horizontal = ya.is_category() && !xa.is_category();
    let (cat, val) = if horizontal { (ya, xa) } else { (xa, ya) };
    let Axis::Continuous { calibration, .. } = val else {
        return false;
    };
    let mut centers: BTreeMap<i64, Vec<Rgb>> = BTreeMap::new();
    let mut points: Vec<(Rgb, Point)> = Vec::new();
    for (color, (x0, y0, x1, y1)) in bars {
        let (center, near, far) = if horizontal {
            (((x0 + x1) / 2.0, (y0 + y1) / 2.0), *x0, *x1)
        } else {
            (((x0 + x1) / 2.0, (y0 + y1) / 2.0), *y1, *y0)
        };
        let (a, b) = (calibration.value_at(near), calibration.value_at(far));
        let value = if a.abs() <= b.abs() { b - a } else { a - b };
        let key = cat.dir().of(center);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "drawing coordinates are small"
        )]
        let bucket = (key * 10.0).round() as i64;
        centers.entry(bucket).or_default().push(*color);
        let x = match cat {
            Axis::Category { axis, dir, .. } => {
                XValue::Category(axis.label_at(dir.of(center)).unwrap_or_default().to_owned())
            }
            Axis::Continuous { .. } => x_value(center, cat),
        };
        points.push((*color, Point { x, y: Some(value) }));
    }
    let stacked = centers.values().any(|colors| {
        let mut c = colors.clone();
        c.sort_unstable();
        c.dedup();
        c.len() > 1
    });
    for (color, p) in points {
        add(color, ChartType::Bar, vec![p]);
    }
    stacked
}

/// Legend entries in order: a text outside the plot with a coloured swatch just left of
/// it, on the same line.
fn legend(scene: &Scene, axes: &Axes, plot: &Plot) -> Vec<(Rgb, String)> {
    let mut entries: Vec<(Pt, Rgb, String)> = Vec::new();
    for (i, text) in scene.texts.iter().enumerate() {
        if axes.labels.contains(&i) || plot.contains(text.at) || is_heading(text) {
            continue;
        }
        let swatch = scene
            .shapes
            .iter()
            .filter(|s| {
                let (cx, cy) = s.center();
                let (x0, y0, x1, y1) = s.bbox();
                let c = s.classes.to_ascii_lowercase();
                let dx = text.at.0 - cx;
                dx > 0.0
                    && dx < 60.0
                    && (text.at.1 - cy).abs() < 12.0
                    && !plot.contains((cx, cy))
                    && (x1 - x0).max(y1 - y0) <= 40.0
                    && !["background", "bg", "toggle"].iter().any(|k| c.contains(k))
            })
            .min_by(|a, b| (text.at.0 - a.center().0).total_cmp(&(text.at.0 - b.center().0)));
        if let Some(color) =
            swatch.and_then(|s| s.fill.filter(|c| *c != (255, 255, 255)).or(s.stroke))
        {
            entries.push((text.at, color, text.content.clone()));
        }
    }
    entries.sort_by(|a, b| a.0.1.total_cmp(&b.0.1).then(a.0.0.total_cmp(&b.0.0)));
    let mut seen = Vec::new();
    entries
        .into_iter()
        .filter(|(_, c, _)| {
            let fresh = !seen.contains(c);
            seen.push(*c);
            fresh
        })
        .map(|(_, c, n)| (c, n))
        .collect()
}

/// A chart or axis title, by class.
fn is_heading(text: &Text) -> bool {
    let c = text.classes.to_ascii_lowercase();
    c.contains("title") && !c.contains("legend")
}

/// The chart title: a text classed as the chart's title, else the topmost text when it
/// stands alone on its line.
fn chart_title(scene: &Scene) -> Option<String> {
    let classed = scene.texts.iter().find(|t| {
        let c = t.classes.to_ascii_lowercase();
        (c.contains("gtitle") || c.contains("role-title")) && !c.contains("subtitle")
    });
    if let Some(t) = classed {
        return Some(t.content.clone());
    }
    let top = scene
        .texts
        .iter()
        .filter(|t| !t.rotated && !t.classes.to_ascii_lowercase().contains("axis"))
        .min_by(|a, b| a.at.1.total_cmp(&b.at.1))?;
    let alone = scene
        .texts
        .iter()
        .filter(|t| (t.at.1 - top.at.1).abs() < 1.5)
        .count()
        == 1;
    alone.then(|| top.content.clone())
}

/// Axis titles by class (Vega's `role-axis-title`, Plotly's `xtitle`/`ytitle`).
fn axis_titles(scene: &Scene) -> (Option<String>, Option<String>) {
    let find = |pred: &dyn Fn(&Text) -> bool| {
        scene
            .texts
            .iter()
            .find(|t| pred(t))
            .map(|t| t.content.clone())
    };
    let x = find(&|t| {
        let c = t.classes.to_ascii_lowercase();
        c.contains("xtitle") || (c.contains("axis-title") && !t.rotated)
    });
    let y = find(&|t| {
        let c = t.classes.to_ascii_lowercase();
        c.contains("ytitle") || (c.contains("axis-title") && t.rotated)
    });
    (x, y)
}

/// A pie: two or more sectors about one centre that together fill most of a circle.
fn pie(
    scene: &Scene,
    title: Option<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ChartData> {
    let mut slices: Vec<(f64, f64, Rgb, Pt)> = Vec::new();
    for shape in &scene.shapes {
        let Some(color) = shape.fill else { continue };
        if !shape.closed || !shape.curved || is_furniture(shape) {
            continue;
        }
        if let Some((start, sweep, center)) = sector(shape) {
            slices.push((start, sweep, color, center));
        }
    }
    let total: f64 = slices.iter().map(|s| s.1).sum();
    if slices.len() < 2 || !(350.0..=370.0).contains(&total) {
        return None;
    }
    let center = slices.first()?.3;
    if slices
        .iter()
        .any(|s| (s.3.0 - center.0).abs() > 1.0 || (s.3.1 - center.1).abs() > 1.0)
    {
        return None;
    }
    slices.sort_by(|a, b| a.0.total_cmp(&b.0));
    let names = legend_colors(scene);
    let points = slices
        .iter()
        .enumerate()
        .map(|(i, (_, sweep, color, _))| Point {
            x: XValue::Category(
                names
                    .iter()
                    .find(|(c, _)| c == color)
                    .map_or_else(|| format!("Slice {}", i + 1), |(_, n)| n.clone()),
            ),
            y: Some(sweep / total * 100.0),
        })
        .collect();
    diagnostics.push(Diagnostic::new(
        &codes::PIE_SHARES,
        "A pie's geometry holds each slice's share, not its value: the values are shares in percent.",
    ));
    let mut series = Series::exact("Share (%)", points);
    series.provenance = Provenance {
        class: ProvenanceClass::Vector,
        confidence: Some(Confidence::CERTAIN),
    };
    series.chart_type = Some(ChartType::Pie);
    Some(ChartData {
        chart_type: Some(ChartType::Pie),
        title,
        series: vec![series],
        ..ChartData::default()
    })
}

/// A sector: its start angle, clockwise from 12 o'clock, and its sweep, in degrees. The
/// centre is the vertex from which every other vertex lies at one radius (a donut's inner
/// rim, at a smaller radius, is allowed); without such a vertex, the local origin.
fn sector(shape: &Shape) -> Option<(f64, f64, Pt)> {
    let distance = |c: Pt, p: &Pt| (p.0 - c.0).hypot(p.1 - c.1);
    let mut candidates: Vec<Pt> = vec![shape.origin];
    candidates.extend(&shape.points);
    let (c, r) = candidates.iter().find_map(|c| {
        let others: Vec<f64> = shape
            .points
            .iter()
            .map(|p| distance(*c, p))
            .filter(|d| *d > 1e-3)
            .collect();
        let r = others.iter().copied().fold(0.0_f64, f64::max);
        let on_rim = others
            .iter()
            .filter(|d| (*d - r).abs() <= r * 0.005)
            .count();
        (r >= 1.0 && on_rim >= 2 && on_rim == others.len()).then_some((*c, r))
    })?;
    let outer: Vec<Pt> = shape
        .points
        .iter()
        .copied()
        .filter(|p| (distance(c, p) - r).abs() <= r * 0.005)
        .collect();
    // Clockwise angle from 12 o'clock in screen coordinates (y down).
    let angle = |p: &Pt| {
        let a = (p.0 - c.0).atan2(-(p.1 - c.1)).to_degrees();
        if a < 0.0 { a + 360.0 } else { a }
    };
    let mut sweep = 0.0;
    let mut direction = 0.0_f64;
    for pair in outer.windows(2) {
        let [a, b] = pair else { continue };
        let mut step = angle(b) - angle(a);
        if step > 180.0 {
            step -= 360.0;
        } else if step < -180.0 {
            step += 360.0;
        }
        if direction == 0.0 {
            direction = step.signum();
        }
        sweep += step;
    }
    let start = if direction >= 0.0 {
        angle(outer.first()?)
    } else {
        angle(outer.last()?)
    };
    // A slice starting at 12 o'clock may read as 359.99°: it comes first.
    let start = if start > 359.5 { start - 360.0 } else { start };
    Some((start, sweep.abs(), c))
}

/// Legend swatch colours and labels anywhere in the scene (pies have no axes).
fn legend_colors(scene: &Scene) -> Vec<(Rgb, String)> {
    let axes = Axes::default();
    let everywhere = Plot {
        x: (f64::INFINITY, f64::NEG_INFINITY),
        y: (f64::INFINITY, f64::NEG_INFINITY),
    };
    legend(scene, &axes, &everywhere)
}

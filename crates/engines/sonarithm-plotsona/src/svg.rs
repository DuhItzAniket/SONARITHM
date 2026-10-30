// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SVG charts: tier 2, `vector` (10 §3, notes/06 §6 and §8).
//!
//! An SVG holds a chart's geometry, not its data. PLOTSONA flattens the document into a
//! [`Scene`] of shapes and texts in one coordinate system (every `transform` applied,
//! fill and stroke inherited, `style` attributes read); the shared [`vector`] reader then
//! finds the axes from their tick labels and tick or grid lines, calibrates them through
//! [`calibrate`](crate::calibrate), and reads the marks back into values:
//! open stroked paths are line series, filled outlines areas, axis-aligned rectangles
//! bars, small repeated glyphs scatter markers, and arc sectors pie slices. Series are
//! named from the legend, matched by colour.
//!
//! Nothing outside the file is read and no script runs: `<script>` elements, external
//! `href`s and `<foreignObject>` are skipped and reported (`SNR-ENG-360`, `361`).
//! Definitions (`<defs>`, clip paths, masks, markers, patterns, symbols) draw nothing
//! by themselves and are skipped.

use roxmltree::Node;
use sonarithm_core::Diagnostic;
use svgtypes::{SimplePathSegment, SimplifyingPathParser, Transform};

use crate::budget::{Budgets, Work};
use crate::codes;
use crate::model::ChartData;
use crate::vector::{self, Pt, Rgb, Scene, Shape, Text};
use crate::xml;

/// Inherited presentation state.
#[derive(Clone, Debug)]
struct Style {
    transform: Transform,
    fill: Option<Rgb>,
    stroke: Option<Rgb>,
    color: Rgb,
    classes: String,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            // SVG's initial values: fill black, stroke none.
            fill: Some((0, 0, 0)),
            stroke: None,
            color: (0, 0, 0),
            classes: String::new(),
        }
    }
}

/// Elements that draw nothing by themselves.
const SKIPPED: [&str; 12] = [
    "defs",
    "clipPath",
    "mask",
    "marker",
    "pattern",
    "symbol",
    "title",
    "desc",
    "metadata",
    "style",
    "linearGradient",
    "radialGradient",
];

/// Extracts the chart an SVG draws.
pub(crate) fn extract(
    bytes: &[u8],
    budgets: &Budgets,
    work: &mut Work,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<ChartData, Box<Diagnostic>> {
    let document = xml::parse(bytes, "SVG image", budgets, work)?;
    let mut scene = Scene::default();
    let mut notes = Notes::default();
    walk(
        document.root_element(),
        &Style::default(),
        &mut scene,
        &mut notes,
        work,
    )?;
    if notes.scripts > 0 {
        diagnostics.push(Diagnostic::new(
            &codes::SVG_SCRIPT_IGNORED,
            format!(
                "{} script element(s) in the SVG were ignored; none ran.",
                notes.scripts
            ),
        ));
    }
    if notes.external > 0 {
        diagnostics.push(Diagnostic::new(
            &codes::SVG_EXTERNAL_REF,
            format!(
                "{} reference(s) to content outside the SVG were not followed.",
                notes.external
            ),
        ));
    }
    vector::read(&scene, budgets, work, diagnostics)
}

#[derive(Debug, Default)]
struct Notes {
    scripts: usize,
    external: usize,
}

fn walk(
    node: Node<'_, '_>,
    parent: &Style,
    scene: &mut Scene,
    notes: &mut Notes,
    work: &mut Work,
) -> Result<(), Box<Diagnostic>> {
    work.spend(1)?;
    let name = node.tag_name().name();
    if name == "script" {
        notes.scripts += 1;
        return Ok(());
    }
    if name == "foreignObject" || SKIPPED.contains(&name) {
        return Ok(());
    }
    if node.attribute("display") == Some("none") || node.attribute("visibility") == Some("hidden") {
        return Ok(());
    }
    let href = node
        .attributes()
        .find(|a| a.name() == "href")
        .map(|a| a.value());
    if href.is_some_and(|h| !h.starts_with('#')) {
        notes.external += 1;
        return Ok(());
    }
    let style = style_of(node, parent);
    match name {
        "g" | "svg" | "a" | "switch" => {
            for child in node.children().filter(Node::is_element) {
                walk(child, &style, scene, notes, work)?;
            }
        }
        "text" => {
            if let Some(text) = text_of(node, &style) {
                scene.texts.push(text);
            }
        }
        "path" | "rect" | "circle" | "ellipse" | "line" | "polyline" | "polygon" => {
            if let Some(shape) = shape_of(node, name, &style, work)? {
                scene.shapes.push(shape);
            }
        }
        _ => {}
    }
    Ok(())
}

/// The element's own presentation attributes and `style` over its parent's.
fn style_of(node: Node<'_, '_>, parent: &Style) -> Style {
    let mut style = parent.clone();
    if let Some(t) = node.attribute("transform")
        && let Ok(own) = t.parse::<Transform>()
    {
        style.transform = multiply(&parent.transform, &own);
    }
    // An outer <svg> with a viewBox maps it to its viewport; charts draw in viewBox units,
    // which is the space PLOTSONA calibrates in, so no extra mapping is applied.
    let mut declarations: Vec<(String, String)> = Vec::new();
    for key in [
        "color",
        "fill",
        "stroke",
        "fill-opacity",
        "stroke-opacity",
        "opacity",
    ] {
        if let Some(v) = node.attribute(key) {
            declarations.push((key.to_owned(), v.to_owned()));
        }
    }
    if let Some(css) = node.attribute("style") {
        for declaration in css.split(';') {
            if let Some((k, v)) = declaration.split_once(':') {
                declarations.push((k.trim().to_owned(), v.trim().to_owned()));
            }
        }
    }
    for (key, value) in &declarations {
        match key.as_str() {
            "color" => {
                if let Some(c) = color(value, parent.color) {
                    style.color = c;
                }
            }
            "fill" => style.fill = paint(value, style.color),
            "stroke" => style.stroke = paint(value, style.color),
            "fill-opacity" if is_zero(value) => style.fill = None,
            "stroke-opacity" if is_zero(value) => style.stroke = None,
            "opacity" if is_zero(value) => {
                style.fill = None;
                style.stroke = None;
            }
            _ => {}
        }
    }
    if let Some(class) = node.attribute("class") {
        style.classes = format!("{} {}", parent.classes, class);
        if style.classes.len() > 512 {
            class.clone_into(&mut style.classes);
        }
    }
    style
}

fn is_zero(value: &str) -> bool {
    value.trim().parse::<f64>().is_ok_and(|v| v <= 0.0)
}

fn paint(value: &str, current: Rgb) -> Option<Rgb> {
    match value.trim() {
        "none" | "transparent" => None,
        "currentColor" | "currentcolor" => Some(current),
        other if other.starts_with("url(") => Some((128, 128, 128)),
        other => color(other, current),
    }
}

fn color(value: &str, current: Rgb) -> Option<Rgb> {
    if value.trim().eq_ignore_ascii_case("currentcolor") {
        return Some(current);
    }
    let c = value.trim().parse::<svgtypes::Color>().ok()?;
    (c.alpha > 0).then_some((c.red, c.green, c.blue))
}

fn multiply(outer: &Transform, inner: &Transform) -> Transform {
    Transform::new(
        outer.a * inner.a + outer.c * inner.b,
        outer.b * inner.a + outer.d * inner.b,
        outer.a * inner.c + outer.c * inner.d,
        outer.b * inner.c + outer.d * inner.d,
        outer.a * inner.e + outer.c * inner.f + outer.e,
        outer.b * inner.e + outer.d * inner.f + outer.f,
    )
}

fn apply(t: &Transform, (x, y): Pt) -> Pt {
    (t.a * x + t.c * y + t.e, t.b * x + t.d * y + t.f)
}

/// The first number of a length list attribute (`x="10 20"` places the first glyph).
fn number(node: Node<'_, '_>, name: &str) -> f64 {
    node.attribute(name)
        .and_then(|v| v.split([' ', ',']).find(|s| !s.is_empty()))
        .and_then(|v| v.trim_end_matches("px").parse::<f64>().ok())
        .filter(|v| v.is_finite())
        .unwrap_or(0.0)
}

fn text_of(node: Node<'_, '_>, style: &Style) -> Option<Text> {
    let content: String = node
        .descendants()
        .filter(Node::is_text)
        .filter_map(|n| n.text())
        .collect::<Vec<_>>()
        .join("");
    let content = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if content.is_empty() {
        return None;
    }
    let at = apply(&style.transform, (number(node, "x"), number(node, "y")));
    let t = &style.transform;
    Some(Text {
        at,
        start: at,
        end: at,
        content,
        classes: style.classes.clone(),
        rotated: t.b.abs() > 1e-6 || t.c.abs() > 1e-6,
    })
}

/// The largest arc a path may draw. The path simplifier turns an arc into cubic segments
/// eagerly, and their number grows with the arc's size: its radius, or, when the radius is
/// too small to reach the end point, half the distance to it, which SVG scales the radius up
/// to. A radius of 10^48 made it allocate gigabytes, and an end point 10^35 away ran for
/// minutes, before PLOTSONA saw one segment (F-169). Bounding the radius and both ends of
/// every arc by a billion units, more than any drawing, bounds the work.
const MAX_ARC: f64 = 1e9;

/// Refuses a path with an arc too large to simplify (or not numbers). The current point is
/// tracked through relative commands, so an arc's start is checked as well as its end.
fn check_arcs(d: &str) -> Result<(), Box<Diagnostic>> {
    use svgtypes::PathSegment as S;
    let within = |v: f64| v.abs() <= MAX_ARC;
    let (mut cur, mut start) = ((0.0_f64, 0.0_f64), (0.0_f64, 0.0_f64));
    for segment in svgtypes::PathParser::from(d) {
        let Ok(segment) = segment else { break };
        let to = |abs: bool, x: f64, y: f64| if abs { (x, y) } else { (cur.0 + x, cur.1 + y) };
        let next = match segment {
            S::MoveTo { abs, x, y } => {
                start = to(abs, x, y);
                start
            }
            S::LineTo { abs, x, y }
            | S::CurveTo { abs, x, y, .. }
            | S::SmoothCurveTo { abs, x, y, .. }
            | S::Quadratic { abs, x, y, .. }
            | S::SmoothQuadratic { abs, x, y } => to(abs, x, y),
            S::HorizontalLineTo { abs, x } => (if abs { x } else { cur.0 + x }, cur.1),
            S::VerticalLineTo { abs, y } => (cur.0, if abs { y } else { cur.1 + y }),
            S::ClosePath { .. } => start,
            S::EllipticalArc {
                abs, rx, ry, x, y, ..
            } => {
                let end = to(abs, x, y);
                let ok = within(rx)
                    && within(ry)
                    && [cur.0, cur.1, end.0, end.1].into_iter().all(within);
                if !ok {
                    return Err(Box::new(Diagnostic::new(
                        &codes::MALFORMED,
                        "An SVG path draws an arc more than a billion units across, beyond any \
                         drawing; the file cannot be read.",
                    )));
                }
                end
            }
        };
        cur = next;
    }
    Ok(())
}

fn shape_of(
    node: Node<'_, '_>,
    name: &str,
    style: &Style,
    work: &mut Work,
) -> Result<Option<Shape>, Box<Diagnostic>> {
    if style.fill.is_none() && style.stroke.is_none() {
        return Ok(None);
    }
    let mut local: Vec<Pt> = Vec::new();
    let mut closed = false;
    let mut curved = false;
    let n = |k| number(node, k);
    match name {
        "rect" => {
            let (left, top, width, height) = (n("x"), n("y"), n("width"), n("height"));
            if width <= 0.0 || height <= 0.0 {
                return Ok(None);
            }
            let (right, bottom) = (left + width, top + height);
            local = vec![(left, top), (right, top), (right, bottom), (left, bottom)];
            closed = true;
        }
        "circle" | "ellipse" => {
            let (cx, cy) = (n("cx"), n("cy"));
            let (rx, ry) = if name == "circle" {
                (n("r"), n("r"))
            } else {
                (n("rx"), n("ry"))
            };
            if rx <= 0.0 || ry <= 0.0 {
                return Ok(None);
            }
            local = vec![(cx + rx, cy), (cx, cy + ry), (cx - rx, cy), (cx, cy - ry)];
            closed = true;
            curved = true;
        }
        "line" => local = vec![(n("x1"), n("y1")), (n("x2"), n("y2"))],
        "polyline" | "polygon" => {
            let values: Vec<f64> = node
                .attribute("points")
                .unwrap_or_default()
                .split([' ', ',', '\n', '\t'])
                .filter_map(|v| v.parse().ok())
                .collect();
            local = values
                .as_chunks::<2>()
                .0
                .iter()
                .map(|[x, y]| (*x, *y))
                .collect();
            closed = name == "polygon";
        }
        _ => {
            let d = node.attribute("d").unwrap_or_default();
            work.spend(crate::budget::len64(d.len()))?;
            check_arcs(d)?;
            for segment in SimplifyingPathParser::from(d) {
                let Ok(segment) = segment else { break };
                match segment {
                    SimplePathSegment::MoveTo { x, y } => {
                        local.push((x, y));
                    }
                    SimplePathSegment::LineTo { x, y } => local.push((x, y)),
                    SimplePathSegment::CurveTo { x, y, .. }
                    | SimplePathSegment::Quadratic { x, y, .. } => {
                        curved = true;
                        local.push((x, y));
                    }
                    SimplePathSegment::ClosePath => closed = true,
                }
            }
        }
    }
    if local.is_empty() {
        return Ok(None);
    }
    let local_extent = local
        .iter()
        .fold(0.0_f64, |m, (x, y)| m.max(x.abs()).max(y.abs()));
    let points = local.iter().map(|p| apply(&style.transform, *p)).collect();
    Ok(Some(Shape {
        points,
        closed,
        curved,
        fill: if name == "line" || name == "polyline" {
            None
        } else {
            style.fill
        },
        stroke: style.stroke,
        origin: apply(&style.transform, (0.0, 0.0)),
        local_extent,
        classes: style.classes.clone(),
    }))
}

/// The calibration a person would enter for the chart this SVG draws, rendered at `scale`
/// pixels per unit (see [`vector::hand_calibration`]).
#[cfg(test)]
pub(crate) fn hand_calibration(bytes: &[u8], scale: f64) -> Option<serde_json::Value> {
    let budgets = Budgets::DEFAULT;
    let mut work = Work::new(u64::MAX);
    let document = xml::parse(bytes, "SVG image", &budgets, &mut work).ok()?;
    let mut scene = Scene::default();
    let mut notes = Notes::default();
    walk(
        document.root_element(),
        &Style::default(),
        &mut scene,
        &mut notes,
        &mut work,
    )
    .ok()?;
    vector::hand_calibration(&scene, scale)
}

#[cfg(test)]
mod tests;

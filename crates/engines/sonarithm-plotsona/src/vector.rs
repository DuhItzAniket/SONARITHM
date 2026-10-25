// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The vector tier's shared reader: a [`Scene`] of shapes and texts, from an SVG document
//! (C01-T07) or a PDF page (C01-T08), read into a chart through its [`axes`] and
//! [`marks`].

mod axes;
mod marks;

pub(crate) use marks::read;

/// An RGB colour.
pub(crate) type Rgb = (u8, u8, u8);

/// A point in the document's user space.
pub(crate) type Pt = (f64, f64);

/// A text element: its anchor point, content and classes.
#[derive(Clone, Debug)]
pub(crate) struct Text {
    /// The anchor: where the text is aligned (its middle when the renderer says so).
    pub(crate) at: Pt,
    /// The start and end of the text along its baseline, where left- and right-aligned
    /// labels line up; the anchor when the renderer gives no glyph widths (SVG).
    pub(crate) start: Pt,
    pub(crate) end: Pt,
    pub(crate) content: String,
    pub(crate) classes: String,
    /// Whether the text is rotated (axis titles on a y axis, slanted tick labels).
    pub(crate) rotated: bool,
}

/// A drawn shape in user space.
#[derive(Clone, Debug)]
pub(crate) struct Shape {
    /// Segment end points, in drawing order.
    pub(crate) points: Vec<Pt>,
    pub(crate) closed: bool,
    pub(crate) curved: bool,
    pub(crate) fill: Option<Rgb>,
    pub(crate) stroke: Option<Rgb>,
    /// The shape's local origin in user space: the centre of a symbol drawn about it.
    pub(crate) origin: Pt,
    /// The largest absolute local coordinate: small for a symbol drawn about its origin.
    pub(crate) local_extent: f64,
    pub(crate) classes: String,
}

impl Shape {
    pub(crate) fn bbox(&self) -> (f64, f64, f64, f64) {
        self.points.iter().fold(
            (
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ),
            |(x0, y0, x1, y1), (x, y)| (x0.min(*x), y0.min(*y), x1.max(*x), y1.max(*y)),
        )
    }

    pub(crate) fn center(&self) -> Pt {
        let (x0, y0, x1, y1) = self.bbox();
        (x0.midpoint(x1), y0.midpoint(y1))
    }
}

/// Every shape and text of a document, in drawing order.
#[derive(Debug, Default)]
pub(crate) struct Scene {
    pub(crate) shapes: Vec<Shape>,
    pub(crate) texts: Vec<Text>,
}

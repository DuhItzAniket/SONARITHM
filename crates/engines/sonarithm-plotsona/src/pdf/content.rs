// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A page's content streams as a vector [`Scene`] (ISO 32000-1 §8, §9).
//!
//! The interpreter tracks the graphics state (`q`/`Q`, `cm`, fill and stroke colours in
//! gray, RGB and CMYK), builds paths (`m`, `l`, `c`, `v`, `y`, `h`, `re`) and records them
//! when painted (`S`, `f`, `B` and their variants; `n` only clips and draws nothing), and
//! places text (`BT`, `Tf`, `Td`, `TD`, `Tm`, `T*`, `Tj`, `TJ`, `'`, `"`). Text is decoded
//! through the font's `ToUnicode` map, or as single-byte Latin text for simple fonts,
//! and measured with the font's widths so a label's middle and end are known. Form
//! `XObjects` are drawn with their matrix, to a bounded depth; images are only counted (they
//! are raster content). Coordinates are flipped to y-down, as SVG's, so the shared reader
//! sees one convention.

use std::collections::BTreeMap;

use sonarithm_core::Diagnostic;

use super::object::{File, Lexer, Obj};
use crate::budget::Work;
use crate::vector::{Pt, Rgb, Scene, Shape, Text};

/// How deep Form `XObjects` may nest (cycles end here too).
const MAX_FORM_DEPTH: usize = 8;

/// What a page held besides vector content.
#[derive(Debug, Default)]
pub(crate) struct PageNotes {
    pub(crate) images: usize,
    pub(crate) undecoded: usize,
}

/// An affine matrix `[a b c d e f]`, as PDF writes them.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Matrix([f64; 6]);

impl Matrix {
    pub(crate) const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    /// `self` followed by `outer` (PDF's `self × outer`).
    #[expect(
        clippy::many_single_char_names,
        reason = "the specification's names for matrix entries"
    )]
    fn then(self, outer: Self) -> Self {
        let [a, b, c, d, e, f] = self.0;
        let [a2, b2, c2, d2, e2, f2] = outer.0;
        Self([
            a * a2 + b * c2,
            a * b2 + b * d2,
            c * a2 + d * c2,
            c * b2 + d * d2,
            e * a2 + f * c2 + e2,
            e * b2 + f * d2 + f2,
        ])
    }

    #[expect(
        clippy::many_single_char_names,
        reason = "the specification's names for matrix entries"
    )]
    fn apply(self, (x, y): Pt) -> Pt {
        let [a, b, c, d, e, f] = self.0;
        (a * x + c * y + e, b * x + d * y + f)
    }

    pub(crate) fn from(values: &[f64]) -> Option<Self> {
        let v: [f64; 6] = values.try_into().ok()?;
        v.iter().all(|x| x.is_finite()).then_some(Self(v))
    }
}

#[derive(Clone, Debug)]
struct State {
    ctm: Matrix,
    fill: Rgb,
    stroke: Rgb,
}

/// A font, as far as placing and reading text needs.
#[derive(Clone, Debug, Default)]
struct Font {
    two_byte: bool,
    to_unicode: BTreeMap<u32, String>,
    /// Glyph widths in thousandths of the font size.
    widths: BTreeMap<u32, f64>,
    default_width: f64,
}

impl Font {
    fn codes(&self, bytes: &[u8]) -> Vec<u32> {
        if self.two_byte {
            bytes
                .chunks(2)
                .map(|c| {
                    u32::from(c.first().copied().unwrap_or(0)) << 8
                        | u32::from(c.get(1).copied().unwrap_or(0))
                })
                .collect()
        } else {
            bytes.iter().map(|b| u32::from(*b)).collect()
        }
    }

    fn text(&self, codes: &[u32]) -> String {
        codes
            .iter()
            .map(|c| {
                self.to_unicode.get(c).cloned().unwrap_or_else(|| {
                    if self.two_byte {
                        String::new()
                    } else {
                        char::from_u32(*c).map(String::from).unwrap_or_default()
                    }
                })
            })
            .collect()
    }

    fn width(&self, code: u32) -> f64 {
        self.widths
            .get(&code)
            .copied()
            .unwrap_or(self.default_width)
    }
}

/// Text state inside `BT`…`ET`.
#[derive(Clone, Copy, Debug)]
struct TextState {
    tm: Matrix,
    line: Matrix,
    size: f64,
    leading: f64,
    char_spacing: f64,
    word_spacing: f64,
    scale: f64,
    rise: f64,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            tm: Matrix::IDENTITY,
            line: Matrix::IDENTITY,
            size: 1.0,
            leading: 0.0,
            char_spacing: 0.0,
            word_spacing: 0.0,
            scale: 1.0,
            rise: 0.0,
        }
    }
}

pub(crate) struct Interpreter<'f> {
    file: &'f mut File,
    scene: Scene,
    page_height: f64,
    pub(crate) notes: PageNotes,
    fonts: BTreeMap<u32, Font>,
    /// The start of the last text run and its em size, for joining split words.
    run: Option<(Pt, f64)>,
}

impl<'f> Interpreter<'f> {
    pub(crate) fn new(file: &'f mut File, page_height: f64) -> Self {
        Self {
            file,
            scene: Scene::default(),
            page_height,
            notes: PageNotes::default(),
            fonts: BTreeMap::new(),
            run: None,
        }
    }

    pub(crate) fn finish(self) -> (Scene, PageNotes) {
        (self.scene, self.notes)
    }

    /// Runs content `data` with `resources`, starting from `ctm`.
    #[expect(
        clippy::too_many_lines,
        clippy::many_single_char_names,
        reason = "one operator table, with the specification's operand names"
    )]
    pub(crate) fn run(
        &mut self,
        data: &[u8],
        resources: &BTreeMap<String, Obj>,
        ctm: Matrix,
        depth: usize,
        work: &mut Work,
    ) -> Result<(), Box<Diagnostic>> {
        let mut lexer = Lexer::new(data, 0, 64);
        let mut operands: Vec<Obj> = Vec::new();
        let mut state = State {
            ctm,
            fill: (0, 0, 0),
            stroke: (0, 0, 0),
        };
        let mut stack: Vec<State> = Vec::new();
        let mut path: Vec<Pt> = Vec::new();
        let mut closed = false;
        let mut curved = false;
        let mut text = TextState::default();
        let mut font = Font::default();
        let mut start: Option<Pt> = None;
        while lexer.at < data.len() {
            let before = lexer.at;
            let Ok(token) = lexer.object(0) else {
                if lexer.at == before {
                    lexer.at += 1;
                }
                operands.clear();
                continue;
            };
            work.spend(1)?;
            let Obj::Op(op) = token else {
                if operands.len() < 64 {
                    operands.push(token);
                }
                continue;
            };
            let nums: Vec<f64> = operands.iter().filter_map(Obj::as_f64).collect();
            let n = |i: usize| nums.get(i).copied().unwrap_or(0.0);
            let user = |p: Pt, ctm: Matrix| ctm.apply(p);
            match op.as_str() {
                "q" => {
                    if stack.len() < 64 {
                        stack.push(state.clone());
                    }
                }
                "Q" => {
                    if let Some(s) = stack.pop() {
                        state = s;
                    }
                }
                "cm" => {
                    if let Some(m) = Matrix::from(&nums) {
                        state.ctm = m.then(state.ctm);
                    }
                }
                "g" => state.fill = gray(n(0)),
                "G" => state.stroke = gray(n(0)),
                "rg" => state.fill = rgb(n(0), n(1), n(2)),
                "RG" => state.stroke = rgb(n(0), n(1), n(2)),
                "k" => state.fill = cmyk(n(0), n(1), n(2), n(3)),
                "K" => state.stroke = cmyk(n(0), n(1), n(2), n(3)),
                "sc" | "scn" => state.fill = any_color(&nums).unwrap_or(state.fill),
                "SC" | "SCN" => state.stroke = any_color(&nums).unwrap_or(state.stroke),
                "m" => {
                    let p = user((n(0), n(1)), state.ctm);
                    start = Some(p);
                    path.push(p);
                }
                "l" => path.push(user((n(0), n(1)), state.ctm)),
                "c" => {
                    curved = true;
                    path.push(user((n(4), n(5)), state.ctm));
                }
                "v" | "y" => {
                    curved = true;
                    path.push(user((n(2), n(3)), state.ctm));
                }
                "h" => {
                    closed = true;
                    if let Some(s) = start {
                        path.push(s);
                    }
                }
                "re" => {
                    let (x, y, w, h) = (n(0), n(1), n(2), n(3));
                    let corners = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
                    path.extend(corners.iter().map(|p| user(*p, state.ctm)));
                    closed = true;
                }
                "S" | "s" | "f" | "F" | "f*" | "B" | "B*" | "b" | "b*" | "n" => {
                    let fills = matches!(op.as_str(), "f" | "F" | "f*" | "B" | "B*" | "b" | "b*");
                    let strokes = matches!(op.as_str(), "S" | "s" | "B" | "B*" | "b" | "b*");
                    if op.starts_with('s') || op.starts_with('b') {
                        closed = true;
                    }
                    if (fills || strokes) && !path.is_empty() {
                        self.shape(
                            &path,
                            closed || fills,
                            curved,
                            fills.then_some(state.fill),
                            strokes.then_some(state.stroke),
                        );
                    }
                    path.clear();
                    closed = false;
                    curved = false;
                    start = None;
                }
                "BT" => text = TextState::default(),
                "Tf" => {
                    text.size = n(0);
                    if let Some(Obj::Name(name)) = operands.first() {
                        font = self.font(resources, name);
                    }
                }
                "TL" => text.leading = n(0),
                "Tc" => text.char_spacing = n(0),
                "Tw" => text.word_spacing = n(0),
                "Tz" => text.scale = n(0) / 100.0,
                "Ts" => text.rise = n(0),
                "Td" | "TD" => {
                    if op == "TD" {
                        text.leading = -n(1);
                    }
                    text.line = Matrix([1.0, 0.0, 0.0, 1.0, n(0), n(1)]).then(text.line);
                    text.tm = text.line;
                }
                "Tm" => {
                    if let Some(m) = Matrix::from(&nums) {
                        text.line = m;
                        text.tm = m;
                    }
                }
                "T*" => {
                    text.line = Matrix([1.0, 0.0, 0.0, 1.0, 0.0, -text.leading]).then(text.line);
                    text.tm = text.line;
                }
                "Tj" | "'" | "\"" | "TJ" => {
                    if op == "'" || op == "\"" {
                        text.line =
                            Matrix([1.0, 0.0, 0.0, 1.0, 0.0, -text.leading]).then(text.line);
                        text.tm = text.line;
                    }
                    let mut pieces: Vec<(Vec<u8>, f64)> = Vec::new();
                    for operand in &operands {
                        match operand {
                            Obj::Str(s) => pieces.push((s.clone(), 0.0)),
                            Obj::Array(items) => {
                                for item in items {
                                    match item {
                                        Obj::Str(s) => pieces.push((s.clone(), 0.0)),
                                        other => {
                                            if let (Some(adjust), Some(last)) =
                                                (other.as_f64(), pieces.last_mut())
                                            {
                                                last.1 += adjust;
                                            } else if let Some(adjust) = other.as_f64() {
                                                pieces.push((Vec::new(), adjust));
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    self.show(&pieces, &font, &mut text, state.ctm);
                }
                "Do" => {
                    if let Some(Obj::Name(name)) = operands.first() {
                        self.xobject(resources, name, state.ctm, depth, work)?;
                    }
                }
                _ => {}
            }
            operands.clear();
        }
        Ok(())
    }

    fn to_screen(&self, (x, y): Pt) -> Pt {
        (x, self.page_height - y)
    }

    fn shape(
        &mut self,
        path: &[Pt],
        closed: bool,
        curved: bool,
        fill: Option<Rgb>,
        stroke: Option<Rgb>,
    ) {
        let mut points: Vec<Pt> = path.iter().map(|p| self.to_screen(*p)).collect();
        if closed && points.len() > 2 && points.first() == points.last() {
            points.pop();
        }
        let origin = points.first().copied().unwrap_or((0.0, 0.0));
        self.scene.shapes.push(Shape {
            points,
            closed,
            curved,
            fill,
            stroke,
            origin,
            local_extent: f64::INFINITY,
            classes: String::new(),
        });
    }

    /// Places one text showing: its start, middle and end along the baseline, in text
    /// space through the text matrix and the CTM (ISO 32000-1 §9.4.4).
    fn show(&mut self, pieces: &[(Vec<u8>, f64)], font: &Font, text: &mut TextState, ctm: Matrix) {
        let m = text.tm.then(ctm);
        let mut content = String::new();
        let mut advance = 0.0;
        for (bytes, adjust) in pieces {
            let codes = font.codes(bytes);
            for code in &codes {
                advance += font.width(*code) / 1000.0 * text.size + text.char_spacing;
                if *code == 32 && !font.two_byte {
                    advance += text.word_spacing;
                }
            }
            advance -= adjust / 1000.0 * text.size;
            content.push_str(&font.text(&codes));
        }
        let advance = advance * text.scale;
        text.tm = Matrix([1.0, 0.0, 0.0, 1.0, advance, 0.0]).then(text.tm);
        let content = content.split_whitespace().collect::<Vec<_>>().join(" ");
        if content.is_empty() {
            return;
        }
        let rise = text.rise;
        let point = |d: f64| m.apply((d, rise));
        let (start, end) = (self.to_screen(point(0.0)), self.to_screen(point(advance)));
        let origin = m.apply((0.0, 0.0));
        let em = {
            let up = m.apply((0.0, text.size));
            (up.0 - origin.0).hypot(up.1 - origin.1)
        };
        // Exporters often show a word in pieces (a glyph per showing, or a kerned split):
        // a showing that starts where the last one ended, on its baseline, extends it.
        if let (Some(run), Some(last)) = (self.run.as_mut(), self.scene.texts.last_mut()) {
            let (dx, dy) = (last.end.0 - run.0.0, last.end.1 - run.0.1);
            let length = dx.hypot(dy).max(1e-9);
            let (ux, uy) = (dx / length, dy / length);
            let (gx, gy) = (start.0 - last.end.0, start.1 - last.end.1);
            let along = gx * ux + gy * uy;
            let across = (gx * uy - gy * ux).abs();
            let same_line = across <= run.1 * 0.2 && along >= -run.1 * 0.3 && along <= run.1 * 0.6;
            let aligned = length < 1e-6 || ((end.0 - start.0) * ux + (end.1 - start.1) * uy) >= 0.0;
            if same_line && aligned {
                if along > run.1 * 0.25 {
                    last.content.push(' ');
                }
                last.content.push_str(&content);
                last.end = end;
                if !last.rotated {
                    last.at = (run.0.0.midpoint(end.0), run.0.1.midpoint(end.1));
                }
                return;
            }
        }
        let middle = self.to_screen(point(advance / 2.0));
        let rotated = (start.1 - end.1).abs() > (start.0 - end.0).abs();
        self.run = Some((start, em));
        self.scene.texts.push(Text {
            at: if rotated { start } else { middle },
            start,
            end,
            content,
            classes: String::new(),
            rotated,
        });
    }

    fn font(&mut self, resources: &BTreeMap<String, Obj>, name: &str) -> Font {
        let Some(Obj::Ref(number)) = resources
            .get("Font")
            .map(|f| self.file.resolve(f))
            .and_then(Obj::dict)
            .and_then(|d| d.get(name))
            .cloned()
        else {
            return self.load_font(resources, name);
        };
        if let Some(f) = self.fonts.get(&number) {
            return f.clone();
        }
        let font = self.load_font(resources, name);
        self.fonts.insert(number, font.clone());
        font
    }

    fn load_font(&mut self, resources: &BTreeMap<String, Obj>, name: &str) -> Font {
        let Some(dict) = resources
            .get("Font")
            .map(|f| self.file.resolve(f).clone())
            .and_then(|f| f.dict().and_then(|d| d.get(name)).cloned())
            .map(|f| self.file.resolve(&f).clone())
            .and_then(|f| f.dict().cloned())
        else {
            return Font {
                default_width: 500.0,
                ..Font::default()
            };
        };
        let mut font = Font {
            two_byte: dict.get("Subtype").and_then(Obj::as_name) == Some("Type0"),
            default_width: 500.0,
            ..Font::default()
        };
        if let Some(stream) = dict.get("ToUnicode").map(|o| self.file.resolve(o).clone())
            && let Ok(Some(cmap)) = self.file.decode(&stream, &mut Work::new(1 << 26))
        {
            font.to_unicode = parse_cmap(&cmap);
        }
        if font.two_byte {
            let descendant = dict
                .get("DescendantFonts")
                .map(|d| self.file.resolve(d).clone())
                .and_then(|d| match d {
                    Obj::Array(a) => a.first().map(|f| self.file.resolve(f).clone()),
                    _ => None,
                })
                .and_then(|d| d.dict().cloned())
                .unwrap_or_default();
            font.default_width = descendant.get("DW").and_then(Obj::as_f64).unwrap_or(1000.0);
            if let Some(Obj::Array(w)) = descendant.get("W").map(|w| self.file.resolve(w).clone()) {
                font.widths = cid_widths(&w);
            }
        } else {
            let first = dict.get("FirstChar").and_then(Obj::as_f64).unwrap_or(0.0);
            if let Some(Obj::Array(w)) = dict.get("Widths").map(|w| self.file.resolve(w).clone()) {
                for (i, v) in w.iter().enumerate() {
                    if let (Some(v), Ok(i)) = (self.file.resolve(v).as_f64(), u32::try_from(i)) {
                        #[expect(
                            clippy::cast_possible_truncation,
                            clippy::cast_sign_loss,
                            reason = "char codes"
                        )]
                        let code = first.max(0.0) as u32 + i;
                        font.widths.insert(code, v);
                    }
                }
            }
        }
        font
    }

    fn xobject(
        &mut self,
        resources: &BTreeMap<String, Obj>,
        name: &str,
        ctm: Matrix,
        depth: usize,
        work: &mut Work,
    ) -> Result<(), Box<Diagnostic>> {
        let Some(stream) = resources
            .get("XObject")
            .map(|x| self.file.resolve(x).clone())
            .and_then(|x| x.dict().and_then(|d| d.get(name)).cloned())
            .map(|x| self.file.resolve(&x).clone())
        else {
            return Ok(());
        };
        let Some(dict) = stream.dict().cloned() else {
            return Ok(());
        };
        match dict.get("Subtype").and_then(Obj::as_name) {
            Some("Image") => self.notes.images += 1,
            Some("Form") if depth < MAX_FORM_DEPTH => {
                let matrix = dict
                    .get("Matrix")
                    .map(|m| self.file.resolve(m).clone())
                    .and_then(|m| match m {
                        Obj::Array(a) => {
                            Matrix::from(&a.iter().filter_map(Obj::as_f64).collect::<Vec<_>>())
                        }
                        _ => None,
                    })
                    .unwrap_or(Matrix::IDENTITY);
                let own = dict
                    .get("Resources")
                    .map(|r| self.file.resolve(r).clone())
                    .and_then(|r| r.dict().cloned())
                    .unwrap_or_else(|| resources.clone());
                match self.file.decode(&stream, work)? {
                    Some(data) => self.run(&data, &own, matrix.then(ctm), depth + 1, work)?,
                    None => self.notes.undecoded += 1,
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn channel(v: f64) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped to 0..=255"
    )]
    let c = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    c
}

fn gray(v: f64) -> Rgb {
    (channel(v), channel(v), channel(v))
}

fn rgb(r: f64, g: f64, b: f64) -> Rgb {
    (channel(r), channel(g), channel(b))
}

fn cmyk(c: f64, m: f64, y: f64, k: f64) -> Rgb {
    rgb(
        (1.0 - c) * (1.0 - k),
        (1.0 - m) * (1.0 - k),
        (1.0 - y) * (1.0 - k),
    )
}

fn any_color(values: &[f64]) -> Option<Rgb> {
    match values {
        [g] => Some(gray(*g)),
        [r, g, b] => Some(rgb(*r, *g, *b)),
        [c, m, y, k] => Some(cmyk(*c, *m, *y, *k)),
        _ => None,
    }
}

/// `/W` arrays: `c [w1 w2 …]` or `c_first c_last w`.
fn cid_widths(w: &[Obj]) -> BTreeMap<u32, f64> {
    let mut out = BTreeMap::new();
    let mut i = 0;
    while let Some(first) = w.get(i).and_then(Obj::as_f64) {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "CIDs"
        )]
        let first = first.max(0.0) as u32;
        match w.get(i + 1) {
            Some(Obj::Array(list)) => {
                for (k, v) in list.iter().enumerate() {
                    if let (Some(v), Ok(k)) = (v.as_f64(), u32::try_from(k)) {
                        out.insert(first + k, v);
                    }
                }
                i += 2;
            }
            Some(last) => {
                let (Some(last), Some(v)) = (last.as_f64(), w.get(i + 2).and_then(Obj::as_f64))
                else {
                    break;
                };
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "CIDs"
                )]
                let last = (last.max(0.0) as u32).min(first.saturating_add(65_535));
                for c in first..=last {
                    out.insert(c, v);
                }
                i += 3;
            }
            None => break,
        }
    }
    out
}

/// A `ToUnicode` map's `bfchar` and `bfrange` entries.
fn parse_cmap(data: &[u8]) -> BTreeMap<u32, String> {
    let mut out = BTreeMap::new();
    let mut lexer = Lexer::new(data, 0, 8);
    let mut tokens = Vec::new();
    while lexer.at < data.len() && tokens.len() < 100_000 {
        let before = lexer.at;
        match lexer.object(0) {
            Ok(t) => tokens.push(t),
            Err(()) if lexer.at == before => lexer.at += 1,
            Err(()) => {}
        }
    }
    let code = |o: &Obj| match o {
        Obj::Str(b) => Some(b.iter().fold(0_u32, |a, x| a << 8 | u32::from(*x))),
        _ => None,
    };
    let utf16 = |o: &Obj| match o {
        Obj::Str(b) => {
            let units: Vec<u16> = b
                .chunks(2)
                .map(|c| {
                    u16::from(c.first().copied().unwrap_or(0)) << 8
                        | u16::from(c.get(1).copied().unwrap_or(0))
                })
                .collect();
            Some(String::from_utf16_lossy(&units))
        }
        _ => None,
    };
    let mut i = 0;
    let mut mode = "";
    while let Some(t) = tokens.get(i) {
        match t {
            Obj::Op(op) if op == "beginbfchar" => mode = "char",
            Obj::Op(op) if op == "beginbfrange" => mode = "range",
            Obj::Op(op) if op.starts_with("end") => mode = "",
            _ if mode == "char" => {
                if let (Some(c), Some(u)) = (code(t), tokens.get(i + 1).and_then(utf16)) {
                    out.insert(c, u);
                    i += 1;
                }
            }
            _ if mode == "range" => {
                if let (Some(lo), Some(hi)) = (code(t), tokens.get(i + 1).and_then(code)) {
                    let hi = hi.min(lo.saturating_add(65_535));
                    match tokens.get(i + 2) {
                        Some(Obj::Array(targets)) => {
                            for (k, target) in (lo..=hi).zip(targets) {
                                if let Some(u) = utf16(target) {
                                    out.insert(k, u);
                                }
                            }
                        }
                        Some(start) => {
                            if let Some(base) = utf16(start) {
                                let mut units: Vec<u16> = base.encode_utf16().collect();
                                for k in lo..=hi {
                                    out.insert(k, String::from_utf16_lossy(&units));
                                    if let Some(last) = units.last_mut() {
                                        *last = last.wrapping_add(1);
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                    i += 2;
                }
            }
            _ => {}
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmaps_and_width_arrays_read() {
        let cmap = b"2 beginbfchar\n<0011> <002E>\n<0018> <0035>\nendbfchar\n1 beginbfrange\n<0013> <0016> <0030>\nendbfrange";
        let map = parse_cmap(cmap);
        assert_eq!(map.get(&0x11).map(String::as_str), Some("."));
        assert_eq!(map.get(&0x15).map(String::as_str), Some("2"));
        let w = cid_widths(&[
            Obj::Int(17),
            Obj::Array(vec![Obj::Real(363.0)]),
            Obj::Int(19),
            Obj::Int(24),
            Obj::Real(635.0),
        ]);
        assert_eq!(w.get(&17), Some(&363.0));
        assert_eq!(w.get(&22), Some(&635.0));
    }
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Vector PDF charts: tier 2, `vector` (10 §3, notes/06 §1 and §6).
//!
//! Following the approach `ChartDetective` published (paths and text runs from the content
//! stream, axes from tick labels, marks inverted through the calibration), each page's
//! content becomes a vector scene ([`content`]) and the shared reader of the SVG tier
//! reads it. The chart region is the page: the first page whose scene holds a readable
//! chart is used, and later pages are reported (`SNR-ENG-340`). A page with no vector
//! content (only images) needs the raster tier, which is reported as such
//! (`SNR-ENG-364`). Objects are read by PLOTSONA's own reader ([`object`], F-160) under
//! the input budgets; encrypted files are refused.

mod content;
mod object;

use std::collections::BTreeMap;

use sonarithm_core::Diagnostic;

use crate::budget::{Budgets, Work};
use crate::codes;
use crate::model::ChartData;
use crate::vector;
use content::{Interpreter, Matrix};
use object::{File, Obj};

/// How deep the page tree is walked (cycles end here too).
const MAX_PAGE_DEPTH: usize = 32;

/// Extracts the first chart of a PDF.
pub(crate) fn extract(
    bytes: &[u8],
    budgets: &Budgets,
    work: &mut Work,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<ChartData, Box<Diagnostic>> {
    let mut file = File::parse(bytes, budgets, work)?;
    let pages = pages(&file);
    if pages.is_empty() {
        return Err(Box::new(Diagnostic::new(
            &codes::NO_CHART,
            "The PDF has no pages.",
        )));
    }
    let mut first_error: Option<Box<Diagnostic>> = None;
    let mut found: Option<(usize, ChartData, Vec<Diagnostic>)> = None;
    let mut readable = 0;
    let mut images_only = 0;
    for (index, page) in pages.iter().enumerate() {
        let (scene, notes) = page_scene(&mut file, page, work)?;
        if scene.shapes.is_empty() && scene.texts.is_empty() {
            if notes.images > 0 {
                images_only += 1;
            }
            continue;
        }
        let mut page_diagnostics = Vec::new();
        match vector::read(&scene, budgets, work, &mut page_diagnostics) {
            Ok(chart) => {
                readable += 1;
                if found.is_none() {
                    found = Some((index, chart, page_diagnostics));
                }
            }
            Err(d) => {
                first_error.get_or_insert(d);
            }
        }
    }
    if let Some((index, chart, page_diagnostics)) = found {
        diagnostics.extend(page_diagnostics);
        if readable > 1 {
            diagnostics.push(Diagnostic::new(
                &codes::OTHER_CHARTS,
                format!("{readable} pages hold charts; page {} was read.", index + 1),
            ));
        }
        return Ok(chart);
    }
    if images_only > 0 && first_error.is_none() {
        return Err(Box::new(Diagnostic::new(
            &codes::PDF_NO_VECTOR,
            "The PDF's pages hold only images, no vector drawing: the chart needs the raster \
             tier.",
        )));
    }
    Err(first_error.unwrap_or_else(|| {
        Box::new(Diagnostic::new(
            &codes::NO_CHART,
            "The PDF's pages draw no chart.",
        ))
    }))
}

/// The pages in reading order: through the catalog's page tree, else every `/Page` object.
fn pages(file: &File) -> Vec<BTreeMap<String, Obj>> {
    let mut out = Vec::new();
    if let Some(root) = file.get(&file.trailer, "Root").and_then(Obj::dict)
        && let Some(tree) = file.get(root, "Pages")
    {
        walk(file, tree, &BTreeMap::new(), &mut out, 0);
    }
    if out.is_empty() {
        out = file
            .of_type("Page")
            .into_iter()
            .filter_map(|p| p.dict().cloned())
            .collect();
    }
    out
}

/// Inherited page attributes (resources, media box) flow from the tree to the leaves.
fn walk(
    file: &File,
    node: &Obj,
    inherited: &BTreeMap<String, Obj>,
    out: &mut Vec<BTreeMap<String, Obj>>,
    depth: usize,
) {
    if depth > MAX_PAGE_DEPTH || out.len() > 10_000 {
        return;
    }
    let Some(dict) = node.dict() else { return };
    let mut here = inherited.clone();
    for key in ["Resources", "MediaBox"] {
        if let Some(v) = dict.get(key) {
            here.insert(key.to_owned(), v.clone());
        }
    }
    match dict.get("Type").and_then(Obj::as_name) {
        Some("Page") => {
            let mut page = dict.clone();
            for (k, v) in here {
                page.entry(k).or_insert(v);
            }
            out.push(page);
        }
        _ => {
            if let Some(Obj::Array(kids)) = file.get(dict, "Kids") {
                for kid in kids {
                    walk(file, file.resolve(kid), &here, out, depth + 1);
                }
            }
        }
    }
}

fn page_scene(
    file: &mut File,
    page: &BTreeMap<String, Obj>,
    work: &mut Work,
) -> Result<(vector::Scene, content::PageNotes), Box<Diagnostic>> {
    let media: Vec<f64> = match file.get(page, "MediaBox") {
        Some(Obj::Array(a)) => a.iter().filter_map(|v| file.resolve(v).as_f64()).collect(),
        _ => Vec::new(),
    };
    let height = match media.as_slice() {
        [_, y0, _, y1] => y1 - y0,
        _ => 792.0,
    };
    let resources = file
        .get(page, "Resources")
        .and_then(Obj::dict)
        .cloned()
        .unwrap_or_default();
    let streams: Vec<Obj> = match file.get(page, "Contents") {
        Some(Obj::Array(parts)) => parts.iter().map(|p| file.resolve(p).clone()).collect(),
        Some(stream @ Obj::Stream(..)) => vec![stream.clone()],
        _ => Vec::new(),
    };
    let mut data = Vec::new();
    let mut undecoded = 0;
    for stream in &streams {
        match file.decode(stream, work)? {
            Some(bytes) => {
                data.extend(bytes);
                data.push(b'\n');
            }
            None => undecoded += 1,
        }
    }
    let mut interpreter = Interpreter::new(file, height);
    interpreter.run(&data, &resources, Matrix::IDENTITY, 0, work)?;
    let (scene, mut notes) = interpreter.finish();
    notes.undecoded += undecoded;
    Ok((scene, notes))
}

#[cfg(test)]
mod tests;

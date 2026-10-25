// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Vector PDFs of the corpus charts extract within 0.5 % (C01-T08 acceptance), with
/// `vector` provenance.
#[test]
fn corpus_charts_extract_within_half_a_percent() {
    let checked = crate::corpus::check(&crate::corpus::folder("pdf"), 0.005);
    assert_eq!(checked.len(), 9, "the PDF corpus has 9 charts");
    for (name, error) in &checked {
        println!("{name}: {:.4} %", error * 100.0);
    }
}

/// Prints what each corpus PDF gives (run with `--ignored --nocapture` while developing).
#[test]
#[ignore = "a development aid"]
fn show_corpus() {
    for entry in std::fs::read_dir(crate::corpus::folder("pdf")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "pdf") {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        match crate::extract(&bytes, "x.pdf", &crate::ExtractOptions::new()) {
            Ok(e) => {
                println!(
                    "== {} {:?} {:?}",
                    path.file_name().unwrap().to_string_lossy(),
                    e.chart.chart_type,
                    e.chart.title
                );
                for d in &e.report.diagnostics {
                    println!("   ! {} {}", d.code.as_str(), d.message);
                }
                for s in &e.chart.series {
                    let pts: Vec<String> = s
                        .points
                        .iter()
                        .take(5)
                        .map(|p| format!("{:?}={:.3}", p.x, p.y.unwrap_or(f64::NAN)))
                        .collect();
                    println!(
                        "   {} n={} res={:?} {}",
                        s.name,
                        s.points.len(),
                        s.residual,
                        pts.join(" ")
                    );
                }
            }
            Err(d) => println!(
                "== {} ERR {} {}",
                path.file_name().unwrap().to_string_lossy(),
                d.code.as_str(),
                d.message
            ),
        }
    }
}

use std::fmt::Write as _;
use std::io::Write as _;

/// A PDF of numbered objects, with a correct trailer; `streams` are (number, dictionary
/// entries, data) and are written with their length.
fn pdf(objects: &[(u32, &str)], streams: &[(u32, &str, &[u8])]) -> Vec<u8> {
    let mut out = b"%PDF-1.7\n".to_vec();
    for (n, body) in objects {
        let _ = writeln!(out, "{n} 0 obj {body} endobj");
    }
    for (n, dict, data) in streams {
        let _ = write!(
            out,
            "{n} 0 obj << {dict} /Length {} >>\nstream\n",
            data.len()
        );
        out.extend_from_slice(data);
        out.extend_from_slice(b"\nendstream\nendobj\n");
    }
    out.extend_from_slice(b"trailer << /Root 1 0 R >>\n%%EOF\n");
    out
}

const CATALOG: (u32, &str) = (1, "<< /Type /Catalog /Pages 2 0 R >>");
const PAGES: (u32, &str) = (2, "<< /Type /Pages /Kids [3 0 R] /Count 1 >>");

fn page(resources: &str) -> String {
    format!(
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 300] /Contents 4 0 R /Resources << {resources} >> >>"
    )
}

fn run(bytes: &[u8]) -> Result<crate::Extraction, Box<sonarithm_core::Diagnostic>> {
    crate::extract(bytes, "chart.pdf", &crate::ExtractOptions::new())
}

/// A bar chart drawn with PDF operators and a standard font: two bars over ticks 0, 5, 10.
fn bar_chart_content() -> String {
    let mut c = String::from("0 0 0 RG 0 0 0 rg\n");
    // Axes and ticks (PDF y up: the x axis at y = 50, 100 units per 5).
    c.push_str("50 50 m 250 50 l S\n");
    for (x, label) in [(100.0, "a"), (200.0, "b")] {
        let _ = write!(
            c,
            "{x} 50 m {x} 44 l S\nBT /F1 10 Tf {} 34 Td ({label}) Tj ET\n",
            x - 2.78
        );
    }
    for (y, label) in [(50.0, "0"), (150.0, "5"), (250.0, "10")] {
        let _ = write!(
            c,
            "50 {y} m 44 {y} l S\nBT /F1 10 Tf 30 {} Td ({label}) Tj ET\n",
            y - 3.0
        );
    }
    c.push_str("0.27 0.51 0.71 rg 80 50 40 140 re f\n180 50 40 40 re f\n");
    c
}

#[test]
fn a_hand_drawn_pdf_bar_chart_reads_with_vector_provenance() {
    let content = bar_chart_content();
    let bytes = pdf(
        &[
            CATALOG,
            PAGES,
            (3, &page("/Font << /F1 5 0 R >>")),
            (5, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"),
        ],
        &[(4, "", content.as_bytes())],
    );
    let read = run(&bytes).unwrap();
    assert_eq!(read.report.extractor, "sonarithm-plotsona/pdf-vector");
    let series = &read.chart.series[0];
    assert_eq!(
        series.provenance.class,
        sonarithm_core::ProvenanceClass::Vector
    );
    let values: Vec<(String, f64)> = series
        .points
        .iter()
        .map(|p| match &p.x {
            crate::XValue::Category(c) => (c.clone(), (p.y.unwrap() * 1e9).round() / 1e9),
            other => (format!("{other:?}"), f64::NAN),
        })
        .collect();
    assert_eq!(values, [("a".to_owned(), 7.0), ("b".to_owned(), 2.0)]);
}

/// Recursive and oversized PDFs fail within budget (C01-T08 acceptance).
#[test]
fn recursive_and_oversized_pdfs_fail_within_budget() {
    // A Form XObject that draws itself, and a page tree that contains itself.
    let looping = pdf(
        &[
            CATALOG,
            (2, "<< /Type /Pages /Kids [3 0 R 2 0 R] /Count 1 >>"),
            (3, &page("/XObject << /X1 5 0 R >>")),
        ],
        &[
            (4, "", b"/X1 Do"),
            (
                5,
                "/Type /XObject /Subtype /Form /Resources << /XObject << /X1 5 0 R >> >>",
                b"0 0 m 10 10 l S /X1 Do",
            ),
        ],
    );
    let mut budgets = crate::Budgets::DEFAULT;
    budgets.max_work = 1_000_000;
    let options = crate::ExtractOptions::new().with_budgets(budgets);
    let err = crate::extract(&looping, "loop.pdf", &options).unwrap_err();
    assert!(
        err.code.as_str().starts_with("SNR-ENG-3"),
        "{}",
        err.code.as_str()
    );
    // A content stream that inflates far past its size: a Flate bomb.
    let mut zeros = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    zeros.write_all(&vec![b' '; 8 << 20]).unwrap();
    let bomb = zeros.finish().unwrap();
    let bytes = pdf(
        &[CATALOG, PAGES, (3, &page(""))],
        &[(4, "/Filter /FlateDecode", &bomb)],
    );
    assert_eq!(run(&bytes).unwrap_err().code.as_str(), "SNR-ENG-302");
    // Larger than the input budget.
    let mut small = crate::Budgets::DEFAULT;
    small.max_bytes = 100;
    let options = crate::ExtractOptions::new().with_budgets(small);
    assert_eq!(
        crate::extract(&bytes, "big.pdf", &options)
            .unwrap_err()
            .code
            .as_str(),
        "SNR-ENG-301"
    );
}

#[test]
fn encrypted_pdfs_are_refused() {
    let mut bytes = pdf(&[CATALOG, PAGES, (3, &page(""))], &[(4, "", b"")]);
    let at = bytes.len() - b"trailer << /Root 1 0 R >>\n%%EOF\n".len();
    bytes.truncate(at);
    bytes.extend_from_slice(b"trailer << /Root 1 0 R /Encrypt 9 0 R >>\n%%EOF\n");
    assert_eq!(run(&bytes).unwrap_err().code.as_str(), "SNR-ENG-311");
}

/// A page with only an image needs the raster tier, and says so (C01-T08 acceptance).
#[test]
fn an_image_only_pdf_reports_the_raster_fallback() {
    let bytes = pdf(
        &[CATALOG, PAGES, (3, &page("/XObject << /Im1 5 0 R >>"))],
        &[
            (4, "", b"q 200 0 0 200 50 50 cm /Im1 Do Q"),
            (
                5,
                "/Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8",
                b"\x80",
            ),
        ],
    );
    let err = run(&bytes).unwrap_err();
    assert_eq!(err.code.as_str(), "SNR-ENG-364");
    assert!(err.message.contains("raster"));
}

/// Objects inside an object stream are read.
#[test]
fn object_streams_are_read() {
    let content = bar_chart_content();
    let inner = "5 0 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>";
    let objstm = format!("/Type /ObjStm /N 1 /First {}", "5 0 ".len());
    let bytes = pdf(
        &[CATALOG, PAGES, (3, &page("/Font << /F1 5 0 R >>"))],
        &[(4, "", content.as_bytes()), (6, &objstm, inner.as_bytes())],
    );
    assert_eq!(run(&bytes).unwrap().chart.points(), 2);
}

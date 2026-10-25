// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/// SVG renders of the corpus charts from Vega-Lite, Plotly and D3 extract within 0.5 %
/// (C01-T07 acceptance), with `vector` provenance.
#[test]
fn corpus_charts_extract_within_half_a_percent() {
    let checked = crate::corpus::check(&crate::corpus::folder("svg"), 0.005);
    assert_eq!(checked.len(), 12, "the SVG corpus has 12 charts");
    for (name, error) in &checked {
        println!("{name}: {:.4} %", error * 100.0);
    }
}

/// Prints what each corpus SVG gives (run with `--ignored --nocapture` while developing).
#[test]
#[ignore = "a development aid"]
fn show_corpus() {
    for entry in std::fs::read_dir(crate::corpus::folder("svg")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "svg") {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        match crate::extract(&bytes, "x.svg", &crate::ExtractOptions::new()) {
            Ok(e) => {
                println!(
                    "== {} {:?} {:?}",
                    path.display(),
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
                        .take(6)
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
                path.display(),
                d.code.as_str(),
                d.message
            ),
        }
    }
}

fn run(svg: &str) -> Result<crate::Extraction, Box<sonarithm_core::Diagnostic>> {
    crate::extract(svg.as_bytes(), "chart.svg", &crate::ExtractOptions::new())
}

/// A small bar chart: x categories a and b, y ticks 0 to 10 at 100 units per 5.
fn bar_chart(extra: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 300">{extra}
<g transform="translate(50,20)">
<line x1="0" y1="200" x2="200" y2="200" stroke="black"/>
<line x1="50" y1="200" x2="50" y2="206" stroke="black"/><text x="50" y="220">a</text>
<line x1="150" y1="200" x2="150" y2="206" stroke="black"/><text x="150" y="220">b</text>
<line x1="0" y1="200" x2="-6" y2="200" stroke="black"/><text x="-9" y="203">0</text>
<line x1="0" y1="100" x2="-6" y2="100" stroke="black"/><text x="-9" y="103">5</text>
<line x1="0" y1="0" x2="-6" y2="0" stroke="black"/><text x="-9" y="3">10</text>
<rect x="30" y="60" width="40" height="140" fill="steelblue"/>
<rect x="130" y="160" width="40" height="40" fill="steelblue"/>
</g></svg>"#
    )
}

#[test]
fn a_hand_drawn_bar_chart_reads_exactly_with_vector_provenance() {
    let read = run(&bar_chart("")).unwrap();
    let series = &read.chart.series[0];
    assert_eq!(
        series.provenance.class,
        sonarithm_core::ProvenanceClass::Vector
    );
    assert_eq!(read.report.extractor, "sonarithm-plotsona/svg");
    let values: Vec<(String, f64)> = series
        .points
        .iter()
        .map(|p| match &p.x {
            crate::XValue::Category(c) => (c.clone(), p.y.unwrap()),
            other => (format!("{other:?}"), f64::NAN),
        })
        .collect();
    assert_eq!(values, [("a".to_owned(), 7.0), ("b".to_owned(), 2.0)]);
    assert_eq!(series.residual, Some(0.0));
}

/// External references and scripts are refused: never loaded or run, and reported
/// (C01-T07 acceptance).
#[test]
fn external_references_and_scripts_are_refused() {
    let hostile = bar_chart(
        r#"<script>alert(1)</script><script href="https://example.invalid/x.js"/>
<image href="https://example.invalid/tracker.png" width="1" height="1"/>
<use href="other.svg#bars"/><foreignObject><div>html</div></foreignObject>"#,
    );
    let read = run(&hostile).unwrap();
    let codes: Vec<&str> = read
        .report
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(codes, ["SNR-ENG-361", "SNR-ENG-360"]);
    assert_eq!(read.chart.points(), 2, "the chart itself is still read");
}

#[test]
fn an_svg_without_axes_says_so() {
    let svg =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><rect x="1" y="1" width="5" height="9"/></svg>"#;
    assert_eq!(run(svg).unwrap_err().code.as_str(), "SNR-ENG-362");
}

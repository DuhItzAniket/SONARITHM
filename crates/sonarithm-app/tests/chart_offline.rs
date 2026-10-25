// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Chart files that point at the network are read without connecting (C01-T06, OD-25,
//! ADR-0012). `cargo xtask network-deny` runs this executable under the OS mechanism that
//! records every outbound connection attempt, so a fetch here fails CI.

use sonarithm_plotsona::{ExtractOptions, extract};

fn code(bytes: &[u8]) -> String {
    match extract(bytes, "chart", &ExtractOptions::new()) {
        Ok(_) => "ok".to_owned(),
        Err(diagnostic) => diagnostic.code.as_str().to_owned(),
    }
}

#[test]
fn chart_sources_stay_offline() {
    // A Vega-Lite spec whose data is a URL: reported, not fetched.
    let remote = br#"{"$schema": "https://vega.github.io/schema/vega-lite/v6.json",
        "data": {"url": "https://example.com/data.json"}, "mark": "line",
        "encoding": {"x": {"field": "x"}, "y": {"field": "y"}}}"#;
    assert_eq!(code(remote), "SNR-ENG-346");
    // A Vega spec with a remote data set beside an inline one: the inline one is read.
    let vega = br#"{"$schema": "https://vega.github.io/schema/vega/v6.json",
        "data": [{"name": "r", "url": "https://example.com/r.csv"},
                 {"name": "t", "values": [{"x": 1, "y": 2}]}],
        "marks": [{"type": "symbol", "from": {"data": "r"}, "encode": {"enter": {"x": {"field": "x"}, "y": {"field": "y"}}}},
                  {"type": "symbol", "from": {"data": "t"}, "encode": {"enter": {"x": {"field": "x"}, "y": {"field": "y"}}}}]}"#;
    assert_eq!(code(vega), "ok");
    // A page that loads its library and its data from the network: no script is loaded
    // or run, and the inline options are read.
    let page = br#"<html><head><script src="https://code.highcharts.com/highcharts.js"></script>
        <script src="https://example.com/data.js"></script></head><body><script>
        Highcharts.chart('c', { series: [{ name: 'A', data: [1, 2, 3] }] });
        fetch('https://example.com/more.json').then(r => r.json());
        </script></body></html>"#;
    assert_eq!(code(page), "ok");
    // A page whose data only a script would fetch.
    let fetched = b"<html><body><script>
        Highcharts.chart('c', { series: [{ data: await (await fetch('https://example.com/d.json')).json() }] });
        </script></body></html>";
    assert_eq!(code(fetched), "SNR-ENG-347");
}

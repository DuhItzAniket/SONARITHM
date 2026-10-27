# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Generates the SVG and vector-PDF parts of PLOTSONA's exact-source chart corpus (DS-007,
C01-T07 and C01-T08).

The same known data drawn by three renderers, each with a ground-truth JSON file:

- Vega-Lite, rendered by vl-convert-python 1.9.0.post1 (BSD-3-Clause);
- Plotly, rendered by plotly.py 6.7.0 with Kaleido 1.5.0 (MIT; it drives a local Chrome);
- D3 7.9.0 (ISC) in jsdom 24.1.3 (MIT), through `d3_charts.mjs` and Node.

Data: DS-001 to DS-004 (CC BY 4.0, examples/data). Values are in data units; pie charts
give each slice's share in percent, the only number a pie's geometry holds.

With `--pdf`, the Vega-Lite and Plotly charts are exported as vector PDF instead (vl-convert
converts through svg2pdf; Kaleido through Chrome's Skia PDF backend) into `pdf/`.

Run from the repository root with the d3 environment's node_modules folder:
  python crates/engines/sonarithm-plotsona/tests/corpus/generate/svg.py <node_modules>
  python crates/engines/sonarithm-plotsona/tests/corpus/generate/svg.py --pdf
"""

import csv
import json
import subprocess
import sys
import tempfile
from pathlib import Path

import plotly.graph_objects as go
import vl_convert

ROOT = Path(__file__).resolve().parents[6]
DATA = ROOT / "examples" / "data"
FORMAT = "svg"
OUT = Path(__file__).resolve().parents[1] / "svg"
HERE = Path(__file__).resolve().parent
MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
VL = "https://vega.github.io/schema/vega-lite/v6.json"


def rows(name):
    with open(DATA / name, newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


REGIONS = rows("ds-003-regions.csv")
REGION_NAMES = list(dict.fromkeys(r["region"] for r in REGIONS))
RAIN = {k: [float(r["rain"]) for r in REGIONS if r["region"] == k] for k in REGION_NAMES}
FRUIT = [(r["fruit"], float(r["sold"])) for r in rows("ds-004-fruit.csv")]
TEMPS = [(float(r["day"]), float(r["temp"])) for r in rows("ds-001-temperature.csv")[:60]]
PRICES = [(float(r["week"]), float(r["price"])) for r in rows("ds-002-prices.csv")]


def truth(chart_type, title, x_kind, series, provenance="vector"):
    return {"chart_type": chart_type, "title": title, "provenance": provenance, "x_kind": x_kind,
            "series": [{"name": n, "x": list(x), "y": list(y)} for n, x, y in series]}


def shares(pairs):
    total = sum(v for _, v in pairs)
    return [v / total * 100.0 for _, v in pairs]


def write(name, data, t):
    OUT.mkdir(parents=True, exist_ok=True)
    if FORMAT == "pdf":
        (OUT / f"{name}.pdf").write_bytes(data)
    else:
        (OUT / f"{name}.svg").write_text(data if data.endswith("\n") else data + "\n", encoding="utf-8", newline="\n")
    (OUT / f"{name}.truth.json").write_text(json.dumps(t, indent=1) + "\n", encoding="utf-8", newline="\n")


def render_vl(spec):
    return vl_convert.vegalite_to_pdf(spec) if FORMAT == "pdf" else vl_convert.vegalite_to_svg(spec)


def vega_lite():
    size = {"width": 800, "height": 480}
    region_rows = [{"month": MONTHS[int(r["month"]) - 1], "region": r["region"], "rain": float(r["rain"])} for r in REGIONS]
    write("vl-line", render_vl({
        "$schema": VL, **size, "title": "Rain by region", "data": {"values": region_rows}, "mark": "line",
        "encoding": {"x": {"field": "month", "type": "ordinal", "sort": None},
                     "y": {"field": "rain", "type": "quantitative"}, "color": {"field": "region", "type": "nominal"}},
    }), truth("line", "Rain by region", "category", [(k, MONTHS, RAIN[k]) for k in sorted(REGION_NAMES)]))
    fruit = sorted(FRUIT)
    write("vl-bar", render_vl({
        "$schema": VL, **size, "title": "Fruit sold", "data": {"values": [{"fruit": f, "sold": s} for f, s in FRUIT]},
        "mark": "bar", "encoding": {"x": {"field": "fruit", "type": "nominal"}, "y": {"field": "sold", "type": "quantitative"}},
    }), truth("bar", "Fruit sold", "category", [("Series 1", [f for f, _ in fruit], [s for _, s in fruit])]))
    write("vl-scatter", render_vl({
        "$schema": VL, **size, "title": "Daily temperature", "data": {"values": [{"day": d, "temp": t} for d, t in TEMPS]},
        "mark": "point", "encoding": {"x": {"field": "day", "type": "quantitative"}, "y": {"field": "temp", "type": "quantitative"}},
    }), truth("scatter", "Daily temperature", "number", [("Series 1", [d for d, _ in TEMPS], [t for _, t in TEMPS])]))
    write("vl-area", render_vl({
        "$schema": VL, **size, "title": "Weekly price", "data": {"values": [{"week": w, "price": p} for w, p in PRICES]},
        "mark": "area", "encoding": {"x": {"field": "week", "type": "quantitative"}, "y": {"field": "price", "type": "quantitative"}},
    }), truth("area", "Weekly price", "number", [("Series 1", [w for w, _ in PRICES], [p for _, p in PRICES])]))
    write("vl-pie", render_vl({
        "$schema": VL, "width": 480, "height": 480, "title": "Fruit sold",
        "data": {"values": [{"fruit": f, "sold": s} for f, s in FRUIT]},
        "mark": "arc", "encoding": {"theta": {"field": "sold", "type": "quantitative"}, "color": {"field": "fruit", "type": "nominal"}},
    }), truth("pie", "Fruit sold", "category", [("Share (%)", [f for f, _ in fruit], shares(fruit))]))


def plotly_svgs():
    layout = {"width": 1000, "height": 640}

    def svg(fig):
        image = fig.to_image(format=FORMAT)
        return image if FORMAT == "pdf" else image.decode("utf-8")

    lines = go.Figure([go.Scatter(x=MONTHS, y=RAIN[k], mode="lines", name=k.title()) for k in REGION_NAMES],
                      layout={**layout, "title": {"text": "Rain by region"}})
    write("plotly-line", svg(lines), truth("line", "Rain by region", "category", [(k.title(), MONTHS, RAIN[k]) for k in REGION_NAMES]))
    bar = go.Figure([go.Bar(x=[f for f, _ in FRUIT], y=[s for _, s in FRUIT], name="Sold")],
                    layout={**layout, "title": {"text": "Fruit sold"}})
    write("plotly-bar", svg(bar), truth("bar", "Fruit sold", "category", [("Series 1", [f for f, _ in FRUIT], [s for _, s in FRUIT])]))
    scatter = go.Figure([go.Scatter(x=[d for d, _ in TEMPS], y=[t for _, t in TEMPS], mode="markers", name="Temperature")],
                        layout={**layout, "title": {"text": "Daily temperature"}})
    write("plotly-scatter", svg(scatter), truth("scatter", "Daily temperature", "number",
                                                  [("Series 1", [d for d, _ in TEMPS], [t for _, t in TEMPS])]))
    pie = go.Figure([go.Pie(labels=[f for f, _ in FRUIT], values=[s for _, s in FRUIT], sort=False, direction="clockwise")],
                    layout={**layout, "title": {"text": "Fruit sold"}})
    write("plotly-pie", svg(pie), truth("pie", "Fruit sold", "category", [("Share (%)", [f for f, _ in FRUIT], shares(FRUIT))]))


def d3_svgs(modules):
    charts = [
        {"name": "d3-line", "kind": "line", "title": "Rain by region", "x": MONTHS,
         "series": [{"name": k.title(), "y": RAIN[k]} for k in REGION_NAMES]},
        {"name": "d3-bar", "kind": "bar", "title": "Fruit sold", "x": [f for f, _ in FRUIT],
         "series": [{"name": "Sold", "y": [s for _, s in FRUIT]}]},
        {"name": "d3-scatter", "kind": "scatter", "title": "Daily temperature", "x": [d for d, _ in TEMPS],
         "series": [{"name": "Temperature", "y": [t for _, t in TEMPS]}]},
    ]
    with tempfile.TemporaryDirectory() as tmp:
        spec = Path(tmp) / "charts.json"
        spec.write_text(json.dumps(charts), encoding="utf-8")
        subprocess.run(["node", str(HERE / "d3_charts.mjs"), modules, str(spec), tmp], check=True)
        kinds = {"line": ("line", "category"), "bar": ("bar", "category"), "scatter": ("scatter", "number")}
        for c in charts:
            chart_type, x_kind = kinds[c["kind"]]
            svg = (Path(tmp) / f"{c['name']}.svg").read_text(encoding="utf-8")
            write(c["name"], svg, truth(chart_type, c["title"], x_kind, [(s["name"], c["x"], s["y"]) for s in c["series"]]))


def main():
    global FORMAT, OUT
    if sys.argv[1:] == ["--pdf"]:
        FORMAT = "pdf"
        OUT = OUT.parent / "pdf"
    vega_lite()
    plotly_svgs()
    if FORMAT == "svg" and len(sys.argv) > 1:
        d3_svgs(sys.argv[1])
    count = len(list(OUT.glob("*.truth.json")))
    print(f"{count} {FORMAT.upper()} charts in {OUT.relative_to(ROOT)}", file=sys.stderr)


if __name__ == "__main__":
    main()

# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Generates the chart-JSON part of PLOTSONA's exact-source chart corpus (DS-007, C01-T06).

Vega-Lite, Vega, Plotly and Highcharts charts with known data, as JSON files and inside
local HTML pages, with a ground-truth JSON file beside each. Data: the project's example
datasets DS-001 to DS-004 (CC BY 4.0, examples/data).

- Vega-Lite and Vega specifications are written here, following the libraries' documented
  formats and their gallery examples' structure.
- Plotly figures come from plotly.py (MIT; `fig.to_json()` and `fig.write_html` without the
  library, so the page only calls `Plotly.newPlot`).
- Highcharts options are written here from the documented options format; the library is
  not open source and is never used or bundled (implementation plan §4.4).

Run from the repository root:  python crates/engines/sonarithm-plotsona/tests/corpus/generate/chartjson.py
"""

import csv
import datetime as dt
import json
import sys
from pathlib import Path

import numpy as np
import plotly.graph_objects as go

ROOT = Path(__file__).resolve().parents[6]
DATA = ROOT / "examples" / "data"
OUT = Path(__file__).resolve().parents[1] / "chartjson"
MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
VL = "https://vega.github.io/schema/vega-lite/v6.json"
VEGA = "https://vega.github.io/schema/vega/v6.json"


def rows(name):
    with open(DATA / name, newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


REGIONS = rows("ds-003-regions.csv")
FRUIT = rows("ds-004-fruit.csv")
TEMPS = rows("ds-001-temperature.csv")[:60]
PRICES = rows("ds-002-prices.csv")
REGION_NAMES = list(dict.fromkeys(r["region"] for r in REGIONS))
RAIN = {k: [float(r["rain"]) for r in REGIONS if r["region"] == k] for k in REGION_NAMES}
FRUIT_SOLD = [(r["fruit"], float(r["sold"])) for r in FRUIT]
DATES = [(dt.date(2026, 1, 5) + dt.timedelta(weeks=int(r["week"]))).isoformat() for r in PRICES]
PRICE = [float(r["price"]) for r in PRICES]


def truth(chart_type, title, x_kind, series):
    return {"chart_type": chart_type, "title": title, "provenance": "exact", "x_kind": x_kind,
            "series": [{"name": n, "x": list(x), "y": list(y)} for n, x, y in series]}


def write(name, document, t, html=None):
    OUT.mkdir(parents=True, exist_ok=True)
    if html is None:
        (OUT / f"{name}.json").write_text(json.dumps(document, indent=1) + "\n", encoding="utf-8", newline="\n")
    else:
        (OUT / f"{name}.html").write_text(html, encoding="utf-8", newline="\n")
    (OUT / f"{name}.truth.json").write_text(json.dumps(t, indent=1) + "\n", encoding="utf-8", newline="\n")


def region_rows():
    return [{"month": MONTHS[int(r["month"]) - 1], "region": r["region"], "rain": float(r["rain"])} for r in REGIONS]


def vega_lite():
    month_x = {"field": "month", "type": "ordinal", "sort": None, "title": "Month"}
    rain_y = {"field": "rain", "type": "quantitative", "title": "Rain (mm)"}
    by_region = [(k, MONTHS, RAIN[k]) for k in REGION_NAMES]
    write("vl-line", {
        "$schema": VL, "title": "Rain by region", "data": {"values": region_rows()},
        "mark": "line", "encoding": {"x": month_x, "y": rain_y, "color": {"field": "region", "type": "nominal"}},
    }, truth("line", "Rain by region", "category", by_region))
    fruit_sorted = sorted(FRUIT_SOLD)
    write("vl-bar", {
        "$schema": VL, "title": "Fruit sold",
        "data": {"values": [{"fruit": f, "sold": s} for f, s in FRUIT_SOLD]},
        "mark": "bar", "encoding": {"x": {"field": "fruit", "type": "nominal"}, "y": {"field": "sold", "type": "quantitative"}},
    }, truth("bar", "Fruit sold", "category", [("sold", [f for f, _ in fruit_sorted], [s for _, s in fruit_sorted])]))
    write("vl-stacked-bar", {
        "$schema": VL, "title": "Rain by month", "data": {"values": region_rows()},
        "mark": "bar", "encoding": {"x": month_x, "y": rain_y, "color": {"field": "region", "type": "nominal"}},
    }, truth("stacked-bar", "Rain by month", "category", by_region))
    q1 = [r for r in region_rows() if r["month"] in MONTHS[:4]]
    write("vl-grouped-bar", {
        "$schema": VL, "title": "Rain, first quarter", "datasets": {"rain": q1}, "data": {"name": "rain"},
        "mark": "bar", "encoding": {"x": month_x, "y": rain_y, "xOffset": {"field": "region"},
                                    "color": {"field": "region", "type": "nominal"}},
    }, truth("bar", "Rain, first quarter", "category", [(k, MONTHS[:4], RAIN[k][:4]) for k in REGION_NAMES]))
    write("vl-scatter", {
        "$schema": VL, "title": "Daily temperature",
        "data": {"values": [{"day": int(r["day"]), "temp": float(r["temp"])} for r in TEMPS]},
        "mark": "point", "encoding": {"x": {"field": "day", "type": "quantitative"}, "y": {"field": "temp", "type": "quantitative"}},
    }, truth("scatter", "Daily temperature", "number", [("temp", [float(r["day"]) for r in TEMPS], [float(r["temp"]) for r in TEMPS])]))
    write("vl-pie", {
        "$schema": VL, "title": "Fruit sold",
        "data": {"values": [{"fruit": f, "sold": s} for f, s in FRUIT_SOLD]},
        "mark": "arc", "encoding": {"theta": {"field": "sold", "type": "quantitative"}, "color": {"field": "fruit", "type": "nominal"}},
    }, truth("pie", "Fruit sold", "category", [("sold", [f for f, _ in fruit_sorted], [s for _, s in fruit_sorted])]))
    write("vl-temporal", {
        "$schema": VL, "title": "Weekly price",
        "data": {"values": [{"date": d, "price": p} for d, p in zip(DATES, PRICE)]},
        "mark": "line", "encoding": {"x": {"field": "date", "type": "temporal"}, "y": {"field": "price", "type": "quantitative"}},
    }, truth("line", "Weekly price", "date", [("price", DATES, PRICE)]))
    spec = {
        "$schema": VL, "title": "Rain by region", "data": {"values": region_rows()},
        "mark": "line", "encoding": {"x": month_x, "y": rain_y, "color": {"field": "region", "type": "nominal"}},
    }
    page = ("<!DOCTYPE html>\n<html>\n<head>\n<script src=\"https://cdn.jsdelivr.net/npm/vega-embed@7\"></script>\n</head>\n"
            "<body>\n<div id=\"vis\"></div>\n<script>\n  var spec = " + json.dumps(spec, indent=1)
            + ";\n  vegaEmbed('#vis', spec);\n</script>\n</body>\n</html>\n")
    write("vl-embed", None, truth("line", "Rain by region", "category", by_region), html=page)


def vega():
    fruit = [{"category": f, "amount": s} for f, s in FRUIT_SOLD]
    write("vega-bar", {
        "$schema": VEGA, "title": "Fruit sold", "width": 400, "height": 200,
        "data": [{"name": "table", "values": fruit}],
        "scales": [
            {"name": "xscale", "type": "band", "domain": {"data": "table", "field": "category"}, "range": "width"},
            {"name": "yscale", "domain": {"data": "table", "field": "amount"}, "nice": True, "range": "height"},
        ],
        "axes": [{"orient": "bottom", "scale": "xscale", "title": "Fruit"}, {"orient": "left", "scale": "yscale", "title": "Sold"}],
        "marks": [{"type": "rect", "from": {"data": "table"}, "encode": {"enter": {
            "x": {"scale": "xscale", "field": "category"}, "width": {"scale": "xscale", "band": 1},
            "y": {"scale": "yscale", "field": "amount"}, "y2": {"scale": "yscale", "value": 0}}}}],
    }, truth("bar", "Fruit sold", "category", [("amount", [f for f, _ in FRUIT_SOLD], [s for _, s in FRUIT_SOLD])]))
    write("vega-line", {
        "$schema": VEGA, "title": "Rain by region", "width": 500, "height": 200,
        "data": [{"name": "table", "values": region_rows()}],
        "scales": [
            {"name": "x", "type": "point", "range": "width", "domain": {"data": "table", "field": "month"}},
            {"name": "y", "type": "linear", "range": "height", "domain": {"data": "table", "field": "rain"}},
            {"name": "color", "type": "ordinal", "range": "category", "domain": {"data": "table", "field": "region"}},
        ],
        "marks": [{"type": "group", "from": {"facet": {"name": "series", "data": "table", "groupby": "region"}},
                   "marks": [{"type": "line", "from": {"data": "series"}, "encode": {"enter": {
                       "x": {"scale": "x", "field": "month"}, "y": {"scale": "y", "field": "rain"},
                       "stroke": {"scale": "color", "field": "region"}}}}]}],
    }, truth("line", "Rain by region", "category", [(k, MONTHS, RAIN[k]) for k in REGION_NAMES]))
    write("vega-scatter", {
        "$schema": VEGA, "title": "Daily temperature", "width": 400, "height": 300,
        "data": [{"name": "temps", "values": [{"day": int(r["day"]), "temp": float(r["temp"])} for r in TEMPS]}],
        "scales": [{"name": "x", "type": "linear", "domain": {"data": "temps", "field": "day"}, "range": "width"},
                   {"name": "y", "type": "linear", "domain": {"data": "temps", "field": "temp"}, "range": "height"}],
        "marks": [{"type": "symbol", "from": {"data": "temps"}, "encode": {"enter": {
            "x": {"scale": "x", "field": "day"}, "y": {"scale": "y", "field": "temp"}}}}],
    }, truth("scatter", "Daily temperature", "number", [("temp", [float(r["day"]) for r in TEMPS], [float(r["temp"]) for r in TEMPS])]))
    write("vega-pie", {
        "$schema": VEGA, "title": "Fruit sold", "width": 200, "height": 200,
        "data": [{"name": "table", "values": [{"id": f, "field": s} for f, s in FRUIT_SOLD],
                  "transform": [{"type": "pie", "field": "field"}]}],
        "scales": [{"name": "color", "type": "ordinal", "domain": {"data": "table", "field": "id"}, "range": {"scheme": "category20"}}],
        "marks": [{"type": "arc", "from": {"data": "table"}, "encode": {"enter": {
            "fill": {"scale": "color", "field": "id"}, "x": {"signal": "width / 2"}, "y": {"signal": "height / 2"},
            "startAngle": {"field": "startAngle"}, "endAngle": {"field": "endAngle"}, "outerRadius": {"signal": "width / 2"}}}}],
    }, truth("pie", "Fruit sold", "category", [("field", [f for f, _ in FRUIT_SOLD], [s for _, s in FRUIT_SOLD])]))


def plotly_figures():
    def save(name, fig, t):
        write(name, json.loads(fig.to_json()), t)

    lines = go.Figure([go.Scatter(x=MONTHS, y=RAIN[k], mode="lines", name=k.title()) for k in REGION_NAMES],
                      layout={"title": {"text": "Rain by region"}, "xaxis": {"title": {"text": "Month"}},
                              "yaxis": {"title": {"text": "Rain (mm)"}}})
    by_region = [(k.title(), MONTHS, RAIN[k]) for k in REGION_NAMES]
    save("plotly-line", lines, truth("line", "Rain by region", "category", by_region))
    stacked = go.Figure([go.Bar(x=MONTHS, y=RAIN[k], name=k.title()) for k in REGION_NAMES],
                        layout={"title": {"text": "Rain by month"}, "barmode": "stack"})
    save("plotly-stacked-bar", stacked, truth("stacked-bar", "Rain by month", "category", by_region))
    days = np.array([int(r["day"]) for r in TEMPS], dtype=np.int32)
    temps = np.array([float(r["temp"]) for r in TEMPS], dtype=np.float64)
    scatter = go.Figure([go.Scatter(x=days, y=temps, mode="markers", name="Temperature")],
                        layout={"title": {"text": "Daily temperature"}})
    save("plotly-scatter", scatter, truth("scatter", "Daily temperature", "number",
                                          [("Temperature", [float(d) for d in days], [float(t) for t in temps])]))
    pie = go.Figure([go.Pie(labels=[f for f, _ in FRUIT_SOLD], values=[s for _, s in FRUIT_SOLD], name="Sold")],
                    layout={"title": {"text": "Fruit sold"}})
    save("plotly-pie", pie, truth("pie", "Fruit sold", "category", [("Sold", [f for f, _ in FRUIT_SOLD], [s for _, s in FRUIT_SOLD])]))
    area = go.Figure([go.Scatter(x=DATES, y=PRICE, fill="tozeroy", mode="lines", name="Price")],
                     layout={"title": {"text": "Weekly price"}, "xaxis": {"type": "date"}})
    save("plotly-area", area, truth("area", "Weekly price", "date", [("Price", DATES, PRICE)]))
    hbar = go.Figure([go.Bar(y=[f for f, _ in FRUIT_SOLD], x=[s for _, s in FRUIT_SOLD], orientation="h", name="Sold")],
                     layout={"title": {"text": "Fruit sold"}})
    save("plotly-horizontal-bar", hbar, truth("bar", "Fruit sold", "category", [("Sold", [f for f, _ in FRUIT_SOLD], [s for _, s in FRUIT_SOLD])]))
    OUT.mkdir(parents=True, exist_ok=True)
    page = lines.to_html(include_plotlyjs=False, full_html=True, div_id="rain")
    write("plotly-page", None, truth("line", "Rain by region", "category", by_region), html=page)


def highcharts():
    by_region = [(k.title(), MONTHS, RAIN[k]) for k in REGION_NAMES]
    write("highcharts-line", {
        "chart": {"type": "line"}, "title": {"text": "Rain by region"},
        "xAxis": {"categories": MONTHS, "title": {"text": "Month"}}, "yAxis": {"title": {"text": "Rain (mm)"}},
        "series": [{"name": k.title(), "data": RAIN[k]} for k in REGION_NAMES],
    }, truth("line", "Rain by region", "category", by_region))
    write("highcharts-stacked-column", {
        "chart": {"type": "column"}, "title": {"text": "Rain by month"}, "xAxis": {"categories": MONTHS},
        "plotOptions": {"column": {"stacking": "normal"}},
        "series": [{"name": k.title(), "data": RAIN[k]} for k in REGION_NAMES],
    }, truth("stacked-bar", "Rain by month", "category", by_region))
    write("highcharts-scatter", {
        "chart": {"type": "scatter"}, "title": {"text": "Daily temperature"},
        "series": [{"name": "Temperature", "data": [[int(r["day"]), float(r["temp"])] for r in TEMPS]}],
    }, truth("scatter", "Daily temperature", "number", [("Temperature", [float(r["day"]) for r in TEMPS], [float(r["temp"]) for r in TEMPS])]))
    write("highcharts-pie", {
        "chart": {"type": "pie"}, "title": {"text": "Fruit sold"},
        "series": [{"name": "Sold", "data": [{"name": f, "y": s} for f, s in FRUIT_SOLD]}],
    }, truth("pie", "Fruit sold", "category", [("Sold", [f for f, _ in FRUIT_SOLD], [s for _, s in FRUIT_SOLD])]))
    # A page as Highcharts' demos write it: a JavaScript literal with unquoted keys, single
    # quotes, a trailing comma and Date.UTC for the first point; weekly points.
    week_ms = 7 * 24 * 3600 * 1000
    page = ("<!DOCTYPE html>\n<html>\n<head>\n<script src=\"https://code.highcharts.com/highcharts.js\"></script>\n</head>\n"
            "<body>\n<div id=\"container\"></div>\n<script>\nHighcharts.chart('container', {\n"
            "    chart: { type: 'area' },\n    title: { text: 'Weekly price' },\n"
            "    xAxis: { type: 'datetime' },\n    yAxis: { title: { text: 'Price ($)' } },\n"
            "    plotOptions: { series: { pointStart: Date.UTC(2026, 0, 5), pointInterval: " + str(week_ms) + " } },\n"
            "    series: [{\n        name: 'Price',\n        data: [" + ", ".join(repr(p) for p in PRICE) + "],\n    }],\n});\n"
            "</script>\n</body>\n</html>\n")
    write("highcharts-page", None, truth("area", "Weekly price", "date", [("Price", DATES, PRICE)]), html=page)


def main():
    vega_lite()
    vega()
    plotly_figures()
    highcharts()
    count = len(list(OUT.glob("*.truth.json")))
    print(f"{count} chart-JSON charts in {OUT.relative_to(ROOT)}", file=sys.stderr)


if __name__ == "__main__":
    main()

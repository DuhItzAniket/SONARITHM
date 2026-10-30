# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Writes the importer examples (C01-T10).

The Vega-Lite examples follow the chart types of the Vega-Lite example gallery, and the
MAIDR examples the layer types of MAIDR's data schema; every document and every number is
written here for SONARITHM (no third-party file is copied). Run from this folder:

    python generate.py
"""
import json
import math
from pathlib import Path

HERE = Path(__file__).parent
VL = "https://vega.github.io/schema/vega-lite/v5.json"
QUARTERS = ["Q1", "Q2", "Q3", "Q4"]
REGIONS = ["North", "South", "East"]


def sales():
    return [
        {"quarter": q, "region": r, "sales": round(10 + 3 * i + 2 * j + (i * j) % 3, 1)}
        for i, q in enumerate(QUARTERS)
        for j, r in enumerate(REGIONS)
    ]


def monthly():
    rows = []
    for j, city in enumerate(["Oslo", "Lima"]):
        for m in range(12):
            temp = round(10 + 8 * math.sin((m - 3 + 6 * j) / 12 * 2 * math.pi), 1)
            rows.append({"month": f"2025-{m + 1:02d}-01", "city": city, "temp": temp})
    return rows


def points():
    return [
        {"height": round(150 + i * 2.5, 1), "weight": round(48 + i * 1.7 + (i % 4) * 2.1, 1),
         "group": "A" if i % 2 else "B"}
        for i in range(16)
    ]


def write(folder, name, doc):
    path = HERE / folder / name
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(doc, f, indent=2, ensure_ascii=False)
        f.write("\n")


def vegalite():
    q = {"field": "quarter", "type": "nominal", "title": "Quarter"}
    s = {"field": "sales", "type": "quantitative", "title": "Sales"}
    totals = [{"quarter": r["quarter"], "sales": r["sales"]} for r in sales() if r["region"] == "North"]
    write("vegalite", "bar.vl.json", {"$schema": VL, "title": "Sales by quarter", "data": {"values": totals},
                                      "mark": "bar", "encoding": {"x": q, "y": s}})
    write("vegalite", "horizontal-bar.vl.json", {"$schema": VL, "data": {"values": totals}, "mark": "bar",
                                                 "encoding": {"y": q, "x": s}})
    write("vegalite", "stacked-bar.vl.json", {"$schema": VL, "data": {"values": sales()}, "mark": "bar",
                                              "encoding": {"x": q, "y": {**s, "aggregate": "sum"},
                                                           "color": {"field": "region", "type": "nominal"}}})
    write("vegalite", "grouped-bar.vl.json", {"$schema": VL, "data": {"values": sales()}, "mark": "bar",
                                              "encoding": {"x": q, "y": s,
                                                           "xOffset": {"field": "region"},
                                                           "color": {"field": "region", "type": "nominal"}}})
    write("vegalite", "mean-bar.vl.json", {"$schema": VL, "data": {"values": sales()}, "mark": "bar",
                                           "encoding": {"x": {"field": "region", "type": "nominal"},
                                                        "y": {**s, "aggregate": "mean", "title": "Mean sales"}}})
    write("vegalite", "multi-line.vl.json", {"$schema": VL, "title": "Monthly temperature",
                                             "data": {"values": monthly()}, "mark": "line",
                                             "encoding": {"x": {"field": "month", "type": "temporal", "title": "Month"},
                                                          "y": {"field": "temp", "type": "quantitative", "title": "°C"},
                                                          "color": {"field": "city", "type": "nominal"}}})
    write("vegalite", "area.vl.json", {"$schema": VL, "data": {"values": [r for r in monthly() if r["city"] == "Oslo"]},
                                       "mark": {"type": "area", "line": True},
                                       "encoding": {"x": {"field": "month", "type": "temporal"},
                                                    "y": {"field": "temp", "type": "quantitative"}}})
    write("vegalite", "scatter.vl.json", {"$schema": VL, "data": {"values": points()}, "mark": "point",
                                          "encoding": {"x": {"field": "height", "type": "quantitative"},
                                                       "y": {"field": "weight", "type": "quantitative"},
                                                       "color": {"field": "group", "type": "nominal"},
                                                       "size": {"field": "weight", "type": "quantitative"},
                                                       "tooltip": [{"field": "height"}, {"field": "weight"}]}})
    write("vegalite", "histogram.vl.json", {"$schema": VL, "data": {"url": "data/waits.csv"}, "mark": "bar",
                                            "encoding": {"x": {"field": "minutes", "bin": {"maxbins": 8}, "title": "Wait"},
                                                         "y": {"aggregate": "count"}}})
    write("vegalite", "pie.vl.json", {"$schema": VL, "data": {"values": [
        {"fruit": f, "units": u} for f, u in [("Apples", 30), ("Bananas", 50), ("Cherries", 20)]]},
        "mark": "arc", "encoding": {"theta": {"field": "units", "type": "quantitative"},
                                    "color": {"field": "fruit", "type": "nominal"}}})
    write("vegalite", "heatmap.vl.json", {"$schema": VL, "data": {"values": sales()}, "mark": "rect",
                                          "encoding": {"x": q, "y": {"field": "region", "type": "nominal"},
                                                       "color": {**s, "aggregate": "sum"}}})
    boxes = [{"site": site, "ms": round(200 + 40 * k + (i * 37) % 90, 1)}
             for k, site in enumerate(["A", "B", "C"]) for i in range(12)]
    write("vegalite", "boxplot.vl.json", {"$schema": VL, "data": {"values": boxes}, "mark": "boxplot",
                                          "encoding": {"x": {"field": "site", "type": "nominal"},
                                                       "y": {"field": "ms", "type": "quantitative"}}})
    write("vegalite", "layered.vl.json", {"$schema": VL, "data": {"values": monthly()},
                                          "transform": [{"filter": {"field": "city", "equal": "Oslo"}}],
                                          "encoding": {"x": {"field": "month", "type": "temporal"}},
                                          "layer": [
                                              {"mark": "line", "encoding": {"y": {"field": "temp", "type": "quantitative"}}},
                                              {"mark": "rule", "encoding": {"y": {"datum": 10}}},
                                          ]})
    write("vegalite", "filtered.vl.json", {"$schema": VL, "data": {"values": points()},
                                           "transform": [
                                               {"filter": {"field": "height", "gte": 160}},
                                               {"calculate": "datum.weight / 2.2", "as": "pounds"},
                                           ],
                                           "params": [{"name": "brush", "select": "interval"}],
                                           "mark": "circle",
                                           "encoding": {"x": {"field": "height", "type": "quantitative"},
                                                        "y": {"field": "weight", "type": "quantitative"}}})
    waits = HERE / "vegalite" / "data" / "waits.csv"
    waits.parent.mkdir(parents=True, exist_ok=True)
    with open(waits, "w", encoding="utf-8", newline="\n") as f:
        f.write("minutes\n")
        for i in range(60):
            f.write(f"{round(-6 * math.log(1 - (i + 0.5) / 60 * 0.98), 2)}\n")


def maidr_doc(layer, title=None, **top):
    doc = {"id": "chart", "subplots": [[{"id": "plot", "layers": [layer]}]]}
    if title:
        doc["title"] = title
    doc.update(top)
    return doc


def maidr():
    totals = [{"x": r["quarter"], "y": r["sales"]} for r in sales() if r["region"] == "North"]
    axes = {"x": {"label": "Quarter"}, "y": {"label": "Sales"}}
    write("maidr", "bar.json", maidr_doc({"id": "0", "type": "bar", "axes": axes, "data": totals}, "Sales by quarter"))
    by_region = [[{"x": r["quarter"], "fill": r["region"], "y": r["sales"]} for r in sales() if r["region"] == reg]
                 for reg in REGIONS]
    write("maidr", "dodged-bar.json", maidr_doc({"id": "0", "type": "dodged_bar", "axes": axes, "data": by_region}))
    write("maidr", "stacked-bar.json", maidr_doc({"id": "0", "type": "stacked_bar", "axes": axes, "data": by_region}))
    lines = [[{"x": m + 1, "y": r["temp"]} for m, r in enumerate(x for x in monthly() if x["city"] == c)]
             for c in ["Oslo", "Lima"]]
    write("maidr", "line.json", maidr_doc({"id": "0", "type": "line",
                                           "axes": {"x": {"label": "Month"}, "y": {"label": "°C"}}, "data": lines},
                                          "Monthly temperature"))
    write("maidr", "step.json", maidr_doc({"id": "0", "type": "step", "stepDirection": "hv", "data": lines[:1]}))
    write("maidr", "point.json", maidr_doc({"id": "0", "type": "point", "data": [
        {"x": p["height"], "y": p["weight"]} for p in points()]}))
    bins = [{"x": 1.5 + 3 * i, "y": c, "xMin": 3 * i, "xMax": 3 * i + 3, "yMin": 0, "yMax": c}
            for i, c in enumerate([14, 11, 8, 6, 4, 2])]
    write("maidr", "hist.json", maidr_doc({"id": "0", "type": "hist", "data": bins}))
    write("maidr", "pie.json", maidr_doc({"id": "0", "type": "pie", "startAngle": 0, "direction": "clockwise",
                                          "axes": {"x": {"label": "Fruit"}, "y": {"label": "Units"}},
                                          "data": [{"x": "Apples", "y": 30}, {"x": "Bananas", "y": 50},
                                                   {"x": "Cherries", "y": 20}]}))
    write("maidr", "heat.json", maidr_doc({"id": "0", "type": "heat", "data": {
        "points": [[r["sales"] for r in sales() if r["region"] == reg] for reg in REGIONS],
        "x": QUARTERS, "y": REGIONS}}))
    write("maidr", "box.json", maidr_doc({"id": "0", "type": "box", "orientation": "vert", "data": [
        {"lowerOutliers": [], "min": 71.4, "q1": 92.6, "q2": 99.6, "q3": 107.7, "max": 118.2,
         "upperOutliers": [150.0], "fill": "Group 1"},
        {"lowerOutliers": [], "min": 65.0, "q1": 80.5, "q2": 88.1, "q3": 95.0, "max": 110.3,
         "upperOutliers": [], "fill": "Group 2"}]}))
    write("maidr", "candlestick.json", maidr_doc({"id": "0", "type": "candlestick", "data": [
        {"value": f"2026-02-{d:02d}", "open": 150 + d * 0.5, "high": 152 + d * 0.5, "low": 149 + d * 0.5,
         "close": 151 + d * 0.4, "volume": 1000 + 10 * d} for d in range(2, 12)]}))
    write("maidr", "legacy-bar.json", {"type": "bar", "id": "legacy", "title": "Legacy format",
                                       "axes": {"x": {"label": "Quarter"}, "y": {"label": "Sales"}},
                                       "data": totals})


if __name__ == "__main__":
    vegalite()
    maidr()

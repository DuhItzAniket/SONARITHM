# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Writes the raster part of DS-007 (C02): the corpus SVGs rendered to PNG.

Each SVG in `../svg/` (drawn by Vega-Lite, Plotly and D3 from DS-001 to DS-004) is rasterized
with vl-convert 1.9.0 (resvg) into `../raster/`:

- every chart anti-aliased at 1x and 2x, as `<name>@1x.png` and `<name>@2x.png` (C02-T03);
- the line and bar charts also at 1.5x, `<name>@1.5x.png`, and at 1x without anti-aliasing
  (`shape-rendering="crispEdges"` on the root element), `<name>@1x-crisp.png` (C02-T08).

JPEG artefacts are not committed: the accuracy harness re-encodes these PNGs as JPEG in
memory with the `image` crate the raster tier already uses.

The ground truth is the SVG's, with the provenance class `cv_estimate`. The raster tests
take their hand-entered anchors from the SVG's axes, scaled to the image, as a person would
click them on the ticks.

Run from the repository root:

  python crates/engines/sonarithm-plotsona/tests/corpus/generate/raster.py
"""

import json
from pathlib import Path

import vl_convert

HERE = Path(__file__).resolve().parent
SVG = HERE.parent / "svg"
OUT = HERE.parent / "raster"
SCALES = (1, 2)
# C02-T08: more resolutions and no anti-aliasing, for the charts the C02 bar covers.
EXTRA_SCALES = (1.5,)
EXTRA_KINDS = ("line", "bar")


def crisp(text):
    """The SVG with anti-aliasing turned off for shapes (resvg honours `crispEdges`)."""
    return text.replace("<svg ", '<svg shape-rendering="crispEdges" ', 1)


def write(name, png, truth):
    (OUT / f"{name}.png").write_bytes(png)
    (OUT / f"{name}.truth.json").write_text(json.dumps(truth, indent=1) + "\n", encoding="utf-8", newline="\n")


def main():
    OUT.mkdir(exist_ok=True)
    count = 0
    for svg in sorted(SVG.glob("*.svg")):
        truth = json.loads((SVG / f"{svg.stem}.truth.json").read_text(encoding="utf-8"))
        truth["provenance"] = "cv_estimate"
        text = svg.read_text(encoding="utf-8")
        for scale in SCALES:
            write(f"{svg.stem}@{scale}x", vl_convert.svg_to_png(text, scale=scale), truth)
            count += 1
        if truth["chart_type"] not in EXTRA_KINDS:
            continue
        for scale in EXTRA_SCALES:
            write(f"{svg.stem}@{scale}x", vl_convert.svg_to_png(text, scale=scale), truth)
            count += 1
        write(f"{svg.stem}@1x-crisp", vl_convert.svg_to_png(crisp(text), scale=1), truth)
        count += 1
    print(f"{count} PNG charts in {OUT}")


if __name__ == "__main__":
    main()

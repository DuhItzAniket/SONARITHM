# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Writes the raster part of DS-007 (C02): the corpus SVGs rendered to PNG.

Each SVG in `../svg/` (drawn by Vega-Lite, Plotly and D3 from DS-001 to DS-004) is rasterized
with vl-convert 1.9.0 (resvg, anti-aliased) at 1x and 2x into `../raster/`, as
`<name>@1x.png` and `<name>@2x.png`. The ground truth is the SVG's, with the provenance class
`cv_estimate`. The raster tests take their hand-entered anchors from the SVG's axes, scaled
to the image, as a person would click them on the ticks.

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


def main():
    OUT.mkdir(exist_ok=True)
    count = 0
    for svg in sorted(SVG.glob("*.svg")):
        truth = json.loads((SVG / f"{svg.stem}.truth.json").read_text(encoding="utf-8"))
        truth["provenance"] = "cv_estimate"
        text = svg.read_text(encoding="utf-8")
        for scale in SCALES:
            name = f"{svg.stem}@{scale}x"
            (OUT / f"{name}.png").write_bytes(vl_convert.svg_to_png(text, scale=scale))
            (OUT / f"{name}.truth.json").write_text(json.dumps(truth, indent=1) + "\n", encoding="utf-8", newline="\n")
            count += 1
    print(f"{count} PNG charts in {OUT}")


if __name__ == "__main__":
    main()

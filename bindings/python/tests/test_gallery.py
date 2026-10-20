# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""The gallery notebook runs top to bottom against the installed package."""

from __future__ import annotations

import json
import shutil
from pathlib import Path

EXAMPLES = Path(__file__).resolve().parents[3] / "examples"


def test_the_gallery_notebook_runs(tmp_path, monkeypatch):
    notebook = json.loads((EXAMPLES / "gallery.ipynb").read_text("utf-8"))
    # Run where the notebook expects to be, in a copy, so its renders stay out of the tree.
    for name in ("specs", "data"):
        shutil.copytree(EXAMPLES / name, tmp_path / name)
    monkeypatch.chdir(tmp_path)
    namespace: dict = {}
    for cell in notebook["cells"]:
        if cell["cell_type"] == "code":
            exec(compile("".join(cell["source"]), "gallery.ipynb", "exec"), namespace)
    assert len(namespace["gallery"]) == 5
    assert namespace["manifest"]["output"]["file"] == "temperature.flac"
    assert (tmp_path / "temperature.render.json").is_file()
    assert "<audio" in namespace["gallery"]["ds-003-regions.spec.json"]._repr_html_()

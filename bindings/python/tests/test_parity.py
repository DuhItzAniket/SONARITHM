# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Score parity with the Rust facade (ADR-0013): Python compiles the parity spec to the
Score the facade checked in, value for value (tests/golden/README.md)."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

import sonarithm as snr

PARITY = Path(__file__).resolve().parents[3] / "tests" / "golden" / "parity"


def test_python_compiles_the_score_the_facade_checked_in():
    compiled = snr.compile(snr.Spec.from_file(PARITY / "temps.spec.json"))
    score = compiled.score()
    score["header"]["engines"] = {engine: "*" for engine in score["header"]["engines"]}
    expected = json.loads((PARITY / "temps.score.json").read_text("utf-8"))
    assert score == expected


def test_python_renders_the_bytes_the_facade_checked_in(tmp_path):
    compiled = snr.compile(snr.Spec.from_file(PARITY / "temps.spec.json"))
    expected = (PARITY / "temps.wav.sha256").read_text("utf-8").strip()
    assert hashlib.sha256(compiled.wav()).hexdigest() == expected
    manifest = compiled.render(tmp_path / "temps.wav")
    assert hashlib.sha256((tmp_path / "temps.wav").read_bytes()).hexdigest() == expected
    assert manifest["output"]["sha256"].endswith(expected)
    assert (tmp_path / "temps.render.json").is_file() and (tmp_path / "temps.cues.vtt").is_file()


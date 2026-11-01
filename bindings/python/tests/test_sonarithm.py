# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""The Python package against an installed wheel: specs, compiles, quick, data inputs,
errors and the notebook display (11 §3)."""

from __future__ import annotations

import concurrent.futures
import importlib.resources
import importlib.util
import json
from pathlib import Path

import pytest

import sonarithm as snr

ROOT = Path(__file__).resolve().parents[3]
PARITY = ROOT / "tests" / "golden" / "parity"

TEMPS = {
    "specVersion": "0.1",
    "name": "Temperatures",
    "data": {
        "source": {
            "kind": "inline",
            "values": [
                {"day": 1, "temp": 8.5},
                {"day": 2, "temp": 9.25},
                {"day": 3, "temp": 7.0},
            ],
        },
        "fields": {"temp": {"unit": "°C", "concept": "temperature"}},
    },
    "substrate": {"time": {"field": "day"}},
    "tracks": [{"id": "temps", "mark": "tone", "channels": {"pitch": {"field": "temp"}}}],
}

COLUMNS = {"year": [2000.0, 2001.0, 2002.0, 2003.0], "anomaly": [0.39, 0.54, 0.63, 0.62]}


def codes(diagnostics: list[dict]) -> list[str]:
    return [d["code"] for d in diagnostics]


def test_versions_and_typing_marker():
    assert snr.__version__.count(".") == 2
    assert snr.SPEC_VERSION == "0.4"
    assert importlib.resources.files("sonarithm").joinpath("py.typed").is_file()


def test_the_schema_is_the_published_one():
    published = json.loads((ROOT / "schemas" / "sonarithm-spec-0.4.schema.json").read_text("utf-8"))
    assert snr.schema() == published
    with pytest.raises(ValueError):
        snr.schema("profile")


def test_specs_round_trip_as_dicts_and_json():
    spec = snr.Spec.from_dict(TEMPS, name="temps.spec.json")
    assert spec.name == "temps.spec.json"
    assert spec.to_dict() == TEMPS
    assert snr.Spec.from_json(spec.to_json()).to_dict() == TEMPS


def test_validation_reports_errors_and_defaults():
    defaults = snr.Spec.from_dict(TEMPS).validate()
    assert "SNR-SPEC-080" in codes(defaults)
    assert all(d["severity"] != "error" for d in defaults)
    reversed_range = json.loads(json.dumps(TEMPS))
    reversed_range["tracks"][0]["channels"]["pitch"]["range"] = {"hz": [1400, 400]}
    errors = [d for d in snr.Spec.from_dict(reversed_range).validate() if d["severity"] == "error"]
    assert codes(errors) == ["SNR-SPEC-031"]
    assert errors[0]["rules"] == ["SDR-15"]
    assert errors[0]["location"] == "/tracks/0/channels/pitch/range"


def test_invalid_json_raises_with_diagnostics():
    with pytest.raises(snr.SonarithmError) as caught:
        snr.Spec.from_json('{ "specVersion": ')
    assert codes(caught.value.diagnostics) == ["SNR-SPEC-001"]


def test_a_spec_compiles_with_its_defaults_reported():
    compiled = snr.compile(TEMPS)
    summary = compiled.summary()
    assert summary["tracks"][0]["notes"] == 3
    assert summary["tracks"][0]["pitch_range_hz"] == [400.0, 1400.0]
    assert any("SDR-08" in d["rules"] for d in compiled.diagnostics if d["severity"] == "info")
    text = compiled.describe()
    assert text.splitlines()[0] == "Temperatures: 1 data track, 0.92 seconds, analysis mode."
    assert compiled.description()["l1"][0] == text.splitlines()[0]
    assert len(compiled.score_hash) == 64
    assert compiled.score()["tracks"][0]["id"] == "temps"
    assert json.loads(compiled.canonical_spec)["specVersion"] == "0.1"


def test_compile_errors_carry_every_diagnostic():
    unknown_field = json.loads(json.dumps(TEMPS))
    unknown_field["tracks"][0]["channels"]["pitch"]["field"] = "humidity"
    with pytest.raises(snr.SonarithmError) as caught:
        snr.compile(unknown_field)
    assert any(d["severity"] == "error" for d in caught.value.diagnostics)
    assert "humidity" in str(caught.value)


def test_missing_files_name_no_directories(tmp_path):
    with pytest.raises(snr.SonarithmError) as caught:
        snr.Spec.from_file(tmp_path / "nested" / "absent.spec.json")
    message = str(caught.value)
    assert "absent.spec.json" in message
    assert "nested" not in message and str(tmp_path) not in message


def test_quick_reads_dicts_records_and_files(tmp_path):
    from_dict = snr.quick(COLUMNS, x="year", y="anomaly")
    records = [dict(zip(COLUMNS, row)) for row in zip(*COLUMNS.values())]
    from_records = snr.quick(records, x="year", y="anomaly")
    assert from_dict.score_hash == from_records.score_hash
    assert from_dict.spec.to_dict()["data"]["source"] == {"kind": "dataset", "id": "main"}
    csv = tmp_path / "temps.csv"
    csv.write_text("year,anomaly\n2000.0,0.39\n2001.0,0.54\n2002.0,0.63\n2003.0,0.62\n", "utf-8")
    from_file = snr.quick(csv, x="year", y="anomaly")
    assert from_file.summary()["tracks"][0]["notes"] == 4
    assert from_file.spec.to_dict()["data"]["source"] == {"kind": "file", "path": "temps.csv"}
    assert from_file.spec.name == "temps.spec.json"


def test_quick_options_shape_the_spec():
    compiled = snr.quick(
        COLUMNS, x="year", y="anomaly", mark="glide", range_hz=(300.0, 900.0), concept="temperature", name="Anomaly"
    )
    spec = compiled.spec.to_dict()
    assert spec["name"] == "Anomaly"
    assert spec["tracks"][0]["mark"] == "glide"
    assert spec["tracks"][0]["channels"]["pitch"]["range"] == {"hz": [300.0, 900.0]}
    assert spec["data"]["fields"]["anomaly"]["concept"] == "temperature"
    track = compiled.summary()["tracks"][0]
    assert track["glides"] == 1 and track["pitch_range_hz"] == [300.0, 900.0]
    with pytest.raises(ValueError):
        snr.quick(COLUMNS, mark="buzz")
    with pytest.raises(ValueError):
        snr.quick(COLUMNS, range_hz=(900.0, 300.0))
    with pytest.raises(snr.SonarithmError):
        snr.quick(COLUMNS, x="year", y="missing")


def test_the_same_values_compile_alike_from_every_library():
    pa = pytest.importorskip("pyarrow")
    pd = pytest.importorskip("pandas")
    pl = pytest.importorskip("polars")
    data = dict(COLUMNS, site=["a", "b", "a", "b"])
    frame = pd.DataFrame(data)
    hashes = {
        "dict": snr.quick(data, x="year", y="anomaly").score_hash,
        "pyarrow": snr.quick(pa.table(data), x="year", y="anomaly").score_hash,
        "pandas": snr.quick(frame, x="year", y="anomaly").score_hash,
        "polars": snr.quick(pl.DataFrame(data), x="year", y="anomaly").score_hash,
        "record batch": snr.quick(pa.record_batch(data), x="year", y="anomaly").score_hash,
    }
    assert len(set(hashes.values())) == 1, hashes


def test_numpy_arrays_and_pandas_without_index():
    np = pytest.importorskip("numpy")
    pd = pytest.importorskip("pandas")
    array = snr.quick(np.array([3.0, 1.0, 2.0]))
    assert array.summary()["tracks"][0]["notes"] == 3
    indexed = pd.DataFrame(COLUMNS, index=pd.Index(["w", "x", "y", "z"], name="label"))
    assert "label" not in snr.dataset(indexed).fields
    nan = snr.quick({"x": [0.0, 1.0, 2.0], "y": [1.0, float("nan"), 2.0]}, x="x", y="y")
    assert nan.summary()["tracks"][0]["notes"] == 2, "a missing value is silent (03 §5)"


def test_dataset_sources_and_reuse():
    pd = pytest.importorskip("pandas")
    frame = pd.DataFrame(COLUMNS)
    dataset = snr.dataset(frame, label="frame")
    assert dataset.rows == 4 and dataset.fields == ["year", "anomaly"]
    spec = {
        "specVersion": "0.1",
        "data": {"source": {"kind": "dataset", "id": "main"}},
        "substrate": {"time": {"field": "year"}},
        "tracks": [{"id": "anomaly", "mark": "tone", "channels": {"pitch": {"field": "anomaly"}}}],
    }
    first = snr.compile(spec, data={"main": dataset})
    second = snr.compile(spec, data={"main": frame})
    assert first.score_hash == second.score_hash
    with pytest.raises(snr.SonarithmError) as caught:
        snr.compile(spec, data={"other": frame})
    assert "SNR-COMP-026" in codes(caught.value.diagnostics)
    with pytest.raises(TypeError):
        snr.compile(spec, data={"main": 42})


def test_compiles_run_in_parallel_threads_with_identical_results():
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        hashes = set(pool.map(lambda _: snr.compile(TEMPS).score_hash, range(8)))
    assert len(hashes) == 1


def test_renders_write_files_and_memory_matches_them(tmp_path):
    compiled = snr.compile(TEMPS)
    manifest = compiled.render(tmp_path / "temps.wav")
    assert manifest["output"]["file"] == "temps.wav"
    assert manifest["data"]["provenance"] == {"exact": 1.0}
    assert str(tmp_path) not in json.dumps(manifest), "no user paths in the manifest"
    assert (tmp_path / "temps.render.json").is_file() and (tmp_path / "temps.cues.vtt").is_file()
    assert compiled.wav() == (tmp_path / "temps.wav").read_bytes()
    flac = compiled.render(tmp_path / "temps.flac", sample_rate=44100, cues=False)
    assert flac["output"]["format"] == "flac" and flac["render"]["sampleRate"] == 44100
    assert not (tmp_path / "temps.cues.vtt").exists() or flac["cues"] is None
    assert compiled.wav(preserve_level=True) != compiled.wav()
    with pytest.raises(snr.SonarithmError):
        compiled.render(tmp_path / "temps.mp3")


def test_play_needs_ipython():
    compiled = snr.compile(TEMPS)
    if importlib.util.find_spec("IPython") is None:
        with pytest.raises(ImportError, match="IPython"):
            compiled.play()
    else:
        assert type(compiled.play()).__name__ == "Audio"


def test_the_notebook_display_is_text_first():
    page = snr.compile(TEMPS)._repr_html_()
    assert 'aria-label="SONARITHM sonification"' in page
    assert '<audio controls preload="none" aria-label="Sonification, 0.9 seconds"' in page
    assert "<table" not in page and "<script" not in page
    assert "Pitch follows temp, 400 to 1400 Hz" in page
    escaped = dict(TEMPS, name="<b>Temps</b>")
    assert "&lt;b&gt;" in snr.compile(escaped)._repr_html_()


CORPUS = ROOT / "crates" / "engines" / "sonarithm-plotsona" / "tests" / "corpus"


def test_a_chart_file_reads_with_its_report_and_plays():
    """C01-T12: a chart's report, its summary sentence and the spec of its template."""
    chart = snr.read_chart(CORPUS / "svg" / "vl-line.svg")
    assert chart.summary.startswith("36 points: 0 exact, 36 calibrated, 0 estimated, median confidence")
    report = chart.report()
    assert report["tier"] == "vector"
    assert report["provenance"] == {"vector": 36}
    assert [s["class"] for s in report["series"]] == ["vector"] * 3
    assert chart.template == "chart-line"
    assert chart.dataset().fields[:3] == ["series", "x", "y"]
    compiled = chart.compile()
    assert compiled.summary()["tracks"][0]["notes"] == 36


def test_a_function_lists_its_critical_points_and_plays():
    """C03-T07: a function's critical points as text, and its spec with their earcons."""
    function = snr.read_function("x^2 - 1", (-2, 2))
    points = function.critical_points()
    assert [p["text"] for p in points] == [
        "zero at x = -1",
        "local minimum at x = 0, y = -1",
        "zero at x = 1",
    ]
    assert function.summary == "f(x) = x^2 - 1 from x = -2 to 2, 1000 samples: 2 zeros, 1 local minimum"
    assert function.dataset().fields[:3] == ["x", "y", "dy"]
    compiled = function.compile()
    tracks = compiled.summary()["tracks"]
    assert tracks[0]["glides"] >= 1
    cues = next(t for t in tracks if t["id"] == "curvasona.cues")
    assert cues["earcons"] == 3


def test_a_function_that_cannot_be_read_raises_with_a_code():
    with pytest.raises(snr.SonarithmError) as raised:
        snr.read_function("2x", (0, 1))
    assert raised.value.diagnostics[0]["code"] == "SNR-ENG-400"


def test_a_file_that_is_no_chart_raises_with_a_code(tmp_path):
    path = tmp_path / "notes.svg"
    path.write_text("not a chart", encoding="utf-8")
    with pytest.raises(snr.SonarithmError) as raised:
        snr.read_chart(path)
    assert raised.value.diagnostics[0]["code"].startswith("SNR-ENG-3")

# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""SONARITHM turns graphs, charts and numerical data into structured, navigable sound.

A declarative spec plus data compiles into a Score, the canonical description of the
sound. Every default comes from published evidence and is reported with its design-rule
ID, so you can hear and read what was assumed.

>>> import sonarithm as snr
>>> c = snr.quick({"year": [2000, 2001, 2002], "anomaly": [0.39, 0.54, 0.63]},
...               x="year", y="anomaly", concept="temperature")
>>> print(c.describe())  # doctest: +SKIP

Data can be a pyarrow table, a polars or pandas DataFrame, any object with the Arrow
PyCapsule interface (``__arrow_c_stream__``), a dict of columns, a list of records, a 1-D
NumPy array, or a path to a CSV, JSON, Parquet, Arrow or spreadsheet file.
"""

from __future__ import annotations

import base64
import html
import json
import math
import os
from collections.abc import Mapping, Sequence
from typing import Any, Union

from . import _core
from ._core import SPEC_VERSION, Dataset, SonarithmError, __version__

__all__ = [
    "SPEC_VERSION",
    "Chart",
    "Compiled",
    "Dataset",
    "Function",
    "SonarithmError",
    "Spec",
    "__version__",
    "compile",
    "dataset",
    "quick",
    "read_chart",
    "read_function",
    "schema",
]

PathLike = Union[str, "os.PathLike[str]"]


class Spec:
    """A spec v0.1 document (RFC-0001). Round-trips as a dict or as JSON text."""

    __slots__ = ("_inner",)

    def __init__(self, inner: _core.Spec) -> None:
        self._inner = inner

    @classmethod
    def from_dict(cls, value: Mapping[str, Any], *, name: str | None = None) -> Spec:
        """A spec from a dict, such as one built in code."""
        return cls(_core.Spec.from_json(json.dumps(value, allow_nan=False), name))

    @classmethod
    def from_json(cls, text: str, *, name: str | None = None) -> Spec:
        """A spec from JSON text."""
        return cls(_core.Spec.from_json(text, name))

    @classmethod
    def from_file(cls, path: PathLike) -> Spec:
        """Reads a spec file; its ``file`` sources resolve next to it."""
        return cls(_core.Spec.from_file(os.fspath(path)))

    @property
    def name(self) -> str:
        """The name used in messages, such as ``temps.spec.json``."""
        return self._inner.name

    def to_dict(self) -> dict[str, Any]:
        """The spec as a dict."""
        return json.loads(self._inner.to_json())

    def to_json(self) -> str:
        """The spec as JSON text."""
        return self._inner.to_json()

    def validate(self) -> list[dict[str, Any]]:
        """Every diagnostic of validation without data (syntax, shape, meaning, defaults)."""
        return self._inner.validate()

    def __repr__(self) -> str:
        return f"Spec(name={self.name!r})"


class Compiled:
    """A compiled sonification: the Score, its diagnostics and a description.

    In a notebook it displays as an accessible summary: headings, the L1/L2 description as
    text, and the diagnostic counts.
    """

    __slots__ = ("_inner", "spec")

    def __init__(self, inner: _core.Compiled, spec: Spec | None = None) -> None:
        self._inner = inner
        #: The spec this was compiled from, when ``quick`` generated it.
        self.spec = spec

    @property
    def diagnostics(self) -> list[dict[str, Any]]:
        """Warnings and applied defaults, each with ``severity``, ``code``, ``rules``,
        ``message``, ``location`` and ``data``."""
        return self._inner.diagnostics

    @property
    def score_hash(self) -> str:
        """The canonical Score hash (hex); equal hashes mean byte-identical Scores."""
        return self._inner.score_hash

    @property
    def canonical_spec(self) -> str:
        """The canonical spec with defaults applied (RFC 8785 JSON)."""
        return self._inner.canonical_spec

    def summary(self) -> dict[str, Any]:
        """What ``sonarithm inspect --json`` prints as ``summary``."""
        return self._inner.summary()

    def describe(self) -> str:
        """The L1/L2 description, one sentence per line. The same Score always gives the
        same text."""
        return self._inner.describe()

    def description(self) -> dict[str, list[str]]:
        """The description as ``{"l1": [...], "l2": [...]}``."""
        return self._inner.description()

    def score(self) -> dict[str, Any]:
        """The Score IR as a dict (05 §4)."""
        return self._inner.score()

    def render(
        self,
        path: PathLike,
        *,
        sample_rate: int | None = None,
        preserve_level: bool = False,
        cues: bool = True,
    ) -> dict[str, Any]:
        """Renders to ``path`` (``.wav`` or ``.flac``) with ``<stem>.render.json`` (the
        render manifest) and ``<stem>.cues.vtt`` next to it; returns the manifest.

        The level is normalized to the spec's loudness target (-16 LUFS within -1 dBTP by
        default) unless ``preserve_level``. The same spec and seed always give the same
        bytes, here and in the ``sonarithm render`` command.
        """
        return self._inner.render(
            os.fspath(path), sample_rate=sample_rate, preserve_level=preserve_level, cues=cues
        )

    def wav(self, *, sample_rate: int | None = None, preserve_level: bool = False) -> bytes:
        """The 24-bit WAV bytes a ``.wav`` render would write, rendered in memory."""
        data, _rate = self._inner.wav(sample_rate=sample_rate, preserve_level=preserve_level)
        return data

    def play(self) -> Any:
        """The sound as an ``IPython.display.Audio`` for a notebook (needs IPython:
        ``pip install sonarithm[notebook]``)."""
        try:
            from IPython.display import Audio
        except ImportError as missing:
            raise ImportError(
                "play() needs IPython (pip install sonarithm[notebook]); "
                "use wav() for the bytes or render(path) for a file"
            ) from missing
        return Audio(data=self.wav())

    def __repr__(self) -> str:
        return repr(self._inner)

    def _repr_html_(self) -> str:
        summary = self.summary()
        description = self.description()
        counts = summary["diagnostics"]
        title = html.escape(description["l1"][0] if description["l1"] else summary["spec"])
        items = "".join(
            f"<li>{html.escape(line)}</li>" for line in description["l1"][1:] + description["l2"]
        )
        seconds = summary["duration_s"]
        if seconds <= _EMBED_SECONDS:
            sound = base64.b64encode(self.wav()).decode("ascii")
            audio = (
                f'<audio controls preload="none" aria-label="Sonification, {seconds:.1f} seconds" '
                f'src="data:audio/wav;base64,{sound}"></audio>'
            )
        else:
            audio = f"<p>{seconds:.1f} seconds: call <code>play()</code> to listen.</p>"
        return (
            '<section class="sonarithm" aria-label="SONARITHM sonification">'
            f"<p><strong>{title}</strong></p>"
            f"<ul>{items}</ul>"
            f"{audio}"
            f"<p>Diagnostics: {counts['errors']} errors, {counts['warnings']} warnings, "
            f"{counts['info']} applied defaults and choices.</p>"
            "</section>"
        )


#: Scores up to this length embed their audio in the notebook display (about 290 kB per
#: second of 48 kHz 24-bit stereo WAV); longer ones point at ``play()``.
_EMBED_SECONDS = 30.0


def _is_pandas(value: object) -> bool:
    return type(value).__module__.split(".")[0] == "pandas"


def _clean(value: Any) -> Any:
    """A JSON-safe scalar: NaN and infinities become null, NumPy scalars plain numbers."""
    if hasattr(value, "item") and not isinstance(value, (str, bytes)):
        value = value.item()
    if isinstance(value, float) and not math.isfinite(value):
        return None
    return value


def _records_json(rows: Sequence[Mapping[str, Any]]) -> str:
    return json.dumps(
        [{str(k): _clean(v) for k, v in row.items()} for row in rows], allow_nan=False
    )


def _columns_json(columns: Mapping[str, Any]) -> str:
    names = [str(name) for name in columns]
    values = [list(column.tolist() if hasattr(column, "tolist") else column) for column in columns.values()]
    lengths = {len(column) for column in values}
    if len(lengths) > 1:
        raise ValueError(f"columns have different lengths: {sorted(lengths)}")
    rows = [dict(zip(names, row)) for row in zip(*values)]
    return _records_json(rows)


def _tabular(data: Any, label: str) -> Any:
    """Data in a form ``_core`` reads: a ``Dataset``, a path, or an Arrow object."""
    if isinstance(data, Dataset):
        return data
    if isinstance(data, (str, os.PathLike)):
        return os.fspath(data)
    if _is_pandas(data):
        try:
            import pyarrow as pa
        except ImportError:
            # pandas writes JSON itself, so pyarrow is optional.
            return _core._records(data.to_json(orient="records", date_format="iso"), label)
        return pa.Table.from_pandas(data, preserve_index=False)
    if hasattr(data, "__arrow_c_stream__") or hasattr(data, "__arrow_c_array__"):
        return data
    if isinstance(data, Mapping):
        return _core._records(_columns_json(data), label)
    if hasattr(data, "ndim") and hasattr(data, "tolist") and getattr(data, "ndim", 0) == 1:
        values = data.tolist()
        return _core._records(_columns_json({"x": list(range(len(values))), "y": values}), label)
    if isinstance(data, Sequence) and data and isinstance(data[0], Mapping):
        return _core._records(_records_json(data), label)
    raise TypeError(
        f"{label}: expected a pyarrow table, a polars or pandas DataFrame, a dict of columns, "
        f"a list of records, a 1-D array or a file path, not {type(data).__name__}"
    )


def dataset(data: Any, *, label: str = "data") -> Dataset:
    """Reads data once, with inferred field types and roles, to compile it many times."""
    tabular = _tabular(data, label)
    return tabular if isinstance(tabular, Dataset) else Dataset(tabular, label)


def compile(spec: Spec | Mapping[str, Any] | str, data: Mapping[str, Any] | None = None) -> Compiled:
    """Compiles a spec. ``data`` maps the IDs of ``{"kind": "dataset", "id": ...}``
    sources to data; ``file`` and ``inline`` sources need none.

    Raises ``SonarithmError`` with ``diagnostics`` when the spec or data has errors.
    """
    if isinstance(spec, Mapping):
        spec = Spec.from_dict(spec)
    elif isinstance(spec, str):
        spec = Spec.from_json(spec)
    tables = {str(key): _tabular(value, str(key)) for key, value in (data or {}).items()}
    return Compiled(_core.compile(spec._inner, tables))


def quick(
    data: Any,
    x: str | None = None,
    y: str | None = None,
    *,
    mark: str | None = None,
    range_hz: tuple[float, float] | None = None,
    concept: str | None = None,
    name: str | None = None,
) -> Compiled:
    """The quick start: ``x`` along time, ``y`` as pitch, every other choice from the
    evidence-backed defaults (400–1400 Hz log-frequency pitch, 3 points per second).

    Without ``x`` or ``y`` the fields are chosen by role, then by kind. ``mark`` is
    ``"tone"`` (default), ``"glide"`` or the experimental ``"auto"``. ``concept`` (such as
    ``"temperature"``) sets the polarity of ``y``. The generated spec is ``.spec`` on the
    result.
    """
    spec, compiled = _core.quick(
        _tabular(data, "data"),
        x=x,
        y=y,
        mark=mark,
        range_hz=range_hz,
        concept=concept,
        name=name,
    )
    return Compiled(compiled, Spec(spec))


class Chart:
    """A chart's data as PLOTSONA extracted it from a chart file (C01-T12).

    ``summary`` is the sentence the app speaks when the chart opens, such as
    ``"36 points: 0 exact, 36 calibrated, 0 estimated, median confidence 0.99"``;
    ``report()`` has the details (tier, series, provenance per point, diagnostics).
    """

    __slots__ = ("_inner",)

    def __init__(self, inner: _core.Chart) -> None:
        self._inner = inner

    @property
    def summary(self) -> str:
        """How many values were read exactly, calibrated from the drawing, or estimated."""
        return self._inner.summary

    @property
    def template(self) -> str | None:
        """The chart-type template the chart's type calls for, or ``None``."""
        return self._inner.template

    def report(self) -> dict[str, Any]:
        """The extraction report, as ``sonarithm inspect --json`` prints it."""
        return self._inner.report()

    def spec(self) -> Spec | None:
        """A spec that plays the chart file through its template, or ``None``."""
        inner = self._inner.spec()
        return None if inner is None else Spec(inner)

    def dataset(self) -> Dataset:
        """The chart's data as a long table: ``series``, ``x``, ``y``, with provenance."""
        return self._inner.dataset()

    def compile(self) -> Compiled:
        """Compiles the chart through its template."""
        spec = self.spec()
        if spec is None:
            raise SonarithmError("no chart-type template plays this chart yet")
        return compile(spec)

    def __repr__(self) -> str:
        return f"Chart({self.summary!r})"


def read_chart(path: PathLike) -> Chart:
    """Extracts the chart in a file: Office (XLSX, PPTX, DOCX), Vega, Vega-Lite, Plotly or
    Highcharts JSON, an HTML page holding one, SVG or vector PDF. The file is recognised by
    its contents. Nothing is fetched and no script runs.

    Raises ``SonarithmError`` with ``diagnostics`` when the file is not a chart it reads.
    """
    return Chart(_core.read_chart(os.fspath(path)))


class Function:
    """A function of ``x`` as CURVASONA sampled it (C03-T07).

    ``critical_points()`` lists its zeros, extrema, inflections, asymptotes, holes, jumps
    and undefined spans, each with a ``text`` such as ``"local maximum at x = 1.5708, y = 1"``:
    the text equivalent of the earcons its spec plays.
    """

    __slots__ = ("_inner",)

    def __init__(self, inner: _core.Function) -> None:
        self._inner = inner

    @property
    def expr(self) -> str:
        """The expression in its canonical form."""
        return self._inner.expr

    @property
    def summary(self) -> str:
        """The expression, the domain and how many of each feature, as one sentence."""
        return self._inner.summary

    def critical_points(self) -> list[dict[str, Any]]:
        """The critical points and spans in increasing ``x``, as ``inspect --json`` lists them."""
        return self._inner.critical_points()

    def spec(self, duration: float | None = None) -> Spec:
        """A spec that plays the function as a glide over ``duration`` seconds (default 8),
        with its critical points as earcons."""
        return Spec(self._inner.spec(duration))

    def dataset(self) -> Dataset:
        """The sampled table: ``x``, ``y`` and ``dy``, provenance ``synthetic``."""
        return self._inner.dataset()

    def compile(self, duration: float | None = None) -> Compiled:
        """Compiles :meth:`spec`."""
        return compile(self.spec(duration))

    def __repr__(self) -> str:
        return f"Function({self.summary!r})"


def read_function(
    expr: str,
    domain: tuple[float, float],
    *,
    samples: int | None = None,
    step: float | None = None,
    variables: Mapping[str, float] | None = None,
) -> Function:
    """Samples ``expr``, a function of ``x`` such as ``"sin(x)/x"``, over ``domain``
    (``samples`` points, default 1000, or points ``step`` apart) and finds its critical
    points. Products are written with ``*``; ``variables`` names constants.

    Raises ``SonarithmError`` with ``diagnostics`` (``SNR-ENG-400`` to ``404``) when the
    expression cannot be read or the grid is too large.
    """
    low, high = domain
    return Function(
        _core.read_function(
            expr,
            (float(low), float(high)),
            samples=samples,
            step=step,
            variables=None if variables is None else dict(variables),
        )
    )


def schema(kind: str = "spec") -> dict[str, Any]:
    """The JSON Schema of ``kind`` (``"spec"``) as a dict."""
    return json.loads(_core.schema(kind))

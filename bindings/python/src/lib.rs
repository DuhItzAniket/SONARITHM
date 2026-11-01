// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `sonarithm._core`, the native module of the Python package (11 §3).
//!
//! A thin layer over the `sonarithm` facade: specs, compiles and the quick builder, with
//! data from any object that implements the Arrow `PyCapsule` interface
//! (`__arrow_c_stream__` or `__arrow_c_array__`: pyarrow, polars, pandas 2.2 and later),
//! read without copying the column buffers. The pure-Python package (`python/sonarithm`)
//! adapts other inputs and adds the notebook display. Compiles release the GIL.
//!
//! Python sees diagnostics as dicts with the same fields as the CLI's `--json` output.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use arrow_array::RecordBatch;
use arrow_pyarrow::FromPyArrow;
use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyString};
use serde_json::Value;
use sonarithm::{
    Datasets, Diagnostics, Format, Mark, Normalization, ReadOptions, RenderOptions, SchemaKind,
    Sonarithm,
};

create_exception!(
    _core,
    SonarithmError,
    PyException,
    "A spec, data or compile problem. `diagnostics` lists every diagnostic as a dict."
);

/// A JSON value as a Python object (dict, list, str, int, float, bool or None).
fn to_py<'py>(py: Python<'py>, value: &Value) -> PyResult<Bound<'py, PyAny>> {
    Ok(match value {
        Value::Null => py.None().into_bound(py),
        Value::Bool(flag) => flag.into_pyobject(py)?.to_owned().into_any(),
        Value::Number(number) => {
            if let Some(int) = number.as_i64() {
                int.into_pyobject(py)?.into_any()
            } else if let Some(int) = number.as_u64() {
                int.into_pyobject(py)?.into_any()
            } else {
                number
                    .as_f64()
                    .unwrap_or(f64::NAN)
                    .into_pyobject(py)?
                    .into_any()
            }
        }
        Value::String(text) => PyString::new(py, text).into_any(),
        Value::Array(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(to_py(py, item)?)?;
            }
            list.into_any()
        }
        Value::Object(map) => {
            let dict = PyDict::new(py);
            for (key, item) in map {
                dict.set_item(key, to_py(py, item)?)?;
            }
            dict.into_any()
        }
    })
}

/// A serializable value as a Python object.
fn serialized<'py>(py: Python<'py>, value: &impl serde::Serialize) -> PyResult<Bound<'py, PyAny>> {
    let value =
        serde_json::to_value(value).map_err(|err| PyValueError::new_err(err.to_string()))?;
    to_py(py, &value)
}

/// A `SonarithmError` carrying `diagnostics`.
fn error_with(py: Python<'_>, message: String, diagnostics: &Diagnostics) -> PyErr {
    let err = SonarithmError::new_err(message);
    if let Ok(list) = serialized(py, diagnostics) {
        let _ = err.value(py).setattr("diagnostics", list);
    }
    err
}

fn facade_error(py: Python<'_>, err: &sonarithm::Error) -> PyErr {
    error_with(py, err.to_string(), &err.diagnostics())
}

/// Python data as a bundle: an object with `__arrow_c_stream__` (a table or reader) or
/// `__arrow_c_array__` (a record batch), or a path to a data file. `label` names it in
/// diagnostics.
fn bundle(py: Python<'_>, data: &Bound<'_, PyAny>, label: &str) -> PyResult<sonarithm::DataBundle> {
    let loaded = if let Ok(path) = data.extract::<PathBuf>() {
        sonarithm::read_file(&path, None, &ReadOptions::new())
    } else if data.hasattr("__arrow_c_stream__")? {
        let table = arrow_pyarrow::Table::from_pyarrow_bound(data)?;
        let (batches, schema) = table.into_inner();
        sonarithm::read_batches(&schema, batches, label, &ReadOptions::new())
    } else if data.hasattr("__arrow_c_array__")? {
        let batch = RecordBatch::from_pyarrow_bound(data)?;
        sonarithm::read_batches(&batch.schema(), vec![batch], label, &ReadOptions::new())
    } else {
        return Err(pyo3::exceptions::PyTypeError::new_err(format!(
            "data `{label}` is not tabular: pass a pyarrow table, a polars or pandas DataFrame, a dict of columns or a file path"
        )));
    };
    loaded.map(|loaded| loaded.bundle).map_err(|err| {
        error_with(py, err.to_string(), &{
            let mut one = Diagnostics::new();
            one.push(err.to_diagnostic());
            one
        })
    })
}

/// JSON records (a list of objects) as a bundle, for inputs without the Arrow interface.
#[pyfunction]
#[pyo3(signature = (records, label = "records"))]
fn _records(py: Python<'_>, records: &str, label: &str) -> PyResult<Dataset> {
    sonarithm::read_bytes(records.as_bytes(), Format::Json, label, &ReadOptions::new())
        .map(|loaded| Dataset {
            bundle: loaded.bundle,
        })
        .map_err(|err| {
            let mut one = Diagnostics::new();
            one.push(err.to_diagnostic());
            error_with(py, err.to_string(), &one)
        })
}

/// Data read into SONARITHM's data layer, with inferred field types and roles.
#[pyclass(frozen, module = "sonarithm._core")]
struct Dataset {
    bundle: sonarithm::DataBundle,
}

#[pymethods]
impl Dataset {
    /// Reads data: an Arrow table, reader or record batch, or a path to a data file.
    #[new]
    #[pyo3(signature = (data, label = "data"))]
    fn new(py: Python<'_>, data: &Bound<'_, PyAny>, label: &str) -> PyResult<Self> {
        Ok(Self {
            bundle: bundle(py, data, label)?,
        })
    }

    /// The number of rows.
    #[getter]
    fn rows(&self) -> u64 {
        self.bundle.primary().map_or(0, sonarithm::Table::num_rows)
    }

    /// The field names, in order.
    #[getter]
    fn fields(&self) -> Vec<String> {
        self.bundle.primary().map_or_else(Vec::new, |table| {
            table
                .schema()
                .fields()
                .iter()
                .map(|field| field.name().clone())
                .collect()
        })
    }

    fn __repr__(&self) -> String {
        format!("Dataset(rows={}, fields={:?})", self.rows(), self.fields())
    }
}

/// A spec document (RFC-0001).
#[pyclass(frozen, module = "sonarithm._core")]
struct Spec {
    inner: sonarithm::Spec,
}

#[pymethods]
impl Spec {
    /// A spec from JSON text.
    #[staticmethod]
    #[pyo3(signature = (text, name = None))]
    fn from_json(py: Python<'_>, text: String, name: Option<&str>) -> PyResult<Self> {
        let spec = sonarithm::Spec::from_json_str(text).map_err(|err| facade_error(py, &err))?;
        Ok(Self {
            inner: match name {
                Some(name) => spec.with_name(name),
                None => spec,
            },
        })
    }

    /// Reads a spec file; its `file` sources resolve next to it.
    #[staticmethod]
    fn from_file(py: Python<'_>, path: PathBuf) -> PyResult<Self> {
        sonarithm::Spec::from_file(path)
            .map(|inner| Self { inner })
            .map_err(|err| facade_error(py, &err))
    }

    /// The JSON text.
    fn to_json(&self) -> &str {
        self.inner.text()
    }

    /// The name used in messages, such as `temps.spec.json`.
    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }

    /// Every diagnostic of validation without data, as dicts (04 §8).
    fn validate<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        serialized(py, &self.inner.validate())
    }

    fn __repr__(&self) -> String {
        format!("Spec(name={:?})", self.inner.name())
    }
}

/// A compiled sonification: the Score, its diagnostics and descriptions.
#[pyclass(frozen, module = "sonarithm._core")]
struct Compiled {
    inner: sonarithm::Compiled,
}

#[pymethods]
impl Compiled {
    /// Warnings and applied defaults, as dicts with `severity`, `code`, `rules`,
    /// `message`, `location` and `data`.
    #[getter]
    fn diagnostics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        serialized(py, self.inner.diagnostics())
    }

    /// The summary `sonarithm inspect --json` prints.
    fn summary<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        serialized(py, &self.inner.summary())
    }

    /// The L1/L2 description as text, one sentence per line (deterministic).
    fn describe(&self) -> String {
        self.inner.describe().to_string()
    }

    /// The L1/L2 description as a dict with `l1` and `l2` lists.
    fn description<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        serialized(py, &self.inner.describe())
    }

    /// The canonical Score hash (hex).
    #[getter]
    fn score_hash(&self, py: Python<'_>) -> PyResult<String> {
        self.inner
            .score_hash()
            .map_err(|err| facade_error(py, &err))
    }

    /// The canonical spec with defaults applied (RFC 8785 JSON).
    #[getter]
    fn canonical_spec(&self) -> &str {
        self.inner.canonical_spec()
    }

    /// The Score as a dict (05 §4).
    fn score<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        serialized(py, self.inner.score())
    }

    /// Renders to `path` (`.wav` or `.flac`) with `<stem>.render.json` and, unless
    /// `cues` is false, `<stem>.cues.vtt`; returns the manifest as a dict. The GIL is
    /// released while rendering.
    #[pyo3(signature = (path, *, sample_rate = None, preserve_level = false, cues = true))]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "PyO3 extracts arguments by value"
    )]
    fn render<'py>(
        &self,
        py: Python<'py>,
        path: PathBuf,
        sample_rate: Option<u32>,
        preserve_level: bool,
        cues: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let options = render_options(sample_rate, preserve_level, cues);
        let rendered = py
            .detach(|| Sonarithm::new().render_to_file(&self.inner, &path, &options))
            .map_err(|err| facade_error(py, &err))?;
        serialized(py, &rendered.manifest)
    }

    /// The 24-bit WAV bytes a `.wav` render would write, and the sample rate, rendered in
    /// memory (for notebook playback). The GIL is released while rendering.
    #[pyo3(signature = (*, sample_rate = None, preserve_level = false))]
    fn wav<'py>(
        &self,
        py: Python<'py>,
        sample_rate: Option<u32>,
        preserve_level: bool,
    ) -> PyResult<(Bound<'py, PyBytes>, u32)> {
        let options = render_options(sample_rate, preserve_level, false);
        let (bytes, rate) = py
            .detach(|| Sonarithm::new().render_wav(&self.inner, &options))
            .map_err(|err| facade_error(py, &err))?;
        Ok((PyBytes::new(py, &bytes), rate))
    }

    fn __repr__(&self) -> String {
        let summary = self.inner.summary();
        format!(
            "Compiled(spec={:?}, tracks={}, duration_s={})",
            summary.spec,
            summary.tracks.len(),
            summary.duration_s
        )
    }
}

fn render_options(sample_rate: Option<u32>, preserve_level: bool, cues: bool) -> RenderOptions {
    let mut options = RenderOptions::new();
    if let Some(hz) = sample_rate {
        options = options.with_sample_rate(hz);
    }
    if preserve_level {
        options = options.with_normalization(Normalization::Preserve);
    }
    if !cues {
        options = options.without_cues();
    }
    options
}

/// Compiles a spec. `data` maps `dataset` source IDs to data; the GIL is released while
/// the compiler runs.
#[pyfunction]
#[pyo3(signature = (spec, data = None))]
fn compile(py: Python<'_>, spec: &Spec, data: Option<&Bound<'_, PyDict>>) -> PyResult<Compiled> {
    let mut datasets = Datasets::new();
    if let Some(data) = data {
        for (key, value) in data.iter() {
            let id: String = key.extract()?;
            let bundle = match value.cast::<Dataset>() {
                Ok(dataset) => dataset.get().bundle.clone(),
                Err(_) => bundle(py, &value, &id)?,
            };
            datasets.insert(id, bundle);
        }
    }
    let spec = spec.inner.clone();
    py.detach(|| Sonarithm::new().compile_with(&spec, &datasets))
        .map(|inner| Compiled { inner })
        .map_err(|err| facade_error(py, &err))
}

/// The quick start: x along time, y as pitch, every other choice from the defaults.
#[pyfunction]
#[pyo3(signature = (data, *, x = None, y = None, mark = None, range_hz = None, concept = None, name = None))]
#[expect(
    clippy::too_many_arguments,
    reason = "keyword arguments of one Python function"
)]
fn quick(
    py: Python<'_>,
    data: &Bound<'_, PyAny>,
    x: Option<String>,
    y: Option<String>,
    mark: Option<&str>,
    range_hz: Option<(f64, f64)>,
    concept: Option<String>,
    name: Option<String>,
) -> PyResult<(Spec, Compiled)> {
    let snr = Sonarithm::new();
    let mut quick = if let Ok(path) = data.extract::<PathBuf>() {
        snr.quick()
            .data_file(path)
            .map_err(|err| facade_error(py, &err))?
    } else {
        let bundle = match data.cast::<Dataset>() {
            Ok(dataset) => dataset.get().bundle.clone(),
            Err(_) => bundle(py, data, "data")?,
        };
        snr.quick().data(bundle)
    };
    if let Some(x) = x {
        quick = quick.x(x);
    }
    if let Some(y) = y {
        quick = quick.y(y);
    }
    if let Some(mark) = mark {
        quick = quick.mark(match mark {
            "tone" => Mark::Tone,
            "glide" => Mark::Glide,
            "auto" => Mark::Auto,
            other => {
                return Err(PyValueError::new_err(format!(
                    "mark `{other}` is not one of `tone`, `glide`, `auto`"
                )));
            }
        });
    }
    if let Some((low, high)) = range_hz {
        if !(low.is_finite() && high.is_finite() && 0.0 < low && low < high) {
            return Err(PyValueError::new_err(format!(
                "range_hz must be (low, high) with 0 < low < high, not ({low}, {high})"
            )));
        }
        quick = quick.range_hz(low, high);
    }
    if let Some(concept) = concept {
        quick = quick.concept(concept);
    }
    if let Some(name) = name {
        quick = quick.name(name);
    }
    let spec = quick.spec().map_err(|err| facade_error(py, &err))?;
    let compiled = py
        .detach(|| quick.compile())
        .map_err(|err| facade_error(py, &err))?;
    Ok((Spec { inner: spec }, Compiled { inner: compiled }))
}

/// A chart's data as PLOTSONA extracted it (C01-T12): the report, the summary sentence and
/// the spec that plays it.
#[pyclass(frozen, module = "sonarithm._core")]
struct Chart {
    inner: sonarithm::Chart,
}

#[pymethods]
impl Chart {
    /// The extraction report as a dict: `locator`, `container`, `tier`, `extractor`,
    /// `chart_type`, `series` (each with `class`, `confidence`, `residual`), `points`,
    /// `provenance` and `diagnostics`; the same as `sonarithm inspect --json`.
    fn report<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        serialized(py, self.inner.report())
    }

    /// The summary sentence, such as "36 points: 0 exact, 36 calibrated, 0 estimated,
    /// median confidence 0.99".
    #[getter]
    fn summary(&self) -> String {
        self.inner.summary()
    }

    /// The chart-type template its type calls for, or `None`.
    #[getter]
    fn template(&self) -> Option<&'static str> {
        self.inner.template()
    }

    /// The spec that plays the chart through its template, or `None`.
    fn spec(&self) -> Option<Spec> {
        self.inner.spec().map(|inner| Spec { inner })
    }

    /// The chart's long table (`series`, `x`, `y`, with provenance) as a dataset.
    fn dataset(&self) -> Dataset {
        Dataset {
            bundle: self.inner.bundle().clone(),
        }
    }

    fn __repr__(&self) -> String {
        format!("Chart({:?})", self.inner.summary())
    }
}

/// Extracts the chart in a file (Office, chart JSON, an HTML page holding one, SVG or
/// vector PDF), recognised by its contents. Nothing is fetched and no script runs.
#[pyfunction]
#[expect(
    clippy::needless_pass_by_value,
    reason = "PyO3 extracts arguments by value"
)]
fn read_chart(py: Python<'_>, path: PathBuf) -> PyResult<Chart> {
    py.detach(|| sonarithm::read_chart(&path))
        .map(|inner| Chart { inner })
        .map_err(|err| facade_error(py, &err))
}

/// A sampled function (C03-T07): its critical points as text, the spec that plays it as a
/// glide with critical-point earcons, and its table.
#[pyclass(frozen, module = "sonarithm._core")]
struct Function {
    inner: sonarithm::Function,
}

#[pymethods]
impl Function {
    /// The expression in its canonical form.
    #[getter]
    fn expr(&self) -> String {
        self.inner.expr().to_owned()
    }

    /// One sentence: the expression, the domain and how many of each feature.
    #[getter]
    fn summary(&self) -> String {
        self.inner.summary()
    }

    /// The critical points in increasing `x`, each a dict with `kind`, `x`, `end` (spans
    /// only), `y` (where it has one) and `text`: the text equivalent of the cues.
    fn critical_points<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        serialized(py, &self.inner.critical_points())
    }

    /// The spec that plays the function as one glide over `duration` seconds (default 8,
    /// the example of 04 §6.3), with its critical points as earcons.
    #[pyo3(signature = (duration = None))]
    fn spec(&self, duration: Option<f64>) -> Spec {
        Spec {
            inner: self
                .inner
                .spec_lasting(duration.unwrap_or(sonarithm::FUNCTION_DURATION_S)),
        }
    }

    /// The table `x`, `y`, `dy` (provenance `synthetic`) as a dataset.
    fn dataset(&self) -> Dataset {
        Dataset {
            bundle: self.inner.bundle().clone(),
        }
    }

    fn __repr__(&self) -> String {
        format!("Function({:?})", self.inner.summary())
    }
}

/// Samples a function of `x` (the expression language of ADR-0028) over `domain` and finds
/// its critical points.
#[pyfunction]
#[pyo3(signature = (expr, domain, *, samples = None, step = None, variables = None))]
fn read_function(
    py: Python<'_>,
    expr: String,
    domain: (f64, f64),
    samples: Option<u32>,
    step: Option<f64>,
    variables: Option<BTreeMap<String, f64>>,
) -> PyResult<Function> {
    let mut source = sonarithm::FunctionSpec::new(expr, [domain.0, domain.1]);
    source.samples = samples;
    source.step = step;
    source.variables = variables.unwrap_or_default();
    py.detach(|| sonarithm::read_function(&source))
        .map(|inner| Function { inner })
        .map_err(|err| facade_error(py, &err))
}

/// The JSON Schema of `kind` (`spec`).
#[pyfunction]
#[pyo3(signature = (kind = "spec"))]
fn schema(kind: &str) -> PyResult<String> {
    match kind {
        "spec" => Ok(sonarithm::schema(SchemaKind::Spec)),
        other => Err(PyValueError::new_err(format!(
            "schema kind `{other}` is not one of `spec`"
        ))),
    }
}

#[pymodule]
fn _core(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", sonarithm::VERSION)?;
    module.add("SPEC_VERSION", sonarithm::SPEC_VERSION.to_string())?;
    module.add("SonarithmError", module.py().get_type::<SonarithmError>())?;
    module.add_class::<Dataset>()?;
    module.add_class::<Spec>()?;
    module.add_class::<Compiled>()?;
    module.add_class::<Chart>()?;
    module.add_function(wrap_pyfunction!(read_chart, module)?)?;
    module.add_class::<Function>()?;
    module.add_function(wrap_pyfunction!(read_function, module)?)?;
    module.add_function(wrap_pyfunction!(compile, module)?)?;
    module.add_function(wrap_pyfunction!(quick, module)?)?;
    module.add_function(wrap_pyfunction!(schema, module)?)?;
    module.add_function(wrap_pyfunction!(_records, module)?)?;
    Ok(())
}

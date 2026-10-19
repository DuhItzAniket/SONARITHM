// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! PB-04 (12 §8): compile time for 100k rows × 4 tracks (budget 200 ms) and for 1M rows
//! (budget 1 s), through the real compiler with NUMISONA. The data is loaded once and handed
//! to every compile, so the benchmark measures compilation, not file reading (PB-05).
//!
//! Run with `cargo bench -p sonarithm-numisona --bench compile`; results go to
//! docs/research-paper/benchmarks-log.md.

#![allow(
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    missing_docs,
    reason = "benchmark harness"
)]

use std::hint::black_box;
use std::sync::Arc;

use arrow_array::{ArrayRef, Float64Array, Int64Array, RecordBatch};
use arrow_schema::{DataType, Field, Schema};
use criterion::{Criterion, criterion_group, criterion_main};
use sonarithm_compiler::Compiler;
use sonarithm_core::engine::{
    CancelToken, CompileContext, CompileError, Engine, EngineManifest, Registrar, RegistryError,
    Source, SourceKind, SpecFragment,
};
use sonarithm_core::ids::{EngineId, TableId};
use sonarithm_core::score::EngineVersion;
use sonarithm_data::{BundleId, DataBundle, FieldMeta, FieldType, Role, SourceDescriptor, Table};
use sonarithm_numisona::Numisona;

/// `rows` rows: an x column and four series (a slow sine, a ramp, a sawtooth, a mix).
fn bundle(rows: usize) -> DataBundle {
    let x: Vec<i64> = (0..rows as i64).collect();
    let series = |f: &dyn Fn(f64) -> f64| -> ArrayRef {
        Arc::new(Float64Array::from_iter_values(
            (0..rows).map(|i| f(i as f64)),
        ))
    };
    let columns: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from(x)),
        series(&|i| sonarithm_core::math::sin(i / 500.0)),
        series(&|i| i * 0.001),
        series(&|i| (i % 97.0) / 97.0),
        series(&|i| sonarithm_core::math::sin(i / 50.0) + (i % 13.0) / 13.0),
    ];
    let x_meta = FieldMeta::new()
        .with_type(FieldType::Quantitative)
        .with_role(Role::X);
    let value = FieldMeta::new().with_type(FieldType::Quantitative);
    let fields = vec![
        x_meta.apply(Field::new("x", DataType::Int64, false)),
        value.apply(Field::new("a", DataType::Float64, false)),
        value.apply(Field::new("b", DataType::Float64, false)),
        value.apply(Field::new("c", DataType::Float64, false)),
        value.apply(Field::new("d", DataType::Float64, false)),
    ];
    let batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), columns).unwrap();
    let table = Table::from_batch(TableId(0), "bench", batch);
    let source = SourceDescriptor::new(sonarithm_data::SourceKind::Memory, "bench", "memory");
    DataBundle::single(BundleId(0), source, table)
}

/// Hands the prepared bundle to every compile.
struct Given(DataBundle);

impl Source<DataBundle> for Given {
    fn ingest(&self, _: &SpecFragment, _: &mut CompileContext) -> Result<DataBundle, CompileError> {
        Ok(self.0.clone())
    }
}

struct Data(DataBundle);

impl Engine<DataBundle> for Data {
    fn manifest(&self) -> EngineManifest {
        EngineManifest::new(
            EngineId::from_static("platform"),
            "bench data",
            EngineVersion("0.0.0".to_owned()),
        )
    }

    fn register(&self, registrar: &mut Registrar<'_, DataBundle>) -> Result<(), RegistryError> {
        registrar.source(SourceKind::File, Arc::new(Given(self.0.clone())))
    }
}

/// Four tone tracks, one per series, with the defaults of the minimal spec; 1000 points/s
/// keeps a million rows inside the compile budget's six hours.
fn spec(tracks: &[&str]) -> String {
    let tracks: Vec<String> = tracks
        .iter()
        .map(|field| format!(r#"{{ "id": "{field}", "mark": "tone", "channels": {{ "pitch": {{ "field": "{field}" }} }} }}"#))
        .collect();
    format!(
        r#"{{ "specVersion": "0.1", "data": {{ "source": {{ "kind": "file", "path": "bench.arrow" }} }},
            "substrate": {{ "time": {{ "pace": {{ "pointsPerSecond": 1000 }} }} }},
            "tracks": [{}] }}"#,
        tracks.join(",")
    )
}

fn compile_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("pb04_compile");
    group.sample_size(20);
    for (name, rows, fields) in [
        ("100k_rows_4_tracks", 100_000, &["a", "b", "c", "d"][..]),
        ("1m_rows_1_track", 1_000_000, &["a"][..]),
        ("1m_rows_4_tracks", 1_000_000, &["a", "b", "c", "d"][..]),
    ] {
        let data = Data(bundle(rows));
        let numisona = Numisona::new();
        let compiler = Compiler::new(&[&data, &numisona]).unwrap();
        let spec = spec(fields);
        let cancel = CancelToken::new();
        assert!(
            compiler.compile(&spec, &cancel).is_ok(),
            "{name} must compile"
        );
        group.bench_function(name, |b| {
            b.iter(|| black_box(compiler.compile(black_box(&spec), &cancel).unwrap()));
        });
    }
    group.finish();
}

criterion_group!(benches, compile_benchmarks);
criterion_main!(benches);

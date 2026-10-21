// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Compiles specs through the real compiler with NUMISONA and AUDAXIS registered and a
//! test source: `inline` sources are read by the data layer, `file` sources are the given
//! bundle. Also renders a navigation tree as an outline for snapshots.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    dead_code,
    reason = "test helpers; clippy treats them as non-test code, and not every test uses every helper"
)]

use std::fmt::Write as _;
use std::sync::Arc;

use sonarithm_audaxis::Audaxis;
use sonarithm_compiler::{Compiled, Compiler, Failure};
use sonarithm_core::engine::{
    CancelToken, CompileContext, CompileError, Engine, EngineManifest, Registrar, RegistryError,
    Source, SourceKind, SpecFragment,
};
use sonarithm_core::ids::EngineId;
use sonarithm_core::nav::{NavKind, NavNodeId, NavTree, XValue};
use sonarithm_core::score::EngineVersion;
use sonarithm_data::{DataBundle, Format, ReadOptions, read_bytes};
use sonarithm_numisona::Numisona;

/// Reads `inline` values with the data layer; any `file` source is the given bundle.
struct TestSource(Option<DataBundle>);

impl Source<DataBundle> for TestSource {
    fn ingest(
        &self,
        source: &SpecFragment,
        _: &mut CompileContext,
    ) -> Result<DataBundle, CompileError> {
        let parsed: sonarithm_spec::Source = source.parse()?;
        match parsed {
            sonarithm_spec::Source::Inline { values } => {
                let bytes = serde_json::to_vec(&values).unwrap();
                let loaded = read_bytes(&bytes, Format::Json, "inline.json", &ReadOptions::new())
                    .map_err(|err| CompileError::Engine {
                    engine: EngineId::from_static("platform"),
                    message: err.to_string(),
                })?;
                Ok(loaded.bundle)
            }
            _ => self.0.clone().ok_or_else(|| CompileError::Engine {
                engine: EngineId::from_static("platform"),
                message: "no bundle was given for a file source".to_owned(),
            }),
        }
    }
}

/// A stand-in for the platform data layer.
struct Platform(Option<DataBundle>);

impl Engine<DataBundle> for Platform {
    fn manifest(&self) -> EngineManifest {
        EngineManifest::new(
            EngineId::from_static("platform"),
            "test platform",
            EngineVersion("0.0.0".to_owned()),
        )
    }

    fn register(&self, registrar: &mut Registrar<'_, DataBundle>) -> Result<(), RegistryError> {
        let source = Arc::new(TestSource(self.0.clone()));
        registrar.source(SourceKind::Inline, source.clone())?;
        registrar.source(SourceKind::File, source)
    }
}

/// Compiles `spec` over `data` with NUMISONA and AUDAXIS; panics with the diagnostics on
/// failure.
pub(crate) fn compile(spec: &str, data: Option<DataBundle>) -> Compiled {
    let platform = Platform(data);
    let compiler = Compiler::new(&[&platform, &Numisona::new(), &Audaxis::new()]).unwrap();
    match compiler.compile(spec, &CancelToken::new()) {
        Ok(done) => done,
        Err(Failure::Invalid(diagnostics)) => {
            let lines: Vec<String> = diagnostics.iter().map(ToString::to_string).collect();
            panic!("compile failed:\n{}", lines.join("\n"))
        }
        Err(other) => panic!("compile failed: {other:?}"),
    }
}

/// The compiled navigation tree; panics if there is none.
pub(crate) fn tree(compiled: &Compiled) -> &NavTree {
    compiled
        .nav
        .as_ref()
        .expect("AUDAXIS builds a navigation tree")
}

/// An outline of `tree`, one node per line. Each segment shows its first and last point.
pub(crate) fn outline(tree: &NavTree) -> String {
    let mut out = String::new();
    walk(tree, tree.root(), 0, &mut out);
    out
}

fn walk(tree: &NavTree, id: NavNodeId, depth: usize, out: &mut String) {
    let node = tree.node(id).unwrap();
    let _ = writeln!(out, "{}{}", "  ".repeat(depth), describe(tree, id));
    let children = tree.children(id);
    let shown: Vec<NavNodeId> = if matches!(node.kind, NavKind::Segment(_)) && children.len() > 2 {
        vec![children[0], children[children.len() - 1]]
    } else {
        children.to_vec()
    };
    for (k, child) in shown.iter().enumerate() {
        if k == 1 && shown.len() < children.len() {
            let hidden = children.len() - 2;
            let _ = writeln!(out, "{}… {hidden} more points", "  ".repeat(depth + 1));
        }
        walk(tree, *child, depth + 1, out);
    }
}

fn describe(tree: &NavTree, id: NavNodeId) -> String {
    let node = tree.node(id).unwrap();
    let label = node
        .label
        .as_ref()
        .map_or_else(String::new, |l| format!(" \"{l}\""));
    let span = node
        .span
        .map_or_else(String::new, |x| format!(" x {}..{}", x.start, x.end));
    let range = |r: Option<sonarithm_core::nav::ValueRange>| {
        r.map_or_else(String::new, |r| format!(" range {}..{}", r.min, r.max))
    };
    match &node.kind {
        NavKind::Dataset(d) => format!(
            "dataset{label} title {:?}, {} series, {} points",
            d.title, d.series, d.points
        ),
        NavKind::Series(s) => format!(
            "series{label} track {} field {:?} unit {:?}, {} points{}{span}",
            s.track,
            s.field,
            s.unit,
            s.points,
            range(s.range)
        ),
        NavKind::Axes => "axes".to_owned(),
        NavKind::Axis(a) => format!(
            "axis {:?}{label} field {:?} unit {:?} values {:?}{}",
            a.axis,
            a.field,
            a.unit,
            a.values,
            range(a.range)
        ),
        NavKind::Insights => "insights".to_owned(),
        NavKind::InsightGroup(g) => format!("insight group {}", g.analyzer),
        NavKind::Insight(i) => format!(
            "insight{label} rows {}..{} value {:?} target {:?}",
            i.rows.start,
            i.rows.end,
            i.value,
            i.target.map(|t| t.0)
        ),
        NavKind::Filters => "filters".to_owned(),
        NavKind::Filter(f) => format!("filter{label} series {}", f.series.0),
        NavKind::Segment(s) => format!(
            "segment{} {} points{}{span}",
            if s.semantic { " (semantic)" } else { "" },
            s.points,
            range(s.range)
        ),
        NavKind::Point(p) => {
            let x = match &p.x {
                Some(XValue::Number(n)) => format!(" x {n}"),
                Some(XValue::Category(c)) => format!(" x \"{c}\""),
                None => String::new(),
            };
            let t =
                p.t.map_or_else(|| " not played".to_owned(), |t| format!(" t {}", t.0));
            let estimate = if p.provenance.class.is_estimate() {
                format!(
                    " estimate {:?} confidence {:?}",
                    p.provenance.class,
                    p.provenance.confidence.map(sonarithm_core::Confidence::get)
                )
            } else {
                String::new()
            };
            format!(
                "point {} row {}{x} value {:?}{t}{estimate}",
                p.index, p.row.row, p.value
            )
        }
        other => format!("{other:?}"),
    }
}

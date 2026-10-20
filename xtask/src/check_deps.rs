// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask check-deps`: enforces the crate dependency rules in
//! docs/architecture/02-repository-and-crate-layout.md §3.
//!
//! | Check | Source |
//! |---|---|
//! | `sonarithm-core` depends on no workspace crate and reaches no async runtime, audio device, file-system or network crate | §3 rule 1 |
//! | Engine crates never depend on another engine's crates | §3 rule 2 |
//! | Only `sonarithm-audio` may reach `cpal`; only stream, app and app or binding crates may reach `tokio` | §3 rule 3 |
//! | Heavy or copyleft dependencies live in their leaf crate, as optional dependencies where 02 §4 makes them features | §3 rule 5 |
//! | Every internal dependency follows the layering graph | §3 diagram |
//!
//! Features are those `cargo metadata` resolves for the whole workspace, which unifies
//! the features dev-dependencies enable. A crate enabling `sonarithm-core/test-support` in
//! its dev-dependencies therefore makes core's optional `proptest` look like a normal
//! dependency here, so test-support dependencies must stay clear of the forbidden crates
//! (proptest is built without `fork` and `timeout`, which pull in `tempfile`; #121).
//!
//! Scope: normal and build dependencies, including optional ones. Dev-dependencies are
//! exempt because they never ship and do not affect the wasm32 build. "Reach" means a
//! direct dependency or a path through external crates; a path that passes through another
//! workspace crate is checked at that crate instead. Paths are taken from the resolved
//! graph of the default features, so a dependency enabled only by a non-default feature is
//! checked where it is declared, not through its transitive dependencies.
//!
//! The rules run on [`CrateDeps`] values rather than on `cargo metadata` output directly,
//! so they can be unit-tested without a Cargo project.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::path::Path;

use anyhow::{Context as _, Result, bail};
use cargo_metadata::{DependencyKind, Metadata, MetadataCommand, Node, PackageId};

/// The nine engine ids (02 §5). An engine's crates are `sonarithm-<engine>` and
/// `sonarithm-<engine>-<part>`, for example `sonarithm-plotsona-ml`. The order also sets
/// each engine's block of `SNR-ENG` code numbers (`codes.rs`).
pub(crate) const ENGINES: [&str; 9] = [
    "numisona",
    "audaxis",
    "sonotrace",
    "plotsona",
    "curvasona",
    "axonote",
    "echometric",
    "tonalens",
    "resonum",
];

/// Stands for every engine crate in [`LAYERS`].
const ENGINE_CRATES: &str = "<engine crates>";

/// The layering graph of 02 §3, plus crates the diagram omits, taken from the crate
/// catalogue (02 §2). A crate may depend on any crate reachable from its entry, and every
/// crate except `sonarithm-core` may depend on `sonarithm-core`. A workspace crate that is
/// missing here is reported: add it to 02 §3 and to this table in the same PR.
const LAYERS: &[(&str, &[&str])] = &[
    ("sonarithm-core", &[]),
    ("sonarithm-spec", &["sonarithm-core"]),
    ("sonarithm-data", &["sonarithm-core"]),
    ("sonarithm-dsp", &["sonarithm-core"]),
    (
        ENGINE_CRATES,
        &[
            "sonarithm-core",
            "sonarithm-spec",
            "sonarithm-data",
            "sonarithm-dsp",
        ],
    ),
    (
        "sonarithm-compiler",
        &["sonarithm-spec", "sonarithm-data", ENGINE_CRATES],
    ),
    ("sonarithm-render", &["sonarithm-dsp"]),
    ("sonarithm-audio", &["sonarithm-render"]),
    ("sonarithm-export", &["sonarithm-render"]),
    ("sonarithm-stream", &["sonarithm-data"]),
    ("sonarithm-speech", &["sonarithm-core"]),
    (
        "sonarithm-app",
        &[
            "sonarithm-compiler",
            "sonarithm-audio",
            "sonarithm-export",
            "sonarithm-stream",
        ],
    ),
    (
        "sonarithm",
        &["sonarithm-compiler", "sonarithm-render", "sonarithm-export"],
    ),
    ("sonarithm-cli", &["sonarithm-app", "sonarithm"]),
    ("sonarithm-desktop", &["sonarithm-app"]),
    ("sonarithm-py", &["sonarithm"]),
    ("sonarithm-wasm", &["sonarithm"]),
    ("sonarithm-plugin-host", &["sonarithm-core"]),
    // xtask reads code tables and schemas from the library crates (codegen, codes).
    ("xtask", &["sonarithm-compiler", "sonarithm-export"]),
];

/// Crates `sonarithm-core` must not reach (§3 rule 1): async runtimes, audio I/O,
/// file-system and network crates. Extend the list when a new crate of these kinds is
/// considered anywhere in the workspace.
const CORE_FORBIDDEN: &[&str] = &[
    // Async runtimes
    "async-std",
    "smol",
    "tokio",
    // Audio devices
    "alsa",
    "coreaudio-rs",
    "cpal",
    "jack",
    "kira",
    "rodio",
    "wasapi",
    // File system
    "directories",
    "dirs",
    "fs-err",
    "notify",
    "tempfile",
    "walkdir",
    // Network and device I/O
    "hyper",
    "mio",
    "reqwest",
    "rosc",
    "rumqttc",
    "serialport",
    "socket2",
    "tokio-tungstenite",
    "tungstenite",
    "ureq",
];

/// An external crate that only some workspace crates may depend on.
struct Restricted {
    name: &'static str,
    owners: &'static [&'static str],
    /// Must be an optional dependency in its owner (a feature in 02 §4).
    optional: bool,
    rule: Rule,
}

const RESTRICTED: &[Restricted] = &[
    Restricted {
        name: "cpal",
        owners: &["sonarithm-audio"],
        optional: false,
        rule: Rule::Three,
    },
    Restricted {
        name: "tokio",
        owners: &[
            "sonarithm-stream",
            "sonarithm-app",
            "sonarithm-cli",
            "sonarithm-desktop",
            "sonarithm-py",
            "sonarithm-wasm",
        ],
        optional: false,
        rule: Rule::Three,
    },
    Restricted {
        name: "wasmtime",
        owners: &["sonarithm-plugin-host"],
        optional: false,
        rule: Rule::Five,
    },
    Restricted {
        name: "tract-onnx",
        owners: &["sonarithm-plotsona-ml"],
        optional: false,
        rule: Rule::Five,
    },
    Restricted {
        name: "ort",
        owners: &["sonarithm-plotsona-ml"],
        optional: true,
        rule: Rule::Five,
    },
    Restricted {
        name: "rumqttc",
        owners: &["sonarithm-stream"],
        optional: true,
        rule: Rule::Five,
    },
    Restricted {
        name: "polars",
        owners: &["sonarithm-data"],
        optional: true,
        rule: Rule::Five,
    },
    Restricted {
        name: "mp3lame-encoder",
        owners: &["sonarithm-export"],
        optional: true,
        rule: Rule::Five,
    },
];

/// The part of 02 §3 a violation breaks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Rule {
    /// Rule 1: `sonarithm-core` is self-contained.
    One,
    /// Rule 2: engines never depend on each other.
    Two,
    /// Rule 3: `cpal` and `tokio` stay in their crates.
    Three,
    /// Rule 5: heavy dependencies live in leaf crates behind features.
    Five,
    /// The layering graph.
    Layering,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::One => "02 §3 rule 1",
            Self::Two => "02 §3 rule 2",
            Self::Three => "02 §3 rule 3",
            Self::Five => "02 §3 rule 5",
            Self::Layering => "02 §3 graph",
        })
    }
}

/// A broken dependency rule.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Violation {
    pub(crate) krate: String,
    /// The offending dependency, or `None` when the crate itself is the problem.
    pub(crate) dependency: Option<String>,
    pub(crate) rule: Rule,
    pub(crate) message: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.dependency {
            Some(dependency) => write!(f, "{} -> {}: {}", self.krate, dependency, self.message)?,
            None => write!(f, "{}: {}", self.krate, self.message)?,
        }
        write!(f, " [{}]", self.rule)
    }
}

/// A workspace crate as the rules see it.
#[derive(Clone, Debug, Default)]
pub(crate) struct CrateDeps {
    pub(crate) name: String,
    /// Declared normal and build dependencies, including optional ones.
    pub(crate) declared: Vec<Declared>,
    /// External crates reachable without passing through another workspace crate, each
    /// with the crates on the path to it (empty for a direct dependency).
    pub(crate) reachable: BTreeMap<String, Vec<String>>,
}

/// A declared dependency.
#[derive(Clone, Debug)]
pub(crate) struct Declared {
    pub(crate) name: String,
    pub(crate) workspace: bool,
    pub(crate) optional: bool,
}

/// Loads the workspace at `manifest`, checks it and prints the result.
pub(crate) fn run(manifest: &Path) -> Result<()> {
    let crates = load(manifest)?;
    let violations = check(&crates);
    if violations.is_empty() {
        println!(
            "check-deps: {} workspace crates follow the dependency rules",
            crates.len()
        );
        return Ok(());
    }
    for violation in &violations {
        eprintln!("  {violation}");
    }
    bail!(
        "{} dependency rule violation(s); see docs/architecture/02-repository-and-crate-layout.md §3",
        violations.len()
    )
}

/// Builds the rule model of the workspace at `manifest` from `cargo metadata`.
pub(crate) fn load(manifest: &Path) -> Result<Vec<CrateDeps>> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest)
        .exec()
        .context("running `cargo metadata`")?;
    from_metadata(&metadata)
}

fn from_metadata(metadata: &Metadata) -> Result<Vec<CrateDeps>> {
    let resolve = metadata
        .resolve
        .as_ref()
        .context("`cargo metadata` returned no dependency graph")?;
    let names: BTreeMap<&PackageId, &str> = metadata
        .packages
        .iter()
        .map(|package| (&package.id, package.name.as_str()))
        .collect();
    let members: BTreeSet<&PackageId> = metadata.workspace_members.iter().collect();
    let member_names: BTreeSet<&str> = members
        .iter()
        .filter_map(|id| names.get(id).copied())
        .collect();
    let nodes: BTreeMap<&PackageId, &Node> =
        resolve.nodes.iter().map(|node| (&node.id, node)).collect();

    let mut crates: Vec<CrateDeps> = metadata
        .workspace_packages()
        .into_iter()
        .map(|package| CrateDeps {
            name: package.name.to_string(),
            declared: package
                .dependencies
                .iter()
                .filter(|dep| dep.kind != DependencyKind::Development)
                .map(|dep| Declared {
                    name: dep.name.clone(),
                    workspace: dep.path.is_some() && member_names.contains(dep.name.as_str()),
                    optional: dep.optional,
                })
                .collect(),
            reachable: reachable_external(&package.id, &nodes, &members, &names),
        })
        .collect();
    crates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(crates)
}

/// Breadth-first search over normal and build edges, stopping at workspace crates.
fn reachable_external(
    root: &PackageId,
    nodes: &BTreeMap<&PackageId, &Node>,
    members: &BTreeSet<&PackageId>,
    names: &BTreeMap<&PackageId, &str>,
) -> BTreeMap<String, Vec<String>> {
    let mut reached: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut seen: BTreeSet<&PackageId> = BTreeSet::from([root]);
    let mut queue: VecDeque<(&PackageId, Vec<String>)> = VecDeque::from([(root, Vec::new())]);
    while let Some((id, path)) = queue.pop_front() {
        let Some(node) = nodes.get(id) else { continue };
        for dep in &node.deps {
            let ships = dep.dep_kinds.is_empty()
                || dep
                    .dep_kinds
                    .iter()
                    .any(|info| info.kind != DependencyKind::Development);
            if !ships || members.contains(&dep.pkg) || !seen.insert(&dep.pkg) {
                continue;
            }
            let name = names
                .get(&dep.pkg)
                .map_or_else(|| dep.name.clone(), |name| (*name).to_owned());
            reached.entry(name.clone()).or_insert_with(|| path.clone());
            let mut next = path.clone();
            next.push(name);
            queue.push_back((&dep.pkg, next));
        }
    }
    reached
}

/// Checks every rule and returns the violations in a stable order. A dependency that
/// breaks several rules is reported once, under the most specific rule.
pub(crate) fn check(crates: &[CrateDeps]) -> Vec<Violation> {
    let mut findings = Findings::default();
    for krate in crates {
        core_isolation(krate, &mut findings);
        engine_independence(krate, &mut findings);
        restricted_crates(krate, &mut findings);
    }
    for krate in crates {
        layering(krate, &mut findings);
    }
    findings.violations.into_iter().collect()
}

#[derive(Default)]
struct Findings {
    violations: BTreeSet<Violation>,
    reported: BTreeSet<(String, Option<String>)>,
}

impl Findings {
    fn add(&mut self, krate: &str, dependency: Option<&str>, rule: Rule, message: String) {
        let key = (krate.to_owned(), dependency.map(str::to_owned));
        if self.reported.insert(key) {
            self.violations.insert(Violation {
                krate: krate.to_owned(),
                dependency: dependency.map(str::to_owned),
                rule,
                message,
            });
        }
    }

    fn reported(&self, krate: &str, dependency: &str) -> bool {
        self.reported
            .contains(&(krate.to_owned(), Some(dependency.to_owned())))
    }
}

/// How a crate reaches an external crate.
enum Reach<'a> {
    Declared(&'a Declared),
    Through(&'a [String]),
}

impl Reach<'_> {
    fn describe(&self) -> String {
        match self {
            Self::Declared(_) | Self::Through([]) => "a direct dependency".to_owned(),
            Self::Through(path) => format!("reached through {}", path.join(" -> ")),
        }
    }
}

fn reach<'a>(krate: &'a CrateDeps, name: &str) -> Option<Reach<'a>> {
    if let Some(declared) = krate
        .declared
        .iter()
        .find(|dep| !dep.workspace && dep.name == name)
    {
        return Some(Reach::Declared(declared));
    }
    krate
        .reachable
        .get(name)
        .map(|path| Reach::Through(path.as_slice()))
}

fn core_isolation(krate: &CrateDeps, findings: &mut Findings) {
    if krate.name != "sonarithm-core" {
        return;
    }
    for dep in krate.declared.iter().filter(|dep| dep.workspace) {
        findings.add(
            &krate.name,
            Some(&dep.name),
            Rule::One,
            "sonarithm-core depends on no other SONARITHM crate".to_owned(),
        );
    }
    for forbidden in CORE_FORBIDDEN {
        if let Some(how) = reach(krate, forbidden) {
            findings.add(
                &krate.name,
                Some(forbidden),
                Rule::One,
                format!(
                    "sonarithm-core reaches no async runtime, audio device, file-system or \
                     network crate ({})",
                    how.describe()
                ),
            );
        }
    }
}

fn engine_independence(krate: &CrateDeps, findings: &mut Findings) {
    let Some(engine) = engine_of(&krate.name) else {
        return;
    };
    for dep in krate.declared.iter().filter(|dep| dep.workspace) {
        if engine_of(&dep.name).is_some_and(|other| other != engine) {
            findings.add(
                &krate.name,
                Some(&dep.name),
                Rule::Two,
                "engines never depend on each other; the compiler wires them through the \
                 engine registry"
                    .to_owned(),
            );
        }
    }
}

fn restricted_crates(krate: &CrateDeps, findings: &mut Findings) {
    for restricted in RESTRICTED {
        let Some(how) = reach(krate, restricted.name) else {
            continue;
        };
        if !restricted.owners.contains(&krate.name.as_str()) {
            findings.add(
                &krate.name,
                Some(restricted.name),
                restricted.rule,
                format!(
                    "only {} may depend on {} ({})",
                    restricted.owners.join(", "),
                    restricted.name,
                    how.describe()
                ),
            );
        } else if restricted.optional
            && let Reach::Declared(declared) = how
            && !declared.optional
        {
            findings.add(
                &krate.name,
                Some(restricted.name),
                restricted.rule,
                format!(
                    "{} must be an optional dependency behind a non-default feature",
                    restricted.name
                ),
            );
        }
    }
}

fn layering(krate: &CrateDeps, findings: &mut Findings) {
    let Some(entry) = layer_of(&krate.name) else {
        findings.add(
            &krate.name,
            None,
            Rule::Layering,
            "workspace crate missing from the dependency rules; add it to 02 §3 and to \
             LAYERS in xtask/src/check_deps.rs"
                .to_owned(),
        );
        return;
    };
    let allowed = closure(entry);
    for dep in krate.declared.iter().filter(|dep| dep.workspace) {
        if findings.reported(&krate.name, &dep.name) || dep.name == "sonarithm-core" {
            continue;
        }
        let same_engine =
            engine_of(&krate.name).is_some() && engine_of(&krate.name) == engine_of(&dep.name);
        let Some(target) = layer_of(&dep.name) else {
            // Reported as an unknown crate on its own.
            continue;
        };
        if !same_engine && !allowed.contains(target) {
            findings.add(
                &krate.name,
                Some(&dep.name),
                Rule::Layering,
                "dependency not allowed by the layering graph".to_owned(),
            );
        }
    }
}

/// The engine a crate belongs to, if it is an engine crate.
pub(crate) fn engine_of(krate: &str) -> Option<&'static str> {
    let rest = krate.strip_prefix("sonarithm-")?;
    ENGINES.into_iter().find(|engine| {
        rest == *engine
            || rest
                .strip_prefix(engine)
                .is_some_and(|part| part.starts_with('-'))
    })
}

/// The [`LAYERS`] entry for a crate.
fn layer_of(krate: &str) -> Option<&'static str> {
    if engine_of(krate).is_some() {
        return Some(ENGINE_CRATES);
    }
    LAYERS
        .iter()
        .map(|(name, _)| *name)
        .find(|name| *name == krate)
}

/// Every [`LAYERS`] entry reachable from `entry`, excluding `entry` itself.
fn closure(entry: &str) -> BTreeSet<&'static str> {
    let mut reached = BTreeSet::new();
    let mut queue: VecDeque<&str> = VecDeque::from([entry]);
    while let Some(current) = queue.pop_front() {
        let Some((_, deps)) = LAYERS.iter().find(|(name, _)| *name == current) else {
            continue;
        };
        for dep in *deps {
            if reached.insert(*dep) {
                queue.push_back(dep);
            }
        }
    }
    reached
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    fn krate(name: &str) -> CrateDeps {
        CrateDeps {
            name: name.to_owned(),
            ..CrateDeps::default()
        }
    }

    impl CrateDeps {
        fn internal(mut self, name: &str) -> Self {
            self.declared.push(Declared {
                name: name.to_owned(),
                workspace: true,
                optional: false,
            });
            self
        }

        fn external(mut self, name: &str, optional: bool) -> Self {
            self.declared.push(Declared {
                name: name.to_owned(),
                workspace: false,
                optional,
            });
            self.reachable.insert(name.to_owned(), Vec::new());
            self
        }

        fn through(mut self, name: &str, path: &[&str]) -> Self {
            self.reachable.insert(
                name.to_owned(),
                path.iter().map(|step| (*step).to_owned()).collect(),
            );
            self
        }
    }

    fn rules_broken(crates: &[CrateDeps]) -> Vec<(String, Option<String>, Rule)> {
        check(crates)
            .into_iter()
            .map(|v| (v.krate, v.dependency, v.rule))
            .collect()
    }

    fn broken(krate: &str, dependency: &str, rule: Rule) -> (String, Option<String>, Rule) {
        (krate.to_owned(), Some(dependency.to_owned()), rule)
    }

    /// The target layout of 02 §1, wired as the diagram in 02 §3 allows.
    fn target_layout() -> Vec<CrateDeps> {
        vec![
            krate("sonarithm-core"),
            krate("sonarithm-spec").internal("sonarithm-core"),
            krate("sonarithm-data").internal("sonarithm-core"),
            krate("sonarithm-dsp").internal("sonarithm-core"),
            krate("sonarithm-numisona")
                .internal("sonarithm-core")
                .internal("sonarithm-spec")
                .internal("sonarithm-data")
                .internal("sonarithm-dsp"),
            krate("sonarithm-plotsona").internal("sonarithm-data"),
            krate("sonarithm-plotsona-ml")
                .internal("sonarithm-plotsona")
                .external("tract-onnx", false)
                .external("ort", true),
            krate("sonarithm-compiler")
                .internal("sonarithm-core")
                .internal("sonarithm-spec")
                .internal("sonarithm-data")
                .internal("sonarithm-numisona")
                .internal("sonarithm-plotsona"),
            krate("sonarithm-render").internal("sonarithm-dsp"),
            krate("sonarithm-audio")
                .internal("sonarithm-render")
                .external("cpal", false),
            krate("sonarithm-export")
                .internal("sonarithm-render")
                .external("mp3lame-encoder", true),
            krate("sonarithm-stream")
                .internal("sonarithm-data")
                .external("tokio", false)
                .external("rumqttc", true),
            krate("sonarithm-app")
                .internal("sonarithm-compiler")
                .internal("sonarithm-audio")
                .internal("sonarithm-export")
                .internal("sonarithm-stream")
                .external("tokio", false),
            krate("sonarithm")
                .internal("sonarithm-compiler")
                .internal("sonarithm-render")
                .internal("sonarithm-export"),
            krate("sonarithm-cli")
                .internal("sonarithm-app")
                .internal("sonarithm"),
            krate("xtask").external("cargo_metadata", false),
        ]
    }

    #[test]
    fn target_layout_passes() {
        assert_eq!(rules_broken(&target_layout()), Vec::new());
    }

    #[test]
    fn engine_depending_on_another_engine_fails() {
        let crates = [
            krate("sonarithm-numisona").internal("sonarithm-audaxis"),
            krate("sonarithm-audaxis"),
        ];
        assert_eq!(
            rules_broken(&crates),
            vec![broken("sonarithm-numisona", "sonarithm-audaxis", Rule::Two)]
        );
    }

    #[test]
    fn engine_part_may_depend_on_its_own_engine() {
        let crates = [
            krate("sonarithm-plotsona-ml").internal("sonarithm-plotsona"),
            krate("sonarithm-plotsona"),
        ];
        assert_eq!(rules_broken(&crates), Vec::new());
    }

    #[test]
    fn engine_part_depending_on_another_engine_fails() {
        let crates = [
            krate("sonarithm-plotsona-ml").internal("sonarithm-echometric"),
            krate("sonarithm-echometric"),
        ];
        assert_eq!(
            rules_broken(&crates),
            vec![broken(
                "sonarithm-plotsona-ml",
                "sonarithm-echometric",
                Rule::Two
            )]
        );
    }

    #[test]
    fn core_depending_on_a_workspace_crate_fails() {
        let crates = [
            krate("sonarithm-core").internal("sonarithm-spec"),
            krate("sonarithm-spec"),
        ];
        assert_eq!(
            rules_broken(&crates),
            vec![broken("sonarithm-core", "sonarithm-spec", Rule::One)]
        );
    }

    #[test]
    fn core_depending_on_tokio_fails_once() {
        let crates = [krate("sonarithm-core").external("tokio", true)];
        assert_eq!(
            rules_broken(&crates),
            vec![broken("sonarithm-core", "tokio", Rule::One)]
        );
    }

    #[test]
    fn core_reaching_cpal_through_an_external_crate_fails() {
        let crates = [krate("sonarithm-core").through("cpal", &["rodio"])];
        let violations = check(&crates);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].dependency.as_deref(), Some("cpal"));
        assert_eq!(violations[0].rule, Rule::One);
        assert!(violations[0].message.contains("through rodio"));
    }

    #[test]
    fn cpal_outside_the_audio_crate_fails() {
        let crates = [
            krate("sonarithm-render").external("cpal", false),
            krate("sonarithm-audio").external("cpal", false),
        ];
        assert_eq!(
            rules_broken(&crates),
            vec![broken("sonarithm-render", "cpal", Rule::Three)]
        );
    }

    #[test]
    fn tokio_in_a_wasm_crate_fails() {
        let crates = [krate("sonarithm-compiler").through("tokio", &["some-client"])];
        assert_eq!(
            rules_broken(&crates),
            vec![broken("sonarithm-compiler", "tokio", Rule::Three)]
        );
    }

    #[test]
    fn copyleft_codec_must_be_optional() {
        let crates = [krate("sonarithm-export").external("mp3lame-encoder", false)];
        assert_eq!(
            rules_broken(&crates),
            vec![broken("sonarithm-export", "mp3lame-encoder", Rule::Five)]
        );
    }

    #[test]
    fn heavy_dependency_outside_its_leaf_crate_fails() {
        let crates = [krate("sonarithm-compiler").external("wasmtime", true)];
        assert_eq!(
            rules_broken(&crates),
            vec![broken("sonarithm-compiler", "wasmtime", Rule::Five)]
        );
    }

    #[test]
    fn upward_dependency_fails() {
        let crates = [
            krate("sonarithm-dsp").internal("sonarithm-render"),
            krate("sonarithm-render").internal("sonarithm-dsp"),
        ];
        assert_eq!(
            rules_broken(&crates),
            vec![broken("sonarithm-dsp", "sonarithm-render", Rule::Layering)]
        );
    }

    #[test]
    fn depending_on_an_app_fails() {
        let crates = [
            krate("sonarithm-app").internal("sonarithm-cli"),
            krate("sonarithm-cli"),
        ];
        assert_eq!(
            rules_broken(&crates),
            vec![broken("sonarithm-app", "sonarithm-cli", Rule::Layering)]
        );
    }

    #[test]
    fn unknown_workspace_crate_fails() {
        let crates = [krate("sonarithm-unplanned")];
        assert_eq!(
            rules_broken(&crates),
            vec![("sonarithm-unplanned".to_owned(), None, Rule::Layering)]
        );
    }

    #[test]
    fn engine_ids_match_whole_names_only() {
        assert_eq!(engine_of("sonarithm-plotsona"), Some("plotsona"));
        assert_eq!(engine_of("sonarithm-plotsona-ml"), Some("plotsona"));
        assert_eq!(engine_of("sonarithm-plotsonax"), None);
        assert_eq!(engine_of("sonarithm-core"), None);
        assert_eq!(engine_of("numisona"), None);
    }

    /// A throwaway Cargo workspace on disk, removed on drop.
    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let root =
                std::env::temp_dir().join(format!("sonarithm-xtask-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            Self { root }
        }

        fn file(&self, path: &str, contents: &str) {
            let path = self.root.join(path);
            fs::create_dir_all(path.parent().expect("fixture paths have a parent"))
                .expect("create fixture directory");
            fs::write(path, contents).expect("write fixture file");
        }

        fn package(&self, dir: &str, name: &str, dependencies: &str) {
            self.file(
                &format!("{dir}/Cargo.toml"),
                &format!(
                    "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
                     publish = false\n\n[dependencies]\n{dependencies}"
                ),
            );
            self.file(&format!("{dir}/src/lib.rs"), "");
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    /// End to end through `cargo metadata`: a deliberate engine -> engine dependency and a
    /// core -> cpal dependency (a local stand-in crate, so no registry access) both fail.
    #[test]
    fn fixture_workspace_with_forbidden_dependencies_fails() {
        let fixture = Fixture::new("check-deps");
        fixture.file(
            "ws/Cargo.toml",
            "[workspace]\nresolver = \"3\"\nmembers = [\"core\", \"numisona\", \"audaxis\"]\n",
        );
        fixture.package(
            "ws/core",
            "sonarithm-core",
            "cpal = { path = \"../../ext/cpal\" }\n",
        );
        fixture.package(
            "ws/numisona",
            "sonarithm-numisona",
            "sonarithm-core = { path = \"../core\" }\n\
             sonarithm-audaxis = { path = \"../audaxis\" }\n",
        );
        fixture.package(
            "ws/audaxis",
            "sonarithm-audaxis",
            "sonarithm-core = { path = \"../core\" }\n",
        );
        fixture.package("ext/cpal", "cpal", "");

        let crates = load(&fixture.root.join("ws/Cargo.toml")).expect("cargo metadata runs");
        let names: Vec<&str> = crates.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            names,
            ["sonarithm-audaxis", "sonarithm-core", "sonarithm-numisona"]
        );
        assert_eq!(
            rules_broken(&crates),
            vec![
                broken("sonarithm-core", "cpal", Rule::One),
                broken("sonarithm-numisona", "sonarithm-audaxis", Rule::Two),
            ]
        );
        assert!(run(&fixture.root.join("ws/Cargo.toml")).is_err());
    }

    #[test]
    fn this_workspace_passes() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../Cargo.toml");
        let crates = load(&manifest).expect("cargo metadata runs");
        assert!(crates.iter().any(|c| c.name == "sonarithm-core"));
        assert_eq!(check(&crates), Vec::new());
    }
}

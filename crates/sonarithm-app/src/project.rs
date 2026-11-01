// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `.sonarithm` project bundles, version 1 ([RFC-0002], [ADR-0018]; B05-T04), with
//! format 1.1's function datasets and unknown storages (RFC-0002 §12; C03-T08).
//!
//! A bundle is a ZIP file: `manifest.json` first, then the specs, embedded data snapshots
//! (stored Parquet), and the reserved `profiles/`, `annotations/` and `renders/` members,
//! each listed in the manifest with its length and SHA-256. Writing is byte-stable: fixed
//! entry times, sorted members, a canonical manifest, no timestamps. Reading never trusts
//! the file: member names, sizes, ratios and hashes are checked before anything is parsed,
//! stored Parquet is read in place, and linked data must be a regular data file whose
//! fingerprint matches.
//!
//! [RFC-0002]: ../../docs/rfcs/0002-project-format-v1.md
//! [ADR-0018]: ../../docs/decisions/0018-project-format-v1.md

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{Cursor, Read as _, Write as _};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use sonarithm_core::TableId;
use sonarithm_data::source::write_parquet;
use sonarithm_data::{
    BundleId, Limits, ReadOptions, Table, read_file, read_parquet_at, table_fingerprint,
};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::codes;
use crate::ipc::IpcError;

/// The bundle format's name in the manifest.
pub const FORMAT: &str = "sonarithm-project";
/// The newest format version this build reads and writes, `MAJOR.MINOR`. It writes 1.1
/// only when a project needs it (a function dataset, or an entry it keeps unread), and
/// [`BASE_VERSION`] otherwise, so projects without them stay byte-identical (RFC-0002 §12).
pub const FORMAT_VERSION: (u32, u32) = (1, 1);
/// The version written when nothing of 1.1 is used.
pub const BASE_VERSION: (u32, u32) = (1, 0);
/// The storages this build reads; any other is kept unread (RFC-0002 §12).
const KNOWN_STORAGES: [&str; 3] = ["embedded", "linked", "function"];
/// The extension of project files.
pub const EXTENSION: &str = "sonarithm";

/// The most members a bundle may have.
pub const MAX_MEMBERS: usize = 10_000;
/// The largest JSON member, in bytes.
pub const MAX_JSON: u64 = 64 << 20;
/// The most uncompressed bytes in all members: A04's limit for one input (RFC-0002 §6).
pub const MAX_TOTAL: u64 = 2 << 30;
/// The highest compression ratio of a deflated member.
pub const MAX_RATIO: u64 = 100;

const MANIFEST: &str = "manifest.json";

/// The data extensions a linked dataset may have (A04's readers).
const DATA_EXTENSIONS: [&str; 8] = [
    "csv", "tsv", "json", "ndjson", "parquet", "arrow", "xlsx", "ods",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    format: String,
    format_version: String,
    generator: Generator,
    name: String,
    specs: Vec<SpecEntry>,
    active_spec: String,
    datasets: Vec<DatasetEntry>,
    members: Vec<Member>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Generator {
    name: String,
    version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SpecEntry {
    id: String,
    path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Storage {
    Embedded,
    Linked,
    /// A function's definition, sampled again on read (format 1.1).
    Function,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DatasetEntry {
    id: String,
    name: String,
    storage: Storage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    link: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    function: Option<FunctionDef>,
    fingerprint: String,
    rows: u64,
}

/// A function dataset's definition (format 1.1): the canonical expression, the domain, the
/// grid and the named constants.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FunctionDef {
    expr: String,
    domain: [f64; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    samples: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    step: Option<f64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    variables: BTreeMap<String, f64>,
}

impl FunctionDef {
    /// The definition of `function`, with its expression in canonical form and the default
    /// grid written out.
    fn of(function: &sonarithm::FunctionSpec) -> Self {
        let canonical = sonarithm::check_expression(&function.expr, &function.variables)
            .canonical
            .unwrap_or_else(|| function.expr.clone());
        Self {
            expr: canonical,
            domain: function.domain,
            samples: function
                .step
                .is_none()
                .then(|| function.samples.unwrap_or(1000)),
            step: function.step,
            variables: function.variables.clone(),
        }
    }

    fn spec(&self) -> sonarithm::FunctionSpec {
        let mut function = sonarithm::FunctionSpec::new(self.expr.clone(), self.domain);
        function.samples = self.samples;
        function.step = self.step;
        function.variables.clone_from(&self.variables);
        function
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Member {
    path: String,
    bytes: u64,
    sha256: String,
}

/// A dataset of a project.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ProjectDataset {
    /// The ID specs name it by, such as `d1`.
    pub id: String,
    /// The original file name, without directories.
    pub name: String,
    /// The data, or `None` when a linked file is missing or changed.
    pub table: Option<Table>,
    /// The canonical file it came from, when it came from one: the target of a link.
    pub origin: Option<PathBuf>,
    /// Whether the project links it rather than embedding it.
    pub linked: bool,
    /// The link as recorded, kept for linked data that is unavailable.
    pub link: Option<String>,
    /// The fingerprint the manifest records (A04-T09).
    pub fingerprint: String,
    /// Rows, as recorded.
    pub rows: u64,
    /// For a function dataset, its definition (format 1.1): saved instead of its samples.
    pub function: Option<sonarithm::FunctionSpec>,
    /// For an entry whose storage this build does not know, the entry as read: unavailable,
    /// and written back unchanged (RFC-0002 §12).
    pub kept: Option<serde_json::Value>,
}

impl ProjectDataset {
    /// A dataset held in memory, embedded when saved. `fingerprint` is the table's
    /// (`sonarithm_data::table_fingerprint`).
    #[must_use]
    pub fn embedded(id: &str, name: &str, table: Table, fingerprint: String) -> Self {
        Self {
            id: id.to_owned(),
            name: file_name(name).to_owned(),
            rows: table.num_rows(),
            table: Some(table),
            origin: None,
            linked: false,
            link: None,
            fingerprint,
            function: None,
            kept: None,
        }
    }

    /// A function dataset: saved as its definition, not its samples (format 1.1).
    #[must_use]
    pub fn function(
        id: &str,
        name: &str,
        table: Table,
        fingerprint: String,
        function: sonarithm::FunctionSpec,
    ) -> Self {
        Self {
            function: Some(function),
            ..Self::embedded(id, name, table, fingerprint)
        }
    }
}

/// A project in memory.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct Project {
    /// The display name.
    pub name: String,
    /// The specs by ID, text exactly as saved, in the order the user sees them.
    pub specs: Vec<(String, String)>,
    /// The ID of the spec that opens first.
    pub active_spec: String,
    /// The datasets.
    pub datasets: Vec<ProjectDataset>,
    /// Reserved members kept byte for byte (`profiles/`, `annotations/`, `renders/`).
    pub reserved: BTreeMap<String, Vec<u8>>,
}

/// A project read from a file, with the warnings to show.
#[derive(Debug)]
#[non_exhaustive]
pub struct Opened {
    /// The project.
    pub project: Project,
    /// Warnings: newer minor version, ignored members, linked data missing or changed.
    pub warnings: Vec<IpcError>,
}

fn damaged(why: &str) -> IpcError {
    IpcError::new(
        &codes::PROJECT_DAMAGED,
        format!("The project cannot be opened safely: {why}."),
    )
}

fn hex(digest: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Whether `id` is a valid spec or dataset ID (RFC-0002 §2).
fn valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphanumeric())
        && id.len() <= 64
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// The kind of a member path, or `None` for a path outside the known folders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Manifest,
    Spec,
    Data,
    Reserved,
}

/// Checks a member name (RFC-0002 §6.1). `Err` for an unsafe name (zip-slip), `Ok(None)`
/// for a safe name in an unknown folder.
fn member_kind(name: &str) -> Result<Option<Kind>, IpcError> {
    let unsafe_name = name.is_empty()
        || name.starts_with('/')
        || name.contains('\\')
        || name.chars().any(|c| c == '\0' || c.is_control())
        || name
            .split('/')
            .any(|part| part == ".." || part == "." || part.is_empty())
        || name
            .split('/')
            .next()
            .is_some_and(|first| first.contains(':'));
    if unsafe_name {
        return Err(damaged("a member has an unsafe name"));
    }
    if name == MANIFEST {
        return Ok(Some(Kind::Manifest));
    }
    let Some((folder, file)) = name.split_once('/') else {
        return Ok(None);
    };
    if file.contains('/') {
        return Ok(None);
    }
    let stem = |suffix: &str| file.strip_suffix(suffix).filter(|id| valid_id(id));
    Ok(match folder {
        "specs" if stem(".json").is_some() => Some(Kind::Spec),
        "data" if stem(".parquet").is_some() => Some(Kind::Data),
        "profiles" if file == "project-profile.json" => Some(Kind::Reserved),
        "annotations" if stem(".json").is_some() => Some(Kind::Reserved),
        "renders" if stem(".render.json").is_some() => Some(Kind::Reserved),
        _ => None,
    })
}

/// A relative path from `from_dir` to `to`, with `/` separators; `None` when there is
/// none (another drive). Both are canonical.
fn relative(from_dir: &Path, to: &Path) -> Option<String> {
    let from: Vec<Component<'_>> = from_dir.components().collect();
    let to: Vec<Component<'_>> = to.components().collect();
    if from.first() != to.first() {
        return None;
    }
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = vec!["..".to_owned(); from.len() - common];
    for component in &to[common..] {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            _ => return None,
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

/// Whether a spec's `file` source names an absolute path (RFC-0002 §4).
fn has_absolute_file_source(spec: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(spec) else {
        return false;
    };
    let source = &value["data"]["source"];
    source["kind"] == "file"
        && source["path"].as_str().is_some_and(|path| {
            Path::new(path).is_absolute()
                || path.starts_with('/')
                || path.starts_with('\\')
                || path.get(1..2) == Some(":")
        })
}

/// The bundle bytes of `project`, to be saved at `target` (whose folder anchors links).
/// With `embed_all`, every dataset is embedded; otherwise datasets that came from a file
/// are linked.
///
/// # Errors
///
/// `SNR-APP-033` for an absolute path a spec names or a link that cannot be relative,
/// `SNR-APP-010`-class write errors as `SNR-APP-022`, and an unavailable dataset as
/// `SNR-APP-034`.
pub fn write(project: &Project, target: &Path, embed_all: bool) -> Result<Vec<u8>, IpcError> {
    let folder = target.parent().unwrap_or_else(|| Path::new(""));
    let mut members: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let specs = spec_members(project, &mut members)?;
    let datasets = project
        .datasets
        .iter()
        .filter(|dataset| dataset.kept.is_none())
        .map(|dataset| dataset_member(dataset, folder, embed_all, &mut members))
        .collect::<Result<Vec<_>, _>>()?;
    let kept: Vec<&serde_json::Value> = project
        .datasets
        .iter()
        .filter_map(|dataset| dataset.kept.as_ref())
        .collect();
    let newer = !kept.is_empty() || datasets.iter().any(|d| d.storage == Storage::Function);
    let version = if newer { FORMAT_VERSION } else { BASE_VERSION };
    for (path, bytes) in &project.reserved {
        members.insert(path.clone(), bytes.clone());
    }
    let manifest = Manifest {
        format: FORMAT.to_owned(),
        format_version: format!("{}.{}", version.0, version.1),
        generator: Generator {
            name: "sonarithm".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        },
        name: project.name.clone(),
        specs,
        active_spec: project.active_spec.clone(),
        datasets,
        members: members
            .iter()
            .map(|(path, bytes)| Member {
                path: path.clone(),
                bytes: bytes.len() as u64,
                sha256: hex(&Sha256::digest(bytes)),
            })
            .collect(),
    };
    let mut manifest = serde_json::to_value(&manifest).map_err(|err| {
        IpcError::new(
            &codes::EXPORT_FAILED,
            format!("The manifest could not be written: {err}."),
        )
    })?;
    // Entries kept unread go back as they came, among the others by ID.
    if let Some(list) = manifest["datasets"].as_array_mut() {
        list.extend(kept.into_iter().cloned());
        list.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    }
    let manifest = sonarithm_spec::canonical::to_canonical(&manifest).map_err(|err| {
        IpcError::new(
            &codes::EXPORT_FAILED,
            format!("The manifest could not be written: {err}."),
        )
    })?;
    zip_members(&manifest, &members)
}

/// The spec members, refusing IDs that cannot name a member and absolute file sources.
fn spec_members(
    project: &Project,
    members: &mut BTreeMap<String, Vec<u8>>,
) -> Result<Vec<SpecEntry>, IpcError> {
    let mut specs = Vec::new();
    for (id, text) in &project.specs {
        if !valid_id(id) {
            return Err(IpcError::new(
                &codes::PROJECT_PATH,
                format!("The spec ID `{id}` cannot name a project member."),
            ));
        }
        if has_absolute_file_source(text) {
            return Err(IpcError::new(
                &codes::PROJECT_PATH,
                format!(
                    "Spec `{id}` reads a file by its absolute path, which a project never stores. \
                     Open the file as a dataset and refer to it by ID."
                ),
            ));
        }
        let path = format!("specs/{id}.json");
        members.insert(path.clone(), text.as_bytes().to_vec());
        specs.push(SpecEntry {
            id: id.clone(),
            path,
        });
    }
    Ok(specs)
}

/// One dataset's manifest entry: linked, or embedded as a Parquet member.
fn dataset_member(
    dataset: &ProjectDataset,
    folder: &Path,
    embed_all: bool,
    members: &mut BTreeMap<String, Vec<u8>>,
) -> Result<DatasetEntry, IpcError> {
    {
        if !valid_id(&dataset.id) {
            return Err(IpcError::new(
                &codes::PROJECT_PATH,
                format!(
                    "The dataset ID `{}` cannot name a project member.",
                    dataset.id
                ),
            ));
        }
        // A function is saved as its definition, with no data member (format 1.1).
        if let Some(function) = &dataset.function {
            return Ok(DatasetEntry {
                id: dataset.id.clone(),
                name: dataset.name.clone(),
                storage: Storage::Function,
                path: None,
                link: None,
                function: Some(FunctionDef::of(function)),
                fingerprint: dataset.fingerprint.clone(),
                rows: dataset.rows,
            });
        }
        let link = match (&dataset.origin, &dataset.table) {
            // Data from a file is linked unless the user embeds everything.
            (Some(origin), Some(_)) if !embed_all => {
                Some(relative(folder, origin).ok_or_else(|| {
                    IpcError::new(
                        &codes::PROJECT_PATH,
                        format!(
                            "`{}` is on another drive than the project, so it cannot be linked. \
                         Embed the data instead.",
                            dataset.name
                        ),
                    )
                })?)
            }
            // Unavailable linked data keeps its link and is never embedded (RFC-0002 §4).
            (_, None) if dataset.linked => dataset.link.clone(),
            _ => None,
        };
        if let Some(link) = link {
            return Ok(DatasetEntry {
                id: dataset.id.clone(),
                name: dataset.name.clone(),
                storage: Storage::Linked,
                path: None,
                link: Some(link),
                function: None,
                fingerprint: dataset.fingerprint.clone(),
                rows: dataset.rows,
            });
        }
        let table = dataset.table.as_ref().ok_or_else(|| {
            IpcError::new(
                &codes::LINKED_DATA,
                format!(
                    "`{}` is not available, so it cannot be embedded; link it again first.",
                    dataset.name
                ),
            )
        })?;
        let bytes = write_parquet(table).map_err(|err| {
            IpcError::new(
                &codes::EXPORT_FAILED,
                format!("`{}` could not be written: {err}.", dataset.name),
            )
        })?;
        let path = format!("data/{}.parquet", dataset.id);
        members.insert(path.clone(), bytes);
        Ok(DatasetEntry {
            id: dataset.id.clone(),
            name: dataset.name.clone(),
            storage: Storage::Embedded,
            path: Some(path),
            link: None,
            function: None,
            fingerprint: dataset.fingerprint.clone(),
            rows: dataset.rows,
        })
    }
}

/// The ZIP: manifest first, then members by path; fixed times and permissions.
fn zip_members(manifest: &str, members: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, IpcError> {
    let fail = |err: &dyn std::fmt::Display| {
        IpcError::new(
            &codes::EXPORT_FAILED,
            format!("The project could not be packed: {err}."),
        )
    };
    let base = SimpleFileOptions::default()
        .last_modified_time(zip::DateTime::default())
        .unix_permissions(0o644);
    let deflated = base
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(6));
    let stored = base.compression_method(CompressionMethod::Stored);
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file(MANIFEST, deflated).map_err(|e| fail(&e))?;
    zip.write_all(manifest.as_bytes()).map_err(|e| fail(&e))?;
    for (path, bytes) in members {
        let options = if path.ends_with(".parquet") {
            stored
        } else {
            deflated
        };
        zip.start_file(path.as_str(), options)
            .map_err(|e| fail(&e))?;
        zip.write_all(bytes).map_err(|e| fail(&e))?;
    }
    Ok(zip.finish().map_err(|e| fail(&e))?.into_inner())
}

/// Saves `project` at `target` crash-safely: a temporary file, flushed, then renamed.
///
/// # Errors
///
/// As [`write()`], and `SNR-APP-022` when the file cannot be written.
pub fn save(project: &Project, target: &Path, embed_all: bool) -> Result<(), IpcError> {
    save_with_fault(project, target, embed_all, None)
}

/// [`save`], interrupted at `fault` (fault-injection tests of crash safety).
///
/// # Errors
///
/// As [`save`].
#[doc(hidden)]
pub fn save_with_fault(
    project: &Project,
    target: &Path,
    embed_all: bool,
    fault: Option<crate::settings::Fault>,
) -> Result<(), IpcError> {
    let bytes = write(project, target, embed_all)?;
    crate::settings::write_atomically(target, &bytes, fault).map_err(|detail| {
        IpcError::new(
            &codes::EXPORT_FAILED,
            format!("The project could not be saved: {detail}."),
        )
    })
}

/// Reads one member whole, at most `limit` bytes.
fn read_member(
    archive: &mut ZipArchive<File>,
    name: &str,
    limit: u64,
) -> Result<Vec<u8>, IpcError> {
    let entry = archive
        .by_name(name)
        .map_err(|_| damaged("a listed member is missing"))?;
    let declared = entry.size();
    if declared > limit {
        return Err(damaged("a member is larger than allowed"));
    }
    let mut bytes = Vec::with_capacity(usize::try_from(declared).unwrap_or(0));
    entry
        .take(declared + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| damaged("a member cannot be read"))?;
    if bytes.len() as u64 != declared {
        return Err(damaged("a member is not the size it declares"));
    }
    Ok(bytes)
}

/// Hashes a stored member in place, 1 MiB at a time.
fn hash_in_place(file: &File, start: u64, len: u64) -> Result<String, IpcError> {
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 1 << 20];
    let mut done = 0;
    while done < len {
        let take = usize::try_from((len - done).min(buffer.len() as u64)).unwrap_or(0);
        sonarithm_data::source::read_exact_at(file, &mut buffer[..take], start + done)
            .map_err(|_| damaged("a member cannot be read"))?;
        hasher.update(&buffer[..take]);
        done += take as u64;
    }
    Ok(hex(&hasher.finalize()))
}

/// Reads the project at `path` (canonical, a regular file). `table_ids` gives each dataset,
/// in manifest order, the table ID it gets.
///
/// # Errors
///
/// `SNR-APP-031` for a newer major version, `SNR-APP-032` for an unsafe, oversized,
/// altered or malformed bundle.
pub fn open(path: &Path, table_ids: &mut dyn FnMut() -> u32) -> Result<Opened, IpcError> {
    let file = File::open(path).map_err(|_| damaged("the file cannot be read"))?;
    let reader = file
        .try_clone()
        .map_err(|_| damaged("the file cannot be read"))?;
    let mut archive = ZipArchive::new(reader).map_err(|_| damaged("it is not a ZIP file"))?;
    if archive.len() > MAX_MEMBERS {
        return Err(damaged("it has too many members"));
    }
    let mut warnings = Vec::new();
    let entries = scan(&mut archive, &mut warnings)?;
    // The manifest, then every listed member against its hash.
    let text = read_member(&mut archive, MANIFEST, MAX_JSON)?;
    let (manifest, kept) = parse_manifest(&text, &mut warnings)?;
    let texts = verify(&file, &mut archive, &manifest, &entries)?;
    let listed: BTreeSet<&str> = manifest.members.iter().map(|m| m.path.as_str()).collect();
    let mut project = Project {
        name: manifest.name.clone(),
        active_spec: manifest.active_spec.clone(),
        ..Project::default()
    };
    for spec in &manifest.specs {
        let bytes = texts
            .get(&spec.path)
            .filter(|_| listed.contains(spec.path.as_str()))
            .ok_or_else(|| damaged("a spec is not listed"))?;
        let text =
            String::from_utf8(bytes.clone()).map_err(|_| damaged("a spec is not UTF-8 text"))?;
        project.specs.push((spec.id.clone(), text));
    }
    if !project
        .specs
        .iter()
        .any(|(id, _)| *id == project.active_spec)
    {
        return Err(damaged("the active spec is not in the project"));
    }
    for (path, bytes) in &texts {
        if entries.get(path).is_some_and(|e| e.kind == Kind::Reserved) {
            project.reserved.insert(path.clone(), bytes.clone());
        }
    }
    let folder = path.parent().unwrap_or_else(|| Path::new(""));
    for entry in &manifest.datasets {
        let ids = table_ids();
        let options = ReadOptions::new()
            .with_limits(Limits::DEFAULT)
            .with_ids(BundleId(u64::from(ids)), TableId(ids));
        let dataset = match entry.storage {
            Storage::Embedded => embedded(&file, entry, &listed, &entries, &options)?,
            Storage::Linked => linked_dataset(folder, entry, &options, &mut warnings),
            Storage::Function => function_dataset(entry, ids, &mut warnings)?,
        };
        project.datasets.push(dataset);
    }
    for value in kept {
        // Every dataset takes an ID, so the caller's numbering matches the list.
        let _ = table_ids();
        project.datasets.push(ProjectDataset {
            id: value["id"].as_str().unwrap_or_default().to_owned(),
            name: value["name"].as_str().unwrap_or_default().to_owned(),
            table: None,
            origin: None,
            linked: false,
            link: None,
            fingerprint: value["fingerprint"].as_str().unwrap_or_default().to_owned(),
            rows: value["rows"].as_u64().unwrap_or(0),
            function: None,
            kept: Some(value),
        });
    }
    Ok(Opened { project, warnings })
}

/// Where a member is and how it is packed.
#[derive(Clone, Copy)]
struct Entry {
    kind: Kind,
    size: u64,
    start: u64,
    method: CompressionMethod,
}

/// Every member: a safe name, within the limits, stored Parquet only. Unknown members are
/// counted into one warning.
fn scan(
    archive: &mut ZipArchive<File>,
    warnings: &mut Vec<IpcError>,
) -> Result<BTreeMap<String, Entry>, IpcError> {
    let mut entries: BTreeMap<String, Entry> = BTreeMap::new();
    let mut total = 0_u64;
    let mut ignored = 0;
    for index in 0..archive.len() {
        let entry = archive
            .by_index_raw(index)
            .map_err(|_| damaged("a member cannot be read"))?;
        let name = entry
            .name()
            .map_err(|_| damaged("a member name is not text"))?
            .into_owned();
        if entry.is_dir() {
            ignored += 1;
            continue;
        }
        let Some(kind) = member_kind(&name)? else {
            ignored += 1;
            continue;
        };
        let (size, packed) = (entry.size(), entry.compressed_size());
        total = total.saturating_add(size);
        if total > MAX_TOTAL {
            return Err(damaged("its members are larger than 2 GiB in all"));
        }
        let method = entry.compression();
        match kind {
            Kind::Data if method != CompressionMethod::Stored => {
                return Err(damaged("a data member is compressed; it must be stored"));
            }
            Kind::Data => {}
            _ if size > MAX_JSON => return Err(damaged("a JSON member is larger than 64 MiB")),
            _ if method == CompressionMethod::Stored => {}
            _ if size > packed.saturating_mul(MAX_RATIO) => {
                return Err(damaged("a member is compressed more than 100 to 1"));
            }
            _ => {}
        }
        let start = entry
            .data_start()
            .ok_or_else(|| damaged("a member's data cannot be located"))?;
        entries.insert(
            name,
            Entry {
                kind,
                size,
                start,
                method,
            },
        );
    }
    if ignored > 0 {
        warnings.push(IpcError::new(
            &codes::PROJECT_NEWER,
            format!("{ignored} member(s) of the project are unknown to this version and were ignored; saving will drop them."),
        ));
    }
    Ok(entries)
}

/// Checks every listed member against its size and hash: stored data in place, JSON read
/// whole (and returned by path).
fn verify(
    file: &File,
    archive: &mut ZipArchive<File>,
    manifest: &Manifest,
    entries: &BTreeMap<String, Entry>,
) -> Result<BTreeMap<String, Vec<u8>>, IpcError> {
    let mut texts: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for member in &manifest.members {
        let Some(entry) = entries.get(&member.path) else {
            return Err(damaged("a listed member is missing"));
        };
        if entry.size != member.bytes {
            return Err(damaged("a member is not the size the manifest records"));
        }
        let digest = if entry.kind == Kind::Data && entry.method == CompressionMethod::Stored {
            hash_in_place(file, entry.start, entry.size)?
        } else {
            let bytes = read_member(archive, &member.path, MAX_JSON)?;
            let digest = hex(&Sha256::digest(&bytes));
            texts.insert(member.path.clone(), bytes);
            digest
        };
        if digest != member.sha256 {
            return Err(damaged("a member was altered or damaged"));
        }
    }
    Ok(texts)
}

/// An embedded dataset, read in place from its stored member and checked against its
/// fingerprint.
fn embedded(
    file: &File,
    entry: &DatasetEntry,
    listed: &BTreeSet<&str>,
    entries: &BTreeMap<String, Entry>,
    options: &ReadOptions,
) -> Result<ProjectDataset, IpcError> {
    let member = entry
        .path
        .as_deref()
        .filter(|p| listed.contains(p))
        .ok_or_else(|| damaged("an embedded dataset is not listed"))?;
    let at = entries
        .get(member)
        .ok_or_else(|| damaged("a listed member is missing"))?;
    let region = file
        .try_clone()
        .map_err(|_| damaged("the file cannot be read"))?;
    let loaded = read_parquet_at(region, at.start, at.size, &entry.name, options)
        .map_err(|err| IpcError::from_diagnostic(&err.to_diagnostic()))?;
    let table = loaded
        .bundle
        .primary()
        .cloned()
        .ok_or_else(|| damaged("an embedded dataset is empty"))?;
    let fingerprint = table_fingerprint(&table)
        .map_err(|err| IpcError::from_diagnostic(&err.to_diagnostic()))?
        .to_string();
    if fingerprint != entry.fingerprint {
        return Err(damaged(
            "an embedded dataset does not match its fingerprint",
        ));
    }
    Ok(ProjectDataset {
        id: entry.id.clone(),
        name: entry.name.clone(),
        rows: table.num_rows(),
        table: Some(table),
        origin: None,
        linked: false,
        link: None,
        fingerprint,
        function: None,
        kept: None,
    })
}

/// A function dataset (format 1.1): sampled again from its definition. A different
/// fingerprint (CURVASONA samples differently from the saving build) is a warning, and
/// the new samples are used; an expression this build cannot read leaves it unavailable,
/// its definition kept for the next save.
fn function_dataset(
    entry: &DatasetEntry,
    ids: u32,
    warnings: &mut Vec<IpcError>,
) -> Result<ProjectDataset, IpcError> {
    let definition = entry
        .function
        .as_ref()
        .ok_or_else(|| damaged("a function dataset has no definition"))?;
    let spec = definition.spec();
    let name = file_name(&entry.name);
    let sampled = sonarithm::read_function_with_ids(&spec, BundleId(u64::from(ids)), TableId(ids))
        .ok()
        .and_then(|function| function.into_bundle().primary().cloned());
    let Some(table) = sampled else {
        warnings.push(IpcError::new(
            &codes::LINKED_DATA,
            format!(
                "The function `{name}` cannot be sampled by this version, so it is unavailable."
            ),
        ));
        return Ok(ProjectDataset {
            id: entry.id.clone(),
            name: entry.name.clone(),
            table: None,
            origin: None,
            linked: false,
            link: None,
            fingerprint: entry.fingerprint.clone(),
            rows: entry.rows,
            function: Some(spec),
            kept: None,
        });
    };
    let fingerprint = table_fingerprint(&table)
        .map_err(|_| damaged("a function dataset cannot be fingerprinted"))?
        .to_string();
    if fingerprint != entry.fingerprint {
        warnings.push(IpcError::new(
            &codes::LINKED_DATA,
            format!(
                "The function `{name}` samples differently in this version; it opens with the new samples, and saving records them."
            ),
        ));
    }
    Ok(ProjectDataset {
        id: entry.id.clone(),
        name: entry.name.clone(),
        rows: table.num_rows(),
        table: Some(table),
        origin: None,
        linked: false,
        link: None,
        fingerprint,
        function: Some(spec),
        kept: None,
    })
}

/// A linked dataset: loaded when RFC-0002 §4's conditions hold, else unavailable with a
/// warning that names the file alone.
fn linked_dataset(
    folder: &Path,
    entry: &DatasetEntry,
    options: &ReadOptions,
    warnings: &mut Vec<IpcError>,
) -> ProjectDataset {
    let (table, origin) = match linked(folder, entry.link.as_deref(), &entry.fingerprint, options) {
        Ok((table, origin)) => (Some(table), Some(origin)),
        Err(why) => {
            warnings.push(IpcError::new(
                &codes::LINKED_DATA,
                format!(
                    "The linked data `{}` {why}, so it is unavailable.",
                    file_name(&entry.name)
                ),
            ));
            (None, None)
        }
    };
    ProjectDataset {
        id: entry.id.clone(),
        name: entry.name.clone(),
        table,
        origin,
        linked: true,
        link: entry.link.clone(),
        fingerprint: entry.fingerprint.clone(),
        rows: entry.rows,
        function: None,
        kept: None,
    }
}

/// The file name part of a recorded name, never directories (RFC-0002 §4).
fn file_name(name: &str) -> &str {
    name.rsplit(['/', '\\']).next().unwrap_or(name)
}

/// Opens a linked dataset under RFC-0002 §4's conditions; `Err` says why not, in words.
fn linked(
    folder: &Path,
    link: Option<&str>,
    fingerprint: &str,
    options: &ReadOptions,
) -> Result<(Table, PathBuf), String> {
    let link = link.ok_or("has no link")?;
    let relative_only = !link.is_empty()
        && !link.starts_with('/')
        && !link.contains('\\')
        && !link.contains(':')
        && !link.contains('\0');
    if !relative_only {
        return Err("has a link that is not relative".to_owned());
    }
    let mut target = folder.to_path_buf();
    for part in link.split('/') {
        target.push(part);
    }
    let extension = target
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    if !extension
        .as_deref()
        .is_some_and(|ext| DATA_EXTENSIONS.contains(&ext))
    {
        return Err("is not a data file".to_owned());
    }
    let canonical = std::fs::canonicalize(&target).map_err(|_| "is missing".to_owned())?;
    let extension = canonical
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    if !extension
        .as_deref()
        .is_some_and(|ext| DATA_EXTENSIONS.contains(&ext))
    {
        return Err("points to a file that is not data".to_owned());
    }
    if !std::fs::metadata(&canonical).is_ok_and(|m| m.is_file()) {
        return Err("is not a regular file".to_owned());
    }
    let loaded = read_file(&canonical, None, options).map_err(|_| "cannot be read".to_owned())?;
    let table = loaded.bundle.primary().cloned().ok_or("holds no table")?;
    let found = table_fingerprint(&table)
        .map_err(|_| "cannot be checked".to_owned())?
        .to_string();
    if found != fingerprint {
        return Err("has changed since the project was saved".to_owned());
    }
    Ok((table, canonical))
}

/// Parses the manifest: the format, the version, then the fields this version knows.
fn parse_manifest(
    text: &[u8],
    warnings: &mut Vec<IpcError>,
) -> Result<(Manifest, Vec<serde_json::Value>), IpcError> {
    let mut value: serde_json::Value =
        serde_json::from_slice(text).map_err(|_| damaged("the manifest is not JSON"))?;
    if value["format"] != FORMAT {
        return Err(damaged("it is not a SONARITHM project"));
    }
    let version = value["formatVersion"].as_str().unwrap_or_default();
    let (major, minor) = version
        .split_once('.')
        .and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?)))
        .ok_or_else(|| damaged("the format version is not MAJOR.MINOR"))?;
    if major > FORMAT_VERSION.0 {
        return Err(IpcError::new(
            &codes::PROJECT_MAJOR,
            format!(
                "The project uses format version {version}, made by a newer SONARITHM; this \
                 version reads format {}.",
                FORMAT_VERSION.0
            ),
        ));
    }
    if major < FORMAT_VERSION.0 {
        return Err(damaged(
            "its format version is older than any this build reads",
        ));
    }
    if minor > FORMAT_VERSION.1 {
        warnings.push(IpcError::new(
            &codes::PROJECT_NEWER,
            format!(
                "The project uses format version {version}, newer than this version's {}.{}; \
                 saving will write {}.{} and drop what this version does not know.",
                FORMAT_VERSION.0, FORMAT_VERSION.1, FORMAT_VERSION.0, FORMAT_VERSION.1
            ),
        ));
    }
    // Entries with a storage this build does not know are kept unread (RFC-0002 §12),
    // before unknown fields are stripped, so they go back exactly as they came.
    let kept = take_unknown_storages(&mut value, warnings);
    if minor > FORMAT_VERSION.1 {
        strip_unknown(&mut value);
    }
    let manifest = serde_json::from_value(value)
        .map_err(|_| damaged("the manifest does not have the format's fields"))?;
    Ok((manifest, kept))
}

/// Removes the dataset entries whose `storage` this build does not know, warning for each
/// (`SNR-APP-035`): they are unavailable and kept as they are for the next save.
fn take_unknown_storages(
    value: &mut serde_json::Value,
    warnings: &mut Vec<IpcError>,
) -> Vec<serde_json::Value> {
    let Some(list) = value["datasets"].as_array_mut() else {
        return Vec::new();
    };
    let (known, unknown): (Vec<_>, Vec<_>) = std::mem::take(list).into_iter().partition(|entry| {
        entry["storage"]
            .as_str()
            .is_some_and(|storage| KNOWN_STORAGES.contains(&storage))
    });
    *list = known;
    for entry in &unknown {
        warnings.push(IpcError::new(
            &codes::PROJECT_NEWER,
            format!(
                "The dataset `{}` ({}) is stored as `{}`, which this version does not know; it is unavailable, and saving keeps it as it is.",
                file_name(entry["name"].as_str().unwrap_or_default()),
                entry["id"].as_str().unwrap_or_default(),
                entry["storage"].as_str().unwrap_or("?")
            ),
        ));
    }
    unknown
}

/// Removes fields a newer minor version may have added, at every level this version reads.
fn strip_unknown(value: &mut serde_json::Value) {
    fn keep(value: &mut serde_json::Value, known: &[&str]) {
        if let Some(map) = value.as_object_mut() {
            map.retain(|key, _| known.contains(&key.as_str()));
        }
    }
    keep(
        value,
        &[
            "format",
            "formatVersion",
            "generator",
            "name",
            "specs",
            "activeSpec",
            "datasets",
            "members",
        ],
    );
    keep(&mut value["generator"], &["name", "version"]);
    for (list, known) in [
        ("specs", &["id", "path"][..]),
        (
            "datasets",
            &[
                "id",
                "name",
                "storage",
                "path",
                "link",
                "function",
                "fingerprint",
                "rows",
            ][..],
        ),
        ("members", &["path", "bytes", "sha256"][..]),
    ] {
        if let Some(items) = value[list].as_array_mut() {
            for item in items {
                keep(item, known);
                if list == "datasets" && item.get("function").is_some() {
                    keep(
                        &mut item["function"],
                        &["expr", "domain", "samples", "step", "variables"],
                    );
                }
            }
        }
    }
}

/// Autosaves of unsaved work (RFC-0002 §8).
#[derive(Debug)]
pub struct Autosaves {
    dir: PathBuf,
    budget: u64,
}

/// One autosave on disk.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Autosave {
    /// Its number; higher is newer.
    pub id: u64,
    /// The file system's modification time, seconds since 1970.
    pub modified_s: f64,
}

/// The most autosaves kept.
pub const AUTOSAVES_KEPT: usize = 5;

/// The autosave folder's size budget. TODO(verify): B05-T07's soak sets it (RFC-0002 §8).
pub const AUTOSAVE_BUDGET: u64 = 2 << 30;

impl Autosaves {
    /// The autosaves in `dir` (the app-data folder's `autosave/`).
    #[must_use]
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            budget: AUTOSAVE_BUDGET,
        }
    }

    /// The same with another size budget (tests).
    #[must_use]
    pub fn with_budget(mut self, budget: u64) -> Self {
        self.budget = budget;
        self
    }

    fn path(&self, id: u64) -> PathBuf {
        self.dir.join(format!("autosave-{id}.{EXTENSION}"))
    }

    /// The autosaves on disk, newest first.
    #[must_use]
    pub fn list(&self) -> Vec<Autosave> {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut found: Vec<Autosave> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let id = name
                    .strip_prefix("autosave-")?
                    .strip_suffix(&format!(".{EXTENSION}"))?
                    .parse()
                    .ok()?;
                let modified_s = entry
                    .metadata()
                    .ok()?
                    .modified()
                    .ok()?
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs_f64();
                Some(Autosave { id, modified_s })
            })
            .collect();
        found.sort_by_key(|a| std::cmp::Reverse(a.id));
        found
    }

    /// Writes a new autosave and prunes to [`AUTOSAVES_KEPT`] and the size budget.
    ///
    /// # Errors
    ///
    /// The file system's error, in words.
    pub fn write(&self, bytes: &[u8]) -> Result<u64, String> {
        std::fs::create_dir_all(&self.dir).map_err(|err| err.to_string())?;
        let id = self.list().first().map_or(1, |newest| newest.id + 1);
        crate::settings::write_atomically(&self.path(id), bytes, None)?;
        let mut kept = 0_usize;
        let mut size = 0_u64;
        for autosave in self.list() {
            let path = self.path(autosave.id);
            let len = std::fs::metadata(&path).map_or(0, |m| m.len());
            // The newest always stays, even alone over the budget.
            if autosave.id != id && (kept >= AUTOSAVES_KEPT || size + len > self.budget) {
                let _ = std::fs::remove_file(path);
            } else {
                kept += 1;
                size += len;
            }
        }
        Ok(id)
    }

    /// The path of autosave `id`, if it exists.
    #[must_use]
    pub fn get(&self, id: u64) -> Option<PathBuf> {
        let path = self.path(id);
        path.is_file().then_some(path)
    }

    /// Deletes every autosave.
    pub fn discard(&self) {
        for autosave in self.list() {
            let _ = std::fs::remove_file(self.path(autosave.id));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_names_are_checked_for_zip_slip() {
        for bad in [
            "../evil.json",
            "/abs.json",
            "specs/../x.json",
            "C:/x.json",
            "specs\\x.json",
            "specs//x.json",
            "a\0b",
        ] {
            assert!(member_kind(bad).is_err(), "{bad}");
        }
        assert_eq!(member_kind("specs/main.json").unwrap(), Some(Kind::Spec));
        assert_eq!(member_kind("data/d1.parquet").unwrap(), Some(Kind::Data));
        assert_eq!(
            member_kind("renders/a.render.json").unwrap(),
            Some(Kind::Reserved)
        );
        assert_eq!(member_kind("extras/x.json").unwrap(), None);
        assert_eq!(member_kind("specs/no space.json").unwrap(), None);
    }

    #[test]
    fn links_are_relative_with_forward_slashes() {
        let base = Path::new("/home/u/projects/p");
        assert_eq!(
            relative(base, Path::new("/home/u/data/a.csv")).as_deref(),
            Some("../../data/a.csv")
        );
        assert_eq!(
            relative(base, Path::new("/home/u/projects/p/a.csv")).as_deref(),
            Some("a.csv")
        );
    }

    #[test]
    fn absolute_file_sources_are_found() {
        assert!(has_absolute_file_source(
            r#"{"data": {"source": {"kind": "file", "path": "C:\\data\\a.csv"}}}"#
        ));
        assert!(has_absolute_file_source(
            r#"{"data": {"source": {"kind": "file", "path": "/data/a.csv"}}}"#
        ));
        assert!(!has_absolute_file_source(
            r#"{"data": {"source": {"kind": "file", "path": "a.csv"}}}"#
        ));
    }
}

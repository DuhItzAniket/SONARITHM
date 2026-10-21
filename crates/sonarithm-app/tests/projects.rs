// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `.sonarithm` project bundles through the protocol and the bundle module (RFC-0002 §11,
//! B05-T04).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use sonarithm_app::ipc::IpcError;
use sonarithm_app::project::{self, Project};
use sonarithm_app::settings::Fault;
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, NullSink};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const CSV: &str = "day,temp\n1,8.5\n2,9.25\n3,7.0\n";

fn spec(dataset: &str) -> String {
    json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "dataset", "id": dataset } },
        "substrate": { "time": { "field": "day" } },
        "tracks": [{ "id": "t", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
    })
    .to_string()
}

fn app(dir: &Path) -> App {
    let config = AppConfig::new(dir.join("config"))
        .with_audio(AudioOutput::null())
        .with_data_dir(dir.join("data"));
    App::new(config, Arc::new(NullSink)).unwrap()
}

fn send(app: &mut App, request: &Value) -> Result<Value, IpcError> {
    app.dispatch_json(&request.to_string())
}

fn ok(app: &mut App, request: &Value) -> Value {
    send(app, request).unwrap_or_else(|err| panic!("{request}: {err}"))
}

fn write_csv(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

/// A session with one dataset and a spec over it.
fn session(dir: &Path) -> App {
    let mut app = app(dir);
    let csv = write_csv(dir, "temps.csv", CSV);
    ok(
        &mut app,
        &json!({ "command": "dataset_open", "args": { "path": csv } }),
    );
    ok(
        &mut app,
        &json!({ "command": "spec_set", "args": { "text": spec("d1") } }),
    );
    app
}

#[test]
fn a_saved_project_reopens_and_saves_to_the_same_bytes() {
    let dir = TempDir::new("roundtrip");
    let mut app = session(dir.path());
    let first = dir.path().join("p.sonarithm");
    let saved = ok(
        &mut app,
        &json!({ "command": "project_save_as", "args": { "path": first, "embedData": true } }),
    );
    assert_eq!(saved["fileName"], "p.sonarithm");
    assert_eq!(saved["unsaved"], false);
    let bytes = std::fs::read(&first).unwrap();

    // A second dataset from a file is linked when data is not embedded.
    let mut app = self::app(dir.path());
    ok(
        &mut app,
        &json!({ "command": "project_open", "args": { "path": first } }),
    );
    let other = write_csv(dir.path(), "more.csv", "day,temp\n1,1.0\n2,2.0\n");
    ok(
        &mut app,
        &json!({ "command": "dataset_open", "args": { "path": other } }),
    );
    let second = dir.path().join("q.sonarithm");
    let summary = ok(
        &mut app,
        &json!({ "command": "project_save_as", "args": { "path": second, "embedData": false } }),
    );
    let linked: Vec<bool> = summary["datasets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["linked"].as_bool().unwrap())
        .collect();
    assert_eq!(linked, [false, true]);

    // Reopen and save without changes: identical bytes, specs byte for byte.
    let mut app = self::app(dir.path());
    ok(
        &mut app,
        &json!({ "command": "project_open", "args": { "path": first } }),
    );
    assert_eq!(
        ok(&mut app, &json!({ "command": "spec_get" }))["text"],
        spec("d1")
    );
    ok(&mut app, &json!({ "command": "project_save" }));
    assert_eq!(
        std::fs::read(&first).unwrap(),
        bytes,
        "re-saved bytes differ"
    );
    let compiled = ok(&mut app, &json!({ "command": "compile" }));
    assert_eq!(compiled["ok"], true, "{compiled}");
}

#[test]
fn reserved_members_and_several_specs_survive_byte_for_byte() {
    let dir = TempDir::new("reserved");
    let mut project = Project::default();
    project.name = "Kept".to_owned();
    project.specs = vec![
        (
            "main".to_owned(),
            "{ \"specVersion\": \"0.1\" }\n".to_owned(),
        ),
        (
            "alt".to_owned(),
            "{\"specVersion\":\"0.1\",\"name\":\"alt\"}".to_owned(),
        ),
    ];
    project.active_spec = "main".to_owned();
    project.reserved.insert(
        "annotations/d1.json".to_owned(),
        b"{\"notes\": [1, 2]}".to_vec(),
    );
    project
        .reserved
        .insert("profiles/project-profile.json".to_owned(), b"{}".to_vec());
    let path = dir.path().join("kept.sonarithm");
    project::save(&project, &path, true).unwrap();
    let opened = project::open(&path, &mut || 1).unwrap();
    assert_eq!(opened.project.specs, project.specs);
    assert_eq!(opened.project.reserved, project.reserved);
    assert_eq!(opened.warnings.len(), 0);
    // Writing again gives the same bytes.
    let again = project::write(&opened.project, &path, true).unwrap();
    assert_eq!(again, std::fs::read(&path).unwrap());
}

#[test]
fn an_interrupted_save_leaves_the_previous_project_readable() {
    let dir = TempDir::new("crashsave");
    let path = dir.path().join("p.sonarithm");
    let mut project = Project::default();
    project.specs = vec![("main".to_owned(), "{\"v\": 1}".to_owned())];
    project.active_spec = "main".to_owned();
    project::save(&project, &path, true).unwrap();
    project.specs[0].1 = "{\"v\": 2}".to_owned();
    for fault in [Fault::PartialTemp, Fault::BeforeRename] {
        assert!(project::save_with_fault(&project, &path, true, Some(fault)).is_err());
        let opened = project::open(&path, &mut || 1).unwrap();
        assert_eq!(opened.project.specs[0].1, "{\"v\": 1}", "{fault:?}");
    }
}

#[test]
fn a_crash_leaves_autosaves_that_the_next_start_recovers() {
    let dir = TempDir::new("recover");
    let app = session(dir.path());
    // Dropped without shutdown: what a crash looks like.
    drop(app);
    let mut next = self::app(dir.path());
    let recovery = ok(&mut next, &json!({ "command": "project_recovery" }));
    let autosaves = recovery["autosaves"].as_array().unwrap().clone();
    assert!(!autosaves.is_empty(), "{recovery}");
    let id = autosaves[0]["id"].clone();
    let recovered = ok(
        &mut next,
        &json!({ "command": "project_recover", "args": { "id": id } }),
    );
    assert_eq!(recovered["unsaved"], true);
    assert_eq!(recovered["fileName"], Value::Null);
    assert_eq!(
        ok(&mut next, &json!({ "command": "spec_get" }))["text"],
        spec("d1")
    );
    assert_eq!(recovered["datasets"][0]["available"], true);
    // A clean exit removes them.
    next.shutdown();
    let mut third = self::app(dir.path());
    let recovery = ok(&mut third, &json!({ "command": "project_recovery" }));
    assert_eq!(recovery["autosaves"], json!([]));
}

#[test]
fn autosaves_keep_the_newest_five_within_the_budget() {
    let dir = TempDir::new("autosaves");
    let autosaves = project::Autosaves::new(dir.path().join("a"));
    for i in 0..8 {
        autosaves.write(format!("bundle {i}").as_bytes()).unwrap();
    }
    let ids: Vec<u64> = autosaves.list().into_iter().map(|a| a.id).collect();
    assert_eq!(ids, [8, 7, 6, 5, 4]);
    let small = project::Autosaves::new(dir.path().join("b")).with_budget(10);
    for i in 0..3 {
        small.write(format!("bundle {i}").as_bytes()).unwrap();
    }
    assert_eq!(small.list().len(), 1, "only the newest fits the budget");
}

/// A bundle with custom members, its manifest listing `listed` with their real hashes.
fn craft(path: &Path, members: &[(&str, Vec<u8>, CompressionMethod)], manifest: &Value) {
    let file = std::fs::File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file("manifest.json", deflated).unwrap();
    zip.write_all(manifest.to_string().as_bytes()).unwrap();
    for (name, bytes, method) in members {
        let options = SimpleFileOptions::default().compression_method(*method);
        zip.start_file(*name, options).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
}

fn member(path: &str, bytes: &[u8]) -> Value {
    let digest = Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut out, b| {
            let _ = write!(out, "{b:02x}");
            out
        });
    json!({ "path": path, "bytes": bytes.len(), "sha256": digest })
}

fn manifest(version: &str, members: &[Value]) -> Value {
    json!({
        "format": "sonarithm-project",
        "formatVersion": version,
        "generator": { "name": "test", "version": "0" },
        "name": "Crafted",
        "specs": [{ "id": "main", "path": "specs/main.json" }],
        "activeSpec": "main",
        "datasets": [],
        "members": members,
    })
}

fn open_code(path: &Path) -> String {
    match project::open(path, &mut || 1) {
        Ok(_) => "ok".to_owned(),
        Err(err) => err.code,
    }
}

#[test]
fn unsafe_damaged_and_newer_bundles_are_refused() {
    let dir = TempDir::new("refusals");
    let spec = b"{\"specVersion\":\"0.1\"}".to_vec();
    let good = [("specs/main.json", spec.clone(), CompressionMethod::Deflated)];
    let listed = [member("specs/main.json", &spec)];
    let case = |name: &str, members: &[(&str, Vec<u8>, CompressionMethod)], manifest: Value| {
        let path = dir.path().join(format!("{name}.sonarithm"));
        craft(&path, members, &manifest);
        open_code(&path)
    };
    assert_eq!(case("good", &good, manifest("1.0", &listed)), "ok");
    // Zip-slip.
    let slip = [
        good[0].clone(),
        ("../evil.json", b"{}".to_vec(), CompressionMethod::Stored),
    ];
    assert_eq!(case("slip", &slip, manifest("1.0", &listed)), "SNR-APP-032");
    // Altered member.
    let altered = [(
        "specs/main.json",
        b"{\"evil\":1}".to_vec(),
        CompressionMethod::Deflated,
    )];
    assert_eq!(
        case("altered", &altered, manifest("1.0", &listed)),
        "SNR-APP-032"
    );
    // Missing listed member.
    assert_eq!(
        case("missing", &[], manifest("1.0", &listed)),
        "SNR-APP-032"
    );
    // A deflated data member.
    let deflated_data = [
        good[0].clone(),
        (
            "data/d1.parquet",
            b"PAR1".to_vec(),
            CompressionMethod::Deflated,
        ),
    ];
    assert_eq!(
        case("deflated", &deflated_data, manifest("1.0", &listed)),
        "SNR-APP-032"
    );
    // A compression bomb: 1 MiB of zeros deflates far beyond 100:1.
    let bomb = [
        good[0].clone(),
        (
            "annotations/x.json",
            vec![b' '; 1 << 20],
            CompressionMethod::Deflated,
        ),
    ];
    assert_eq!(case("bomb", &bomb, manifest("1.0", &listed)), "SNR-APP-032");
    // A newer major version is refused; a newer minor one opens with a warning.
    assert_eq!(
        case("major", &good, manifest("2.0", &listed)),
        "SNR-APP-031"
    );
    let mut newer = manifest("1.7", &listed);
    newer["futureField"] = json!(true);
    let path = dir.path().join("minor.sonarithm");
    craft(&path, &good, &newer);
    let opened = project::open(&path, &mut || 1).unwrap();
    assert_eq!(opened.warnings[0].code, "SNR-APP-035");
    // Not a ZIP at all.
    let junk = dir.path().join("junk.sonarithm");
    std::fs::write(&junk, b"not a zip").unwrap();
    assert_eq!(open_code(&junk), "SNR-APP-032");
}

#[test]
fn linked_data_that_is_missing_changed_or_not_data_stays_unavailable() {
    let dir = TempDir::new("links");
    let mut app = session(dir.path());
    let path = dir.path().join("linked.sonarithm");
    ok(
        &mut app,
        &json!({ "command": "project_save_as", "args": { "path": path, "embedData": false } }),
    );
    // Unchanged: available.
    let mut app = self::app(dir.path());
    let opened = ok(
        &mut app,
        &json!({ "command": "project_open", "args": { "path": path } }),
    );
    assert_eq!(opened["datasets"][0]["available"], true);
    // Changed: unavailable, named by file only, and kept as a link on the next save.
    std::fs::write(dir.path().join("temps.csv"), "day,temp\n1,99\n").unwrap();
    let mut app = self::app(dir.path());
    let opened = ok(
        &mut app,
        &json!({ "command": "project_open", "args": { "path": path } }),
    );
    assert_eq!(opened["datasets"][0]["available"], false);
    let warning = &opened["diagnostics"][0];
    assert_eq!(warning["code"], "SNR-APP-034");
    assert!(warning["message"].as_str().unwrap().contains("`temps.csv`"));
    assert!(
        !warning["message"]
            .as_str()
            .unwrap()
            .contains(&*dir.path().to_string_lossy())
    );
    ok(&mut app, &json!({ "command": "project_save" }));
    let reopened = project::open(&std::fs::canonicalize(&path).unwrap(), &mut || 1).unwrap();
    assert!(reopened.project.datasets[0].linked);
    assert!(reopened.project.datasets[0].table.is_none());

    // Links that point at something that is not a regular data file are refused unread.
    for (name, target) in [("notes", "notes.txt"), ("folder", "folder.csv")] {
        let bundle = dir.path().join(format!("{name}.sonarithm"));
        craft_linked(&bundle, target);
        let opened = project::open(&bundle, &mut || 1).unwrap();
        assert_eq!(opened.warnings[0].code, "SNR-APP-034", "{name}");
        assert!(opened.project.datasets[0].table.is_none(), "{name}");
    }
    std::fs::write(dir.path().join("notes.txt"), "secret").unwrap();
    std::fs::create_dir_all(dir.path().join("folder.csv")).unwrap();
    for (name, target) in [("notes2", "notes.txt"), ("folder2", "folder.csv")] {
        let bundle = dir.path().join(format!("{name}.sonarithm"));
        craft_linked(&bundle, target);
        let opened = project::open(&bundle, &mut || 1).unwrap();
        assert_eq!(opened.warnings[0].code, "SNR-APP-034", "{name}");
    }
    // A symbolic link to a file that is not data, where the OS lets the test make one.
    std::fs::write(dir.path().join("secret.txt"), "secret").unwrap();
    let link = dir.path().join("sneaky.csv");
    if make_symlink(&dir.path().join("secret.txt"), &link) {
        let bundle = dir.path().join("sneaky.sonarithm");
        craft_linked(&bundle, "sneaky.csv");
        let opened = project::open(&bundle, &mut || 1).unwrap();
        assert_eq!(opened.warnings[0].code, "SNR-APP-034");
        assert!(opened.project.datasets[0].table.is_none());
    }
    // A device.
    #[cfg(unix)]
    {
        let device = dir.path().join("zero.csv");
        if std::os::unix::fs::symlink("/dev/zero", &device).is_ok() {
            let bundle = dir.path().join("device.sonarithm");
            craft_linked(&bundle, "zero.csv");
            let opened = project::open(&bundle, &mut || 1).unwrap();
            assert!(opened.project.datasets[0].table.is_none());
        }
    }
}

fn make_symlink(target: &Path, link: &Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link).is_ok()
    }
}

/// A bundle with one spec and one linked dataset pointing at `link`.
fn craft_linked(path: &Path, link: &str) {
    let spec = b"{}".to_vec();
    let mut manifest = manifest("1.0", &[member("specs/main.json", &spec)]);
    manifest["datasets"] = json!([{
        "id": "d1", "name": link, "storage": "linked", "link": link,
        "fingerprint": "0".repeat(64), "rows": 1
    }]);
    craft(
        path,
        &[("specs/main.json", spec, CompressionMethod::Deflated)],
        &manifest,
    );
}

#[test]
fn no_absolute_path_is_ever_written() {
    let dir = TempDir::new("nopaths");
    let mut app = session(dir.path());
    let other = write_csv(dir.path(), "more.csv", "day,temp\n1,1\n");
    ok(
        &mut app,
        &json!({ "command": "dataset_open", "args": { "path": other } }),
    );
    let folder = dir
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    for embed in [true, false] {
        let path = dir.path().join(format!("p{embed}.sonarithm"));
        ok(
            &mut app,
            &json!({ "command": "project_save_as", "args": { "path": path, "embedData": embed } }),
        );
        let bytes = std::fs::read(&path).unwrap();
        let haystack = String::from_utf8_lossy(&bytes);
        assert!(
            !haystack.contains(&folder),
            "the folder's name is in the bundle"
        );
        let opened = project::open(&std::fs::canonicalize(&path).unwrap(), &mut || 1).unwrap();
        let _ = opened;
    }
    // A spec that reads a file by an absolute path cannot be saved.
    let absolute = json!({ "specVersion": "0.1",
        "data": { "source": { "kind": "file", "path": dir.path().join("temps.csv") } },
        "tracks": [] })
    .to_string();
    ok(
        &mut app,
        &json!({ "command": "spec_set", "args": { "text": absolute } }),
    );
    let err = send(
        &mut app,
        &json!({ "command": "project_save_as",
                 "args": { "path": dir.path().join("abs.sonarithm"), "embedData": true } }),
    )
    .unwrap_err();
    assert_eq!(err.code, "SNR-APP-033");
}

#[test]
fn project_paths_are_checked_and_never_echoed() {
    let dir = TempDir::new("projpaths");
    let mut app = session(dir.path());
    for path in [
        json!("relative.sonarithm"),
        json!(dir.path().join("x.zip")),
        json!(dir.path().join("gone").join("x.sonarithm")),
    ] {
        let err = send(
            &mut app,
            &json!({ "command": "project_save_as", "args": { "path": path, "embedData": true } }),
        )
        .unwrap_err();
        assert_eq!(err.code, "SNR-APP-060", "{path}");
    }
    let err = send(&mut app, &json!({ "command": "project_save" })).unwrap_err();
    assert_eq!(err.code, "SNR-APP-030");
    let path = dir.path().join("p.sonarithm");
    ok(
        &mut app,
        &json!({ "command": "project_save_as", "args": { "path": path, "embedData": true } }),
    );
    let recent = ok(&mut app, &json!({ "command": "project_recent" }));
    assert_eq!(
        recent["projects"],
        json!([{ "index": 0, "name": "p.sonarithm" }])
    );
    let reopened = ok(
        &mut app,
        &json!({ "command": "project_open_recent", "args": { "index": 0 } }),
    );
    assert_eq!(reopened["fileName"], "p.sonarithm");
}

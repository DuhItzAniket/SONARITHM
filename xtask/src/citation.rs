// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask citation`: keeps `CITATION.cff` and `.zenodo.json` in step with the
//! workspace version (docs/engineering/release-process.md, "Zenodo DOI"). release-plz does
//! not touch either file, and Zenodo reads `.zenodo.json` in preference to `CITATION.cff`,
//! so the two must agree.
//!
//! - `cargo xtask citation` (or `--check`) fails when either file names another version
//!   than `[workspace.package]`, when `.zenodo.json` lacks a required field, or when the
//!   two files list different keywords. Placeholders the owner still has to fill
//!   (`TODO`) are listed as notes.
//! - `cargo xtask citation --release YYYY-MM-DD` writes the workspace version into both
//!   files and the date into `date-released`; the release PR runs it.

use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Context as _, Result, bail};
use serde_json::Value;

/// Runs the task with its options.
pub(crate) fn run(root: &Path, options: &[String]) -> Result<()> {
    let version = workspace_version(&read(root, "Cargo.toml")?)?;
    match options {
        [] => check(root, &version),
        [flag] if flag == "--check" => check(root, &version),
        [flag, date] if flag == "--release" => release(root, &version, date),
        _ => bail!("expected no option, `--check`, or `--release YYYY-MM-DD`"),
    }
}

fn read(root: &Path, file: &str) -> Result<String> {
    std::fs::read_to_string(root.join(file)).with_context(|| format!("reading {file}"))
}

fn write(root: &Path, file: &str, text: &str) -> Result<()> {
    std::fs::write(root.join(file), text).with_context(|| format!("writing {file}"))
}

fn check(root: &Path, version: &str) -> Result<()> {
    let cff = read(root, "CITATION.cff")?;
    let zenodo = read(root, ".zenodo.json")?;
    let problems = problems(&cff, &zenodo, version)?;
    for note in todos(&cff, &zenodo) {
        println!("citation: note: {note}");
    }
    if problems.is_empty() {
        println!("citation: CITATION.cff and .zenodo.json name version {version}");
        Ok(())
    } else {
        bail!(
            "citation metadata is out of step:\n- {}\nRun `cargo xtask citation --release YYYY-MM-DD` in the release PR.",
            problems.join("\n- ")
        )
    }
}

fn release(root: &Path, version: &str, date: &str) -> Result<()> {
    if !is_date(date) {
        bail!("`{date}` is not a date; write YYYY-MM-DD");
    }
    let cff = set_cff(&read(root, "CITATION.cff")?, version, date)?;
    let zenodo = set_zenodo(&read(root, ".zenodo.json")?, version, date)?;
    write(root, "CITATION.cff", &cff)?;
    write(root, ".zenodo.json", &zenodo)?;
    println!("citation: CITATION.cff and .zenodo.json now name {version}, released {date}");
    for note in todos(&cff, &zenodo) {
        println!("citation: note: {note}");
    }
    Ok(())
}

/// `version` of `[workspace.package]` in the root manifest.
fn workspace_version(manifest: &str) -> Result<String> {
    let mut in_section = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line == "[workspace.package]";
        } else if in_section
            && let Some(value) = line
                .strip_prefix("version")
                .and_then(|rest| rest.trim_start().strip_prefix('='))
        {
            return quoted(value).context("[workspace.package] version is not a string");
        }
    }
    bail!("Cargo.toml has no [workspace.package] version")
}

/// The first double-quoted string in `text`.
fn quoted(text: &str) -> Option<String> {
    let start = text.find('"')? + 1;
    let end = start + text.get(start..)?.find('"')?;
    text.get(start..end).map(str::to_owned)
}

fn is_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
}

/// The top-level `version:` of a CITATION.cff file.
fn cff_version(cff: &str) -> Option<String> {
    cff.lines()
        .find_map(|line| line.strip_prefix("version:"))
        .and_then(|value| quoted(value).or_else(|| Some(value.trim().to_owned())))
}

/// The `keywords:` list of a CITATION.cff file, in order.
fn cff_keywords(cff: &str) -> Vec<String> {
    let mut keywords = Vec::new();
    let mut in_list = false;
    for line in cff.lines() {
        if line.starts_with("keywords:") {
            in_list = true;
        } else if in_list {
            match line.trim_start().strip_prefix("- ") {
                Some(keyword) if line.starts_with(' ') => {
                    keywords.push(keyword.trim().trim_matches('"').to_owned());
                }
                _ if line.trim_start().starts_with('#') => {}
                _ => in_list = false,
            }
        }
    }
    keywords
}

fn problems(cff: &str, zenodo: &str, version: &str) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    match cff_version(cff) {
        Some(found) if found == version => {}
        Some(found) => problems.push(format!(
            "CITATION.cff names version {found}, the workspace {version}"
        )),
        None => problems.push("CITATION.cff has no version".to_owned()),
    }
    let json: Value = serde_json::from_str(zenodo).context(".zenodo.json is not JSON")?;
    match json.get("version").and_then(Value::as_str) {
        Some(found) if found == version => {}
        Some(found) => problems.push(format!(
            ".zenodo.json names version {found}, the workspace {version}"
        )),
        None => problems.push(".zenodo.json has no version".to_owned()),
    }
    for field in [
        "title",
        "upload_type",
        "description",
        "creators",
        "license",
        "keywords",
    ] {
        if json.get(field).is_none() {
            problems.push(format!(".zenodo.json has no `{field}`"));
        }
    }
    if json.get("upload_type").and_then(Value::as_str) != Some("software") {
        problems.push(".zenodo.json `upload_type` must be `software`".to_owned());
    }
    let zenodo_keywords: Vec<String> = json
        .get("keywords")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    if zenodo_keywords != cff_keywords(cff) {
        problems.push("CITATION.cff and .zenodo.json list different keywords".to_owned());
    }
    Ok(problems)
}

/// Placeholders the owner still fills before the first DOI.
fn todos(cff: &str, zenodo: &str) -> Vec<String> {
    let mut notes = Vec::new();
    if cff.lines().any(|line| line.contains("TODO")) {
        notes.push("CITATION.cff still has TODO placeholders (author names, ORCID)".to_owned());
    }
    let creators_without_orcid = serde_json::from_str::<Value>(zenodo)
        .ok()
        .and_then(|json| json.get("creators").and_then(Value::as_array).cloned())
        .is_some_and(|creators| creators.iter().any(|c| c.get("orcid").is_none()));
    if creators_without_orcid {
        notes.push(".zenodo.json has creators without an ORCID".to_owned());
    }
    notes
}

fn set_cff(cff: &str, version: &str, date: &str) -> Result<String> {
    let mut out = String::with_capacity(cff.len() + 32);
    let mut dated = false;
    let mut versioned = false;
    for line in cff.lines() {
        if line.starts_with("version:") {
            let _ = write!(out, "version: \"{version}\"");
            versioned = true;
        } else if line.starts_with("date-released:") || line.starts_with("# date-released:") {
            let _ = write!(out, "date-released: \"{date}\"");
            dated = true;
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    if !versioned {
        bail!("CITATION.cff has no `version:` line");
    }
    if !dated {
        let _ = writeln!(out, "date-released: \"{date}\"");
    }
    Ok(out)
}

fn set_zenodo(zenodo: &str, version: &str, date: &str) -> Result<String> {
    let mut json: Value = serde_json::from_str(zenodo).context(".zenodo.json is not JSON")?;
    let object = json
        .as_object_mut()
        .context(".zenodo.json is not a JSON object")?;
    object.insert("version".to_owned(), Value::from(version));
    object.insert("publication_date".to_owned(), Value::from(date));
    let mut text = serde_json::to_string_pretty(&json)?;
    text.push('\n');
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CFF: &str = "cff-version: 1.2.0\ntitle: SONARITHM\nauthors:\n  # TODO: names\n  - alias: \"someone\"\nkeywords:\n  - sonification\n  - Rust\nlicense:\n  - MIT\nversion: \"0.0.0\"\n# date-released: omitted until the first release\n";
    const ZENODO: &str = r#"{ "title": "SONARITHM", "upload_type": "software", "description": "d",
        "creators": [{ "name": "someone" }], "license": "MIT",
        "keywords": ["sonification", "Rust"], "version": "0.0.0" }"#;

    #[test]
    fn the_workspace_version_is_read_from_its_section() {
        let manifest = "[package]\nversion = \"9.9.9\"\n[workspace.package]\n# one version\nversion = \"0.1.0-alpha\"\nedition = \"2024\"\n";
        assert_eq!(workspace_version(manifest).unwrap(), "0.1.0-alpha");
        assert!(workspace_version("[workspace]\nmembers = []\n").is_err());
    }

    #[test]
    fn matching_files_pass_and_differences_are_named() {
        assert_eq!(
            problems(CFF, ZENODO, "0.0.0").unwrap(),
            Vec::<String>::new()
        );
        let stale = problems(CFF, ZENODO, "0.1.0-alpha").unwrap();
        assert_eq!(stale.len(), 2, "{stale:?}");
        let other_keywords = ZENODO.replace("\"Rust\"", "\"Python\"");
        assert!(
            problems(CFF, &other_keywords, "0.0.0")
                .unwrap()
                .iter()
                .any(|p| p.contains("keywords"))
        );
        assert_eq!(cff_keywords(CFF), ["sonification", "Rust"]);
    }

    #[test]
    fn a_release_writes_version_and_date_into_both_files() {
        let cff = set_cff(CFF, "0.1.0-alpha", "2026-10-22").unwrap();
        assert!(cff.contains("version: \"0.1.0-alpha\"\n"));
        assert!(cff.contains("date-released: \"2026-10-22\"\n"));
        assert!(!cff.contains("# date-released"));
        let zenodo = set_zenodo(ZENODO, "0.1.0-alpha", "2026-10-22").unwrap();
        assert_eq!(
            problems(&cff, &zenodo, "0.1.0-alpha").unwrap(),
            Vec::<String>::new()
        );
        assert!(zenodo.contains("\"publication_date\": \"2026-10-22\""));
        assert!(is_date("2026-10-22") && !is_date("2026-1-22") && !is_date("22-10-2026"));
    }

    #[test]
    fn placeholders_are_noted() {
        let notes = todos(CFF, ZENODO);
        assert_eq!(notes.len(), 2, "{notes:?}");
    }
}

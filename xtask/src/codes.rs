// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask codes`: checks every diagnostic code table and writes the registry page
//! `docs/reference/error-codes.md` (12 §1, coding-standards §4.2).
//!
//! Each table is already checked at compile time in its crate (ascending, well-formed, no
//! duplicates). This task adds the checks that need all tables at once: one table per area
//! and no code in two tables.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Context as _, Result, bail};
use sonarithm_core::codes::{CodeStatus, CodeTable, validate};

/// Where the registry page is written, relative to the workspace root.
pub(crate) const OUTPUT: &str = "docs/reference/error-codes.md";

/// Every code table in the workspace. Add a crate's table here when it gets one.
fn tables() -> Vec<CodeTable> {
    vec![
        sonarithm_spec::codes::CODES,
        sonarithm_data::codes::CODES,
        sonarithm_compiler::codes::CODES,
        sonarithm_render::codes::CODES,
    ]
}

/// Checks the tables and writes the registry page.
pub(crate) fn run(root: &Path) -> Result<()> {
    let tables = tables();
    check(&tables)?;
    let path = root.join(OUTPUT);
    let page = render(&tables);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    std::fs::write(&path, page).with_context(|| format!("writing {}", path.display()))?;
    let count: usize = tables.iter().map(|table| table.codes.len()).sum();
    println!(
        "codes: {count} codes in {} table(s) -> {OUTPUT}",
        tables.len()
    );
    Ok(())
}

/// Checks each table, then that areas and codes are unique across tables.
fn check(tables: &[CodeTable]) -> Result<()> {
    let mut areas = BTreeMap::new();
    let mut codes: BTreeMap<&str, &str> = BTreeMap::new();
    for table in tables {
        if let Err(err) = validate(table) {
            bail!("{} ({:?}): {err}", table.owner, table.area);
        }
        if let Some(other) = areas.insert(table.area, table.owner) {
            bail!(
                "{} and {other} both declare a table for {:?}; an area has one owner",
                table.owner,
                table.area
            );
        }
        for info in table.codes {
            if let Some(other) = codes.insert(info.code, table.owner) {
                bail!(
                    "{} is declared by both {other} and {}",
                    info.code,
                    table.owner
                );
            }
        }
    }
    Ok(())
}

/// Renders the registry page. The output depends only on the tables, so `codegen` can
/// diff it in CI.
fn render(tables: &[CodeTable]) -> String {
    let mut sorted: Vec<&CodeTable> = tables.iter().collect();
    sorted.sort_by_key(|table| table.area.prefix());
    let mut page = String::from(
        "# Diagnostic code registry\n\n\
         > **Generated** by `cargo xtask codes` from the code tables in the crates; do not edit \
         this file. To add or retire a code, change the table in the owning crate and run \
         `cargo xtask codegen`.\n\
         > **Related:** [coding-standards §4.2](../engineering/coding-standards.md#42-diagnostic-codes) \
         · [12 §1](../architecture/12-cross-cutting-concerns.md#1-error-handling-and-diagnostics) \
         · [05 §7](../architecture/05-compiler-and-score-ir.md#7-diagnostics)\n\n\
         Every diagnostic carries a stable code `SNR-<AREA>-<NNN>`. A code, once released, is \
         never reused or renumbered; retired codes stay listed. Severity is the default; \
         settings such as `--deny-warnings` can raise it.\n",
    );
    for table in sorted {
        let area = table.area.prefix().trim_end_matches('-');
        let _ = write!(
            page,
            "\n## {area}\n\nOwner: `{}`.\n\n| Code | Severity | Meaning | How to fix | Rules |\n|---|---|---|---|---|\n",
            table.owner
        );
        for info in table.codes {
            let meaning = match info.status {
                CodeStatus::Active => info.summary.to_owned(),
                CodeStatus::Retired => format!("RETIRED: {}", info.summary),
            };
            let rules = if info.rules.is_empty() {
                "—".to_owned()
            } else {
                info.rules.join(", ")
            };
            let _ = writeln!(
                page,
                "| `{}` | {} | {meaning} | {} | {rules} |",
                info.code, info.severity, info.help
            );
        }
    }
    page
}

#[cfg(test)]
mod tests {
    use sonarithm_core::codes::{Area, CodeInfo};
    use sonarithm_core::diagnostic::Severity;

    use super::*;

    const fn info(code: &'static str) -> CodeInfo {
        CodeInfo {
            code,
            severity: Severity::Warning,
            summary: "Test",
            help: "Fix it.",
            rules: &["SDR-19"],
            status: CodeStatus::Active,
        }
    }

    fn table(area: Area, owner: &'static str, codes: &'static [CodeInfo]) -> CodeTable {
        CodeTable { area, owner, codes }
    }

    #[test]
    fn workspace_tables_are_consistent() {
        check(&tables()).unwrap();
    }

    #[test]
    fn two_tables_for_one_area_fail() {
        const A: &[CodeInfo] = &[info("SNR-COMP-001")];
        const B: &[CodeInfo] = &[info("SNR-COMP-002")];
        let err = check(&[table(Area::Comp, "a", A), table(Area::Comp, "b", B)]).unwrap_err();
        assert!(err.to_string().contains("both declare"), "{err}");
    }

    #[test]
    fn an_invalid_table_fails() {
        const DUPLICATE: &[CodeInfo] = &[info("SNR-COMP-001"), info("SNR-COMP-001")];
        let err = check(&[table(Area::Comp, "a", DUPLICATE)]).unwrap_err();
        assert!(err.to_string().contains("duplicate"), "{err}");
    }

    #[test]
    fn the_page_lists_codes_by_area() {
        const CODES: &[CodeInfo] = &[info("SNR-COMP-001")];
        let page = render(&[table(Area::Comp, "sonarithm-compiler", CODES)]);
        assert!(page.contains("## SNR-COMP\n\nOwner: `sonarithm-compiler`."));
        assert!(page.contains("| `SNR-COMP-001` | warning | Test | Fix it. | SDR-19 |"));
    }
}

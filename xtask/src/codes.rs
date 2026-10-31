// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask codes`: checks every diagnostic code table and writes the registry page
//! `docs/reference/error-codes.md` (12 §1, coding-standards §4.2).
//!
//! Each table is already checked at compile time in its crate (ascending, well-formed, no
//! duplicates). This task adds the checks that need all tables at once: no code in two
//! tables, and one table per area, except `SNR-ENG`, which the engine crates share in
//! blocks of 100 numbers, one block per engine (coding-standards §4.2).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::ops::RangeInclusive;
use std::path::Path;

use anyhow::{Context as _, Result, bail};
use sonarithm_core::codes::{Area, CodeStatus, CodeTable, validate};

use crate::check_deps::{ENGINES, engine_of};

/// Where the registry page is written, relative to the workspace root.
pub(crate) const OUTPUT: &str = "docs/reference/error-codes.md";

/// Every code table in the workspace. Add a crate's table here when it gets one.
fn tables() -> Vec<CodeTable> {
    vec![
        sonarithm_spec::codes::CODES,
        sonarithm_data::codes::CODES,
        sonarithm_compiler::codes::CODES,
        sonarithm_render::codes::CODES,
        sonarithm_app::codes::CODES,
        sonarithm_numisona::codes::CODES,
        sonarithm_audaxis::codes::CODES,
        sonarithm_echometric::codes::CODES,
        sonarithm_sonotrace::codes::CODES,
        sonarithm_plotsona::codes::CODES,
        sonarithm_curvasona::codes::CODES,
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

/// The `SNR-ENG` numbers an engine crate may use: block `i` of 100 for the `i`-th engine
/// of [`ENGINES`] (the table in coding-standards §4.2), so NUMISONA has 001–099, AUDAXIS 100–199 and ECHOMETRIC 600–699. An engine's
/// crates share its block. `None` for a crate that is not an engine's.
fn eng_block(owner: &str) -> Option<RangeInclusive<u16>> {
    let engine = engine_of(owner)?;
    let index = ENGINES.iter().position(|known| *known == engine)?;
    let start = u16::try_from(index).ok()? * 100;
    Some(start.max(1)..=start + 99)
}

/// Checks that every code of an `SNR-ENG` table lies in its engine's block.
fn check_eng_block(table: &CodeTable) -> Result<()> {
    let Some(block) = eng_block(table.owner) else {
        bail!(
            "{} declares SNR-ENG codes, but SNR-ENG belongs to engine crates only",
            table.owner
        );
    };
    for info in table.codes {
        let number = info
            .code
            .strip_prefix(Area::Eng.prefix())
            .and_then(|digits| digits.parse::<u16>().ok());
        if !number.is_some_and(|number| block.contains(&number)) {
            bail!(
                "{} is outside the block of {}: SNR-ENG-{:03} to SNR-ENG-{:03} (coding-standards §4.2)",
                info.code,
                table.owner,
                block.start(),
                block.end()
            );
        }
    }
    Ok(())
}

/// Checks each table, then that areas (except the shared `SNR-ENG`) and codes are unique
/// across tables.
fn check(tables: &[CodeTable]) -> Result<()> {
    let mut areas = BTreeMap::new();
    let mut codes: BTreeMap<&str, &str> = BTreeMap::new();
    for table in tables {
        if let Err(err) = validate(table) {
            bail!("{} ({:?}): {err}", table.owner, table.area);
        }
        if table.area == Area::Eng {
            check_eng_block(table)?;
        } else if let Some(other) = areas.insert(table.area, table.owner) {
            bail!(
                "{} and {other} both declare a table for {:?}; an area has one owner (only SNR-ENG is shared, in per-engine blocks)",
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
    sorted.sort_by_key(|table| {
        (
            table.area.prefix(),
            table.codes.first().map(|info| info.code),
        )
    });
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
    let mut previous = None;
    for table in sorted {
        let area = table.area.prefix().trim_end_matches('-');
        if previous != Some(area) {
            let _ = write!(page, "\n## {area}\n");
            previous = Some(area);
        }
        let block = match (table.area, eng_block(table.owner)) {
            (Area::Eng, Some(block)) => format!(
                " Numbers {area}-{:03} to {area}-{:03}.",
                block.start(),
                block.end()
            ),
            _ => String::new(),
        };
        let _ = write!(
            page,
            "\nOwner: `{}`.{block}\n\n| Code | Severity | Meaning | How to fix | Rules |\n|---|---|---|---|---|\n",
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
    fn engines_share_snr_eng_in_their_own_blocks() {
        const NUMISONA: &[CodeInfo] = &[info("SNR-ENG-001"), info("SNR-ENG-099")];
        const ECHOMETRIC: &[CodeInfo] = &[info("SNR-ENG-600"), info("SNR-ENG-699")];
        const ECHOMETRIC_PART: &[CodeInfo] = &[info("SNR-ENG-650")];
        check(&[
            table(Area::Eng, "sonarithm-numisona", NUMISONA),
            table(Area::Eng, "sonarithm-echometric", ECHOMETRIC),
            table(Area::Eng, "sonarithm-echometric-extra", ECHOMETRIC_PART),
        ])
        .unwrap();
    }

    #[test]
    fn an_engine_code_outside_its_block_fails() {
        const STRAY: &[CodeInfo] = &[info("SNR-ENG-012")];
        let err = check(&[table(Area::Eng, "sonarithm-echometric", STRAY)]).unwrap_err();
        assert!(
            err.to_string().contains("SNR-ENG-600 to SNR-ENG-699"),
            "{err}"
        );
    }

    #[test]
    fn snr_eng_codes_outside_an_engine_crate_fail() {
        const CODES: &[CodeInfo] = &[info("SNR-ENG-001")];
        let err = check(&[table(Area::Eng, "sonarithm-compiler", CODES)]).unwrap_err();
        assert!(err.to_string().contains("engine crates only"), "{err}");
    }

    #[test]
    fn one_engine_code_in_two_crates_fails() {
        const A: &[CodeInfo] = &[info("SNR-ENG-650")];
        let err = check(&[
            table(Area::Eng, "sonarithm-echometric", A),
            table(Area::Eng, "sonarithm-echometric-extra", A),
        ])
        .unwrap_err();
        assert!(err.to_string().contains("declared by both"), "{err}");
    }

    #[test]
    fn engine_blocks_follow_the_engine_order() {
        assert_eq!(eng_block("sonarithm-numisona"), Some(1..=99));
        assert_eq!(eng_block("sonarithm-audaxis"), Some(100..=199));
        assert_eq!(eng_block("sonarithm-echometric"), Some(600..=699));
        assert_eq!(eng_block("sonarithm-resonum"), Some(800..=899));
        assert_eq!(eng_block("sonarithm-render"), None);
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
        assert!(page.contains("## SNR-COMP\n\nOwner: `sonarithm-compiler`.\n"));
        assert!(page.contains("| `SNR-COMP-001` | warning | Test | Fix it. | SDR-19 |"));
    }

    #[test]
    fn the_page_gives_each_engine_its_block_under_one_heading() {
        const NUMISONA: &[CodeInfo] = &[info("SNR-ENG-001")];
        const ECHOMETRIC: &[CodeInfo] = &[info("SNR-ENG-600")];
        let page = render(&[
            table(Area::Eng, "sonarithm-echometric", ECHOMETRIC),
            table(Area::Eng, "sonarithm-numisona", NUMISONA),
        ]);
        assert_eq!(page.matches("## SNR-ENG").count(), 1);
        let numisona = page
            .find("Owner: `sonarithm-numisona`. Numbers SNR-ENG-001 to SNR-ENG-099.")
            .unwrap();
        let echometric = page
            .find("Owner: `sonarithm-echometric`. Numbers SNR-ENG-600 to SNR-ENG-699.")
            .unwrap();
        assert!(numisona < echometric);
    }
}

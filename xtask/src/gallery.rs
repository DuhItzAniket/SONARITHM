// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask gallery`: writes the example datasets DS-001 to DS-005
//! (docs/research-paper/datasets.md) as CSV under `examples/data`, from NUMISONA's seeded
//! generators (`sonarithm_numisona::examples`), so the gallery's data is reproducible from
//! code and identical on every platform. `--check` (CI) compares instead of writing.

use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Context as _, Result, bail};
use arrow_array::cast::AsArray as _;
use arrow_array::types::{Float64Type, Int32Type, Int64Type, UInt8Type, UInt16Type, UInt32Type};
use arrow_array::{Array, ArrayRef};
use arrow_schema::DataType;
use sonarithm_data::{DataBundle, DataError};
use sonarithm_numisona::examples;

type Generator = fn() -> Result<DataBundle, DataError>;

/// The gallery datasets: file, generator, columns in order.
const DATASETS: [(&str, Generator, &[&str]); 5] = [
    (
        "ds-001-temperature.csv",
        examples::temperature,
        &["day", "temp"],
    ),
    ("ds-002-prices.csv", examples::prices, &["week", "price"]),
    (
        "ds-003-regions.csv",
        examples::regions,
        &["month", "region", "rain"],
    ),
    ("ds-004-fruit.csv", examples::fruit, &["fruit", "sold"]),
    ("ds-005-signal.csv", examples::signal, &["t", "amplitude"]),
];

/// Runs the task.
pub(crate) fn run(root: &Path, options: &[String]) -> Result<()> {
    let check = match options {
        [] => false,
        [flag] if flag == "--check" => true,
        _ => bail!("expected no option or `--check`"),
    };
    let dir = root.join("examples").join("data");
    let mut stale = Vec::new();
    for (file, generate, columns) in DATASETS {
        let bundle = generate().with_context(|| format!("generating {file}"))?;
        let text = csv(&bundle, columns).with_context(|| format!("writing {file}"))?;
        let path = dir.join(file);
        if check {
            let current = std::fs::read_to_string(&path).unwrap_or_default();
            if current.replace("\r\n", "\n") != text {
                stale.push(file);
            }
        } else {
            std::fs::create_dir_all(&dir).context("creating examples/data")?;
            std::fs::write(&path, &text).with_context(|| format!("writing {file}"))?;
        }
    }
    if !stale.is_empty() {
        bail!(
            "examples/data is out of date: {}. Run `cargo xtask gallery` and state the reason in the PR.",
            stale.join(", ")
        );
    }
    println!(
        "gallery: {} datasets {} examples/data",
        DATASETS.len(),
        if check { "match" } else { "written to" }
    );
    Ok(())
}

/// The primary table's `columns` as CSV with a header row and `\n` line ends.
fn csv(bundle: &DataBundle, columns: &[&str]) -> Result<String> {
    let table = bundle.primary().context("the dataset has no table")?;
    let arrays: Vec<ArrayRef> = columns
        .iter()
        .map(|name| {
            table
                .column(name)
                .with_context(|| format!("no column `{name}`"))
        })
        .collect::<Result<_>>()?;
    let mut out = columns.join(",");
    out.push('\n');
    let rows = arrays.first().map_or(0, Array::len);
    for row in 0..rows {
        let cells: Vec<String> = arrays
            .iter()
            .map(|array| cell(array, row))
            .collect::<Result<_>>()?;
        let _ = writeln!(out, "{}", cells.join(","));
    }
    Ok(out)
}

/// One value as CSV text; text is quoted when it needs to be.
fn cell(array: &ArrayRef, row: usize) -> Result<String> {
    if array.is_null(row) {
        return Ok(String::new());
    }
    let text = match array.data_type() {
        DataType::Int64 => array.as_primitive::<Int64Type>().value(row).to_string(),
        DataType::Float64 => array.as_primitive::<Float64Type>().value(row).to_string(),
        DataType::Utf8 => quote(array.as_string::<i32>().value(row)),
        DataType::LargeUtf8 => quote(array.as_string::<i64>().value(row)),
        DataType::Dictionary(key, _) => {
            let (values, index) = match key.as_ref() {
                DataType::UInt8 => {
                    let d = array.as_dictionary::<UInt8Type>();
                    (d.values(), usize::from(d.keys().value(row)))
                }
                DataType::UInt16 => {
                    let d = array.as_dictionary::<UInt16Type>();
                    (d.values(), usize::from(d.keys().value(row)))
                }
                DataType::UInt32 => {
                    let d = array.as_dictionary::<UInt32Type>();
                    (d.values(), usize::try_from(d.keys().value(row))?)
                }
                DataType::Int32 => {
                    let d = array.as_dictionary::<Int32Type>();
                    (d.values(), usize::try_from(d.keys().value(row))?)
                }
                other => bail!("dictionary keys of type {other} are not written"),
            };
            cell(values, index)?
        }
        other => bail!("columns of type {other} are not written"),
    };
    Ok(text)
}

fn quote(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_dataset_writes_its_columns() {
        for (file, generate, columns) in DATASETS {
            let text = csv(&generate().unwrap(), columns).unwrap();
            let mut lines = text.lines();
            assert_eq!(lines.next().unwrap(), columns.join(","), "{file}");
            assert!(
                lines.all(|line| line.split(',').count() == columns.len()),
                "{file}"
            );
        }
    }

    #[test]
    fn text_is_quoted_only_when_needed() {
        assert_eq!(quote("north"), "north");
        assert_eq!(quote("a,b"), "\"a,b\"");
        assert_eq!(quote("say \"hi\""), "\"say \"\"hi\"\"\"");
    }
}

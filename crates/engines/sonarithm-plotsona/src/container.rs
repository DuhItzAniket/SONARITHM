// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Office Open XML packages (XLSX, PPTX, DOCX) read under the container budgets (12 §6).
//!
//! Before inflating a part, PLOTSONA checks the entry count, the part's declared size and
//! the running total; while inflating, it stops at the part and total budgets and at the
//! compression-ratio budget, whatever the headers claim. Encrypted entries are refused.
//! Nothing is written to disk, so entry names cannot escape a directory.

use std::io::{Cursor, Read};

use sonarithm_core::Diagnostic;
use zip::ZipArchive;
use zip::result::ZipError;

use crate::budget::{Budget, Budgets, Work, len64, over};
use crate::codes;

/// How many bytes are inflated between two budget checks.
const CHUNK: usize = 64 << 10;

/// An open package and its decompression totals.
pub(crate) struct Package<'a> {
    archive: ZipArchive<Cursor<&'a [u8]>>,
    budgets: Budgets,
    inflated: u64,
}

impl<'a> Package<'a> {
    /// Opens `bytes` as a zip package.
    ///
    /// # Errors
    ///
    /// `SNR-ENG-302` above the entry budget, `SNR-ENG-307` if the central directory cannot
    /// be read.
    pub(crate) fn open(bytes: &'a [u8], budgets: &Budgets) -> Result<Self, Box<Diagnostic>> {
        let archive = ZipArchive::new(Cursor::new(bytes)).map_err(zip_error)?;
        let entries = len64(archive.len());
        if entries > budgets.max_entries {
            return Err(over(Budget::Entries, entries, budgets.max_entries));
        }
        Ok(Self {
            archive,
            budgets: *budgets,
            inflated: 0,
        })
    }

    /// Whether the package has a part named `name`.
    pub(crate) fn contains(&self, name: &str) -> bool {
        self.archive.index_for_name(name).is_some()
    }

    /// The part names, sorted (the central directory's order is the writer's choice).
    pub(crate) fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .archive
            .file_names()
            .filter_map(Result::ok)
            .map(std::borrow::Cow::into_owned)
            .collect();
        names.sort_unstable();
        names
    }

    /// Inflates the part `name`, or `None` when the package has no such part.
    ///
    /// # Errors
    ///
    /// `SNR-ENG-311` for an encrypted part, `SNR-ENG-302` past a decompression budget,
    /// `SNR-ENG-305` past the work budget and `SNR-ENG-307` for a damaged part.
    pub(crate) fn read(
        &mut self,
        name: &str,
        work: &mut Work,
    ) -> Result<Option<Vec<u8>>, Box<Diagnostic>> {
        let Some(index) = self.archive.index_for_name(name) else {
            return Ok(None);
        };
        let budgets = self.budgets;
        let mut file = self.archive.by_index(index).map_err(zip_error)?;
        if file.encrypted() {
            return Err(Box::new(Diagnostic::new(
                &codes::ENCRYPTED,
                format!("The part `{name}` is encrypted."),
            )));
        }
        let declared = file.size();
        if declared > budgets.max_part_bytes {
            return Err(over(Budget::PartBytes, declared, budgets.max_part_bytes));
        }
        let compressed = file.compressed_size().max(1);
        let capacity = usize::try_from(declared.min(budgets.max_part_bytes))
            .unwrap_or(0)
            .min(CHUNK * 16);
        let mut out = Vec::with_capacity(capacity);
        let mut chunk = vec![0_u8; CHUNK];
        loop {
            let read = file.read(&mut chunk).map_err(|err| {
                Box::new(Diagnostic::new(
                    &codes::MALFORMED,
                    format!("The part `{name}` cannot be decompressed: {err}."),
                ))
            })?;
            if read == 0 {
                break;
            }
            work.spend(len64(read))?;
            let part = len64(out.len()).saturating_add(len64(read));
            if part > budgets.max_part_bytes {
                return Err(over(Budget::PartBytes, part, budgets.max_part_bytes));
            }
            let total = self.inflated.saturating_add(len64(read));
            if total > budgets.max_inflated_bytes {
                return Err(over(
                    Budget::InflatedBytes,
                    total,
                    budgets.max_inflated_bytes,
                ));
            }
            if part > Budgets::RATIO_FLOOR && part / compressed > budgets.max_ratio {
                return Err(over(Budget::Ratio, part / compressed, budgets.max_ratio));
            }
            self.inflated = total;
            out.extend_from_slice(chunk.get(..read).unwrap_or_default());
        }
        Ok(Some(out))
    }
}

fn zip_error(err: ZipError) -> Box<Diagnostic> {
    let message = match err {
        ZipError::UnsupportedArchive(what) => {
            format!("The package uses a zip feature PLOTSONA does not read: {what}.")
        }
        other => format!("The package is not a readable zip archive: {other}."),
    };
    Box::new(Diagnostic::new(&codes::MALFORMED, message))
}

#[cfg(test)]
pub(crate) mod tests {
    use std::io::Write;

    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    use super::*;

    /// A zip package with Deflate-compressed `parts`.
    pub(crate) fn package(parts: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (name, body) in parts {
            writer.start_file(*name, options).expect("start");
            writer.write_all(body).expect("write");
        }
        writer.finish().expect("finish").into_inner()
    }

    #[test]
    fn reads_parts_and_reports_absent_ones() {
        let bytes = package(&[("a.xml", b"<a/>"), ("b/c.xml", b"<c/>")]);
        let mut package = Package::open(&bytes, &Budgets::DEFAULT).expect("open");
        let mut work = Work::new(1 << 20);
        assert_eq!(package.names(), ["a.xml", "b/c.xml"]);
        assert!(package.contains("b/c.xml"));
        assert_eq!(
            package.read("a.xml", &mut work).expect("read").as_deref(),
            Some(&b"<a/>"[..])
        );
        assert_eq!(package.read("missing.xml", &mut work).expect("read"), None);
    }

    #[test]
    fn a_zip_bomb_stops_at_the_ratio_budget() {
        // 8 MiB of zeros deflates to about 8 KiB: a ratio near 1000:1.
        let zeros = vec![0_u8; 8 << 20];
        let bytes = package(&[("xl/charts/chart1.xml", &zeros)]);
        assert!(bytes.len() < 1 << 20, "the bomb is small on disk");
        let mut package = Package::open(&bytes, &Budgets::DEFAULT).expect("open");
        let mut work = Work::new(u64::MAX);
        let err = package
            .read("xl/charts/chart1.xml", &mut work)
            .expect_err("bomb");
        assert_eq!(err.code.as_str(), "SNR-ENG-302");
        assert!(err.message.contains("compression ratio"), "{}", err.message);
        // The ratio is checked against the whole entry's compressed size, so it trips once
        // the inflated bytes pass 200 times that: at about 1.6 MiB here, not 8 MiB.
        assert!(
            work.spent() < 2 << 20,
            "stopped long before the end, at {} bytes",
            work.spent()
        );
    }

    #[test]
    fn part_entry_and_total_budgets() {
        let body = vec![b'x'; 10_000];
        let bytes = package(&[("a", &body), ("b", &body)]);
        let mut budgets = Budgets::DEFAULT;
        budgets.max_entries = 1;
        assert_eq!(
            Package::open(&bytes, &budgets)
                .err()
                .expect("entries")
                .code
                .as_str(),
            "SNR-ENG-302"
        );
        let mut budgets = Budgets::DEFAULT;
        budgets.max_part_bytes = 5_000;
        let mut package = Package::open(&bytes, &budgets).expect("open");
        let mut work = Work::new(u64::MAX);
        assert_eq!(
            package
                .read("a", &mut work)
                .expect_err("part")
                .code
                .as_str(),
            "SNR-ENG-302"
        );
        let mut budgets = Budgets::DEFAULT;
        budgets.max_inflated_bytes = 15_000;
        let mut package = Package::open(&bytes, &budgets).expect("open");
        assert!(package.read("a", &mut work).is_ok());
        assert_eq!(
            package
                .read("b", &mut work)
                .expect_err("total")
                .code
                .as_str(),
            "SNR-ENG-302"
        );
    }

    #[test]
    fn not_a_zip_is_malformed() {
        assert_eq!(
            Package::open(b"PK\x03\x04 truncated", &Budgets::DEFAULT)
                .err()
                .expect("malformed")
                .code
                .as_str(),
            "SNR-ENG-307"
        );
    }
}

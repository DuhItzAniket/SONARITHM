// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Checks shared by the sonarithm-data fuzz targets.

use sonarithm_data::source::write_ipc;
use sonarithm_data::{
    Format, Limits, Loaded, ReadOptions, Table, read_bytes, table_fingerprint,
};

/// Lets the data layer contain panics of third-party decoders (arrow-ipc, parquet,
/// calamine) as it does in production: libfuzzer-sys installs a hook that aborts on every
/// panic, even one that `sonarithm_data` catches and turns into an error. A panic that
/// escapes `read_bytes` still reaches libfuzzer-sys's own `catch_unwind`, which aborts, so
/// the fuzzer still finds it.
pub fn contain_decoder_panics() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| std::panic::set_hook(Box::new(|_| {})));
}

/// Options with limits small enough that no input can exhaust the fuzzer's memory.
pub fn options() -> ReadOptions {
    let mut limits = Limits::DEFAULT;
    limits.max_bytes = 4 << 20;
    limits.max_rows = 200_000;
    limits.max_fields = 512;
    limits.max_cells = 2_000_000;
    ReadOptions::new().with_limits(limits)
}

/// A loaded table's fingerprint. Every loaded table has one: the readers drop the columns
/// RFC-0008 §3 does not encode, such as intervals (#511), so an error is a finding.
fn fingerprint(table: &Table) -> String {
    match table_fingerprint(table) {
        Ok(digest) => digest.to_string(),
        Err(error) => panic!("a loaded table has a fingerprint: {error}"),
    }
}

/// What every successful read must satisfy: the table has a fingerprint, survives an Arrow
/// IPC round trip unchanged, and keeps its fingerprint.
pub fn check(loaded: &Loaded) {
    let Some(table) = loaded.bundle.primary() else {
        return;
    };
    let before = fingerprint(table);
    let bytes = write_ipc(table).expect("a loaded table can be written");
    let again = read_bytes(
        &bytes,
        Format::Arrow,
        "fuzz.arrow",
        &options().without_inference(),
    )
    .expect("a written table reads back");
    let reread = again.bundle.primary().expect("one table");
    assert_eq!(reread.schema(), table.schema(), "the schema round-trips");
    assert_eq!(
        reread.to_batch().expect("batch"),
        table.to_batch().expect("batch"),
        "the data round-trips"
    );
    assert_eq!(fingerprint(reread), before, "the fingerprint is stable");
}

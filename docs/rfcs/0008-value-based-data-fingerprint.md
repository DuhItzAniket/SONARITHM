# RFC 0008: A value-based data fingerprint (scheme v2)

> **Status:** **Accepted** on 2026-11-26 by the coordinator under the owner's delegation of contract approvals; the 7-day final comment period was waived under the same delegation. The three questions were answered as recorded in [Resolved questions](#resolved-questions): linked data saved under scheme 1 stays unavailable until the user checks and relinks it (§6), the scheme is a separate header field, and Utf8View collapses into `string` after its own fix lands (§10). **Author(s):** project lead (drafted by an AI coding agent, Claude Code, for review by the coordinator under the owner's delegation). **Decision:** an amendment to ADR-0013, with the implementation.
> **Related:** [RFC process](README.md) · [03 §2](../architecture/03-data-model.md#2-core-structures) · [05 §4](../architecture/05-compiler-and-score-ir.md#4-score-ir) · [ADR-0013](../decisions/0013-determinism-and-provenance.md) · [RFC-0002](0002-project-format-v1.md) · [RFC-0007 §4](0007-function-cues-and-audification.md) · F-222, F-223 · [#469](https://github.com/DuhItzAniket/SONARITHM/issues/469), [#476](https://github.com/DuhItzAniket/SONARITHM/issues/476)

| Field | Value |
|---|---|
| RFC | 0008 |
| Start date | 2026-11-26 |
| RFC PR | (this PR) |
| Affects | Score IR (a header field and the IR version); persisted formats (render manifest v3, project format 1.2); the data layer's fingerprint |
| Phase | maintenance, after the current queue (#477, CURVASONA's re-record, Lane U's toggles); before the next arrow minor upgrade |
| Related ADR(s) | An amendment to [ADR-0013](../decisions/0013-determinism-and-provenance.md), written with the implementation |
| Supersedes | none |

## Summary

The data fingerprint becomes SHA-256 over an encoding SONARITHM defines (the data's logical values, field names, logical types and `snr.*` metadata), instead of over the bytes `arrow_ipc::writer::StreamWriter` writes. Every record that stores a fingerprint also says which scheme computed it: the Score header, render manifests and project manifests. Upgrading Arrow then moves no fingerprint, and a record made under the old scheme is recognised as such instead of looking like changed data.

## Motivation

- **F-222.** Bumping arrow from 60.0 to 60.2, a minor release taken for a security fix (#470), moved every data fingerprint without changing any data. It forced a re-record of 2 pinned digests, 16 insta snapshots, 4 golden Scores and 49 replication records. The current scheme hashes arrow-ipc's encoder output, so every Arrow upgrade can do this again.
- **F-223.** The same move made every earlier project with embedded data unopenable, because the reader treated a fingerprint mismatch as damage. RFC-0002 §13 fixes embedded data, but linked data cannot be fixed under the current format: a linked entry records no hash of the file, so "the file changed" and "the fingerprint scheme changed" look the same.
- **ADR-0013's claim.** A render manifest or replication record should identify its input data across builds. Under v1, a newer build computes a different fingerprint for the same data, so old records cannot be checked against it.
- **If we do nothing**, each Arrow upgrade costs a re-record PR and leaves users with linked data that reopens as unavailable.

## Guide-level explanation

Nothing changes for spec authors, CLI or Python users, except that fingerprints change once, when this ships. After that, the fingerprint of a dataset depends only on what SONARITHM reads from it: column names and order, logical types, `snr.*` metadata and values. It does not depend on the Arrow version, on whether the data came from CSV, Parquet, pyarrow or polars, on how it was split into batches, or on whether text was stored as `string`, `large_string` or `string_view`.

A render manifest says `"fingerprintScheme": 2` next to the fingerprint. A project opened from an older build re-checks its embedded data and warns once, then records the new fingerprint when saved. A linked file from an older build cannot be checked against the project, so it opens as unavailable, and the user can choose **Check and relink this file** to read it again and confirm it (§6).

## Reference-level explanation

### 1. The invariant

**Equal fingerprints imply equal compile output** for the same spec, seed and engine versions. The encoding therefore covers everything a compile path can read from the data, and leaves out everything it cannot. Each inclusion or exclusion below cites the code it rests on; the implementation PR adds a test for each (§7).

### 2. Table fingerprint input

The input is the concatenation of the following, fed to SHA-256 as a stream:

1. **Tag:** the bytes `sonarithm-data table fingerprint v2` and a NUL byte.
2. **Schema:** a u64 byte length, then RFC 8785 canonical JSON (the form spec hashes use):
   - `fields`, an array in schema order, each element `{ "name", "type", "meta" }`;
   - `type` is the logical type (§3);
   - `meta` holds the field's metadata keys that start with `snr.`, sorted (RFC 8785 sorts them);
   - `meta` at the top level holds the schema's metadata keys that start with `snr.`. CURVASONA reads `snr.function.expr` and `snr.function.variables` there (`sonarithm-curvasona/src/lib.rs:81`).
3. **Row count:** u64, little-endian.
4. **Columns,** in schema order, each preceded by its own byte length as u64. Within a column, rows are encoded in order across all batches: a tag byte (`0` null, `1` valid), then the value bytes of §3 for a valid row only.

**Left out, with the reason:**

- **Non-`snr.*` metadata** (`pandas`, `ARROW:*`, library keys): no compile path reads it, and it varies with the library that produced the data (F-100).
- **Nullability:** `Field::is_nullable` is read in one place, `sonarithm-data/src/infer/semantic.rs:169`, where it is only carried over to rebuild the field (line 185). No decision depends on it. Leaving it out lets a non-nullable pyarrow field and a nullable CSV field with the same values agree.
- **Batch boundaries, offsets, slicing, alignment, padding and bytes under nulls:** layout only.
- **The table's name, ID and generation:** as in v1, they identify a table, not its content.

### 3. Logical types and value bytes

| Logical type (`type`) | Arrow types | Value bytes |
|---|---|---|
| `bool` | Boolean | 1 byte: 0 or 1 |
| `int8` … `int64`, `uint8` … `uint64` | the integer types | fixed width, two's complement, little-endian |
| `float16`, `float32`, `float64` | Float16, Float32, Float64 | the IEEE 754 bit pattern, little-endian (§4) |
| `decimal(p,s)` | Decimal128, Decimal256 | 16 or 32 bytes, little-endian |
| `date32`, `date64` | Date32, Date64 | i32 or i64, little-endian |
| `time(unit)`, `duration(unit)`, `timestamp(unit,tz)` | Time32/64, Duration, Timestamp | i32 or i64, little-endian; `tz` is the time zone string, or absent |
| `string` | Utf8, LargeUtf8, Utf8View | u64 byte length, then the UTF-8 bytes |
| `binary` | Binary, LargeBinary, BinaryView | u64 byte length, then the bytes |
| `fixed_binary(n)` | FixedSizeBinary | n bytes |
| `dictionary<T>` | Dictionary with any key type and values of logical type T | see below |
| `list<T>` | List, LargeList, ListView, LargeListView | u64 element count, then each element as a row (tag byte and value) |
| `fixed_list<T,n>` | FixedSizeList | n elements, each as a row |
| `struct{…}` | Struct | each child, in order, as a row; the child names and types are in `type` |
| `map<K,V>` | Map | as `list<struct{key,value}>` in stored order; `type` records whether it is sorted |
| `union{…}` | Union, sparse or dense | the type id (u8), then the selected child's value as a row; the type ids and children are in `type` |
| the values' type | RunEndEncoded | each row encodes its run's value; run ends are layout |

**Dictionaries are not resolved into plain values.** NUMISONA takes a category column's order from its dictionary: `encode/columns.rs` builds `order` from the dictionary's values, including values no row uses. The semantic inference writes ordinal columns as dictionaries in their inferred order (`infer/semantic.rs`, the `level` test: "low", "Medium", "High"). A dictionary column and a plain column with the same strings can therefore compile differently. The resolution, with the reasons:

- **The logical type stays distinct:** `dictionary<string>`, not `string`.
- **The value encoding** is the dictionary's values as a column of logical type T (all of them, in order, with a u64 count first), then, for each row, the tag byte and the key as a u64 index into those values.
- **Key width is layout:** UInt8 and Int32 keys with the same indices hash equally.
- **Unused values and their order count,** because NUMISONA's order uses them.

**String layouts collapse into one logical type** (`string` for Utf8, LargeUtf8 and Utf8View) only if every compile path treats the three alike. Two places do not today:

- `infer/semantic.rs:293` classifies Utf8, LargeUtf8 and dictionaries as text, but not Utf8View;
- `view.rs:306` reads dictionary values only when they are Utf8 or LargeUtf8.

Both are fixed first, in a small PR of their own with tests (§10): each place accepts Utf8View as it accepts Utf8, and a test checks that the same table in each string layout infers the same field semantics and compiles to the same Score. Only after that PR has landed does the scheme-2 encoder collapse the three layouts into `string`. BinaryView is checked the same way and collapses into `binary` only if every compile path already treats it like Binary.

### 4. Floats are hashed bit-exact

-0.0 and 0.0 hash differently, and so do NaNs that differ in sign or payload. There is no canonicalisation, because compile paths order floats with `f64::total_cmp`. It separates -0.0 from 0.0, and NaNs by sign and payload where a path does not drop them first. Examples: the navigation tree's row order (`sonarithm-audaxis/src/nav/tree.rs:231`, which drops NaN x values before sorting) and the reference tone's value ordering (`sonarithm-audaxis/src/context.rs:418`). Folding them together could give two datasets one fingerprint and different Scores, against §1.

This changes nothing in practice for text input:

- the text readers turn `NaN`, `nan`, `NAN`, `-NaN` and `-nan` into missing values (`infer/text.rs:33`, `Cell::NaN`, reported as `SNR-DATA-033`);
- the transform expressions map NaN results to null (`transform/expr.rs:242`).

NaN values reach a table only from binary input: Parquet, Arrow IPC and data handed over in memory (`read_batches`, Python). v1 hashes the same bits, since IPC writes them raw.

### 5. Bundle fingerprint

SHA-256 over the bytes `sonarithm-data bundle fingerprint v2` and a NUL byte, then each table's 32-byte fingerprint in table order (v1's shape, with a new tag).

### 6. Naming the scheme in every record

The scheme is a number: `1` for the Arrow IPC scheme (the A04-T09 design, as changed by F-222), `2` for this one. A record without it was written under scheme 1.

| Record | Field | Change |
|---|---|---|
| Score header (`sonarithm_core::score::ScoreHeader`) | `data_fingerprint_scheme: u32` beside `data_fingerprint` | A new header field is a snapshot-format change. Scores written with it carry the next IR version (RFC-0007 §4: the lowest version that can read them), and a reader defaults the field to 1 when it is absent |
| Render manifest (`sonarithm-export`, `schemas/`) | `data.fingerprintScheme` | `MANIFEST_VERSION` 3 with a new generated schema; versions 1 and 2 still read, as scheme 1 |
| Project manifest (RFC-0002 §3) | `datasets[].fingerprintScheme`; for linked datasets also `datasets[].bytes` and `datasets[].sha256` of the linked file | Format 1.2, written by every writer from then on. An older 1.x reader ignores the fields with `SNR-APP-035` (RFC-0002 §5) |

**Reading a project under this RFC:**

- **Embedded, scheme 1:** recompute under scheme 2. A difference is expected and is reported as `SNR-APP-093` (RFC-0002 §13).
- **Linked, scheme 1:** the entry records no hash of the file, so this build cannot tell a changed file from the scheme change. A matching row count is not enough to open it: a file of the same length with different values would compile silently, and a listener who does not see the data would hear wrong values with no visual cue that anything differs. The dataset therefore **stays unavailable** with `SNR-APP-034`, and the user can check and relink it (below).
- **Linked, scheme 2:**
  - the file's size and SHA-256 match: open; a fingerprint difference then means only a scheme change, so it is treated as embedded data is (`SNR-APP-093`);
  - they differ: `SNR-APP-034`, as today, and the same check-and-relink action is offered.

**Check and relink this file.** An unavailable linked dataset that this build could read gets an explicit action. The English strings below are the catalogue's source text (every string is externalised); a translation keeps their content.

- **Where:** the dataset's actions in the Project region and the command palette, keyboard-operable like every other dataset action. The action's accessible name is the visible label.
- **Label:** "Check and relink this file".
- **The warning on open** (`SNR-APP-034`), for a scheme-1 link: "The linked data `{file}` was linked by an older SONARITHM build and cannot be verified against the project, so it is unavailable. Choose Check and relink this file to read it again." For a changed file the existing wording stays ("has changed since the project was saved"), followed by the same pointer to the action.
- **Choosing the action:** the reader opens the file under RFC-0002 §4's safety rules and limits, but nothing else sees the data yet: it is not shown, analysed or compiled. A dialog then opens. Its title and description are announced when it opens, and focus starts on **Cancel**:
  - title: "Relink `{file}`?";
  - description: "This data could not be verified against the project: the project has no record of the file's contents to compare it with. It has {rows} rows now; the project recorded {recordedRows}. If you relink it, SONARITHM uses the file as it is now. Relinking changes the project: save it to keep the link, or close it without saving to undo." When the row counts differ, the description adds: "The number of rows has changed." (wording amended, §11);
  - buttons: **Relink** and **Cancel**.
- **Accepting:**
  - the dataset becomes available;
  - its entry takes the file's size, SHA-256 and scheme-2 fingerprint, which the next save records;
  - the project is marked as having unsaved changes;
  - the announcement is "`{file}` is relinked. Save the project to keep it, or close without saving to undo.";
  - the action is not on the undo history, as no dataset change is; closing the project without saving reverts it (§11).
- **Cancelling** leaves the dataset unavailable and announces nothing more. The dialog closes and focus returns to the action.
- **Protocol:** two additive commands (ADR-0017 §5).
  - `dataset_relink_check { id }` reads the file and returns `{ name, rows, recordedRows, reason }`, where `reason` is `olderBuild` or `changed` (spelling amended, §11). It refuses when the file cannot be read under §4, with that warning's wording.
  - `dataset_relink { id }` applies the relink. The file is read again, so a file changed between the check and the relink is what gets linked, and the reply carries its row count.

Scheme-1 fingerprints are never recomputed. They depend on the arrow-ipc version that wrote them, so no later build can reproduce them.

### 7. Testing

- **Layout independence (property tests):** the same logical table built as one batch and as many, sliced, with garbage under nulls, with Utf8, LargeUtf8 and Utf8View, and with UInt8 and Int32 dictionary keys, gives one fingerprint.
- **Sensitivity (property tests):**
  - a change to any value, name, logical type, `snr.*` key, dictionary order or float bit pattern changes the fingerprint;
  - a change to non-`snr.*` metadata, nullability or batching does not.
- **The invariant:** for the example datasets, two tables with equal fingerprints compile to identical Scores. One case is a dictionary column against a plain column with the same strings, which must differ in fingerprint, because their category order differs.
- **Pinned digests:** the corpus digests (as `tests/fingerprints.rs` today), checked on all three CI operating systems, now with a stated expectation that an Arrow upgrade moves none of them.
- **Records:** a Score, a render manifest and a project written before this RFC read as scheme 1. A project fixture from before it opens as §6 says.
- **Check and relink:**
  - with the pre-#470 fixture (`crates/sonarithm-app/tests/fixtures/pre-470`), the linked dataset stays unavailable on open with the scheme-1 wording;
  - `dataset_relink_check` reports its rows and `olderBuild`;
  - `dataset_relink` makes it available and marks the project unsaved;
  - the next save writes `bytes`, `sha256` and a scheme-2 fingerprint, and reopening opens the link without a warning;
  - a same-length file with different values is never opened without the action;
  - the dialog gets an axe check, a keyboard test (focus starts on Cancel, Escape cancels) and a Storybook story, and the PR carries a manual screen-reader note.
- **Fuzzing:** the `sonarithm-data` fuzz targets keep their round-trip check (a loaded table survives Arrow IPC with the same fingerprint). The check now also holds across layouts.

### 8. Performance

`bundle_fingerprint` runs inside every compile (`sonarithm-compiler/src/data.rs:43`), so it counts against PB-04 (≤ 1 s compile for 1M rows) and against project save and open.

**Expected shape (not a measurement).**

- **v1:** concatenates the batches (a copy when there are several), rebuilds every flat column (a second copy), encodes IPC into a `Vec` (a third copy), then hashes about 8 to 9 bytes per numeric cell.
- **v2:** reads values in place and feeds the hasher through a small buffer. It makes no table-sized copy, but adds one tag byte per cell and per-value work for strings and nested types.
- **Overall:** SHA-256 throughput should dominate both.

**The bench** is `crates/sonarithm-data/benches/fingerprint.rs`, written with criterion beside `csv_ingest.rs`:

| Group | Input | Reports |
|---|---|---|
| `v1_ipc`, `v2_values` | 1M rows × 5 columns (f64, i64, a Utf8 nominal, a Dictionary<UInt8, Utf8> ordinal, a microsecond timestamp) plus `__snr_prov` and `__snr_conf`, built from NUMISONA's seeded generators | median time; throughput in MB/s of logical data; peak allocated bytes (a counting global allocator in the bench) |
| variants | one batch and 16 batches; 0 % and 10 % nulls | as above |

- **Where:** on CI-LINUX, the PB reference, and on the reference laptop under the shared-machine rules.
- **Recording:** results go in the benchmarks log against PB-04, with the compile bench run before and after.
- **No numbers are claimed here.**
- **Acceptance condition:** v2 is not slower than v1 at the median and uses less peak memory; if it is slower, the RFC's outcome records the trade-off.

### 9. Security, privacy and licences

- No new input is parsed and no new dependency is added: `sha2` is already used through `sonarithm-core::Sha256Digest`, and canonical JSON through `sonarithm-spec`.
- Fingerprints reveal no more about the data than today; the linked-file SHA-256 reveals no more than the file's fingerprint does.

## Drawbacks

- **One more move of every fingerprint, on purpose:** the same re-record as #470, plus the header field, in one PR.
- **We own an encoding:** the type table needs care when Arrow adds a type, and an unsupported type is an error (`DataError`, so the compiler records the zero digest, as today).
- **Three formats change at once:** the Score IR, the manifest and the project format.

## Rationale and alternatives

- **Keep v1 and pin arrow-ipc.** This blocks security fixes such as RUSTSEC-2026-0442, and the bytes still depend on the arrow-ipc version in use.
- **Keep v1 and record the arrow-ipc version in each record.** This makes the move visible, but not avoidable, and a record still cannot be checked by a later build.
- **Hash a sorted or row-order-independent form.** Row order is meaning in SONARITHM (time order, navigation), so it must count.
- **Canonicalise floats** (fold -0.0 into 0.0, and NaN payloads into one). This would break §1, because compile paths order with `total_cmp` (§4).
- **Resolve dictionaries into plain values.** This would break §1, because NUMISONA's category order comes from the dictionary (§3).
- **A faster hash (BLAKE3, xxHash).** It would add a dependency, and records and papers already say SHA-256. A different hash would need its own case.

## Prior art

- **Content-addressed storage** (Git, IPFS) hashes a format the system defines, never a library's serialization.
- **Lance and Delta Lake** checksum their own file formats.
- **Frictionless Data packages** record a file's byte hash, as §6 proposes for linked files.

## Accessibility impact

- **Warnings:** the `SNR-APP-093` and `SNR-APP-034` warnings on opening an older project are diagnostics. They reach the screen reader like the other project warnings, name the dataset by file name and need no colour or position.
- **The check-and-relink action** (§6) is new interaction:
  - it is a keyboard-operable action with its visible label as its accessible name;
  - its dialog announces its title and description on opening, starts focus on the safe choice (Cancel), closes on Escape, and returns focus to the action;
  - its outcome is announced.
- **Why the stricter reading:** it protects listeners who rely on sound alone. Opening a link on a row count could play different values with no way to notice; the action makes the uncertainty explicit and puts the decision with the user.
- **Manual checks in the implementation PR:**
  - open the pre-#470 fixture with NVDA and hear each warning once;
  - relink with the keyboard alone and hear the dialog's description and the result;
  - check that the high-contrast theme shows the dialog's focus.

## Research impact

- **Rules:** none. No design rule or default changes.
- **Reproducibility:**
  - replication records and the paper's provenance claim get stronger, since fingerprints survive library upgrades;
  - the paper's method sentence ("SHA-256 over a canonical Arrow IPC encoding") changes;
  - `datasets.md` records new fingerprints;
  - F-222 and F-223 are resolved by this RFC.

### 10. Implementation order

1. **Utf8View as text:** `infer/semantic.rs:293` and `view.rs:306` accept Utf8View as they accept Utf8. This is a small PR of its own, with the tests of §3 (the same table in each string layout infers the same semantics and compiles to the same Score). It changes no fingerprint.
2. **Scheme 2**, after step 1 has landed, in one PR:
   - the encoder and the header, manifest and project fields;
   - the check-and-relink action and its protocol commands;
   - the ADR-0013 amendment;
   - one re-record of every record that holds a fingerprint (the set #470 moved, plus the header field and `datasets.md`), with golden audio unchanged.

### 11. Amendment 1 (2026-11-26): relinking and the undo history; reason spelling

| Date | Change | Approved |
|---|---|---|
| 2026-11-26 | Relinking is not an undoable edit; the dialog and the announcement say how to undo it instead. `reason` values are camelCase | Coordinator, under the owner's delegation of contract approvals (OD-32, closed) |

- **Undo.** §6 said the relink is "one undoable edit, like other dataset changes". No dataset change is on the undo history: undo and redo restore the spec and corrections only, and `dataset_open` and `dataset_close` are not undoable either. Relinking follows them. The safety net is that relinking marks the project as having unsaved changes, so closing it without saving reverts the relink. The dialog's description and the announcement after relinking say so plainly (§6, wording above).
- **Spelling.** `reason` is `olderBuild` or `changed`, not `older-build`, to match the protocol's camelCase values (ADR-0017, for example `jumpForward` and `oneSeries`).
- **Contract.** ADR-0017 "Relinking linked data" holds the commands, replies, codes (`SNR-APP-095` for a dataset with nothing to relink) and the dialog's wording.

## Resolved questions

Decided by the coordinator on 2026-11-26, under the owner's delegation:

1. **Linked data saved under scheme 1:** the stricter reading. A matching row count alone does not open it, because a same-length file with different values would compile silently and a listener without the visual would hear wrong data. It stays unavailable (`SNR-APP-034`), with the explicit **Check and relink this file** action, whose wording and announcements §6 fixes. Accepting it records the file's size, SHA-256 and scheme-2 fingerprint.
2. **The scheme in the Score header:** a separate `data_fingerprint_scheme` field. Explicit is better, and the records quote the header.
3. **Utf8View:** collapse it into `string`, but only after the two fixes (inference and the data view treating Utf8View as text) have landed as their own small PR with tests (§3, §10).

## Unresolved questions

None.

## Future possibilities

- If dataset changes (opening, closing) join the undo history, relinking joins it too (§11).

- A per-column fingerprint, to say which column of a linked file changed.
- Recording the linked file's hash also lets a future build detect a file edited in place, without re-reading it.

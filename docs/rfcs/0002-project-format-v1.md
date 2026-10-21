# RFC 0002: Project format v1 (`.sonarithm` bundles)

> **Status:** **Accepted** on 2026-10-21 by the coordinator under the owner's delegation of contract approvals, with the three proposed answers to the unresolved questions and two conditions added (linked-data safety, §4; memory, §6). The 7-day final comment period was waived by the coordinator under the same delegation (fast-track, sole maintainer, no other participants). Outcome recorded in [ADR-0018](../decisions/0018-project-format-v1.md). **Author(s):** project lead (drafted by an AI coding agent, Claude Code, for review by the coordinator under the owner's delegation).
> **Related:** [RFC process](README.md) · [12 §4 project file format](../architecture/12-cross-cutting-concerns.md#4-project-file-format) · [09 §5 persistence](../architecture/09-desktop-app.md#5-persistence) · [ADR-0013](../decisions/0013-determinism-and-provenance.md) · [ADR-0012](../decisions/0012-privacy-and-telemetry.md) · [ADR-0017](../decisions/0017-ipc-protocol-v1.md)

| Field | Value |
|---|---|
| RFC | 0002 |
| Start date | 2026-10-21 |
| RFC PR | Opened with this file (branch `docs/rfc-0002-project-format`) |
| Affects | Persisted formats (the project bundle); IPC protocol (additive commands) |
| Phase | B05 (task B05-T04, [#236](https://github.com/DuhItzAniket/SONARITHM/issues/236)) |
| Related ADR(s) | [ADR-0018](../decisions/0018-project-format-v1.md) records the outcome |
| Supersedes | none |

## Summary

This RFC fixes version 1 of the `.sonarithm` project bundle sketched in [12 §4](../architecture/12-cross-cutting-concerns.md#4-project-file-format): a ZIP container with a manifest that lists every member with its SHA-256, the project's specs, embedded data snapshots as Parquet or links to data files with fingerprints, and folders reserved for profiles, annotations and render manifests. It pins down the manifest schema, the version and migration rules, what a reader refuses (zip-slip, size bombs, tampered members, newer major versions), what a writer never writes (absolute paths, secrets, timestamps), byte-identical writing, and autosave with crash recovery. It also adds the project commands to the IPC protocol (additive under ADR-0017 §5).

## Motivation

B05 lets desktop users save their work: the data they opened, the specs they edited and the renders they made. A project file outlives the app version that wrote it, is shared between people and machines, and is opened from untrusted sources, so it is a contract (AGENTS.md rule 11; [rfcs/README §1](README.md#1-when-an-rfc-is-required), persisted formats). 12 §4 gives the layout but leaves open:

- the manifest's fields and how a reader checks members against it;
- how linked data is recorded without absolute paths (rule 9, ADR-0012);
- what makes the format reproducible: the same project content should give the same bytes, so projects can be hashed, diffed and cited (ADR-0013);
- how a reader defends against crafted files (12 §6: zip-slip, size and time budgets);
- how versions and migrations work, and what a v1 reader does with members it does not know;
- where autosaves go and how recovery is offered (FR-110, NFR-11).

Doing nothing means B05-T04 would make these choices silently in code.

## Guide-level explanation

A desktop user chooses **Save** and gets one file, `temperatures.sonarithm`. It holds the spec they edited and a snapshot of the data, so it plays and renders the same anywhere, without the original CSV. For a large dataset they can choose **Link data instead of copying**: the project then records where the file is, relative to the project, and its fingerprint; if the file is missing or has changed when the project is opened, they hear and read a warning naming the dataset, and the rest of the project opens.

Opening a project from someone else is safe: SONARITHM checks every member against the manifest before using it, refuses anything that tries to write outside the project or claims to be bigger than allowed, and never runs code from a project.

If SONARITHM crashes, the next start offers the last autosave ("Recover unsaved work from 14:05?"; the time comes from the file system, not the file). Recovering opens it as an unsaved project; declining deletes the autosaves.

Programmatically, through the IPC protocol of ADR-0017:

```json
{ "command": "project_save_as", "args": { "path": "C:/Users/me/Documents/temperatures.sonarithm", "embedData": true } }
{ "command": "project_open", "args": { "path": "C:/Users/me/Documents/temperatures.sonarithm" } }
```

## Reference-level explanation

### 1. Container

- A ZIP file (PKWARE APPNOTE 6.3.x subset) with the extension `.sonarithm` and the media type `application/vnd.sonarithm.project+zip` (not registered; used in the desktop file association only).
- Members are **stored** (no compression) when they are Parquet, and **deflated** when they are JSON, as 12 §4 says.
- No ZIP64 in v1 writers; readers accept ZIP64 only within the size limits of §6. No encryption, no multi-disk archives, no symbolic links.

### 2. Members

| Path | Required | Content |
|---|---|---|
| `manifest.json` | Yes, first member | §3 |
| `specs/<spec-id>.json` | At least one | A spec as UTF-8 JSON text, exactly as the user last saved it (not re-serialized), so editor formatting and comments-free key order survive |
| `data/<dataset-id>.parquet` | One per embedded dataset | A snapshot of the dataset's primary table as written by `sonarithm_data::write_parquet`: `snr.*` field metadata and the canonical provenance columns (03 §4) |
| `profiles/project-profile.json` | No | Per-project overrides (B03-T08 defines its schema). Reserved in v1: kept byte for byte when present |
| `annotations/<dataset-id>.json` | No | User notes and the PLOTSONA corrections log (C01). Reserved in v1: kept byte for byte |
| `renders/<name>.render.json` | No | Render manifests of exports made from the project (B05-T05); audio files stay outside the project |

IDs in paths match `^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$`. Dataset IDs are the IDs specs use in `{"kind": "dataset", "id": "d1"}` sources.

### 3. The manifest

```json
{
  "format": "sonarithm-project",
  "formatVersion": "1.0",
  "generator": { "name": "sonarithm", "version": "0.5.0-beta" },
  "name": "Temperatures",
  "specs": [
    { "id": "main", "path": "specs/main.json" }
  ],
  "activeSpec": "main",
  "datasets": [
    { "id": "d1", "name": "temps.csv", "storage": "embedded", "path": "data/d1.parquet",
      "fingerprint": "sha256:9f2c…", "rows": 365 },
    { "id": "d2", "name": "sensor.parquet", "storage": "linked", "link": "../raw/sensor.parquet",
      "fingerprint": "sha256:41ab…", "rows": 1048576 }
  ],
  "members": [
    { "path": "data/d1.parquet", "bytes": 18234, "sha256": "4c1d…" },
    { "path": "specs/main.json", "bytes": 912, "sha256": "a7e0…" }
  ]
}
```

| Field | Type | Rule |
|---|---|---|
| `format` | string | Always `sonarithm-project` |
| `formatVersion` | string `MAJOR.MINOR` | `1.0` for this RFC |
| `generator` | object | The writing software's name and version, for bug reports. Not used to decide anything |
| `name` | string | Display name, 1–200 characters |
| `specs` | array | Each spec member, in the order the user sees them |
| `activeSpec` | string | The ID of the spec that opens first; must be in `specs` |
| `datasets` | array | Each dataset. `name` is the original file name only (no directories). `storage` is `embedded` (with `path`) or `linked` (with `link`). `fingerprint` is the `sonarithm_data` table fingerprint of the primary table (A04-T09). `rows` is informational |
| `members` | array | Every member except `manifest.json`, sorted by `path`, with its byte length and lowercase hex SHA-256 |

The manifest is JSON with sorted object keys and no insignificant whitespace (RFC 8785 canonical form, as the spec hash uses), so it is byte-stable. Unknown manifest fields are refused in a reader of the same major version and ignored in a reader of an older minor version (§5).

**No timestamps** (resolved question 1). The manifest has no `created` or `modified` field: the file system keeps those times, timestamps would make every re-save differ in bytes (ADR-0013), and they reveal when someone worked (ADR-0012). 12 §4 is updated to match.

### 4. Linked data and paths

- `link` is a relative path from the project file's folder to the data file, with `/` separators. A writer computes it; if no relative path exists (another drive on Windows), the writer refuses with `SNR-APP-033` and the hint to embed the data.
- A writer never writes an absolute path, a drive letter, a UNC path, a home-directory shorthand or a URL anywhere in the bundle. It checks the specs too: a spec whose `file` source names an absolute path is refused with `SNR-APP-033` and the hint to open the file as a dataset and refer to it by ID.
- On open, a missing linked file or a fingerprint mismatch is a warning (`SNR-APP-034`) naming the dataset by its file name only, never a directory (resolved question 3); the dataset is then unavailable and specs that use it report `SNR-COMP-026` when compiled.

**Linked-data safety** (acceptance condition A). A shared project chooses its own `link`, so `../../` can point anywhere the user can read. On open, before reading a linked file, the reader:

1. resolves the link against the project's folder and canonicalises it (symbolic links followed);
2. requires a regular file with a data extension A04 reads (`.csv`, `.tsv`, `.json`, `.ndjson`, `.parquet`, `.arrow`, `.xlsx`, `.ods`); a directory, device, pipe, socket or any other extension is refused with `SNR-APP-034`, without opening it;
3. reads it with A04's reader limits (`Limits::DEFAULT`);
4. compares its fingerprint with the manifest's before anything else sees the data: on a mismatch the dataset stays unavailable and is never shown, analysed, compiled or embedded on a later save; the warning names the file.

Tests cover a link to a non-data file, to a device path (`/dev/zero`, `NUL`), to a directory, and a symbolic link whose target is not allowed (where the OS lets the test create one).

### 5. Versions and migrations

- A reader refuses a newer **major** version (`SNR-APP-031`, "made by a newer SONARITHM").
- A reader of version 1.x opens a newer **minor** version 1.y, ignores manifest fields it does not know, and warns (`SNR-APP-035`) that saving will write version 1.x and drop them.
- Older versions are migrated explicitly, one step at a time, as settings and specs are (`migrate_v<N>_to_v<N+1>`). Version 1 is the first, so v1 has no migrations; the framework and its test harness land with it.
- Members under unknown top-level folders are ignored with `SNR-APP-035` and not written back. The reserved folders of §2 are kept byte for byte.

### 6. Reading untrusted bundles

A reader:

1. Refuses member names that are absolute, contain `..`, `\`, a drive letter, a NUL or a control character, or lie outside the folders of §2 (zip-slip, `SNR-APP-032`). It never writes members to disk; it reads them into memory.
2. Applies limits: at most 10 000 members; at most 64 MiB per JSON member; at most 2 GiB uncompressed in total, A04's limit for one input, so a project never opens more than its files could directly; a compression ratio of at most 100:1 per deflated member. It stops reading a member as soon as it exceeds its declared size. Over a limit: `SNR-APP-032`. Each embedded table also meets A04's `Limits` (rows, fields, cells). The limits are adjustable for trusted files.

   **Memory** (acceptance condition B). The total limit must not mean holding it in RAM on a 16 GB laptop. Stored (uncompressed) Parquet members are read in place: the reader takes the member's offset and length from the ZIP directory and hands the Parquet decoder a bounded file-slice reader, so the member's bytes are never buffered whole; only the decoded table is in memory, as for a Parquet file opened directly. This needs a data-layer entry point that reads Parquet from a seekable source (A04's reader copies its input into memory today); B05-T04 adds it. Deflated members are JSON and at most 64 MiB each. A Parquet member that is not stored (a hand-made bundle) is refused with `SNR-APP-032`, since deflate cannot be read in place.
3. Checks every member listed in the manifest against its length and SHA-256 before parsing it; a mismatch or a listed member that is missing refuses the project (`SNR-APP-032`, "damaged or altered"). Members not listed are ignored.
4. Parses specs with `sonarithm-spec` (diagnostics reported, never fatal to opening) and embedded data with A04's Parquet reader and its limits.

A fuzz target (`project_open`) runs the reader on arbitrary bytes in `nightly.yml`.

### 7. Writing

- Members in a fixed order: `manifest.json`, then the other members sorted by path.
- Every ZIP entry has the DOS epoch timestamp (1980-01-01 00:00:00), no extra fields, no comments, and fixed permissions; deflate at level 6.
- The same project content therefore gives the same bytes with the same build ("a saved project reopens bit-identically": open, then save without changes, gives an identical file). Parquet writer and deflate versions can change bytes between builds; the content round trip (specs byte for byte, data equal by fingerprint) holds across builds.
- Saving is crash-safe: the writer writes `<file>.tmp` in the same folder, flushes it, then renames it over the target, as the settings store does (B01-T01).
- No secrets: the format has no field for credentials, and stream sources (B06) store only a URL without its user-info part.

### 8. Autosave and crash recovery

- While a project has unsaved changes, the app core writes an autosave bundle to `<app-data>/autosave/` after each change, at most once every 30 s, keeping the last 5 (`autosave-<n>.sonarithm`, `n` counting up) (resolved question 2). Autosaves embed all data, so a recovery never depends on a linked file.
- The autosave folder has a total size budget: when a new autosave would exceed it, the oldest are deleted first, and if one autosave alone exceeds it, the app keeps only that newest one and says so in the status log. The budget is TODO(verify): B05-T07's memory and disk soak sets it, because five copies of a 1M-row dataset are the likely problem; until then it is 2 GiB.
- A clean exit or an explicit save deletes the autosaves of that project.
- On start, autosaves that remain mean the last session did not end cleanly: `project_recovery` lists them (newest first, with their names and the file system's modification time) and the UI offers the newest. `project_recover { id }` opens one as an unsaved project; `project_discard_recovery` deletes them.

### 9. IPC protocol additions (ADR-0017 §5: additive)

| Command | Reply | Notes |
|---|---|---|
| `project_new` | `project` | An empty session |
| `project_open { path }` | `project` | Replaces the session; warnings in `ProjectSummary.diagnostics` |
| `project_save` | `project` | To the current path; `SNR-APP-030` if the project has none |
| `project_save_as { path, embedData }` | `project` | `embedData: false` links datasets that came from files |
| `project_recent` | `recent` | Up to 10 paths, kept in the app-config folder (a local file, never in a project) |
| `project_recovery` | `recovery` | Autosaves left by a crash |
| `project_recover { id }` | `project` | |
| `project_discard_recovery` | `recovery` | |

`ProjectSummary`: name, path (as the user chose it), whether it has unsaved changes, specs, datasets with their storage and status, and diagnostics.

### 10. Codes

`SNR-APP-030` no project path; `031` newer major version; `032` unsafe or damaged bundle; `033` absolute path refused; `034` linked data missing or changed (warning); `035` newer minor version or unknown members (warning).

### 11. Testing

- Round trip: a project with two specs, an embedded and a linked dataset and each reserved folder reopens with specs byte for byte and data equal by fingerprint; saving it again without changes gives identical bytes.
- A property test: random spec texts and small tables round-trip.
- Crash injection: an interrupted save leaves the previous file readable; a session dropped without a clean exit leaves autosaves that the next start offers, and recovering yields the last snapshot.
- Refusals: zip-slip names, an oversized member, a bad ratio, a tampered member, a missing listed member, a deflated Parquet member, a newer major version; one test per code.
- Linked data: a link to a non-data file, a device, a directory and a symbolic link to a disallowed target are refused unread; a fingerprint mismatch leaves the dataset unavailable and out of the next save.
- Memory: opening a project with a large stored Parquet member does not allocate the member's size before decoding (checked with a counting allocator around the reader, as A05-T10 does for the audio thread).
- No absolute path is ever written: a property test scans every byte of saved bundles for the temporary folder's absolute path and for drive or root prefixes in manifest paths.
- `cargo fuzz` target `project_open` in `nightly.yml`.

## Drawbacks

- ZIP is more complex than a folder; the reader must defend against crafted archives.
- Embedded snapshots make projects large; linking trades size for portability.
- Dropping timestamps loses information some users might expect in the file.

## Rationale and alternatives

- **A folder instead of a ZIP:** easy to diff, but not one file to share or attach; 12 §4 already chose ZIP.
- **SQLite:** one file and incremental writes, but a large dependency, non-deterministic page layout, and harder to inspect without tools.
- **Arrow IPC instead of Parquet for data:** faster to load, larger on disk; Parquet is A04's archival format and keeps `snr.*` metadata (A04-T03).
- **Timestamps in the manifest:** rejected above for determinism and privacy; the recent list and recovery use the file system's times.

## Prior art

- OpenDocument and OOXML: ZIP containers with a manifest; ODF's `mimetype` first member inspired "manifest first".
- Reproducible builds (`SOURCE_DATE_EPOCH`, fixed ZIP timestamps in `jar` and `zip -X`): fixed entry times for byte-stable archives.
- Python wheels' `RECORD` file: per-member SHA-256 and sizes, checked on install.

## Accessibility impact

Opening, saving and recovery are announced through the app core's events and replies, which carry text for every outcome: warnings name the dataset and say what still works (A11Y-17, A11Y-24). Recovery is a single, dismissible prompt, not a modal that traps focus (09 §7). The format stores no presentation-only information, so nothing visual is lost for screen-reader users. The manual check is part of B05-T07 and B07's matrix.

## Research impact

None on design rules or defaults: the project stores specs, and specs carry every perceptual choice. Byte-identical bundles strengthen ADR-0013's reproducibility claim (H-08): a project's SHA-256 can be cited next to a render manifest.

## Unresolved questions

All three were resolved at acceptance (2026-10-21):

1. `created` and `modified` are dropped from the manifest (§3); 12 §4 is updated.
2. At most 5 autosaves, at most every 30 s, within a total size budget set from B05-T07 (§8; the budget is TODO(verify)).
3. A broken link shows the dataset's file name only, never directories (§4).

The size budget's value is the only open number; B05-T07 measures it.

## Future possibilities

- Signed projects (a detached signature member) for shared research artifacts.
- Embedding rendered audio for archival, behind an explicit option.
- A `sonarithm project` CLI command to inspect, verify and extract bundles.

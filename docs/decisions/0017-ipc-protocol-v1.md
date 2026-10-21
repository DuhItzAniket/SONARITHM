---
status: accepted
date: 2026-10-21
decision-makers: ["@DuhItzAniket"]
consulted: ["coordinator agent (Claude Code), under the owner's delegation of ADR approvals"]
---

# 0017 · IPC protocol v1: one request type in the headless app core

## Context and problem statement

Stage B builds the desktop app as a thin Tauri host over the headless app core `sonarithm-app` ([09 §1–§2](../architecture/09-desktop-app.md), [01 §4](../architecture/01-overview.md#4-containers-c4-level-2)). [ADR-0011](0011-typed-ipc-ts-rs.md) chose ts-rs for the payload types but left the protocol itself open: where it lives, how requests, replies, events and errors are shaped, how it is versioned, and how binary views are laid out.

OD-15 was closed on 2026-10-21 on condition that the Tauri host stays a thin adapter: all app logic, session state, transport and the IPC protocol live in `sonarithm-app` and are testable without Tauri, so a later fallback to Electron changes only the host ([implementation plan §3.1](../planning/implementation-plan.md#31-how-stage-b-runs)). The IPC protocol is a contract (AGENTS.md rule 11; [rfcs/README §1](../rfcs/README.md#1-when-an-rfc-is-required)), so its first version is recorded here before B01-T01 implements it.

## Decision drivers

* OD-15: the host only translates; Electron (napi-rs) or a test harness must be able to drive the same protocol.
* Rust is authoritative for session, spec, transport and cursor state (09 §3).
* No JSON on hot paths; bulk data is binary (F-040, F-041, PB-09).
* Generated types only (AGENTS.md rule 10, ADR-0011); typed errors with `code`, `message`, `hint` and `pointer` (12 §1).
* Mismatched host and UI builds fail fast with a clear error (09 §2.2).
* Accessibility: playback starts only on a user command (A11Y-15), and every change a listener must know about reaches the UI as an event it can announce.

## Considered options

* **A. One `Request` enum and one `Reply` enum in `sonarithm-app::ipc`, an `Event` enum for channels, a dispatch function the host calls.**
* B. One `#[tauri::command]` function per command, with argument structs in `sonarithm-app`.
* C. tauri-specta for typed commands.

## Decision outcome

Chosen option: **A**, because the whole protocol, including command names, then lives in one framework-agnostic crate that tests drive directly, and a host is a few lines of translation.

### 1. Where it lives

`sonarithm-app` has a module `ipc` with `Request`, `Reply`, `Event`, `IpcError`, `PROTOCOL_VERSION` and the binary view header. `App::dispatch(Request) -> Result<Reply, IpcError>` is the only entry point for commands; `App` is `Send`, and a host keeps it behind one lock. The app core drives itself: its own control thread owns the audio player (whose device stream is not `Send` on every OS) and ticks it every 10 ms, and long work runs in the job queue on worker threads. Those threads hand events to the host's `EventSink` directly, so the host never polls. `sonarithm-app` has no Tauri, Electron or webview types.

### 2. Commands and replies (JSON)

A request is an adjacently tagged object; the command names are those of [09 §2.1](../architecture/09-desktop-app.md#21-command-surface-initial):

```json
{ "command": "transport_seek", "args": { "seconds": 12.5 } }
{ "command": "transport_play" }
```

A reply is tagged by `reply`, for example `{ "reply": "transport", "status": "paused", "positionS": 12.5, … }`. Field names are camelCase in JSON, as in the spec. The host returns the reply as the command's result and the `IpcError` as its rejection. A host may expose one handler per command or one generic handler; the JSON is the same either way.

Every transport command replies with the authoritative `TransportState`. The UI renders it and never assumes an intent succeeded.

Spec and data problems are results, not errors: `compile` replies with a `CompileReport` (diagnostics with codes, rules and JSON pointers, and whether a Score was produced). `IpcError` is for commands that could not run.

### 3. Errors

```json
{ "code": "SNR-APP-003", "message": "No dataset is open with the ID d7.", "hint": "dataset_open returns the IDs of open datasets.", "pointer": null }
```

`code` is a stable `SNR-<AREA>-<NNN>` code from the registry (12 §1); `sonarithm-app` adds the area `APP`. `message` and `hint` follow the 12 §1 message style; `pointer` is a JSON pointer into the spec when the error concerns one.

### 4. Events (channels)

`Event` is tagged by `event`. The app core hands events to an `EventSink` the host implements; each event names its channel:

| Channel | Events | Rate |
|---|---|---|
| `clock` | Clock anchors: transport status, position, speed and the host time of the block | On change and every 2 s (06 §9) |
| `jobs` | Job progress, finished, failed, cancelled | On change, at most 10 per second per job |
| `diagnostics` | Diagnostics outside a reply (for example from a background compile) | On change |
| `announcements` | Text for the announcer (B03) | Rate-limited per 08 §4 |
| `app` | Rare app-wide notices: device changed or lost, settings recovered | Rare |
| `meters` | Binary meter frames (B01-T04) | 30 Hz, batched |

### 5. Versioning

`PROTOCOL_VERSION` is one integer, `1` for this ADR. The UI's first request is `hello { protocol }`. A different number fails with `SNR-APP-001` and a message naming both versions and the fix (build host and UI from the same commit). Host and UI ship together, so a mismatch only happens in development.

* Adding a command, an optional field, a reply variant or an event kind does not change the number.
* Removing or renaming a command or field, or changing a meaning or unit, increments it, with a CHANGELOG entry. Once tests, plugins or external consumers depend on the protocol, such a change needs an RFC (rfcs/README §1).

### 6. Binary views

Bulk responses (`view_get`, `table_page`; B04) are one little-endian buffer: a 32-byte header, a column directory and 8-byte-aligned column data, so the UI makes typed-array views without copying.

| Offset | Size | Field |
|---|---|---|
| 0 | 4 | Magic `SNRV` |
| 4 | 2 | Header version, `1` |
| 6 | 2 | Kind: `1` M4 tile, `2` LTTB tile, `3` table page |
| 8 | 4 | Column count `c` |
| 12 | 4 | Reserved, `0` |
| 16 | 8 | Row count `n` |
| 24 | 8 | Data generation (changes when the dataset changes) |
| 32 | 16 × `c` | Column directory: data type (u16, below), flags (u16: bit 0 has a validity bitmap), name length in bytes (u32), data offset from the buffer start (u64) |

Column names follow the directory, then each column's validity bitmap (if any, `⌈n/8⌉` bytes) and data, each padded to 8 bytes.

| Data type code | Values |
|---|---|
| `1` f64 | 8 bytes per row |
| `2` i64 | 8 bytes per row |
| `3` UTF-8 | `n + 1` u32 offsets, padded to 8, then the bytes |
| `4` bool | 1 byte per row, `0` or `1` |
| `5` provenance class | 1 byte per row: the position in the 03 §4 list, `0` `exact`, `1` `vector`, `2` `cv_estimate`, `3` `ml_estimate`, `4` `cloud_estimate`, `5` `user_edited`, `6` `derived`, `7` `synthetic` (the same codes as the canonical `__snr_prov` dictionary) |
| `6` confidence | 4 bytes per row (f32 in `[0, 1]`); null (not applicable) through the validity bitmap |

**Provenance (AGENTS.md rule 7, 03 §4).** A table page of a table that has provenance columns carries them as a `5` column named `__snr_prov` and a `6` column named `__snr_conf`, so the UI can qualify estimates. A table without them is all `exact`, and its pages omit both columns (03 §4 rule 1). Tiles carry no provenance: they are for drawing; the UI reads a point's provenance from its table row.

**Readers are strict.** A reader rejects a buffer with another magic, an unknown header version, an unknown view kind or an unknown data-type code, or one that ends early, with `SNR-APP-050` ("not a binary view this version reads"); it never guesses. Adding a data-type code or a view kind, or changing the layout, increments the header version; host and UI ship together, so a mismatch is a development error, as for `PROTOCOL_VERSION`.

B01-T02 generates the header constants for TypeScript from the Rust constants; B04-T03 implements the kinds.

### 7. Privacy (AGENTS.md rule 9, ADR-0012)

* `IpcError` messages and hints, diagnostics, replies and events never contain an absolute user path or personal data. A file is named by its file name alone, as the data layer's redacted locator does (03 §2, `locator_redacted`); an export result names the files it wrote, not their folder.
* Paths a user passes in (a dataset to open, an export target, later a project) are canonicalised and validated in the app core, not in the host: the file must exist and be a regular file of a supported type, or, for a target, its folder must exist; nothing outside what the user chose is read or written.
* The protocol has no command that sends data off the machine; network features (B06 streams) need their own per-action consent (ADR-0012).

### 8. Layering

`sonarithm-app` may depend on the facade `sonarithm`, as the plan's B01-T01 row says ("sessions over the facade"), so the app reuses its compile and render paths instead of copying them. [02 §3](../architecture/02-repository-and-crate-layout.md#3-dependency-rules-enforced) and `cargo xtask check-deps` gain the edge `sonarithm-app → sonarithm`. A rule that `sonarithm-desktop` depends on nothing below `sonarithm-app` except `sonarithm-core` types comes with B01-T03.

### Consequences

* Good: the scenarios of testing-strategy §10 drive the exact protocol a host exposes, headless and on three OSes; an Electron host only forwards JSON.
* Good: estimates stay qualified in every view, and nothing in the protocol leaks a user's folders.
* Good: one enum per direction gives one generated TypeScript union per direction (ADR-0011), so the UI cannot send an unknown command.
* Bad: Tauri's per-command permissions cannot tell commands apart if a host uses one generic handler. All commands are the app's own and the webview loads no remote content (09 §4), so the risk is small; B01-T03 decides the handler shape.
* Bad: the typed client still maps each command to its reply variant by hand (ADR-0011's known gap); a contract test per command covers it.

### Confirmation

B01-T01: integration scenarios drive `App::dispatch` with JSON requests and the null audio backend, and check that no reply, error or event carries the test's absolute folder. B01-T02: the generated TypeScript types, the version-mismatch test, the header constants and the strict reader. B04-T03: payload tests of each binary kind, provenance columns in table pages included.

Accepted on 2026-10-21 by the coordinator under the owner's delegation of ADR approvals, with the provenance encodings (§6) and the privacy rules (§7) added at review.

## Pros and cons of the options

### A. One request enum in the app core

* Good: protocol and dispatch are tested without any host; names cannot drift between hosts.
* Neutral: the host still needs one Tauri command (or one per request) and a channel per `Event` channel.

### B. One Tauri command per request

* Good: matches Tauri's examples; per-command permissions.
* Bad: the command list lives in the host, so an Electron host would repeat it; dispatch logic tends to leak into the host, against OD-15.

### C. tauri-specta

* Good: generated command signatures.
* Bad: still a release candidate ([ADR-0011](0011-typed-ipc-ts-rs.md)); ties the protocol to Tauri.

## More information

* Builds on [ADR-0011](0011-typed-ipc-ts-rs.md) and OD-15 ([open-decisions](../planning/open-decisions.md#closed-decisions)); evidence from A02 spike 3 (F-040, F-041, F-042).
* Revisit if the protocol gains external consumers (plugins, a remote UI), which would need an RFC for the versioning policy.

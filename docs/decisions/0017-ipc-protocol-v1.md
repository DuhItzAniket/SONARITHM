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
| `clock` | Clock anchors: transport status, position, speed, the host time of the block and the output latency (`outputLatencyS`, see Clock anchors below) | On change and every 2 s (06 §9) |
| `jobs` | Job progress, finished, failed, cancelled | On change, at most 10 per second per job |
| `diagnostics` | Diagnostics outside a reply (for example from a background compile) | On change |
| `announcements` | Text for the announcer (B03) | Rate-limited per 08 §4 |
| `app` | Rare app-wide notices: device changed or lost, settings recovered | Rare |
| `cursor` | The one cursor: every move and every change of the data under it (see Cursor below) | On change |
| `meters` | Binary meter frames, never JSON (see Meter frames below) | At most one every 1/30 s while playing, plus one last frame when playback stops |

#### Clock anchors

> Amended 2026-10-30, B04-T04: `outputLatencyS` added, approved by the coordinator, additive.

A `ClockAnchor` carries the transport, `hostTimeS` (the device's playback time of the anchor's block, when the backend reports one), `seq`, and `outputLatencyS: number | null`. That is how far the heard sound lags the engine's position, in seconds; the UI's playhead shows `positionS − outputLatencyS × speed` (06 §9).

The order of precedence is:

1. the device's calibrated latency where one is stored (ADR-0016, "Per-device latency calibration": "a loopback self-test where the platform offers one, otherwise a stored offset"; stored as `audio.calibratedLatencyS` in the settings, B04-T04);
2. then the backend's reported latency;
3. then `null`, which the UI takes as "no correction", never as a guessed value.

The app core computes it on the control thread when it builds the anchor, never in the audio callback. Adding the field did not change `PROTOCOL_VERSION` (§5).

| Date | Change |
|---|---|
| 2026-10-30 | `outputLatencyS` added (B04-T04), citing ADR-0016 §"Per-device latency calibration"; approved by the coordinator, additive |
| 2026-10-31 | `cursor_set`, `cursor_get` and the `cursor` event and channel added (B04-T06): Rust's one cursor, with an origin that decides what is spoken (INFER, OD-30); approved by the coordinator, additive |
| 2026-11-01 | `nav_move`, `nav_where` and the `nav` reply added (B03-T09); `CursorState.node` may be any navigation node, `path` levels gain the axes, insights and filters levels, `CursorState.excluded` lists the filtered series and `TransportState.excludedTracks` the tracks playback leaves out; approved by the coordinator, additive |
| 2026-11-01 | `braille_route`, `CursorOrigin::braille` and `CursorState.braille` (a `BrailleState`) added, and the settings gain `braille` (`mode`, `cells`) (B03-T10); approved by the coordinator, additive |

#### Cursor

> Amended 2026-10-31, B04-T06: `cursor_set`, `cursor_get` and the `cursor` event added, approved by the coordinator, additive (as `outputLatencyS`).

**One owner (A11Y-10, 09 §3).** Rust's cursor is the only cursor. The table, the plot, the breadcrumb and the readout show it. B03-T09's `nav_move` and `nav_where` must read and update this same cursor; they do not add a second one.

* `cursor_set { dataset, row, origin }` moves it to a dataset row and replies `cursor`. `row` counts from 0 in the dataset's first table (a table page's `__snr_row`). A row outside the dataset fails with `SNR-APP-087`, an unknown dataset with `SNR-APP-002`.
* `cursor_get` replies `cursor`.
* A `cursor` event on the `cursor` channel follows every `cursor_set` and every change Rust makes, so every view shows the same cursor.
* `CursorState { seq, dataset, row, x, node, path, origin }`:
  * `seq` orders them.
  * `dataset` and `row` are `null` when there is no cursor.
  * `node` and `x` come from the compiled Score's navigation tree when the row is one of its points; a row read by several tracks takes the first series' point.
  * `path` lists `{ level, label }` from the dataset down. `level` is `dataset`, `series`, `segment` or `point`, which the UI names through its catalogue; `label` is a data name, shown as it is.
  * `origin` is `table`, `plot` or `nav` for a `cursor_set`, and `null` when Rust moved it.

**What is spoken (INFER, OD-30).** A move from the table is not announced: the screen reader already reads the focused row, and SONARITHM speaking it too would say it twice. Moves from the plot and the explorer are announced as arriving at the point, in the listener's verbosity. No design rule covers this yet, so the choice is marked INFER; the NVDA walkthrough confirms or changes it (OD-30). The point cue on a move is not built yet: the app core has no point-cue builder (B03-T09 and B03-T12).

**The data changes under it.** After every command, Rust checks the cursor against the open datasets:

* a closed dataset, or a project opened or started (which replaces every dataset, even one with the same ID), clears it;
* a dataset that lost rows (corrections, undo) clamps it to its last row;
* a compile that makes its row a point adds `node`, `x` and the path.

Each change sends a `cursor` event with `origin: null`. A `cursor` event never moves the keyboard focus (B04-T06).

#### Navigation

> Amended 2026-11-01, B03-T09: `nav_move`, `nav_where` and the `nav` reply added, and `CursorState` widened, approved by the coordinator, additive.

**The same cursor (A11Y-10).** The chart explorer's verbs move the cursor of `cursor_set`. Beside the row, the cursor keeps the explorer's node, its x position, its series and the series the filters took out (AUDAXIS's `Cursor`).

* A table or plot move puts the node on the row's point, keeping the excluded series. A row that is not a point leaves no node.
* A verb moves the node, and the row follows it: a point's own row; for a series or a segment, the series' point nearest the cursor's x. An axis, an insight group or a filter leaves the row where it was.
* A compile renumbers the nodes, so the node is found again from the row, and the excluded series by their names.

**Commands.**

* `nav_move { verb }` applies a verb (keyboard-and-navigation §4.2) and replies `nav`. `verb` is tagged by `verb`: `next`, `previous`, `jumpForward`, `jumpBack`, `first`, `last`, `into`, `out`, `activate`, `nextSeries`, `previousSeries`, `maximum`, `minimum`, `nextInsight`, `previousInsight`, `{ verb: "landmark", landmark }` (`xAxis`, `yAxis`, `insights`, `filters`, `dataset`), `{ verb: "goTo", x }` (`{ number }` or `{ category }`) and `{ verb: "findValue", value }`. It sends a `cursor` event with origin `nav` for every verb, also one that kept the cursor.
* `nav_where` announces "where am I" (keyboard §6, A11Y-07) and replies `announce`. The cursor does not move, and no `cursor` event is sent.
* `nav_query { query }` answers a question of the query menu about the cursor's series (keyboard §7), as a response announcement, and replies `announce`. `query` is tagged by `query`: `summary`, `minimum`, `maximum`, `range` or `{ query: "valueAt", x }` (the B03 subset; the statistics join with ECHOMETRIC). The targets are found on a copy of the cursor, so it does not move, and no `cursor` event is sent. Added 2026-11-01, additive.
* Each fails with `SNR-APP-090` when no compiled Score has a navigation tree.

**`NavResult`.** `{ cursor, outcome, edge, reason, included, earcon, highlight, announced }`:

* `cursor` is the `CursorState` the `cursor` event carried.
* `outcome` is `moved`, `boundary`, `invalid`, `describe` (Enter on a point) or `toggled` (Enter on a filter).
* `edge` (`first`, `last`, `top`, `bottom`), `reason` (such as `oneSeries`, `noInsights`, `notHere`) and `included` belong to a boundary, an invalid verb and a toggle.
* `earcon` is the outcome's earcon ID, such as `E-BOUNDARY` (A11Y-08). `highlight` is the node's x span.
* `announced` says whether an announcement was queued. Its text goes out on the `announcements` channel only, never in the reply, so nothing is spoken twice. Explorer moves are announced, as plot moves are (INFER, OD-30).

**`CursorState`, widened.** `node` may be any node of the navigation tree, not only a point. `path` levels add `axes`, `axis`, `insights`, `insightGroup`, `insight`, `filters` and `filter`; a group's label is empty. A reader that meets a level it does not know shows the label alone. `excluded` lists `{ node, label }` for each series a filter took out.

**Filters and playback (keyboard §3).** A filter takes its series out of playback and view.

* The transport plays a copy of the compiled Score with the excluded series' tracks removed and nothing else changed: the header, the markers and every other track are the compiled Score's own, bit for bit (tested). It is still the compiler's Score IR (ADR-0005); nothing new synthesizes sound, and nothing changes on the audio thread.
* Loading it pauses playback, which never continues on its own (A11Y-15).
* The exclusion is recorded where it applies, so what is playing is never taken for the canonical Score: `CursorState.excluded` in the session, and `TransportState.excludedTracks` (the removed track IDs, empty when the compiled Score plays as it is) in the transport. A live Score plays whole and clears it.
* Exports render the whole compiled Score: the filter is session state, not part of the spec or the project. While a series is excluded, starting an export announces "Export includes all series; exclusions apply to playback only" (`announce-export-all-series`), until the export dialog (B05-T05) says it. A spec-level filter can follow with `spec_apply` (B05-T01).

**Point cue (A11Y-10, A11Y-20).** A move is to play the point's cue, then speak. The transport has no one-shot cue path yet, so the cue arrives in its own change, with a `cue` field added to `NavResult` then.

#### Braille

> Amended 2026-11-01, B03-T10: `braille_route`, the `braille` origin, `CursorState.braille` and the `braille` settings added, approved by the coordinator, additive.

**The same cursor (A11Y-10).** With braille output on (the effective `modalities.braille`), every `cursor` event carries `braille: BrailleState | null` for the cursor's node:

* `mode`: `"review"` or `"sparkline"`. It says what `text` is: in sparkline mode, a node that is not a point shows its review line, with `mode: "review"`.
* `text`: the review line (the composer's parts in symbol form, in the listener's terse order with the position added, whole parts only) or the sparkline (one 8-dot cell per point of the cursor's segment, U+2800 to U+28FF; MAIDR's four heights; a blank cell for a missing value) (keyboard §10; A11Y-21).
* `caret`: the cell of the cursor's point in a sparkline, 0 in review mode.
* `cells`: the display's cell count from the settings, or `null` when nothing is fitted and the screen reader pans.

The UI shows `text` in a read-only text field, with the caret at `caret` (keyboard §14). A routing-key press sends `braille_route { cell }`, the caret's cell. On a sparkline it moves the one cursor to the point under the cell with origin `"braille"` and replies `cursor`. Elsewhere (review mode, past the last cell, no compiled Score) the cursor stays and the reply is where it is. A `braille` move is not announced, as for the table: the display and the screen reader already show the field (INFER, OD-30).

**Settings.** `settings.braille = { mode: "review" | "sparkline", cells?: number }` holds Shift+B's mode and the display's cell count, which a web page cannot read. They are app settings, not profile fields: they belong to the user's device. `cells: 0` is refused (`SNR-APP-013`). Files without `braille` read the defaults (review, no cell count).

#### Meter frames

> Amended 2026-10-30, B01-T04, approved by the coordinator under delegation.

A meter frame is not an `Event`. The app core hands it to `EventSink::meters`, and the host sends it on the `meters` channel as 32 raw little-endian bytes, so a frame can never travel as JSON (09 §2).

| Offset | Size | Field |
|---|---|---|
| 0 | 4 | Magic `SNRM` |
| 4 | 2 | Frame version, `1` |
| 6 | 2 | Flags: bit 0 the master limiter engaged in the window; bit 1 the last frame (the transport stopped, paused or lost its device). The other bits are `0`, and readers ignore them |
| 8 | 8 | Sequence number (u64) |
| 16 | 8 | Transport position in seconds (f64) |
| 24 | 4 | Peak, left (f32, linear; `1.0` is full scale) |
| 28 | 4 | Peak, right (f32) |

* **Rate.** While playing, at most one frame every 1/30 s. When playback stops, pauses or loses its device, one frame with bit 1 set and zero peaks follows, so a meter falls to silence instead of freezing at its last peak.
* **Peaks.** Each peak is the maximum over the render graph's last complete 50 ms window (`sonarithm_render::METER_WINDOW`). That window is longer than the gap between two reads, so no peak is missed. The window runs on the audio thread with fixed state and no allocation (checked under `allocation-counter`). It reaches the control thread through the engine state's triple buffer, with no lock.
* **Versioning.** A change of size, field order or meaning increments the frame version. A new flag bit does not, because readers ignore unknown bits. Adding this frame kind did not change `PROTOCOL_VERSION` (§5).
* **Readers are strict, as for views (§6).** A buffer of another length, magic or version is refused with `SNR-APP-050` and dropped, never guessed at.
* **Accessibility.** Meter frames are never announced through live regions: they would bury every other announcement. A meter shows limiter engagement as text or an accessible state, not by colour alone (DL-X08).

Layout: `crates/sonarithm-app/src/ipc/meters.rs`. TypeScript: the generated `METERS` constants and `readMeterFrame` in `@sonarithm/ipc`.

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

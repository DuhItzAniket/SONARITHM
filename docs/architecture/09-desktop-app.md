# 09 · Desktop Application: Tauri Host, IPC and Frontend

> **Status:** Baseline v1 (2026-10-18). Built in Stage B (B01–B05). The shell choice is provisional until the A02 gate.
> **Decisions:** [ADR-0002 Tauri](../decisions/0002-desktop-shell-tauri.md), [ADR-0003 React + React Aria](../decisions/0003-frontend-react-and-react-aria.md), [ADR-0011 typed IPC](../decisions/0011-typed-ipc-ts-rs.md).
> **Design:** [design-language.md](../design/design-language.md), [keyboard-and-navigation.md](../design/keyboard-and-navigation.md).

## 1. Process and module structure

The desktop app is **one process**. The Rust core runs in-process with the Tauri host. The UI is a React application in the system webview (WebView2, WKWebView or WebKitGTK).

```text
apps/desktop/
├── src-tauri/                     # crate sonarithm-desktop
│   ├── src/main.rs                # builder, plugins, window, menus
│   ├── src/commands/              # #[tauri::command] handlers, thin: validate → call sonarithm-app
│   │   ├── dataset.rs  compile.rs  transport.rs  nav.rs  inspector.rs  export.rs  project.rs  settings.rs
│   ├── src/channels.rs            # clock anchors, meters, diagnostics, announcements
│   ├── src/a11y/                  # optional native announcement plugin (OD-05)
│   ├── capabilities/*.json        # Tauri permission sets (least privilege)
│   └── tauri.conf.json            # CSP, bundle, updater
└── src/                           # React UI
    ├── app/        # frame, regions, routing-free workspace layout, command palette
    ├── features/   # data-tree, plot, table, spec-editor, inspector, transport, diagnostics, announcements, tutorial
    ├── state/      # Zustand stores; XState machines (transport, export, validation)
    ├── ipc/        # re-exports @sonarithm/ipc (generated types + typed client)
    └── main.tsx
```

Shared packages: `@sonarithm/ui` (tokens and React Aria components), `@sonarithm/viz` (uPlot, WebGL2 layers, playhead), `@sonarithm/ipc` (generated TypeScript types and client). See [02-repository-and-crate-layout.md](02-repository-and-crate-layout.md).

## 2. IPC design

Tauri offers three primitives with very different performance ([notes/05 §1](../research/notes/05-desktop-frontend-stack.md)): **commands** (request/response), **channels** (fast, ordered streams) and **events** (JSON, not designed for throughput).

| Traffic | Primitive | Encoding | Rate |
|---|---|---|---|
| Requests (open, compile, set channel, export, nav move) | Command | JSON (serde ↔ generated TS types) | On demand |
| Bulk data (downsampled views, table pages, spectrogram tiles) | Command returning `tauri::ipc::Response` | **Binary** (little-endian typed arrays plus a small header) | On demand |
| Transport clock anchors | Channel | Compact JSON or binary | On change, plus every 2 s |
| Meters | Channel | Binary frames, batched | 30 Hz |
| Diagnostics, announcements, job progress | Channel | JSON | On change, rate-limited |
| App-wide notifications (device changed, update available) | Event | JSON | Rare |

**Rules.** No JSON events on hot paths. No per-frame IPC: the playhead is extrapolated in the UI ([06 §9](06-audio-engine.md#9-latency-and-clock-synchronization)). Every payload type is generated from Rust (ts-rs), so the frontend never hand-writes protocol types.

**Handler shape (B01-T03, as ADR-0017 left to it).** The host has four generic commands, not one per request: `ipc_dispatch` (a JSON `Request` to `App::dispatch`), `ipc_binary` (to `App::dispatch_binary`, replying with a raw `ipc::Response`), `ipc_subscribe` (a Tauri channel for one ADR-0017 channel; returns an ID) and `ipc_unsubscribe`. A request that is not a protocol message fails with the app core's `SNR-APP-009`. Commands run on Tauri's worker pool, never on the window's thread. The UI side is `createTauriTransport` in `apps/desktop/src/ipc/`; it rejects a binary reply that arrives as JSON (F-042). Tauri's per-command permissions therefore cannot tell requests apart (ADR-0017, consequences); the capability allows only these commands and no plugin.

**As built (B01-T04): hot paths.**

- **Clock anchors** are JSON on `clock`: one on every change (play, pause, stop, seek, speed, loop, device change, end of playback) and one every 2 s while playing, which a change restarts. A `Pacer` in `sonarithm-app/src/transport.rs` times both anchors and meters, and the unit tests drive it with synthetic instants.
- **Meters** are binary and never JSON. The render graph keeps the master's peak over 50 ms windows (`sonarithm_render::METER_WINDOW`) and publishes the last complete one in `GraphStats::meter`, with no allocation. A window lasts longer than the gap between two reads, so a 30 Hz reader misses no peak. While playing, the control thread sends one `MeterFrame` at most every 1/30 s through `EventSink::meters`, which is not part of the `Event` enum. When playback stops, pauses or loses its device, it sends one last frame with zero peaks and the last flag set, so meters never freeze at a peak. The host sends its 32 bytes raw on the `meters` channel. Tauri hands frames under 1 KiB to the page as an `ArrayBuffer`, and `IpcTransport.meters` reads them with `readMeterFrame`. That reader is strict: another magic or version is dropped with `SNR-APP-050`, never guessed at. Meters are never announced through live regions (a test checks that no UI code reading meters reaches the announcer), and a meter shows limiter engagement as text or an accessible state, not by colour alone. The layout is the ADR-0017 §4 amendment ("Meter frames", approved 2026-10-30); it is in `crates/sonarithm-app/src/ipc/meters.rs`, and its constants are generated as `METERS`:
  - magic `SNRM`;
  - a u16 version;
  - u16 flags (bit 0: the limiter engaged; bit 1: the last frame; other bits zero, and readers ignore them);
  - a u64 sequence number;
  - the f64 position in seconds;
  - two f32 linear peaks.
- **The rest.** Job progress stays at 10 per second per job (B01-T01), and announcements are paced by the sequencer (08 §4). The host sends no Tauri events at all: a test checks its source.
- **PB-09.** The desktop smoke run times 30 `view_get` calls of a cached 1.2 MB M4 view, from request to `ArrayBuffer`, on the three CI operating systems, and reports p50, p95 and max. CI runs a debug build on shared runners, so it reports the number but does not gate on it. The [benchmarks log](../research-paper/benchmarks-log.md) records release-build runs on HW-01.

### 2.1 Command surface (initial)

| Command | Returns | Phase |
|---|---|---|
| `dataset_open(path) / dataset_close(id)` | `DatasetSummary` (fields, inferred types, L1/L2 summary, nav root) | B01 |
| `view_get(dataset, x_range, px_width)` | Binary M4 tile | B04 |
| `table_page(dataset, offset, limit, sort)` | Binary columnar page + header | B04 |
| `spec_get / spec_apply(patch) / spec_validate(text)` | Spec, diagnostics | B05 |
| `compile(spec_id)` | `CompileReport` (diagnostics, duration, track summary) | B01 |
| `transport_play / pause / stop / seek / set_speed / set_loop` | `TransportState` (authoritative) | B01 |
| `nav_move(verb) / nav_where / nav_query(kind)` | `nav_move`: `NavResult` (the cursor, the outcome, its earcon, the highlight span, whether it was announced; the cue ID follows with the point cue). `nav_where` and `nav_query`: `announce`. They read and move the one cursor of `cursor_set`, never a second one ([ADR-0017 "Navigation"](../decisions/0017-ipc-protocol-v1.md#navigation)). `nav_query` follows in B03-T09's next change | B03 |
| `cursor_set(dataset, row, origin) / cursor_get` | `CursorState`, and a `cursor` event to every view (B04-T06) | B04 |
| `inspector_set(track, channel, value)` | `SpecPatchResult` (undo token) | B05 |
| `history_undo / history_redo` | `SpecPatchResult` | B05 |
| `export_render(options)` | Job id; progress over a channel; `ExportResult` | B05 |
| `project_open / save / save_as / recent` | `ProjectSummary` | B05 |
| `settings_get / set`, `profile_get / set / import / export` | Settings and profile | B02, B03 |
| `stream_connect / disconnect / pause` | Stream status | B06 |
| `devices_list / device_select` | `DeviceInfo[]` | B01 |
| `function_check / function_open / audify_calculate` | Canonical form and diagnostics; the function as a dataset with its critical points; the SDR-31 calculator ([ADR-0017 "Functions"](../decisions/0017-ipc-protocol-v1.md#functions)) | C03 |

**As built (B01-T01).** The protocol lives in `sonarithm-app::ipc` ([ADR-0017](../decisions/0017-ipc-protocol-v1.md)): one `Request` enum (`{"command": "<name>", "args": {...}}`), one `Reply` enum and one `Event` enum, and `App::dispatch` as the only entry point. Besides the commands above, B01 has `hello` (the version handshake), `dataset_list`, `spec_get`, `spec_set` (undoable), `transport_state`, `stop_all`, `job_status`, `job_cancel` and `job_list`; `history_undo` and `history_redo` already step through `spec_set` edits. `compile` compiles the session's spec against the open datasets, which specs name as `{"kind": "dataset", "id": "d1"}`. Spec and data problems come back as diagnostics in a `CompileReport`; `IpcError` (`SNR-APP` codes) is for commands that could not run. The transport runs on a control thread inside the app core that owns A05's `Player` (F-140), so the device opens on the first play and clock anchors, job progress, device changes and the end of playback reach the host's `EventSink` without polling. Replies, errors and events name files by their file name only, and the app core canonicalises and checks every path the UI passes (a regular data file to open, an existing folder to export into), so no user folder crosses the protocol (ADR-0017 §7).

**As built (B06-T07): live playback.** `live_start(stream)` plays an open stream through the session's spec, whose source must be a stream with a pace of `secondsPerUnit` or `pointsPerSecond` (`SNR-APP-073` otherwise). A worker thread in the app core takes a snapshot of the stream about once a second, narrows it to the spec's `window`, compiles it with `Sonarithm::compile_stream` and moves the Score onto the stream's own timeline (`Score::offset`: the first window starts at 0, and a row keeps its Score time from one window to the next). The transport's control thread swaps each Score in while playing (`Player::swap`), keeps the playhead between 4 s and 0.5 s behind the newest data (2 s when it has to move it; a proposal, TODO(verify) with HP-11), and does not stop at a window's end. Nothing sounds until `transport_play` (A11Y-15). A window that does not compile sends one `live_notice` event (`SNR-APP-074`) per distinct problem and playback goes on with the last window that did. `live_stop`, or a `compile`, leaves live mode and pauses. `TransportState` gained `live` and `xruns` (underruns since the session started). The stream keeps 100,000 rows whatever the spec says, so memory stays bounded at any rate (PB-11).

### 2.2 Versioning

The IPC protocol has a version constant shared by host and UI. Because both ship together, mismatches only happen in development. They fail fast with a clear error.

### 2.3 Electron fallback seam

The UI calls an `IpcTransport` interface (`invoke`, `subscribe`, `binary`). The Tauri implementation is the default. If the A02 gate triggers the fallback ([ADR-0002](../decisions/0002-desktop-shell-tauri.md)), an Electron implementation (Node bindings to `sonarithm-app` via napi-rs) replaces it, and the React code does not change.

**As built (B01-T02).** `@sonarithm/ipc` exports the generated protocol (`packages/ipc/src/generated/protocol.ts`: `Request`, `Reply`, `Event`, every payload type, `PROTOCOL_VERSION`, `REPLY_KINDS`, `EVENT_CHANNELS` and the `BINARY` layout), the `IpcTransport` interface, `connect(transport)`, which sends `hello` first and rejects with `ProtocolMismatchError` on a version mismatch, `createMockTransport` for tests and Storybook, and `readView`, which reads a binary view into typed arrays without copying and refuses, with `SNR-APP-050`, any header version, kind or data type it does not know. Provenance travels as its own data types (class codes named by the generated `PROVENANCE_CLASSES`, and confidences), so the UI can qualify estimates (ADR-0017 §6). `client.call(request)` resolves with the reply type of its command (`ReplyFor<C>`, generated from the Rust command table), so the hand-written part of the client is small.

## 3. Frontend architecture

| Concern | Approach |
|---|---|
| Framework | React 19.2, TypeScript strict, Vite, React Compiler |
| Components | `@sonarithm/ui` on **React Aria Components**: Tree, Table, GridList, Slider, Toolbar, Menu, Dialog, Tabs, ComboBox. No hand-rolled ARIA where RAC has a pattern |
| Layout | Fixed regions (menu/command bar + transport, data tree, centre tabs Plot \| Table \| Spec, inspector, status/diagnostics/announcement log) with resizable splitters. **No free-floating docking** as the primary layout; F6 cycles regions ([design-language](../design/design-language.md)) |
| UI state | Zustand stores (workspace, selection, panels, preferences) with transient subscriptions for hot values |
| Machines | XState v5: `transport` (mirrors the Rust authority), `export` (job lifecycle), `specValidation` |
| Authoritative state | **Rust owns** session, spec, transport and nav cursor. The UI renders projections and sends intents. This avoids split-brain state between UI and audio |
| Hot paths | Playhead and meters are drawn from `requestAnimationFrame` using refs and canvas or CSS transforms, **bypassing React reconciliation** |
| Plots | uPlot for line and time series (canvas); a custom WebGL2 layer for dense, waveform and spectrogram views; WebGPU is optional. All downsampling happens in Rust |
| Spec editor | CodeMirror 6 with the generated JSON Schema, diagnostics from Rust validation, and keyboard-accessible completion |
| Styling | CSS Modules + cascade layers + CSS custom-property tokens generated in OKLCH (`@sonarithm/ui/tokens.css`); the host sets `data-theme` on the root element from the `theme` setting, and `system` follows the OS ([design-language §5.4](../design/design-language.md#54-theme-selection-forced-colors-and-prefers-contrast)); `forced-colors`, `prefers-contrast` and `prefers-reduced-motion` honoured |
| i18n | Fluent message IDs shared with Rust announcement templates |
| Command palette | React Aria ComboBox + Menu (announced results), listing every action with its shortcut |
| Application frame | `apps/desktop/src/app/frame` (B02-T06): five landmark regions in the F6 order of design-language §4.1, keyboard splitters (`@sonarithm/ui` `Splitter`, APG window splitter), collapsed regions as an "Expand" control in the F6 cycle, and the Medium and Narrow layouts of §4.3 |
| Menu bar | **Native menus on every OS** (decided in B02-T06): the system menu bar on macOS, and the window's native menu bar on Windows and Linux, built with Tauri's menu API. React Aria Components has no menubar pattern, and native menus are what NVDA, Narrator, JAWS, VoiceOver and Orca already read. The menu lists commands from the command registry (B02-T07). Menu events reach the UI as commands, so the host keeps no command logic; if this needs a host command or an event channel beyond ADR-0017, B02-T07 proposes it to the coordinator first. Until then the app has no menu bar, and every command stays reachable from the palette (DL-P06) |

**As built (B01-T05): the transport machine.**

- `apps/desktop/src/transport/machine.ts` is the XState v5 `transport` machine, with two parallel regions.
  - The `status` region has `unknown`, `stopped`, `playing` and `paused`. It moves only when the app core reports a `TransportState`, in a command's reply or in a `clock` anchor, never on an intent.
  - The `request` region sends intents one at a time. A newer intent replaces one still waiting, and a failure keeps the last reported state.
- So the Rust state wins over any intent it did not carry out, and the machine never sends a command on its own (A11Y-15).
- `useTransport` runs one machine per connection: it asks for `transport_state` once, then follows the `clock` channel.
- The top bar's `TransportBar` has Play/Pause, Stop and Loop: one Tab stop with arrows inside. The buttons are disabled until a Score is loaded. Stop stays enabled while one is loaded, so pressing it never drops the focus.
- Play/Pause, Stop and Loop are commands (`transport.playPause`, `transport.stop` with Escape in the explorer, and `transport.loop`, unbound), so the palette reaches them.
- Seek and speed are intents of the machine; their controls come with the explorer (B03-T09) and the rate steps of keyboard §5.3.

**As built (B04-T02): the dense layer.**

- `@sonarithm/viz`'s `DenseLayer` draws three kinds of scene: waveforms (one min/max span per pixel column, from the app core's M4 tiles), spectrograms (dB bytes through a colormap texture) and lines of a million points or more.
- **WebGL2 first, Canvas2D otherwise.** WebGL2 is probed once on a throwaway canvas: a context opens and the shaders compile. A canvas that took a WebGL2 context cannot give a 2D one, so the real canvas asks for the context that works. The Canvas2D fallback reduces dense lines to M4 per pixel column, and `forceCanvas` takes it anyway.
- **Drawn once.** Positions are normalized to [0, 1] on upload, so 32-bit floats keep full precision. The data is drawn when the scene, size or theme changes, never per frame. What moves, such as the playhead (B04-T04), goes in the layer's overlay.
- **Accessible as one image.** Like `Plot`, the layer is named by the summary's L1, described by its L2, and not focusable.
- **PB-08.** The `perf` Vitest project records the script time of up to 60 zoom redraws of a million-point line, measuring for at most 8 s so a slow runner still writes the trace (it notes `cutShort`). CI's frontend-browser job reports it without gating (SwiftShader on a shared runner). B04-T08 runs the same trace in the app's own webview through the desktop smoke run (`pb08` in `smoke.json`) on all three CI hosts; the HW-01 WebView2 run is the owner's (#129).
- **Discrete GPU (F-044).** The `gpu` setting (12 §3) chooses the high-performance GPU on Windows; see the start-up arguments in §7.

**As built (B04-T05): the data table.**

- **Windows, not one collection.** React Aria's Table builds a node for every row and takes `aria-rowcount` and `aria-rowindex` from that collection, with no way to override them. A million rows cannot be one collection, and patching those attributes by hand would be hand-rolled ARIA (AGENTS rule 6). The Table tab therefore holds one window of 1,000 rows from `table_page` (`WINDOW_ROWS`, under the command's 10,000 limit), virtualized inside the window (approved by the coordinator, 2026-10-31; F-177).
- **The true position is content.** The first column is a row header holding the row's number in the whole table, in the table's current order ("500,001"), formatted for the locale. A screen reader names every cell by it. `aria-rowcount` describes the window. A visible line above the grid says which rows are loaded: "Rows 10,001 to 11,000 of 1,000,000".
- **Moving between windows.** ArrowDown or PageDown on a window's last row loads the next window and focuses its first row. ArrowUp or PageUp on the first row loads the previous window and focuses its last row. Primary+Home and Primary+End go to the first and last cell of the whole table. "Go to row…" (command palette, `table.goToRow`) loads the window that holds a row and focuses it. Every move keeps the focus in the same column, and the range is announced politely only when the window changes. A slower reply never replaces a newer window.
- **Sorting.** Enter on a field's header sorts through `table_page`'s `sort` and starts again at row 1. React Aria sets `aria-sort`, and "Sorted by temperature (°C), ascending" is announced.
- **Provenance (AGENTS rule 7).** A page with `__snr_prov` and `__snr_conf` gets a Provenance column. Estimates use speech's own messages (`announce-provenance`, `announce-provenance-confidence`, two decimals), so the table and speech say the same thing. Values stated plainly read "exact", "calibrated" or "edited". A missing value reads "no value", never a blank cell.
- **Not yet.** The Table tab shows the first open dataset until the project tree chooses one (B03). Space to play from the focused row, and the focused row as the shared cursor, are B04-T06. Primary+F search needs a search command.

**As built (B04-T06): the linked cursor.**

- **One cursor.** Rust owns it (ADR-0017 "Cursor"). The table, the plot, the readout and the breadcrumb show the same `cursor` event in the same render, so they never disagree.
- **Table.** Its focused row is the cursor (origin `table`, not announced; OD-30), and the cursor's row is shown selected. A cursor moved elsewhere loads its window and scrolls its row into view only while the focus is outside the table; it never moves the focus.
- **Plot tab.** It draws the dataset's first time or numeric field as x and the next numeric field as y, from an M4 tile (`view_get`) as wide as the plot in device pixels. The cursor is a dashed crosshair with a square ring at the point, in the selection accent (`Highlight` under forced colours), moved without redrawing the plot. If the tile does not hold the cursor's row, its two values come from a one-row `table_page`.
- **Click to move.** A click on the plot moves the cursor to the nearest drawn point (origin `plot`), which Rust announces. The plot stays one image (A11Y-01): the click is a pointer convenience, and the table and the chart explorer (B03-T09) are the keyboard ways. B04-T07 adds the other non-drag alternatives.
- **Readout.** The Status region shows "Data row N" and the breadcrumb. It is not a live region.

**As built (B04-T07): zoom and pan without dragging (A11Y-26).**

- **Drag zoom.** A drag across the plot selects an x range, and the plot zooms to it (`view_get` with that range). A press and release in place is a click, which moves the cursor; a drag never also moves it. Pointer events on uPlot's overlay read the overlay's box at each event (F-179).
- **Alternatives.** Every drag has one that needs neither a drag nor a pointer: a "Plot view" toolbar (React Aria Toolbar) with Zoom in, Zoom out, Reset zoom, Pan left and Pan right. The same commands come from the keyboard (`view.zoomIn` Ctrl+Plus, `view.zoomOut` Ctrl+Minus, `view.zoomReset` Ctrl+0; Cmd on macOS) and from the palette (`view.panLeft`, `view.panRight`, unbound).
- **Behaviour.** Zooming halves or doubles the view around the cursor when it is in view (it keeps its place), else around the middle. Panning moves a quarter of the view and stops at the data's ends. The shown range is visible text ("Showing time (s) from 0 to 99.9") and is announced politely when it changes.
- **Targets.** Every toolbar button is at least 24 × 24 CSS pixels (WCAG 2.5.8; the controls' `--row-default` and `--target-min`), checked by an E2E test.

## 4. Security model

| Control | Setting |
|---|---|
| Content Security Policy | `default-src 'self'`; no remote scripts or styles; `connect-src` limited to the IPC origins (`ipc: http://ipc.localhost`, without which binary IPC silently falls back to JSON, F-042); no `unsafe-eval` or `unsafe-inline`; `object-src`, `base-uri`, `form-action` and `frame-ancestors` closed (`apps/desktop/src-tauri/tauri.conf.json`) |
| Navigation | The host refuses every navigation away from the bundled UI's origin (`navigation.rs`, NFR-04 tests) |
| Browser accelerator keys | Off: WebView2's `AreBrowserAcceleratorKeysEnabled` is set to false and read back in the smoke test ([ADR-0025](../decisions/0025-unsafe-register-desktop-host.md)); a page-level guard covers every webview (keyboard §12.3) |
| Remote content | None. The app never loads remote URLs in the webview |
| Tauri capabilities | Minimal permission sets per window. **No shell plugin.** File system access only through user-initiated dialogs, and only for paths the user chose |
| IPC validation | Every command validates its inputs in Rust and returns typed errors. No path traversal from UI-supplied strings |
| Updates | Tauri updater with signature verification (cannot be disabled). Keys held by two custodians ([ci-cd](../engineering/ci-cd.md)) |
| Network | Off by default. Stream sources and opt-in cloud features ask for explicit, per-action consent. A **network-deny integration test** runs in CI (B01) |
| Untrusted files | Parsers are fuzzed. Importers run with size and time budgets. ML models are loaded only from the app's model directory with checksums |

## 5. Persistence

| Data | Location | Format |
|---|---|---|
| Settings | OS app-config dir (`tauri::path`) | JSON, schema-validated, versioned |
| Profiles | App-config dir `/profiles` | JSON (portable) |
| Projects | User-chosen path | `.sonarithm` bundle ([12 §4](12-cross-cutting-concerns.md#4-project-file-format), [RFC-0002](../rfcs/0002-project-format-v1.md)); the recent list in the app-config dir (`recent-projects.json`) |
| Autosave and crash recovery | App-data dir `/autosave` | The newest 5 bundles, at most one every 30 s while there are unsaved changes, 2 GiB in all (TODO(verify) in B05-T07); offered on next start; removed by a save or a clean exit |
| Logs | App-log dir | Rotating text logs, local only ([12 §2](12-cross-cutting-concerns.md#2-logging-tracing-and-observability)) |
| Model weights (D04) | App-data dir `/models` | Downloaded on explicit request, checksummed |

**As built (B05-T04).** `sonarithm-app::project` reads and writes bundles ([ADR-0018](../decisions/0018-project-format-v1.md)). A save writes the whole bundle to a temporary file, flushes it and renames it over the old one, the same crash-safe write as settings. Opening checks every member against the manifest's SHA-256 before using it, reads embedded Parquet in place from the stored member (a positional reader per column chunk; no copy of the member in memory, F-141), and opens linked data only when the link resolves, after canonicalisation, to a regular file with a data extension whose fingerprint matches; otherwise the dataset is listed as unavailable with a warning that names the file alone. The protocol names projects by file name: `project_recent` lists file names with an index, and `project_open_recent` opens by that index, so no folder reaches the UI (ADR-0017 §7). Editing commands mark the session unsaved and write an autosave when the last one is 30 s old; `App::shutdown` removes the autosaves, so only a crash leaves them for `project_recovery`.

## 6. Packaging

| OS | Artifacts | Signing (C06) |
|---|---|---|
| Windows | NSIS installer + MSI | Azure Artifact Signing if eligible, otherwise an OSS signing service ([ci-cd](../engineering/ci-cd.md)) |
| macOS | DMG + app bundle (universal) | Developer ID + notarization |
| Linux | AppImage, .deb, .rpm; Flatpak later | AppImage GPG signature |

## 7. Startup sequence (target ≤ 2 s cold)

1. Rust initializes the logger, settings, the profile and the audio backend (device opened lazily on first play, so start-up is never blocked by audio drivers).
2. The webview loads the bundled UI. The UI requests the session state.
3. If autosave exists, recovery is offered. Otherwise the app opens the last project or an **empty workspace** with keyboard focus in the data tree. There is no splash screen and no marketing panel ([design-language](../design/design-language.md)).
4. The first-run experience offers the built-in tutorial (B07) as a single, dismissible, accessible prompt.

**Start-up arguments (B04-T02).**

- With `gpu` set to `highPerformance`, the host builds the window with WebView2 browser arguments: wry's defaults (`--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection`) plus `--force_high_performance_gpu`, because passing arguments replaces the defaults.
- **Fallback.** WebView2 refuses arguments that differ from a running instance's on the same user-data folder: a second window, or the last run still closing. If creating the window fails with the switch, the host builds it once more without it. The UI then says, and shows in the Status region, that the setting applies after a full restart. The setting never stops the app from starting.
- The start-up facts reach the UI through the window's initialization script (`window.__SONARITHM_START__`), as the smoke configuration does.
- **Smoke runs never hang silently.** A classic script, `public/smoke-guard.js`, loads before the app's modules. In a smoke run it sends a failing report if a page error comes before the run starts, or if the run has not started after 60 s (a module that fails to load runs no code). Once `runSmoke` starts, its watchdog fails the run after 120 s and names the check still waiting, with the checks done so far. CI's step limit is 5 minutes.

**As built (B01-T08): devices, volume and stop all audio.**

- **Stop all audio.** `transport.stopAll` (Ctrl+Period, Cmd+Period) calls `stop_all` from every region and inside dialogs; it stops sonification, earcons and self-voicing, and never moves the focus. In the render graph it acts in the next block and fades over 20 ms (sound-design §5). Output is silent after one block, the fade and the limiter's 2.1 ms look-ahead, about 28 ms at 48 kHz and 256 frames: inside T-13's interaction budget, though the plan's "within one audio block" wording is stricter.
- **Preferences.** `app.preferences` (Ctrl+Comma, Cmd+Comma) opens a Preferences dialog. Its Audio section picks the output device (`devices_list`, `device_select`) and sets the sonification volume from −60 to 0 dB (`settings_set`), said in decibels.
- **The volume is SONARITHM's own level.** It is the render graph's master gain, and no audio backend has a system-volume API; the dialog says so in words.
- **Device messages.** Device changes and losses are app-core announcements (Fluent `announce-audio-device-changed`, `announce-audio-lost`), so the existing announcement feed speaks and logs them.

**As built (B01-T07).**

- **Start-up never waits for a driver.** The app core loads its settings and answers requests without calling the audio driver; the device opens on the first play. A scenario checks this with a null driver that takes 2 s per call: start-up and the first requests finish sooner, and only the first play waits.
- **Focus on start.** Once the app core answers, the focus lands in the project region: on its first control, which is the project tree once it holds one, otherwise on the region itself. In Narrow it lands on the workspace, which is shown first (design-language §4.3). It moves only if nothing has the focus yet. There is no splash and no dialog in the way.
- **PB-07.** The smoke run measures from the host's start to the first focus in a region, and reports it as `pb07` without gating. ci.yml runs a debug build, and desktop-nightly a release build of the installed app. The origin is read before the smoke fixture is written, so the number is an upper bound, and the operating system's process creation before `main` is not counted.
- **App Nap** (macOS) handling for the control and telemetry threads waits on HP-03, which checks whether F-041's message timing suffers under App Nap.

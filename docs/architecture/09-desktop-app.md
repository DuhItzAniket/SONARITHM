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

### 2.1 Command surface (initial)

| Command | Returns | Phase |
|---|---|---|
| `dataset_open(path) / dataset_close(id)` | `DatasetSummary` (fields, inferred types, L1/L2 summary, nav root) | B01 |
| `view_get(dataset, x_range, px_width)` | Binary M4 tile | B04 |
| `table_page(dataset, offset, limit, sort)` | Binary columnar page + header | B04 |
| `spec_get / spec_apply(patch) / spec_validate(text)` | Spec, diagnostics | B05 |
| `compile(spec_id)` | `CompileReport` (diagnostics, duration, track summary) | B01 |
| `transport_play / pause / stop / seek / set_speed / set_loop` | `TransportState` (authoritative) | B01 |
| `nav_move(verb) / nav_where / nav_query(kind)` | `NavResult` (node, highlight, announcement, cue id) | B03 |
| `inspector_set(track, channel, value)` | `SpecPatchResult` (undo token) | B05 |
| `history_undo / history_redo` | `SpecPatchResult` | B05 |
| `export_render(options)` | Job id; progress over a channel; `ExportResult` | B05 |
| `project_open / save / save_as / recent` | `ProjectSummary` | B05 |
| `settings_get / set`, `profile_get / set / import / export` | Settings and profile | B02, B03 |
| `stream_connect / disconnect / pause` | Stream status | B06 |
| `devices_list / device_select` | `DeviceInfo[]` | B01 |

### 2.2 Versioning

The IPC protocol has a version constant shared by host and UI. Because both ship together, mismatches only happen in development. They fail fast with a clear error.

### 2.3 Electron fallback seam

The UI calls an `IpcTransport` interface (`invoke`, `subscribe`, `binary`). The Tauri implementation is the default. If the A02 gate triggers the fallback ([ADR-0002](../decisions/0002-desktop-shell-tauri.md)), an Electron implementation (Node bindings to `sonarithm-app` via napi-rs) replaces it, and the React code does not change.

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
| Styling | CSS Modules + cascade layers + CSS custom-property tokens generated in OKLCH; `forced-colors`, `prefers-contrast` and `prefers-reduced-motion` honoured |
| i18n | Fluent message IDs shared with Rust announcement templates |
| Command palette | React Aria ComboBox + Menu (announced results), listing every action with its shortcut |

## 4. Security model

| Control | Setting |
|---|---|
| Content Security Policy | `default-src 'self'`; no remote scripts or styles; `connect-src` limited to the IPC origin; no `unsafe-eval` |
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
| Projects | User-chosen path | `.sonarithm` bundle ([12 §4](12-cross-cutting-concerns.md#4-project-file-format)) |
| Autosave and crash recovery | App-data dir `/autosave` | Last N snapshots; offered on next start |
| Logs | App-log dir | Rotating text logs, local only ([12 §2](12-cross-cutting-concerns.md#2-logging-tracing-and-observability)) |
| Model weights (D04) | App-data dir `/models` | Downloaded on explicit request, checksummed |

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

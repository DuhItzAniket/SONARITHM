---
status: accepted (provisional)
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0002 · Tauri 2 as the desktop shell, Electron as fallback

## Context and problem statement

The desktop app needs a GUI that blind and low-vision screen-reader users can operate fully on Windows, macOS and Linux, while sighted analysts get a dense, fast, professional UI. Which shell?

## Decision drivers

* **Screen-reader accessibility on all three OSes** (the deciding criterion: A11Y-17, A11Y-18, A11Y-32).
* In-process Rust core with low-latency IPC.
* Rendering performance for 1M-point plots and spectrograms.
* Security model, signed updates, installer size.

## Considered options

* Tauri 2.12 (system webviews + Rust)
* Electron 44 (bundled Chromium + napi-rs bindings)
* Native Rust GUI: egui, iced, Slint, gpui, Dioxus native, Makepad, Xilem, Freya

## Decision outcome

Chosen option: **Tauri 2.12**, with **Electron as the documented fallback**. Webviews inherit mature platform accessibility trees and the whole ARIA pattern ecosystem. Native Rust GUIs are not yet good enough with screen readers ([notes/05 §1](../research/notes/05-desktop-frontend-stack.md)):

* egui has AccessKit but no live regions.
* Slint exposes text inputs only partly, and is GPLv3 or commercial.
* iced and released gpui have no screen-reader support.

### Consequences

* Good: the best available accessibility baseline; the Rust core stays in-process; capability-scoped security; signed updater; small installers.
* Bad: Tauri's accessibility is **inherited, not owned** (tracking issue #207 has been open since 2019).
* Bad: **Linux WebKitGTK** performance and GPU issues (an anecdote of about 40 vs 240 fps against Electron; NVIDIA DMABUF workarounds).
* Bad: Tauri events are JSON and slow, so hot paths must use Channels and binary responses ([09 §2](../architecture/09-desktop-app.md#2-ipc-design)).
* Neutral: the frontend talks to an `IpcTransport` interface, so a switch to Electron does not touch React code.

### Confirmation

The **A02 prototype gate** measures:

1. Channel and binary IPC latency and throughput on all three OSes (PB-09);
2. WebGL2 1M-point waveform and spectrogram frame rates in WebKitGTK (Intel, AMD, NVIDIA), WebView2 and WKWebView (PB-08);
3. NVDA, VoiceOver and Orca walkthroughs of a React Aria Tree, Slider and Toolbar inside a Tauri window, including browse/focus-mode behaviour.

**If (2) or (3) fails on Linux and cannot be mitigated within one week, switch to Electron** and record the outcome as ADR-0002a. Results go in `docs/sdlc/reviews/A02-prototype-gate.md`.

## Pros and cons of the options

### Electron 44
* Good: identical Chromium rendering and accessibility on every OS; best Linux performance.
* Bad: about 100 MB larger installers; a heavier memory footprint; Rust only via Node bindings (napi-rs).

### Native Rust GUI
* Good: one language, GPU rendering, no webview variance.
* Bad: screen-reader support is immature across the board, and AccessKit itself has a bus factor of 2. Re-evaluate in 2027 for a possible "lite" build.

## More information

Revisit when AccessKit integrations mature (live regions, rich text) or when Tauri's CEF runtime stabilizes.

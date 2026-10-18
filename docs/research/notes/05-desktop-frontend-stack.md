# Desktop Shell, Frontend Stack and Visual Design for SONARITHM (as of 18 October 2026)

Scope: GUI architecture for SONARITHM, an open-source app with a Rust core that sonifies graphs and multivariate data. Blind and low-vision screen-reader users are a core audience, alongside scientists, analysts and educators. Screen-reader accessibility on Windows, macOS and Linux is treated as a deciding criterion throughout.

Version legend: "verified" means confirmed on an official or primary page during this research. "Secondary" means it came from blogs or aggregators, so treat it with caution. "Outdated" marks sources that have been superseded.

---

## 1. Shell options compared: Tauri v2 vs Electron vs native Rust GUIs (with screen-reader accessibility as the deciding criterion)

### Takeaway
Only two families give SONARITHM production-grade screen-reader support on all three desktop OSes in October 2026: webview shells (Tauri 2.12.x, Electron 44/45, Dioxus desktop) and, at a lower ceiling, egui or Slint through AccessKit. A webview shell inherits the browser engines' mature accessibility trees plus the whole ARIA/APG ecosystem for complex widgets (trees, grids, sliders, toolbars). The native Rust toolkits make you hand-build accessibility for every custom widget, and several (iced, released gpui, Blitz, Makepad) have none or only pre-release support. Tauri 2.12 is the best fit for a Rust core. Its main risk is Linux (WebKitGTK performance and consistency), and Electron is the hedge for that.

### Cited Findings

**Tauri v2: versions and release status**
- Tauri 2.12.0 was released on 26 Sep 2026 and is described as "the biggest update so far in the 2.x releases". It adds a webview permission-request handler (`on_permission_request`) and Explicit Resource Management (`using`) for JS `Resource` objects. It drops official Windows 7 support because the latest WebView2 installer crashes on Win7, and raises the MSRV to Rust 1.90. — [Announcing Tauri 2.12](https://v2.tauri.app/blog/tauri-2.12/)
- Patch releases followed: 2.12.1 (30 Sep 2026; fixes the need for the macOS private-API flag for transparency/fullscreen) and 2.12.3 (10 Oct 2026, the latest patch listed). — [Tauri Core Releases](https://tauri.app/release/core/); [tauri@2.12.0](https://v2.tauri.app/release/tauri/v2.12.0/)
- OUTDATED: Wikipedia still lists Tauri 2.10.1 (March 2026) as the latest stable. — [Wikipedia: Tauri](https://en.wikipedia.org/wiki/Tauri_(software_framework))
- Tauri 2.0 stable (Oct 2024) added iOS and Android targets to the desktop ones. — [Tauri 2.0 Stable Release](https://v2.tauri.app/blog/tauri-20/)
- 2.12's mobile changes include faster dev-server page loading, `$VIDEO`/`video_dir()` moving to the app-specific Movies folder (a breaking path change), and an Android template upgrade to Gradle 9, Kotlin 2 and targetSdk 37. — [Announcing Tauri 2.12](https://v2.tauri.app/blog/tauri-2.12/)

**Tauri IPC model (commands, channels, events) and throughput**
- **Events**: Event payloads "are always JSON strings", which makes them "not suitable for bigger messages". Events are "not designed for low latency or high throughput situations", are not strongly typed, and are not governed by the capabilities system. With async listeners, rapid events may be processed out of order. — [Tauri: Calling the Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/)
- **Channels**: They are designed to be "fast and deliver ordered data", and Tauri uses them internally for download progress, child-process output and WebSocket messages. They are the recommended mechanism for streaming. — [Tauri: Calling the Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/)
- **Commands**: Return values are JSON-serialized by default, which "can slow down your application if you try to return a large data such as a file". Wrapping array buffers in `tauri::ipc::Response` is "an optimized way" to return binary. Commands can also receive raw `ArrayBuffer`/`Uint8Array` request bodies. — [Tauri: Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/)
- No official Tauri benchmark for Channel throughput or latency was found. A Tauri discussion notes that v2 binary IPC still goes through the fetch API. One non-scientific community test measured about 5 ms for 10 MB on macOS versus about 200 ms on Windows, and the thread notes only the Windows webview supports shared memory. — [tauri-apps discussion #11915](https://github.com/orgs/tauri-apps/discussions/11915)
- A third-party benchmark (excluding the WebView bridge) put Tauri `invoke()` dispatch at 722 ns for 25 B and 2.27 ms for 64 KB, versus a binary dispatcher at 80 ns and 202 µs. The author attributes small-payload cost to WebView transport and the 1 KB+ cost to JSON. — [tauri-conduit](https://github.com/userFRM/tauri-conduit) (third-party; methodology not reviewed)

**Tauri security, updater and signing**
- Capabilities bind sets of permissions (per plugin and per command) to specific windows or webviews. Frontend code can only call what the capabilities grant. — [Tauri: Capabilities](https://v2.tauri.app/security/capabilities/)
- The official updater plugin requires signed update artifacts, and signature verification cannot be disabled. — [Tauri: Updater plugin](https://v2.tauri.app/plugin/updater/)
- Platform code-signing guides cover Windows and macOS (including notarization). — [Tauri: Windows code signing](https://v2.tauri.app/distribute/sign/windows/); [Tauri: macOS code signing](https://v2.tauri.app/distribute/sign/macos/)

**Tauri accessibility (deciding criterion)**
- The Tauri accessibility tracking issue #207, opened Dec 2019, is still open, with no maintainer statement and no official accessibility docs. A blind developer commented in May 2024 that Tauri is "about as accessible as the underlying webview and native controls such as file dialogs". — [tauri-apps/tauri #207](https://github.com/tauri-apps/tauri/issues/207)
- In Sep 2026, a commenter on #207 shared two third-party plugins:
  - **Tauri Plugin Prism** provides native screen-reader speech output. Its author argues that ARIA live regions are too slow for rapid feedback. Tested on Linux, macOS, iOS and Windows, but not Android.
  - **Tauri Plugin DirectTouch** detects VoiceOver and defines direct-touch regions on iOS 17+.
  — [tauri-apps/tauri #207](https://github.com/tauri-apps/tauri/issues/207)
- A 2026 GitHub ecosystem survey calls Tauri the "best free baseline in the study" for accessibility, because the webview exposes HTML semantics to platform accessibility APIs. — [GoldStrikeArch rust-gui-desktop-ecosystem-state](https://github.com/GoldStrikeArch/rust-gui-desktop-ecosystem-state/blob/main/report/data/stack-rows.md)
- FACT-CHECK: That survey cites Tauri issue #4315 as "Linux AT-SPI gaps". The issue is actually a "Could not determine the accessibility bus address" warning, mostly harmless, sometimes with a blank window in containers or NixOS. It is not a documented Orca usability defect. — [tauri-apps/tauri #4315](https://github.com/tauri-apps/tauri/issues/4315)

**Electron**
- Release cadence and support:
  - Electron 44.0.0 went stable on 25 Aug 2026 (Chromium M152, Node 24.18.1).
  - Electron 45.0.0 is scheduled for stable on 20 Oct 2026 (M156).
  - Releases ship on an 8-week cadence, and the latest three majors are supported.
  — [Electron release schedule](https://releases.electronjs.org/schedule); [Electron timelines](https://www.electronjs.org/docs/latest/tutorial/electron-timelines)
- Electron's accessibility is Chromium's accessibility tree on every OS. Its docs describe enabling it automatically when assistive technology is detected, or manually via `app.setAccessibilitySupportEnabled`. — [Electron: Accessibility](https://www.electronjs.org/docs/latest/tutorial/accessibility)
- Linear's desktop client is Electron, and Linear designs navigation to work "as a native Electron app and in a browser". — [Linear: How we redesigned the Linear UI](https://linear.app/now/how-we-redesigned-the-linear-ui)

**Native Rust GUIs: AccessKit status**
- AccessKit's own site lists Bevy, egui, Freya, Slint and Xilem as integrations. Adapters exist for Windows, macOS and Unix (AT-SPI), plus `accesskit_winit`. — [accesskit.dev](https://accesskit.dev/)
- The current crate (0.25.0) also has an Android adapter. — [docs.rs accesskit 0.25.0](https://docs.rs/crate/accesskit/0.25.0)
- NLnet has funded AccessKit iOS work. — [NLnet: AccessKit-iOS](https://nlnet.nl/project/AccessKit-iOS/)
- GTK merged an AccessKit backend in 4.18, which is the first time GTK apps can be accessible on Windows and macOS. — [GTK blog: An accessibility update (May 2025)](https://blog.gtk.org/2025/05/12/an-accessibility-update/)
- Per-toolkit status below is from the GoldStrikeArch 2026 survey (a secondary but evidence-based GitHub report; issue and PR numbers are given there). — [stack-rows.md](https://github.com/GoldStrikeArch/rust-gui-desktop-ecosystem-state/blob/main/report/data/stack-rows.md)
  - **egui 0.35.0 (2026-06-25)**
    - AccessKit is mandatory and on by default in eframe.
    - Windows UIA, macOS, Linux AT-SPI and Android adapters are at "rough feature parity". Web has only an experimental built-in screen reader.
    - Gaps include live regions (#2647) and `labelled_by` panics (#3647).
  - **Slint 1.17.1 (2026-07-07)**
    - AccessKit is on by default, with `accessible-*` DSL properties. IME works, including macOS CJK.
    - Text-input widgets are incompletely exposed (#2895, #8732), and RTL is essentially unsupported (#1317).
  - **iced 0.14.0 (2025-12-07)**: No AccessKit in stable. Issue #552 has been open since 2020. PR #3281 was closed unmerged on 2026-03-14 by hecrj, who said he would do it himself. System76's pop-os/iced fork ships `iced_accessibility`. There is no Tab navigation (#489).
  - **gpui**: Released 0.2.2 has zero accessibility (#41138). AccessKit was merged on main on 2026-05-27 (PR #56065) but is not yet on crates.io, and Zed's own components are not annotated.
  - **Dioxus 0.7.9 desktop**: Accessibility is good today because it uses the webview accessibility tree. **Blitz** (Dioxus native; blitz-dom 0.3.0-alpha.6) uses AccessKit but is pre-alpha, "not yet recommended", and has essentially one maintainer.
  - **Xilem 0.4.0 (2025-10-29)**: AccessKit is wired through masonry and parley (described as the deepest in the ecosystem), but screen-reader polish is undocumented.
  - **Freya 0.4.0**: AccessKit in the default tree with per-element roles. Screen-reader depth was not exercised.
  - **Ecosystem risk**: AccessKit has a "bus factor 2 and lapsed grant funding", and Linux AT-SPI support is weak.
- Slint licensing: GPLv3 for open-source apps. Royalty-free and commercial licenses cover proprietary desktop apps. — [Slint pricing/licensing](https://slint.dev/pricing)
- Pro-audio precedent for screen-reader support in a dense instrument UI:
  - Ableton Live 12 added screen-reader support on macOS and Windows. Ableton tests with VoiceOver on macOS and with NVDA, Narrator and JAWS on Windows.
  - Gaps remain, for example Wavetable's modulation matrix, automation editing and Max for Live.
  — [Ableton: Accessibility in Live Overview](https://help.ableton.com/hc/en-us/articles/11550373507868); [Live 12 manual: Accessibility and Keyboard Navigation](https://www.ableton.com/en/live-manual/12/accessibility-and-keyboard-navigation); [MusicRadar](https://www.musicradar.com/news/ableton-live-accessibility-revolution)

### Inferences
- **Why webview wins for SONARITHM.** SONARITHM's UI is dominated by complex widgets: a data/graph tree, tables of variables, mapping sliders, a transport toolbar, menus, dialogs and a spec editor. On the web, each has a WAI-ARIA APG pattern and battle-tested implementations ([WAI-ARIA APG](https://www.w3.org/WAI/ARIA/apg/)). In AccessKit toolkits you would author roles, states, actions and keyboard models yourself, and egui's missing live-region support (#2647) directly hurts sonification-status announcements.
- **Tauri vs Electron for the Rust core.**
  - Tauri lets the Rust core live in-process, with typed commands, channels and the capability security model. Electron would need the core as a Node native addon (e.g., napi-rs) or a sidecar process, which adds a layer.
  - Electron's advantage is identical Chromium rendering and accessibility on Linux. Choose Electron only if Linux/Orca parity or WebKitGTK performance proves blocking in prototype testing.
- **Hedge.** Keep the frontend shell-agnostic: a thin `bridge` module wraps `invoke` and `Channel`. That way a later move to Electron, or to Tauri's experimental CEF runtime, is a port, not a rewrite.
- **Streaming design for ~60 Hz playhead and meters.**
  - Do not emit per-frame events. Send a transport "anchor" (sample position, audio-clock timestamp, rate, output latency) over a Channel only on state changes. The frontend then extrapolates the playhead in `requestAnimationFrame`, which removes IPC jitter from the visual path.
  - Batch meter values into compact binary frames over a Channel at 30–60 Hz.
  - Ship large datasets and downsampled tiles as `ipc::Response` ArrayBuffers, never JSON.
  - Benchmark Windows specifically, given the anecdotal 10 MB/200 ms figure.
- **Native Rust re-evaluation.** It is worth revisiting in 2027–28. Slint and egui could become viable for a secondary "lite" or embedded build, but not for the primary accessible UI today.
- **Direct speech output (Prism-style).** Rapid sonification feedback, such as "value 3.2, peak", may need direct screen-reader speech in addition to ARIA live regions. This is a reason to stay on a shell with a plugin ecosystem.

### Gaps
- No independent, reproducible benchmark of Tauri Channel latency or throughput (messages/s, jitter) was found for any OS. SONARITHM must measure it.
- No first-hand 2026 reports were found of NVDA/JAWS/VoiceOver/Orca testing of Tauri apps specifically, beyond the blind developer's comment on #207.
- No evidence found either way on Makepad's accessibility or AccessKit integration.
- I could not open the 2026 Rust GUI survey at blog.wybxc.cc (HTTP 403).
- Release notes for Tauri 2.12.2/2.12.3 were not inspected.

---

## 2. Webview realities: WebView2 (Windows), WKWebView (macOS), WebKitGTK (Linux)

### Takeaway
On Windows and macOS the system webviews are modern and well accessible: WebView2 is evergreen Chromium, and WKWebView is the Safari engine, with WebGPU in Safari 26. Linux's WebKitGTK is the weak link. It is markedly slower in reported cases, has GPU-driver (especially NVIDIA/DMABUF) issues needing environment-variable workarounds, and has no confirmed WebGPU. Design for WebGL2 + Canvas2D as the baseline, treat WebGPU as an optional accelerator, and test Linux early.

### Cited Findings

**WebKitGTK (Linux)**
- WebKitGTK 2.46 (Sep 2024) switched 2D rendering from Cairo to Skia and made GPU rendering the default. — [WebKitGTK 2.46.0 release](https://webkitgtk.org/2024/09/17/webkitgtk2.46.0-released.html); [Igalia: Graphics improvements in 2.46](https://blogs.igalia.com/carlosgc/?p=1029); [Phoronix](https://www.phoronix.com/news/WebKitGTK-2.46-Skia)
- WebKitGTK 2.52.0 (18 Mar 2026) brought text blending in linear color space ("make text look like in other browsers"), GPU-dependent tile sizes, better composition scheduling, and faster accelerated 2D canvas via batched replay. The 2.52.x series continued through at least 2.52.6. — [WebKitGTK 2.52.0 released](https://webkitgtk.org/2026/03/18/webkitgtk2.52.0-released.html); [Debian changelog 2.52.6](https://tracker.debian.org/media/packages/w/webkit2gtk/changelog-2.52.6-1)
- Tauri's official Linux graphics page documents driver conflicts, mostly on NVIDIA, with environment-variable workarounds such as `WEBKIT_DISABLE_DMABUF_RENDERER=1` that trade speed for stability. — [Tauri: Linux Graphics Issues](https://v2.tauri.app/develop/debug/linux-graphics/)
- One July 2026 report says a Tauri app reached about 40 fps on Ubuntu WebKitGTK versus about 240 fps after porting to Electron (anecdote, not a benchmark).
  - A maintainer says Chromium via CEF is "not off the table" but has no ETA, and gtk4 and CEF work has begun.
  - A March 2026 reply calls the CEF branch "very experimental".
  — [tauri-apps discussion #8524](https://github.com/orgs/tauri-apps/discussions/8524)
- Tauri hosts `cef-rs` Rust bindings to the Chromium Embedded Framework. A March 2026 issue requests navigation APIs for CEF-backed webviews. — [cef-rs (via docsearch)](https://docsearch.algolia.com/mcp/docs/repo/tauri-apps/cef-rs); [tauri issue 14661 mirror](https://hub.harborframework.com/tasks/abundant/tauri-apps__tauri-14661)
- A 2023 Tauri issue says the Linux and macOS webviews expose no way to pass browser flags such as WebGPU enablement. Possibly outdated. — [tauri-apps/tauri #6381](https://github.com/tauri-apps/tauri/issues/6381)

**WKWebView (macOS)**
- Safari 26 (macOS Tahoe 26, Sep 2025) enabled WebGPU by default on all Apple platforms. WebKit says WebGPU "supersedes WebGL" on Apple platforms. — [Privacy Guides summary of WebKit WWDC25 post](https://discuss.privacyguides.net/t/news-from-wwdc25-webkit-in-safari-26-beta/28226); [Cinevva news](https://app.cinevva.com/news/2025-09-15-safari-webgpu)

**WebView2 (Windows)**
- Chrome shipped WebGPU by default in Chrome 113 on Windows, macOS and ChromeOS. — [Chrome for Developers: WebGPU release](https://developer.chrome.com/blog/webgpu-release)
- WebView2 Evergreen auto-updates with Edge, while a Fixed Version runtime does not. — [Microsoft Edge WebView2](https://developer.microsoft.com/microsoft-edge/webview2)

### Inferences
- **WebGPU availability.**
  - **Windows**: WebGPU should be available in Evergreen WebView2, which is Chromium ≥113. Verify in-app.
  - **macOS 26+**: WebGPU should be available in WKWebView. On macOS 13–15, WKWebView lacks it because WKWebView's engine is tied to the OS version.
  - **Linux**: Assume no WebGPU.
  - Implication: a WebGPU-only renderer would exclude Linux and older macOS. Use WebGL2 as primary and WebGPU as progressive enhancement.
- **Fonts.** Font rendering differs by platform: DirectWrite (Windows), Core Text (macOS), and FreeType/Skia with linear blending since 2.52 (Linux). Bundle the UI fonts as WOFF2 rather than relying on system fonts, and use `font-variant-numeric: tabular-nums` so number widths are identical everywhere.
- **Accessibility differences across webviews.**
  - Windows: WebView2 exposes Chromium's UIA/IA2 tree, the best NVDA/JAWS path.
  - macOS: WKWebView maps to NSAccessibility for VoiceOver.
  - Linux: WebKitGTK maps to AT-SPI for Orca.
  - Each needs its own manual test pass, because ARIA support differs per engine.
- **Forced colors.** `forced-colors` is primarily a Windows High Contrast concept (Chromium/WebView2). macOS exposes `prefers-contrast: more` instead. Support both media queries. — [MDN forced-colors](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/forced-colors); [MDN prefers-contrast](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-contrast)

### Gaps
- No primary 2026 source confirms WebGPU status in WebKitGTK 2.52, or WKWebView parity with Safari 26 for WebGPU. Test directly.
- No quantified WebGL2/Canvas2D benchmark comparing the three webviews in 2026 was found.
- No primary source found on 2026 WebKitGTK ATK-vs-AT-SPI accessibility backend changes or Orca quality inside WebKitGTK.

---

## 3. Frontend framework for a dense pro tool: React 19 vs SolidJS vs Svelte 5 vs Vue 3

### Takeaway
Solid and Svelte 5 lead raw benchmarks, but the hot paths in SONARITHM (playhead, meters, plots) should bypass any framework's render cycle anyway, using canvas, WebGL and `requestAnimationFrame`. The deciding factor is therefore accessibility tooling and ecosystem depth. React 19.2 + React Aria Components is the most complete accessible-widget ecosystem. Solid is mid-transition to 2.0 (RC only), and Vue 3.6 Vapor is still RC.

### Cited Findings

**React**
- React 19.2 shipped on 1 Oct 2025 with `<Activity>` (keeps hidden UI state while deferring updates) and `useEffectEvent`. — [React 19.2 blog](https://react.dev/blog/2025/10/01/react-19-2)
- React Compiler is reported as stable. — [LogRocket](https://blog.logrocket.com/react-19-2-is-here/) (secondary)
- A secondary source claims React 19.3.0 shipped 9 Sep 2026. — [tech-insider](https://tech-insider.org/solidjs-vs-react-vs-svelte-2026/) (UNVERIFIED)
- React Server Components were affected by CVE-2025-55182 in 19.0.0–19.2.2 per a third-party guide; verify against official advisories. Client-only desktop apps are less exposed. — [MakerKit](https://makerkit.dev/blog/md/tutorials/react-19-2)

**SolidJS**
- Solid 2.0 skipped alpha and went to beta, then to release candidate (RC announced 13 Aug 2026; rc.14 is the newest seen). There is no stable 2.0 yet. APIs differ from 1.x (split-phase `createEffect`, draft-first store setters). — [Solid v2.0.0-beta.0](https://github.com/solidjs/solid/releases/tag/v2.0.0-beta.0); [solid releases](https://github.com/solidjs/solid/releases); [socket.dev rc.14](https://socket.dev/npm/package/solid-js/overview/2.0.0-rc.14); [InfoQ May 2026](https://www.infoq.com/news/2026/05/solidjs-2-async/)

**Vue**
- Vue 3.6 (Vapor Mode) is still pre-release: v3.6.0-rc.9 on 18 Sep 2026 and rc.10 on 30 Sep 2026. The npm `latest` tag is 3.5.43. — [Vue 3.6 status summary](https://blog.codercops.com/blog/vue-36-vapor-mode-no-virtual-dom-2026); [versionlog](https://versionlog.com/vuejs/3.6/) (secondary; check [vuejs/core releases](https://github.com/vuejs/core/releases))

**Svelte**
- Svelte 5.57.0 (28 Aug 2026) is reported by a secondary source. — [tech-insider](https://tech-insider.org/solidjs-vs-react-vs-svelte-2026/) (UNVERIFIED)

**Benchmarks and ecosystem**
- Secondary summaries of js-framework-benchmark give geometric means relative to vanilla JS:
  - Solid ≈1.02–1.08×
  - Svelte 5 ≈1.07–1.10×
  - React 19 without the compiler ≈2.05×
  - Vue 3 ≈1.30×, with Vapor described as "competitive with Solid"
  — [pkgpulse](https://www.pkgpulse.com/guides/solidjs-vs-svelte-5-vs-react-reactivity-2026); [tech-insider](https://tech-insider.org/solidjs-vs-react-vs-svelte-2026/). Verify at the [official js-framework-benchmark results](https://krausest.github.io/js-framework-benchmark/).
- Weekly npm downloads are cited as roughly React 190M, Svelte 2.7M, Solid 1.5M. Solid topped State of JS satisfaction again in 2025. — [tech-insider](https://tech-insider.org/solidjs-vs-react-vs-svelte-2026/) (secondary)
- Zustand documents "transient updates" (subscribing to a store outside React render) for frequent state changes. — [Zustand README](https://github.com/pmndrs/zustand)

### Inferences
- **React 19.2 + TypeScript + Vite** is the recommendation for SONARITHM.
  - React Aria Components (section 4) is decisive for screen-reader users.
  - The ecosystem covers docking (dockview-react), editors (CodeMirror wrappers), cmdk and Storybook.
  - Contributor familiarity matters for an open-source project.
  - For 60 fps paths, write directly to canvas and WebGL from rAF loops, reading Zustand via `subscribe` or refs, so React re-renders only on discrete state changes.
- **Solid** would be the choice if raw DOM update performance dominated. Its accessible-component layer (Kobalte/Ark) is thinner than React Aria for trees and grids, and 2.0 API churn is imminent.
- **Svelte 5** (Bits UI, Ark Svelte) is viable but has a smaller pro-tool ecosystem.
- **Vue 3.5** is stable but Vapor is not yet. No clear advantage here.

### Gaps
- Official js-framework-benchmark numbers for the current versions were not retrieved, so the figures above are secondary.
- React 19.3 and Svelte 5.57 version claims are unverified on primary pages.

---

## 4. Accessible headless component libraries

### Takeaway
React Aria Components (Adobe) is the strongest choice for SONARITHM's complex widgets. It is the only mainstream headless library with production Tree, Table (with expandable rows), GridList, Virtualizer, accessible drag-and-drop, Slider, Toolbar, Menu and Dialog, plus extensive i18n and Adobe-funded screen-reader testing. Base UI 1.x is a solid, actively maintained second choice for simpler primitives but has no Tree or Grid. Radix's momentum has slowed. Ark UI/Zag is the best cross-framework option and has a Tree View. Kobalte and Bits UI are framework-specific and self-described APG-aligned, without independent audits.

### Cited Findings

**React Aria Components**
- Components include Slider, Table (column resizing), Menu, Dialog, Modal, Popover, ComboBox, Select, ListBox, DatePicker and Calendar.
- They are "extensively tested using many popular screen readers and devices" (readers not named).
- i18n covers 30+ languages, 13 calendar systems, 5 numbering systems and RTL.
- Drag-and-drop is accessible.
— [react-aria.adobe.com](https://react-aria.adobe.com/)
- Tree and Virtualizer reached GA in the 5 Mar 2025 release. — [React Spectrum release 2025-03-05](https://react-spectrum.adobe.com/releases/2025-03-05.html)
- Tree drag-and-drop shipped 5 Jun 2025 (also works with Table and GridList). — [React Spectrum release 2025-06-05](https://react-spectrum.adobe.com/releases/2025-06-05.html)
- RAC v1.17.0 added expandable Table rows (file-tree layouts), Virtualizer window scrolling, horizontally virtualized GridList/ListBox, and "90% fewer dependencies". — [RAC v1.17.0 release](https://react-aria.adobe.com/releases/v1-17-0)
- RAC has been stable since Dec 2023 and is Adobe-funded. — [Untitled UI comparison](https://www.untitledui.com/blog/base-ui-vs-react-aria) (secondary)

**Base UI**
- v1.0.0 was released on 11 Dec 2025. The latest is v1.9.0 (9 Oct 2026). — [Base UI releases](https://base-ui.com/react/overview/releases)
- It has 37 components: Slider, Toolbar, Menu, Menubar, Context Menu, Dialog, Number Field, Tabs, Combobox, Toast, Meter and others. There is no Tree and no data grid. — [Base UI releases](https://base-ui.com/react/overview/releases)
- InfoQ reports v1 shipped "35 accessible components" with long-term maintenance by a dedicated team (Radix + Floating UI + MUI creators). — [InfoQ Feb 2026](https://infoq.com/news/2026/02/baseui-v1-accessible/)

**Radix**
- shadcn/ui made Base UI the default for new projects in July 2026, while Radix remains supported (`shadcn init -b radix`). — [OpenReplay](https://blog.openreplay.com/shadcn-ui-radix-base-ui-switch/); [byteiota](https://byteiota.com/shadcn-ui-base-ui-default-what-react-devs-must-know-now/) (secondary)
- Commentary describes slower Radix maintenance after the WorkOS acquisition (not an official WorkOS statement). — [DEV Community](https://dev.to/mashuktamim/is-your-shadcn-ui-project-at-risk-a-deep-dive-into-radixs-future-45ei); [InfoQ](https://infoq.com/news/2026/02/baseui-v1-accessible/)

**Ark UI / Zag, Kobalte, Bits UI / Melt UI**
- Ark UI is built on Zag.js finite state machines and supports React, Solid, Vue and Svelte. Components include Tree View, JSON Tree View, Slider, Angle Slider, Splitter, Color Picker, Listbox, Menu, Dialog and Number Input; no Toolbar was seen. `@ark-ui/vue` is at 5.39.3 (5 Oct 2026). — [Ark UI llms-vue.txt](https://ark-ui.com/llms-vue.txt)
- Kobalte (Solid) follows WAI-ARIA APG "whenever possible" and handles focus and keyboard. — [Kobalte](https://github.com/kobaltedev/kobalte)
- Bits UI (Svelte) "strives to follow W3C ARIA Authoring Practices", and Melt UI powered Bits UI v1. — [Bits UI](https://next.bits-ui.com/)

**Supporting libraries and patterns**
- cmdk implements the ARIA combobox pattern, was tested with VoiceOver, and requires a `label` prop. — [cmdk accessibility guide](https://mintlify.wiki/dip/cmdk/guides/accessibility)
- The reference keyboard and ARIA patterns (slider, tree, treegrid, grid, toolbar, menu, dialog) are defined by the APG. — [WAI-ARIA Authoring Practices Guide](https://www.w3.org/WAI/ARIA/apg/)

### Inferences
- Use **React Aria Components** for everything: tree (data/graph hierarchy), Table (variables and mapping matrix), GridList, Slider and NumberField-style controls, Toolbar (transport), Menu/MenuBar-like structures, Dialog, Tabs and DnD.
- Supplement with custom APG-conformant widgets where RAC has none, notably the plot "keyboard explorer" (a focusable canvas region with roving cursor, live-region readout and sonified feedback).
- Avoid mixing Radix, Base UI and RAC in one app, because focus management and portal conventions conflict. Pick RAC as the single primitive layer.
- If SONARITHM chose Solid or Svelte, Ark UI (Tree View, Slider, Splitter) would be the best substitute. Its lack of a documented Toolbar means building one to the APG toolbar pattern.

### Gaps
- No independent 2026 screen-reader audit comparing these libraries widget by widget was found. All accessibility claims are vendor self-descriptions.
- The exact current RAC version and date (2026) were not confirmed. v1.17.0's date was not visible.

---

## 5. State management and styling (theming: dark, light, high-contrast, forced-colors)

### Takeaway
Pair Zustand (UI and app state, with transient subscriptions for hot paths) with XState v5 state machines for the transport/playback and sonification-session lifecycles, where illegal states must be impossible. For styling, use plain CSS (CSS Modules + cascade layers) driven by a token layer of CSS custom properties. Generate themes (dark, light, high-contrast) from a few parameters, following Linear's LCH approach, and add explicit `forced-colors` and `prefers-contrast` rules. Tailwind v4 is fine technically, but its defaults are the root of the generic look, so if used it must be fully re-themed.

### Cited Findings

**State management**
- Zustand is a small hooks-based store with documented transient (non-rendering) subscriptions. — [Zustand](https://github.com/pmndrs/zustand)
- XState v5 provides actor and state-machine modeling. — [Stately/XState docs](https://stately.ai/docs/xstate)
- Jotai uses atomic state. — [Jotai](https://jotai.org/)
- TanStack Store is a framework-agnostic store. — [TanStack Store](https://tanstack.com/store/latest)
- (Basic descriptions only. Versions not verified in this research.)
- Ark UI's components are themselves built on finite state machines (Zag.js), a precedent for FSM-driven UI. — [Ark UI](https://ark-ui.com/llms-vue.txt)

**Styling**
- Tailwind CSS 4.2.0 (18 Feb 2026) added a first-class webpack plugin, new palettes (mauve, olive, mist, taupe) and logical-property utilities. 4.3.x releases followed (4.3.2 on 29 Jun 2026). — [InfoQ: Tailwind 4.2](https://infoq.com/news/2026/04/tailwind-css-4-2-webpack); [versionlog](https://versionlog.com/tailwind-css/)
- vanilla-extract and Panda CSS are both zero-runtime, build-time CSS-in-TS. No 2026 maintenance data was found. — [pkgpulse comparison](https://www.pkgpulse.com/guides/vanilla-extract-vs-panda-css-vs-tailwind-2026) (secondary)

**Theming references**
- Linear builds themes from three inputs ("base color, accent color, and contrast") in LCH, replacing about 98 hand-set variables. The contrast variable lets the system "automatically include super high-contrast themes". — [Linear: How we redesigned the Linear UI](https://linear.app/now/how-we-redesigned-the-linear-ui)
- Carbon themes are token sets mapped per theme. The dark themes use Gray 100 (#161616) and Gray 90 backgrounds, with layering tokens for nested surfaces. — [Carbon v10 color overview](https://v10.carbondesignsystem.com/guidelines/color/overview/)
- Relevant media queries are `forced-colors: active` (system colors such as `Canvas`, `CanvasText`, `Highlight`, `ButtonText`; use `forced-color-adjust` sparingly), `prefers-contrast` and `prefers-reduced-motion`. — [MDN forced-colors](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/forced-colors); [MDN prefers-contrast](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-contrast); [MDN prefers-reduced-motion](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion)

### Inferences
- **Token architecture**: three layers.
  1. Primitives: an OKLCH/LCH ramp generated from base, accent and contrast.
  2. Semantic tokens: `--surface-0..3`, `--text-primary/secondary/disabled`, `--border-subtle/strong`, `--focus-ring`, `--signal-ok/warn/clip`, `--playhead`, `--selection`, `--series-1..8`.
  3. Component tokens.
  - Themes swap only the semantic layer.
- **High-contrast theme**: a first-class theme, not a fallback. Under `forced-colors`, canvas plots must redraw with system-color equivalents read via `getComputedStyle`, because canvas content is not auto-adjusted.
- **CSS Modules + custom properties** keep zero runtime, work with any framework, and avoid Tailwind's utility-default aesthetics. vanilla-extract is a good alternative if typed tokens are desired.
- **XState** fits transport (stopped/playing/paused/scrubbing/rendering/exporting) and spec-validation flows. Zustand fits panels, selection and preferences.

### Gaps
- Current versions and maintenance health of Zustand, Jotai, XState, TanStack Store, vanilla-extract and Panda CSS were not verified.
- No authoritative source found on `forced-colors` support inside WKWebView or WebKitGTK.

---

## 6. High-performance visualization in a webview (waveforms, spectrograms, 60 fps playhead over 1M+ points)

### Takeaway
Use uPlot (Canvas2D, about 50 KB, MIT) for standard time-series and multivariate line panels. Add a small custom WebGL2 layer (twgl-style helpers) for spectrograms, dense waveforms and 1M+ point scatter or graph views. Do the downsampling (M4/min-max per pixel column, LTTB for shape-preserving overviews) in the Rust core, and send only pixel-resolution data. Draw the playhead on a separate overlay canvas so it never re-renders the data layer. Keep ECharts (which has LTTB sampling and an ARIA/decal module) as the option for breadth chart types. WebGPU stays optional (section 2).

### Cited Findings

**uPlot**
- Performance (from the project's own 2023-03-11 benchmarks on Chrome 113):
  - Interactive chart with 166,650 points in 25 ms cold start, then "~100,000 pts/ms".
  - Streaming 3,600 points at 60 fps uses about 10% CPU / 12.3 MB, versus Chart.js 40% / 77 MB and ECharts 70% / 85 MB.
  - Size is about 50 KB min.
- Limitations:
  - It "may begin to struggle beyond 100k in-view points".
  - It is Canvas2D only (no WebGL/WASM), and suggests WebGL/WebGPU libraries for heavy streaming.
  - No built-in drag panning and no accessibility statements.
- License: MIT.
— [uPlot README](https://github.com/leeoniya/uPlot)
- A July 2026 vendor benchmark (ApexCharts) with uPlot 1.6.32 and ECharts 6.1.0 found uPlot renders a 100k-point line in 9 ms but takes about 6 s on a 50k-point scatter, where Chart.js was fastest at 89 ms. — [ApexCharts benchmark](https://apexcharts.com/blog/javascript-chart-library-benchmark/) (vendor; caution)

**ECharts**
- ECharts 6.1.0 exists as of mid-2026. — [ApexCharts benchmark](https://apexcharts.com/blog/javascript-chart-library-benchmark/)
- ECharts line series support `sampling: 'lttb'` (plus average/min/max/minmax). — [ECharts option: series-line.sampling](https://echarts.apache.org/en/option.html#series-line.sampling)
- ECharts' `aria` option generates chart descriptions and decal patterns. — [ECharts option: aria](https://echarts.apache.org/en/option.html#aria)

**Downsampling references**
- LTTB comes from Steinarsson's 2013 thesis "Downsampling Time Series for Visual Representation". — [Skemman](https://skemman.is/handle/1946/15343)
- M4 (Jugel et al., VLDB 2014) is a visualization-oriented aggregation that keeps the min/max/first/last per pixel column for error-free line rasterization. — [PVLDB vol. 7](https://www.vldb.org/pvldb/vol7/p797-jugel.pdf)

**Vendor commentary**
- A SciChart 2026 article says uPlot and ECharts suit "basic applications" and that uPlot drops to 6 fps for 100k OHLC candles. — [SciChart](https://www.scichart.com/blog/fastest-chart-libraries-for-quantitative-analysis/) (vendor; caution)

### Inferences
- **Library roles.**
  - **D3**: Use its modules (scales, axes, shapes, d3-force/d3-hierarchy for graph layout) as math only. Avoid D3 DOM rendering for large data.
  - **Observable Plot and visx**: SVG-based. Good for static summary charts and docs, not for 1M points or 60 fps.
  - **deck.gl**: Map- and geospatial-oriented and heavy. Use only if geo data matters.
  - **PixiJS v8**: A general 2D WebGL/WebGPU scene graph. Overkill unless building a node-graph canvas.
  - **Plotly**: Its scattergl uses WebGL but is a large bundle with a less "instrument-like" look.
- **Waveforms.** Rust computes min/max per column at each zoom level (a mipmap pyramid, like DAWs). The frontend draws vertical spans in one WebGL draw call or with Canvas2D `fillRect` batches.
- **Spectrograms.** Rust computes STFT magnitudes into dB-scaled `Uint8`/`Float16` textures, tiled by time. WebGL2 maps them through a perceptual colormap LUT texture (e.g., viridis/magma, which are color-vision-deficiency-friendly). Scrolling appends texture columns (ring buffer). For accessibility, offer a parallel table and summary.
- **Playhead at 60 fps over 1M+ points.** The data layer is redrawn only on zoom, pan or data change. The playhead lives on a separate `<canvas>` or absolutely positioned element, moved with CSS `transform` on rAF, extrapolated from the audio-clock anchor (section 1). It costs about one quad per frame regardless of N.
- **Canvas2D vs WebGL2 vs WebGPU.**
  - Canvas2D is simplest and works everywhere, and is fine up to about 100k in-view points (uPlot's guidance).
  - WebGL2 is universal across all three webviews and needed beyond that.
  - WebGPU offers compute shaders (on-GPU downsampling/FFT) but lacks Linux WebKitGTK and older macOS.
- **Accessibility of charts.** Every canvas plot needs:
  - an accessible name and description;
  - a keyboard "explorer" mode (arrow keys move a data cursor, a live region reports values, and SONARITHM sonifies the cursor);
  - a "view as table" alternative.
  - No library provides this out of the box, except ECharts' basic `aria` description.

### Gaps
- No 2026 head-to-head benchmark at 1M points across uPlot, ECharts 6 (WebGL/large mode) and custom WebGL in the three webviews was found.
- Maintenance status of regl, twgl and visx in 2026 was not verified.
- Graph/network-specific renderers (e.g., sigma.js/graphology, cosmos) were not researched and may matter for graph sonification.

---

## 7. Docking/layout, editors (JSON/TOML spec with schema validation), command palettes

### Takeaway
Use dockview (MIT core) for dockable panels. It ships ARIA roles, focus indicators and live-region announcements of layout changes, but its licence page lists spatial keyboard navigation and keyboard docking as Enterprise-only, so plan to implement F6-style region cycling yourself or verify what core includes. Use CodeMirror 6 for the spec editor, with JSON Schema linting in the editor and TOML validated by the Rust core (e.g., Taplo) feeding CM6 diagnostics. For command palettes, use cmdk with a mandatory label, or a React Aria ComboBox/Menu composition.

### Cited Findings

**dockview**
- Accessibility features:
  - An accessibility pack (core and framework packages) with keyboard navigation (opt-in via `keyboardNavigation`).
  - Default bindings: Ctrl+]/Ctrl+[ for tabs, F6/Shift+F6 to move between groups, Ctrl+Shift+\ to reach the tab strip.
  - Live-region announcements of panel open/close/float/dock/maximize, on by default.
  - The React bindings moved to `dockview-react` in v7.
  — [dockview: Accessibility](https://dockview.dev/docs/advanced/accessibility); [dockview v7 what's new](https://dockview.dev/docs/releases/whats-new/whats-new-v7)
- Licensing:
  - Dual-tier: MIT open-source core plus a commercial "Dockview Enterprise".
  - Enterprise-only features include advanced overflow, multi-row tabs, pinned tabs, layout history, **spatial keyboard navigation** and **keyboard docking**. The nav menu tags "Keyboard navigation" as Enterprise.
  - ARIA roles/labels, announcements and focus indicators are in both tiers.
  — [dockview licence](https://dockview.dev/docs/overview/licence)
- CONFLICT: The accessibility docs present `keyboardNavigation` as part of the pack, while the licence page flags keyboard navigation features as Enterprise. Verify exactly which bindings ship in MIT core. — [accessibility](https://dockview.dev/docs/advanced/accessibility) vs [licence](https://dockview.dev/docs/overview/licence)

**golden-layout**
- The latest is 2.6.0. The v3 dev branch is "quite unstable", and Best of JS showed the last commit about 2 years ago. — [golden-layout README](https://cdn.jsdelivr.net/npm/golden-layout@2.6.0/README.md); [Best of JS](https://bestofjs.org/projects/golden-layout)

**Editors**
- CodeMirror 6 says it was "designed from the start with accessibility in mind", using a native editable control (self-description). — [CodeMirror 6 (archived page)](https://webcf.waybackmachine.org/web/20211102230227/https://codemirror.net/6)
- Monaco supports screen readers using a strategy "based on paging the text", plus a Tab-trapping toggle and an exhaustive command palette. — [Monaco Editor Accessibility Guide](https://github.com/Microsoft/monaco-editor/wiki/Monaco-Editor-Accessibility-Guide)
- Replit moved from Monaco to CodeMirror 6 mainly for mobile support and extensibility. GitLab's spike also cited Monaco's lack of touch support. — [Replit blog](https://blog.repl.it/codemirror); [GitLab work item 387586](https://gitlab.com/gitlab-org/gitlab/-/work_items/387586)
- Taplo is a Rust TOML toolkit (parser, formatter, LSP) with JSON-Schema-based validation. — [Taplo](https://github.com/tamasfe/taplo)

**Command palettes**
- cmdk follows the ARIA combobox pattern, was tested with VoiceOver, and requires a `label`. — [cmdk accessibility](https://mintlify.wiki/dip/cmdk/guides/accessibility)

### Inferences
- **CodeMirror 6 over Monaco.** CM6 is smaller, modular and themeable with tokens, and does not depend on Monaco's web-worker and AMD/ESM packaging. It works the same in WebKitGTK and WKWebView. Monaco remains a defensible choice if a VS Code-identical editing model is valued by screen-reader users familiar with VS Code.
- **Schema single source of truth.** Define the spec schema once (JSON Schema generated from Rust types, e.g., via `schemars`). Validate in Rust for both JSON and TOML, and stream diagnostics to CM6's lint gutter. Then the frontend never re-implements validation.
- **Layout fallback.** If dockview Enterprise keyboard features are needed but not wanted (licence), a simpler fixed-region layout with resizable splitters plus F6 landmark cycling may be more accessible than free-form docking. Free-form floating panels are hard for screen-reader users anyway.
- **FlexLayout** status was not found. dockview appears the most actively maintained React-friendly option.

### Gaps
- No independent 2026 screen-reader comparison of CodeMirror 6 vs Monaco (NVDA/JAWS/VoiceOver/Orca) was found.
- FlexLayout's 2026 maintenance state is unknown.
- I did not verify the current maintenance state of a CM6 JSON Schema package (e.g., `codemirror-json-schema`).

---

## 8. Design language: credible pro-audio/scientific look, typography, color and contrast, avoiding "AI-generated" UI, icons

### Takeaway
Credible pro tools share these traits:
- Content-first, low-chrome layouts with clear chrome-vs-content separation.
- Dense but aligned information; neutral, restrained palettes with a single purposeful accent.
- Semantic signal colors (level, clip, selection, playhead) rather than decorative ones.
- Tabular numerals; keyboard-first operation.
- First-class contrast themes. Linear's three-variable LCH theming and Carbon's layered gray surfaces are good primary references, and Ableton Live 12 shows a dense music instrument can be screen-reader accessible.

Use WCAG 2.2 AA as the compliance bar, and APCA only as a supplementary design check, since APCA is not in the WCAG 3 draft. Avoid the template tells: indigo/purple gradients, glassmorphism, Inter-by-default, oversized rounded cards with soft shadows, centered heroes, emoji icons and default shadcn styling.

### Cited Findings

**Reference design systems and tools**
- Linear's redesign aims to "reduce visual noise, maintain visual alignment, and increase the hierarchy and density of navigation elements".
  - It distinguishes global chrome, which "controls the content in the main view", from content.
  - It reduced blue in color calculations for "a more neutral and timeless appearance", and uses LCH because colors at the same lightness look equally bright.
  - It uses Inter Display for headings and Inter for body.
  — [Linear: How we redesigned the Linear UI](https://linear.app/now/how-we-redesigned-the-linear-ui)
- Linear also found dividing lines had "quietly proliferated", and softened their contrast. It moved from cool blue-ish grays toward warmer gray, warning that too warm "risks looking muddy". — [Linear: Behind the latest design refresh](https://linear.app/now/behind-the-latest-design-refresh)
- Carbon's dark themes are Gray 100 (#161616) and Gray 90, with per-theme token mapping and layering. — [Carbon v10 color](https://v10.carbondesignsystem.com/guidelines/color/overview/)
- Material's dark theme recommends a dark gray (#121212) surface rather than pure black. — [Material Design 2: Dark theme](https://m2.material.io/design/color/dark-theme.html)
- Ableton Live 12 offers screen-reader support plus keyboard navigation, with Accessibility options such as "Speak Minimum and Maximum Slider Values". — [Live 12 manual: Accessibility Options](https://www.ableton.com/en/live-manual/12/accessibility-options-in-live/); [Ableton Accessibility FAQ](https://help.ableton.com/hc/en-us/articles/11762577169948)

**Contrast standards**
- WCAG 2.2 AA requires 4.5:1 for normal text and 3:1 for large text (SC 1.4.3), 3:1 for UI components and graphical objects (SC 1.4.11), and that color is not the only means of conveying information (SC 1.4.1). — [WCAG 2.2](https://www.w3.org/TR/WCAG22/)
- The WCAG 3.0 Working Draft (3 Mar 2026) leaves its contrast algorithm "yet to be determined". APCA was removed from the drafts in 2023. — [Adrian Roselli: WCAG3 Contrast as of April 2026](https://adrianroselli.com/2026/04/wcag3-contrast-as-of-april-2026.html)
- WCAG 3 Recommendation status is not expected until about 2028–2030. — [webability](https://www.webability.io/blog/wcag-3-0-explained) (secondary)
- CONFLICT: One tooling doc claims APCA is in WCAG 3 at "Silver". This contradicts the W3C-based sources above and is treated as unreliable. — [contrast-advanced.md](https://cdn.jsdelivr.net/npm/@hegemonart/get-design-done@1.60.4/reference/contrast-advanced.md)

**Typography (all fonts below are under the SIL OFL 1.1)**
- `font-variant-numeric: tabular-nums` gives equal-width digits. — [MDN font-variant-numeric](https://developer.mozilla.org/en-US/docs/Web/CSS/font-variant-numeric)
- Inter supports tabular numbers, a slashed zero and contextual alternates. It "can be used commercially but not sold by itself". — [Inter](https://github.com/rsms/inter)
- IBM Plex (Sans, Serif, Mono, and others). — [Wikipedia: IBM Plex](https://en.wikipedia.org/wiki/IBM_Plex)
- Geist Sans and Geist Mono, with no Reserved Font Name. — [vercel/geist-font](https://github.com/vercel/geist-font); [CTAN geist-font](https://www.ctan.org/pkg/geist-font)
- JetBrains Mono (build source under Apache 2.0). It has an increased x-height for small sizes, plus stylistic sets and character variants. — [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono)
- Source Sans 3 (Adobe). — [adobe-fonts/source-sans](https://github.com/adobe-fonts/source-sans) (license not re-fetched in this session)

**What "AI-generated" or template UI looks like**
- Commonly cited markers:
  - Inter or Roboto, purple or indigo accents/gradients, a centered hero with a CTA.
  - Three icon boxes in a row, rounded cards with soft shadows.
  - The "default shadcn/Tailwind look".
  — [prg.sh: Why your AI keeps building the same purple gradient website](https://prg.sh/ramblings/Why-Your-AI-Keeps-Building-the-Same-Purple-Gradient-Website); [frontend-design-deslop skill](https://mcpservers.org/ko/agent-skills/samber/frontend-design-deslop) (community sources, not peer-reviewed)
- The prg.sh post traces the purple default to Tailwind UI's `bg-indigo-500` buttons, which Adam Wathan publicly apologized for in 2025. — [prg.sh](https://prg.sh/ramblings/Why-Your-AI-Keeps-Building-the-Same-Purple-Gradient-Website) (secondary account)

**Icons**
- Lucide is under the ISC License. Icons derived from Feather are additionally under MIT (© Cole Bemis). — [Lucide LICENSE](https://github.com/lucide-icons/lucide/blob/main/LICENSE)

### Inferences
- **Credibility cues from pro audio and scientific software.** Ableton, Bitwig, iZotope RX, MATLAB, Grafana and Observable are characterizations, not sourced. They share:
  - Neutral dark gray canvases (not pure black), with color reserved for data and state.
  - Hairline separators and small radii (0–4 px).
  - Compact controls (24–28 px rows) with generous keyboard affordances.
  - Monospaced or tabular numerics in readouts.
  - Explicit units everywhere (Hz, dB, s, samples).
  - Persistent transport and status bars.
  - Visible, precise selection and playhead states.
  - Docked panels instead of floating cards.
  - Minimal motion.
- **Fonts for SONARITHM.**
  - **Recommended**: IBM Plex Sans + IBM Plex Mono. Plex has an engineering heritage and is distinct from the overused Inter/Geist "AI default".
  - **Alternative**: Source Sans 3 + JetBrains Mono.
  - Use Inter only with deliberate tuning (tnum, slashed zero, Inter Display for headings).
  - Never rely on proportional figures in readouts.
- **Contrast checking.** Use WCAG 2.2 ratios as pass/fail in CI (token pairs checked automatically). Use APCA Lc values as an additional design-time sanity check for thin text on dark backgrounds, where WCAG 2's math is known to be weak.
- **Data colors.**
  - Use CVD-safe categorical palettes (e.g., Okabe–Ito-style).
  - Use perceptually uniform sequential maps (viridis/magma/cividis) for spectrograms.
  - Always provide redundant encodings (line dash, marker shape, labels). This is also SC 1.4.1.
- **Icons.** Lucide is fine licence-wise (ISC + MIT notice). Keep icon use sparse, always paired with accessible names, and avoid emoji as icons.

### Gaps
- No primary design writing from Ableton, Bitwig, iZotope, MATLAB, Grafana, Observable or Figma on their visual principles was retrieved. The pro-audio characterization above is inference.
- I did not verify IBM Plex's tabular-figure defaults or Source Sans 3's license text in this session.
- No primary data-visualization color guidance from Carbon, Spectrum or Primer was retrieved.

---

## 9. Frontend testing: Vitest, Playwright, Tauri WebDriver per OS, axe-core, Storybook vs Ladle, visual regression

### Takeaway
Use a four-layer test strategy:
1. Vitest 4 (unit + browser-mode component tests with `toMatchScreenshot`).
2. Storybook 10 with the Vitest addon and the a11y (axe-core) addon, as the component workbench.
3. Playwright against the frontend in Chromium, WebKit and Firefox with Tauri IPC mocked (`mockIPC`), for cross-engine E2E and visual regression.
4. WebdriverIO's `@wdio/tauri-service` for real-app E2E on Windows, Linux and macOS. The official `tauri-driver` still does not support macOS.

Add a mandatory manual screen-reader matrix, because automation cannot judge announcements.

### Cited Findings

**Tauri WebDriver per OS**
- Official `tauri-driver` supports Windows and Linux only on desktop, because macOS has no WKWebView driver tool. The docs recommend WebdriverIO's `@wdio/tauri-service`, whose embedded provider runs a WebDriver server inside the app on Windows, Linux and macOS. — [Tauri: WebDriver](https://v2.tauri.app/develop/tests/webdriver/); [WebdriverIO Tauri service](https://webdriver.io/docs/wdio-tauri-service); [WebdriverIO platform support](https://webdriver.io/docs/desktop-testing/tauri/platform-support)
- CrabNebula's `@crabnebula/tauri-driver` requires a paid API key for macOS. Open-source community options exist (tauri-webdriver; tauri-plugin-webdriver). — [@crabnebula/tauri-driver](https://www.npmjs.com/package/@crabnebula/tauri-driver); [Daniel Raffel, Feb 2026](https://danielraffel.me/2026/02/14/i-built-a-webdriver-for-wkwebview-tauri-apps-on-macos/); [Choochmeque/tauri-webdriver](https://github.com/Choochmeque/tauri-webdriver)
- OPEN QUESTION: An October 2026 GitHub issue lists the embedded provider on macOS 26 with Tauri 2.12 as unverified. — [gerchowl/crank #35](https://github.com/gerchowl/crank/issues/35)
- Tauri provides IPC mocking (`@tauri-apps/api/mocks`, `mockIPC`) for frontend tests without the Rust backend. — [Tauri: Mocking](https://v2.tauri.app/develop/tests/mocking/)

**Vitest and Storybook**
- Vitest 4.0 graduated Browser Mode to stable and added visual regression (`toMatchScreenshot`, pixel diff against committed baselines).
  - It requires a provider package: `@vitest/browser-playwright` (recommended), `-webdriverio` or `-preview`.
  - Teams report OS rendering differences in baselines.
  — [Vitest 4 blog](https://main.vitest.dev/blog/vitest-4); [InfoQ](https://www.infoq.com/news/2025/12/vitest-4-browser-mode); [Argos guide](https://argos-ci.com/blog/vitest-visual-testing)
- Storybook docs (v10.3) recommend the Vitest addon in browser mode (Playwright Chromium), with accessibility testing "out of the box" via the Accessibility addon. The addon replaces the Jest-based test runner for Vite-based React, Vue, Svelte, Preact and Web Components projects. — [Storybook Vitest addon](https://storybook.js.org/docs/writing-tests/integrations/vitest-addon); [migration guide (v10.3)](https://storybook.js.org/docs/writing-tests/integrations/vitest-addon/migration-guide.md)
- Playwright offers screenshot assertions for visual regression. — [Playwright: Visual comparisons](https://playwright.dev/docs/test-snapshots)

**Limits of automation**
- Automated scanners cannot detect incorrect announcements, wrong reading order, missing context, focus traps or broken keyboard interactions. — [screen-reader-testing skill (community)](https://tessl.io/registry/skills/github/wshobson/agents/screen-reader-testing)
- Ableton's own practice is to test VoiceOver on macOS and NVDA, Narrator and JAWS on Windows. — [Ableton](https://help.ableton.com/hc/en-us/articles/11550373507868)

### Inferences
- **Visual baselines.** Take them on one pinned OS/browser image in CI (e.g., Linux Chromium in Docker) to avoid font-rendering diffs, plus a small per-OS smoke set.
- **Accessibility CI.** Run axe-core via the Storybook a11y addon on every story and via `@axe-core/playwright` on key E2E flows. Gate CI on serious and critical violations.
- **Screen-reader automation.** Guidepup ([guidepup](https://github.com/guidepup/guidepup)) can automate VoiceOver and NVDA for regression smoke tests. Its maturity was not verified.
- **Manual matrix per release.**
  - NVDA + WebView2 (Windows)
  - JAWS + WebView2 (Windows)
  - Narrator (Windows)
  - VoiceOver + WKWebView (macOS 14/15/26)
  - Orca + WebKitGTK (Ubuntu LTS / Fedora)
  - Involve blind testers early, as Ableton did.
- **Storybook vs Ladle.** Storybook 10 is preferable given its first-party Vitest and a11y integration. Ladle (Vite-based, lighter) lacks equivalent integrated a11y/test tooling.

### Gaps
- Ladle's 2026 maintenance state and Storybook 10's exact release notes were not verified on primary pages.
- No verified report of `@wdio/tauri-service` stability on macOS 26 + Tauri 2.12.
- axe-core's current version and coverage statistics were not retrieved.

---

## 10. Synthesis (1): Recommended stack for SONARITHM

### Takeaway

| Layer | Recommendation (Oct 2026) | Rationale | Main trade-off / fallback |
|---|---|---|---|
| Desktop shell | **Tauri 2.12.x** (Rust core in-process; WebView2 / WKWebView / WebKitGTK) | Inherits mature webview accessibility trees (best free baseline); in-process Rust; capabilities security; signed updater; mobile path | WebKitGTK performance and GPU issues on Linux; no WebGPU on Linux. **Fallback: Electron 44/45** (Chromium everywhere, Rust via napi-rs) if Linux/Orca tests fail; watch Tauri's experimental CEF runtime |
| Native Rust GUI | **Not for the primary UI.** Re-evaluate Slint/egui in 2027 | AccessKit integrations exist (egui, Slint) but have gaps (egui live regions; Slint text inputs). iced has none, released gpui has none, and Blitz is pre-alpha | Could serve a future "lite" or embedded build |
| IPC pattern | Commands for requests; **Channels** for meters and transport anchors; `ipc::Response` ArrayBuffers for bulk data; no JSON events on hot paths | Official guidance: events are JSON and not for throughput; channels are ordered and fast | Benchmark Windows binary IPC early |
| Framework | **React 19.2 + TypeScript + Vite** (React Compiler on) | Best accessible-component ecosystem; contributor pool; hot paths bypass React anyway | Solid 2.0 faster, but RC and a thinner accessibility layer |
| Headless components | **React Aria Components** (single primitive layer) | Tree, Table (expandable rows), GridList, Virtualizer, DnD, Slider, Toolbar, Menu, Dialog; Adobe-tested; i18n | More assembly than styled kits. Base UI 1.9 for simple primitives only if ever needed (no Tree or Grid) |
| App state | **Zustand** (UI/app state, transient subscriptions) + **XState v5** (transport, render/export, spec-validation machines) | Hot-path reads without re-render; impossible states eliminated | Jotai is an alternative for fine-grained derived state |
| Styling and theming | **CSS Modules + cascade layers + CSS custom-property tokens**, generated from LCH/OKLCH (base, accent, contrast); explicit `forced-colors`, `prefers-contrast` and `prefers-reduced-motion` | Zero runtime; framework-agnostic; avoids Tailwind-default look; high-contrast as a first-class theme | vanilla-extract if typed tokens are wanted; Tailwind v4 only with a fully custom `@theme` |
| Time-series plots | **uPlot** (Canvas2D, MIT) | Fastest small library for lines and streaming; about 50 KB | Struggles beyond about 100k in-view points, so feed it downsampled data |
| Dense, waveform and spectrogram views | **Custom WebGL2 layer** (twgl-style helpers); WebGPU as optional enhancement | Universal across webviews; textures for spectrograms; min/max mipmaps for waveforms | More code to own. ECharts 6 (LTTB sampling, `aria`) for breadth charts |
| Downsampling | **In Rust**: M4 / min-max per pixel column; LTTB for overviews | Pixel-exact lines; small IPC payloads | Requires zoom-level tiling logic |
| Playhead | Overlay canvas or CSS-transform element driven by rAF, extrapolated from audio-clock anchors | Constant cost per frame regardless of N; IPC-jitter-free | Must compensate output latency |
| Docking | **dockview** (MIT core) or fixed regions + splitters | ARIA, focus indicators, live announcements in core | Spatial keyboard nav and keyboard docking are Enterprise-only, so implement F6 cycling or buy |
| Spec editor | **CodeMirror 6** + JSON lint; TOML/JSON validated in Rust (schemars-generated JSON Schema; Taplo for TOML) | Modular, light, accessible-by-design; one schema source of truth | Monaco if VS Code parity is preferred (heavier) |
| Command palette | **cmdk** (with required `label`) or React Aria ComboBox + Menu | ARIA combobox pattern | Ensure results are announced |
| Icons | **Lucide** (ISC; Feather-derived icons MIT) | Permissive, consistent stroke set | Keep sparse; always label |
| Fonts | **IBM Plex Sans + IBM Plex Mono** (OFL 1.1), bundled WOFF2; `tabular-nums` | Engineering credibility; not the Inter/Geist "default" | Alternative: Source Sans 3 + JetBrains Mono (OFL) |
| Testing | **Vitest 4** (browser mode, `toMatchScreenshot`), **Storybook 10** + a11y addon, **Playwright** (Chromium/WebKit/Firefox, `mockIPC`), **@wdio/tauri-service** (real app, all OSes), manual SR matrix | Covers unit, component, cross-engine and real-shell testing; axe in CI | macOS real-app WebDriver still maturing; `tauri-driver` is Win/Linux only |

### Cited Findings
- Tauri 2.12 release and IPC guidance — [Tauri 2.12](https://v2.tauri.app/blog/tauri-2.12/); [Calling the frontend](https://v2.tauri.app/develop/calling-frontend/); [Calling Rust](https://v2.tauri.app/develop/calling-rust/)
- Accessibility status of native toolkits — [stack-rows.md](https://github.com/GoldStrikeArch/rust-gui-desktop-ecosystem-state/blob/main/report/data/stack-rows.md); [accesskit.dev](https://accesskit.dev/)
- Electron schedule — [releases.electronjs.org/schedule](https://releases.electronjs.org/schedule)
- React Aria capabilities — [RAC v1.17.0](https://react-aria.adobe.com/releases/v1-17-0); [React Spectrum 2025-03-05](https://react-spectrum.adobe.com/releases/2025-03-05.html)
- uPlot performance and limits — [uPlot](https://github.com/leeoniya/uPlot)
- dockview licensing — [dockview licence](https://dockview.dev/docs/overview/licence)
- Testing — [Tauri WebDriver](https://v2.tauri.app/develop/tests/webdriver/); [Vitest 4](https://main.vitest.dev/blog/vitest-4)

### Inferences
- **Prototype gate before committing.** Build a two-week spike measuring three things:
  1. Channel latency and throughput on all three OSes.
  2. WebGL2 spectrogram + 1M-point waveform fps in WebKitGTK on Intel/AMD/NVIDIA.
  3. NVDA, VoiceOver and Orca walkthroughs of a RAC Tree + Slider + Toolbar inside the Tauri window.
  If (2) or (3) fails on Linux, switch the shell to Electron (frontend unchanged).
- **License compatibility.** Every recommended dependency is permissive (MIT/ISC/Apache/OFL). The only copyleft or commercial tiers to watch are Slint (GPLv3 or commercial, not recommended here anyway) and dockview Enterprise.

### Gaps
- No end-to-end reference app combining Tauri + React Aria + WebGL plots with published screen-reader test results was found.

---

## 11. Synthesis (2): Design principles and anti-patterns for the SONARITHM UI

### Takeaway
SONARITHM should look and behave like a calibrated instrument: neutral, dense, precise, keyboard-first and fully operable without sight. Every visual flourish must earn its place by encoding data or state.

### Cited Findings
- Reduce visual noise, increase density and hierarchy, and separate chrome from content. — [Linear](https://linear.app/now/how-we-redesigned-the-linear-ui)
- Soften proliferating dividers, and choose neutral rather than cool-blue grays. — [Linear refresh](https://linear.app/now/behind-the-latest-design-refresh)
- Generate themes from base, accent and contrast so high-contrast themes are automatic. — [Linear](https://linear.app/now/how-we-redesigned-the-linear-ui)
- Use dark gray, not pure black, surfaces (#161616 Carbon; #121212 Material). — [Carbon](https://v10.carbondesignsystem.com/guidelines/color/overview/); [Material](https://m2.material.io/design/color/dark-theme.html)
- WCAG 2.2: 4.5:1 text, 3:1 non-text, no color-only meaning. APCA is not yet a standard. — [WCAG 2.2](https://www.w3.org/TR/WCAG22/); [Roselli 2026](https://adrianroselli.com/2026/04/wcag3-contrast-as-of-april-2026.html)
- Tabular numerals. — [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/font-variant-numeric)
- Pro-audio screen-reader precedent, including slider min/max announcements. — [Ableton Live 12](https://www.ableton.com/en/live-manual/12/accessibility-options-in-live/)
- Template "AI look" markers. — [prg.sh](https://prg.sh/ramblings/Why-Your-AI-Keeps-Building-the-Same-Purple-Gradient-Website); [deslop skill](https://mcpservers.org/ko/agent-skills/samber/frontend-design-deslop)

### Inferences

**Design principles (do)**
1. **Instrument, not marketing page.** Open straight into the workspace: data/graph tree, plot/sonification canvas, mapping inspector, transport bar and status bar. No hero, no splash, no onboarding carousel.
2. **Neutral surfaces, one accent, semantic signals.**
   - Use 3–4 near-neutral dark grays (about L* 8–20) for surfaces, and an equivalent light set.
   - Use one restrained accent for focus and selection.
   - Use reserved signal colors for playhead, clip/over-range, warning and OK, never reused decoratively.
3. **Density with alignment.** Use a 4 px base grid, 24–28 px control rows, 1 px hairline separators at low contrast, and radii 0–4 px. Align numbers on decimal points.
4. **Numbers are first-class.**
   - Use tabular, lining figures; a slashed zero where available; and a monospace face for readouts.
   - Always show units (Hz, dB, s, samples, BPM, data units) and precision appropriate to the data.
5. **Visible state everywhere.** Use high-visibility focus rings (≥3:1 against adjacent colors, 2 px+), and keep the selection, armed/playing state and current data cursor unambiguous in both visual and spoken form.
6. **Keyboard-first and screen-reader-complete.**
   - Every action is in a menu and the command palette and has a shortcut.
   - F6 cycles regions; APG patterns apply to every widget.
   - Each plot has an "explore" mode (arrows move the cursor, readout goes to a live region and direct speech, and the cursor is sonified), plus a table view.
   - Slider announcements include value, unit and range, following Ableton's min/max option.
7. **Themes generated, not hand-painted.** Dark, light and high-contrast come from base/accent/contrast in OKLCH, with `forced-colors` and `prefers-contrast` honored and canvases redrawn with system colors.
8. **Data color discipline.** Use CVD-safe categorical palettes and perceptually uniform colormaps (viridis/magma/cividis) for spectrograms, plus redundant encodings (dash, marker, label).
9. **Motion only for meaning.** Allowed motion is the playhead, meters and short state transitions (≤120 ms). Honor `prefers-reduced-motion`. Nothing animates on idle.
10. **Typography with a point of view.**
    - IBM Plex Sans/Mono (or Source Sans 3 + JetBrains Mono), 12–13 px UI text, a tight type scale (about 11/12/13/15/18), and weight contrast rather than size contrast.
    - Sentence-case labels, no all-caps paragraphs.
11. **Copy like a lab instrument.** Terse, precise, unit-bearing labels. Error messages state cause and fix. No exclamation marks or chatty microcopy.

**Anti-patterns (don't)**
1. No purple/indigo/blue gradients, gradient text, or "glow" accents.
2. No glassmorphism, frosted blur panels, or translucent layered cards.
3. No oversized rounded cards (≥12 px radii) with soft drop shadows as the main layout unit. Use docked panels and hairlines instead.
4. No centered hero layouts, "three feature boxes" rows, or big empty-state illustrations.
5. No emoji as icons or status indicators. Use labeled Lucide icons sparingly, never icon-only without accessible names and tooltips.
6. No untouched shadcn/Tailwind defaults: default slate/zinc + indigo palette, default `rounded-lg`/`shadow-md`, default Inter/Geist pairing without tuning.
7. No pure-black (#000) backgrounds with pure-white text (halation). No low-contrast gray-on-gray "aesthetic" text below WCAG 4.5:1.
8. No color-only encodings (e.g., red/green status without shape or text).
9. No decorative motion (parallax, floating blobs, shimmer skeletons on local data).
10. No proportional digits in changing readouts (causes jitter), and no unitless numbers.
11. No hover-only affordances. Everything reachable by keyboard and exposed to assistive technology.
12. No free-floating draggable windows as the primary layout. Use predictable regions that screen-reader users can learn.

### Gaps
- These principles are a synthesis. Primary design writing from Ableton, Bitwig, iZotope, MATLAB, Grafana and Observable was not retrieved and would strengthen the pro-audio/scientific rationale.
- No user research with SONARITHM's blind and low-vision audience was found. Validate the principles with them.

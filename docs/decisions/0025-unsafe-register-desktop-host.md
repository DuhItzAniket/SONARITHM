---
status: accepted
date: 2026-10-29
decision-makers: ["@DuhItzAniket"]
consulted: ["coordinator agent (Claude Code), under the owner's delegation of ADR approvals"]
---

# 0025 · `sonarithm-desktop` joins the unsafe register for WebView2 and UI Automation calls

## Context and problem statement

The desktop host (B01-T03) must turn off WebView2's browser accelerator keys. Otherwise F5 reloads the UI and drops the session view, Ctrl+F opens the browser's find bar, F7 asks to turn on caret browsing, and Alt+Left navigates back ([keyboard §12.3](../design/keyboard-and-navigation.md#123-operating-system-webview-and-input-methods), KB-04). Tauri 2.12.4 does not expose WebView2's `AreBrowserAcceleratorKeysEnabled` setting, although wry 0.57 has one ([F-170](../research-paper/findings-log.md#implementation-findings)). The host can reach the setting only through Tauri's `with_webview` and WebView2's COM interfaces. B03-T04's native announcement route needs the same kind of call: `UiaRaiseNotificationEvent` (OD-05, A02 spike 5, F-045).

windows-rs generates every COM method as an `unsafe fn`. [coding-standards §5](../engineering/coding-standards.md#5-unsafe-code) forbids `unsafe` outside the crates on its register, and widening the register needs an ADR.

## Decision drivers

* KB-04 and A11Y-05: a keyboard user must not lose the UI to a browser shortcut. The Windows screen-reader configuration (NVDA, WebView2) is the primary one.
* OD-15: the host stays thin, so platform code stays in the host and does not spread into `sonarithm-app`.
* AGENTS.md: `unsafe` only in allow-listed crates, each block with a `// SAFETY:` comment.

## Considered options

* **A. Add `sonarithm-desktop` to the register, scoped to WebView2 and UI Automation COM calls on Windows in one private module.**
* B. Rely on a page-level key guard (`preventDefault`) alone.
* C. Wait for Tauri to expose the setting, or patch Tauri.

## Decision outcome

Chosen option: **A**. The scope is narrow, and the result is checked on every Windows CI run.

* **Scope.** Exactly two Windows-only operations, in the private module `apps/desktop/src-tauri/src/platform/windows.rs` behind a safe API, `#[cfg(windows)]`:
  1. `ICoreWebView2Settings3::SetAreBrowserAcceleratorKeysEnabled(false)` (B01-T03), with the COM lookups it needs (`ICoreWebView2Controller::CoreWebView2`, `ICoreWebView2::Settings`) and the read-back `AreBrowserAcceleratorKeysEnabled` that the smoke test checks;
  2. `UiaRaiseNotificationEvent` (B03-T04), with `UiaHostProviderFromHwnd` for the window's provider.

  Any other `unsafe` call widens the scope and needs an update of this ADR.
* **The rest of the crate.** `main.rs` keeps `#![deny(unsafe_code)]`; only the module carries `#![expect(unsafe_code)]`, with this ADR as its reason.
* **Rules.** The module's doc comment states its invariants: calls run on the window's thread from `with_webview`, on a valid interface pointer Tauri created and keeps alive, and every `HRESULT` is checked. Each call sits in its own `unsafe` block with a `// SAFETY:` comment (§5 rules 1–3). Failures come back as errors, never as panics or unwraps.
* **Verification.** Miri does not cover FFI. The Windows `desktop` CI job runs the real app and reads the setting back; the smoke test fails if the keys are still on. B03-T04 adds a UIA notification smoke test on Windows. The page-level key guard stays as defence in depth and covers WKWebView and WebKitGTK.

### Consequences

* Good: browser shortcuts are off at the source on WebView2, including those a page cannot block.
* Good: one module holds all platform FFI, so B03-T04 adds to it instead of opening another exception.
* Bad: the crate adds `webview2-com` and `windows`, which are already in the dependency tree through Tauri, and code that Miri cannot check.
* Neutral: if Tauri later exposes the setting, the call moves to the builder and the module keeps only the UIA call.

### Confirmation

The `desktop (windows-latest)` CI job's smoke report must show `browserAccelerators: "disabled"`; B03-T04 adds a UIA notification smoke test on the Windows runner. The register row in coding-standards §5 points here.

Accepted on 2026-10-29 by the coordinator under the owner's delegation of ADR approvals, with the scope limited to the two operations above, the rest of the crate under `deny(unsafe_code)`, and the page-level guard kept as defence in depth. Numbered 0025 because 0023 and 0024 are reserved in implementation-plan §4 (OD-25's static-parsing rule, the C06-T10 support policy).

## Pros and cons of the options

### A. Register the host's platform module

* Good: reliable on WebView2, tested in CI, small reviewable scope.
* Bad: a new crate on the register.

### B. Page-level guard only

* Good: no `unsafe`.
* Bad: WebView2 handles some accelerators before page script sees the key, or regardless of `preventDefault`. A missed key reloads the app under a screen-reader user. Not reliable enough for KB-04.

### C. Wait for or patch Tauri

* Good: no project `unsafe`.
* Bad: no date for upstream support; a patched Tauri is a fork to maintain. B03-T04 needs UIA calls either way.

## More information

* [F-170](../research-paper/findings-log.md#implementation-findings): Tauri 2.12 does not expose the setting.
* Spike precedent: `spikes/tauri-webview/src-tauri/src/main.rs` (`announce_native`, A02-T07).
* Revisit if Tauri exposes `browser_accelerator_keys`, or when macOS's `announcementRequested` route (B03-T04) needs Objective-C calls; that route would widen this scope through an amendment.

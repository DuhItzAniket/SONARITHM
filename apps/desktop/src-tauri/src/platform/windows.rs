// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! WebView2 COM calls, the only `unsafe` code in this crate (coding-standards §5 register,
//! ADR-0023). Invariants: every call is made on the window's thread from Tauri's
//! `with_webview`, on a controller Tauri created and keeps alive for the call; each COM
//! method returns an `HRESULT` that is checked; interface pointers are reference-counted by
//! `windows-core` and released when they go out of scope.
#![expect(unsafe_code, reason = "WebView2 COM methods are unsafe fns (ADR-0023)")]

use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2Controller, ICoreWebView2Settings3,
};
use windows::core::{BOOL, Interface as _, Result};

/// Sets `AreBrowserAcceleratorKeysEnabled` to false and reads it back; `Ok(true)` when
/// the setting is off.
pub(super) fn disable_browser_accelerators(controller: &ICoreWebView2Controller) -> Result<bool> {
    // SAFETY: `controller` is a live WebView2 controller (module invariants); the call
    // takes no pointer arguments and its HRESULT is checked.
    let webview = unsafe { controller.CoreWebView2() }?;
    // SAFETY: `webview` is the live ICoreWebView2 returned above; no pointer arguments.
    let settings = unsafe { webview.Settings() }?;
    // Settings3 exists from WebView2 runtime 1.0.864; `cast` fails cleanly on older ones.
    let settings: ICoreWebView2Settings3 = settings.cast()?;
    // SAFETY: `settings` is live; the method takes a plain bool.
    unsafe { settings.SetAreBrowserAcceleratorKeysEnabled(false) }?;
    let mut enabled = BOOL::from(true);
    // SAFETY: `enabled` is a valid, writable BOOL that outlives the call.
    unsafe { settings.AreBrowserAcceleratorKeysEnabled(&raw mut enabled) }?;
    Ok(!enabled.as_bool())
}

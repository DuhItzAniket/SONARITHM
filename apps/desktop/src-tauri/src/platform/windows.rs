// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! WebView2 and UI Automation calls, the only `unsafe` code in this crate (coding-standards
//! §5 register, ADR-0025). Invariants: every call is made on the window's thread (from
//! Tauri's `with_webview`, or a synchronous command, which Tauri runs on that thread), on a
//! controller or window handle Tauri created and keeps alive for the call; each COM method
//! returns an `HRESULT` that is checked; interface pointers are reference-counted by
//! `windows-core` and released when they go out of scope.
#![expect(unsafe_code, reason = "WebView2 COM methods are unsafe fns (ADR-0025)")]

use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2Controller, ICoreWebView2Settings3,
};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{
    NotificationKind_Other, NotificationProcessing_ImportantMostRecent,
    NotificationProcessing_MostRecent, UiaHostProviderFromHwnd, UiaRaiseNotificationEvent,
};
use windows::core::{BOOL, BSTR, Interface as _, Result};

/// The activity ID of every notification, so a screen reader can group them.
const ACTIVITY: &str = "sonarithm.announce";

/// Raises a UI Automation notification with `text` on the window's host provider: the
/// native announcer route (B03-T04, OD-05). `assertive` asks the screen reader to speak it
/// before what is queued (`ImportantMostRecent`); otherwise it replaces the last one
/// (`MostRecent`).
pub(super) fn raise_notification(
    hwnd: *mut core::ffi::c_void,
    text: &str,
    assertive: bool,
) -> Result<()> {
    // SAFETY: `hwnd` is the live top-level window of this app (module invariants); the
    // call writes only its out-parameter, and its HRESULT is checked.
    let provider = unsafe { UiaHostProviderFromHwnd(HWND(hwnd)) }?;
    let processing = if assertive {
        NotificationProcessing_ImportantMostRecent
    } else {
        NotificationProcessing_MostRecent
    };
    let (text, activity) = (BSTR::from(text), BSTR::from(ACTIVITY));
    // SAFETY: `provider` is the live provider returned above; both BSTRs outlive the call,
    // and its HRESULT is checked.
    unsafe {
        UiaRaiseNotificationEvent(
            &provider,
            NotificationKind_Other,
            processing,
            &text,
            &activity,
        )
    }
}

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

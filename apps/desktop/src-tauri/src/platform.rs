// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Per-webview settings Tauri does not expose. Browser accelerator keys (reload, find,
//! caret browsing, developer tools, history; keyboard §12.3) exist only in WebView2;
//! WKWebView and WebKitGTK have none without a browser's menus, and the UI's key guard
//! covers what a page can block on every OS.

use serde::Serialize;

/// What happened to the webview's browser accelerator keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Accelerators {
    /// Turned off, and read back as off (WebView2).
    #[cfg_attr(
        all(not(windows), not(test)),
        expect(dead_code, reason = "only WebView2 has the setting")
    )]
    Disabled,
    /// This webview has no browser accelerator keys (WKWebView, WebKitGTK).
    #[cfg_attr(windows, expect(dead_code, reason = "only WKWebView and WebKitGTK"))]
    None,
    /// WebView2 refused the setting.
    #[cfg_attr(
        all(not(windows), not(test)),
        expect(dead_code, reason = "only WebView2 has the setting")
    )]
    Failed,
}

#[cfg(windows)]
mod windows;

/// Speaks `text` through the platform's notification API: the native announcer route
/// (B03-T04, OD-05). Only Windows has one so far (UI Automation); elsewhere it fails, and the
/// UI's announcer falls back to live regions.
pub(crate) fn announce(
    window: &tauri::WebviewWindow,
    text: &str,
    assertive: bool,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        let hwnd = window.hwnd().map_err(|err| err.to_string())?;
        windows::raise_notification(hwnd.0, text, assertive).map_err(|err| err.to_string())
    }
    #[cfg(not(windows))]
    {
        let _ = (window, text, assertive);
        Err("This platform has no native announcer; live regions are used.".to_owned())
    }
}

/// Turns off the webview's browser accelerator keys.
pub(crate) fn disable_browser_accelerators(
    webview: &tauri::webview::PlatformWebview,
) -> Accelerators {
    #[cfg(windows)]
    {
        match windows::disable_browser_accelerators(&webview.controller()) {
            Ok(true) => Accelerators::Disabled,
            Ok(false) | Err(_) => Accelerators::Failed,
        }
    }
    #[cfg(not(windows))]
    {
        let _ = webview;
        Accelerators::None
    }
}

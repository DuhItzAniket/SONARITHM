// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The webview never loads a remote URL (09 §4, NFR-04): every navigation away from the
//! app's own origin is refused, whatever a link or script asks for.

use tauri::Url;

/// Whether the webview may navigate to `url`: only the bundled UI. Tauri serves it from
/// `tauri://localhost` on macOS and Linux and `http(s)://tauri.localhost` on Windows; debug
/// builds also allow the Vite dev server of `tauri dev`.
pub(crate) fn allowed(url: &Url) -> bool {
    let host = url.host_str();
    match url.scheme() {
        "tauri" => host == Some("localhost"),
        "http" | "https" if host == Some("tauri.localhost") => true,
        "http" if cfg!(debug_assertions) => {
            host == Some("localhost") && url.port() == Some(DEV_PORT)
        }
        _ => false,
    }
}

/// The Vite dev server's port (`vite.config.ts`, `tauri.conf.json`).
const DEV_PORT: u16 = 1420;

#[cfg(test)]
mod tests {
    use super::*;

    fn check(url: &str) -> bool {
        Url::parse(url).is_ok_and(|url| allowed(&url))
    }

    #[test]
    fn nfr_04_the_bundled_ui_loads_on_every_os() {
        assert!(check("tauri://localhost/index.html"));
        assert!(check("http://tauri.localhost/"));
        assert!(check("https://tauri.localhost/assets/index.js"));
    }

    #[test]
    fn nfr_04_no_remote_url_loads() {
        for url in [
            "https://example.org/",
            "http://tauri.localhost.example.org/",
            "https://localhost:1420/",
            "http://127.0.0.1:1420/",
            "file:///etc/passwd",
            "data:text/html,hi",
            "javascript:alert(1)",
            "tauri://evil/",
        ] {
            assert!(!check(url), "{url} must not load");
        }
    }

    #[test]
    fn the_dev_server_loads_only_in_debug_builds() {
        assert_eq!(check("http://localhost:1420/"), cfg!(debug_assertions));
        assert!(!check("http://localhost:8080/"));
    }
}

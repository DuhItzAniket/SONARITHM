// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A02-T05..T07 spike host: IPC benchmarks (Channel, binary `ipc::Response`), WebGL2 frame
//! timing and accessibility prototypes. Usage: `README.md`. Throwaway code.
//!
//! Command line: `--page <file.html>` (default `index.html`), `--autorun <suite>` (run the
//! suite, write the report and exit), `--out <file>` (report path; stdout if absent) and,
//! on Windows, `--gpu high|low` (asks WebView2 for that adapter).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::ipc::{Channel, InvokeResponseBody, Response};
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

struct Opts {
    page: String,
    autorun: Option<String>,
    out: Option<PathBuf>,
    gpu: Option<String>,
}

fn opts() -> Opts {
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    Opts {
        page: arg("--page").unwrap_or_else(|| "index.html".into()),
        autorun: arg("--autorun"),
        out: arg("--out").map(PathBuf::from),
        gpu: arg("--gpu"),
    }
}

/// Wall clock in milliseconds. The webview compares it with `performance.timeOrigin +
/// performance.now()` for one-way Channel latency (both read the system clock).
fn epoch_ms() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64() * 1e3)
}

/// The suite to run without interaction, if any.
#[tauri::command]
fn autorun(opts: State<'_, Opts>) -> Option<String> {
    opts.autorun.clone()
}

/// Round-trip baseline: a command with no payload.
#[tauri::command]
fn bench_ping() {}

/// A binary view of `bytes` bytes returned as `ipc::Response` (09 §2: bulk data).
#[tauri::command]
fn bench_binary(bytes: usize) -> Response {
    let mut v = vec![0x5Au8; bytes.max(8)];
    v[..8].copy_from_slice(&epoch_ms().to_le_bytes());
    Response::new(v)
}

/// The same payload serialized as JSON (a number array), for contrast.
#[tauri::command]
fn bench_json(bytes: usize) -> Vec<u8> {
    vec![0x5A; bytes]
}

/// Streams `count` binary messages of `size` bytes over a Channel, `interval_us` apart
/// (0: as fast as possible). Each message starts with its index and send time (f64 LE).
#[tauri::command]
fn bench_channel(channel: Channel<InvokeResponseBody>, size: usize, count: usize, interval_us: u64) {
    std::thread::spawn(move || {
        for i in 0..count {
            let mut b = vec![0u8; size.max(16)];
            b[..8].copy_from_slice(&(i as f64).to_le_bytes());
            b[8..16].copy_from_slice(&epoch_ms().to_le_bytes());
            if channel.send(InvokeResponseBody::Raw(b)).is_err() {
                break;
            }
            if interval_us > 0 {
                std::thread::sleep(Duration::from_micros(interval_us));
            }
        }
    });
}

/// A02-T07 (OD-05): speaks `text` through a UI Automation notification on the window
/// (`UiaRaiseNotificationEvent`), the native route of 08 §4. Windows only.
#[tauri::command]
fn announce_native(window: tauri::WebviewWindow, text: String, assertive: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::Accessibility::{
            NotificationKind_Other, NotificationProcessing_ImportantMostRecent, NotificationProcessing_MostRecent,
            UiaHostProviderFromHwnd, UiaRaiseNotificationEvent,
        };
        use windows::core::BSTR;
        let hwnd = HWND(window.hwnd().map_err(|e| e.to_string())?.0);
        let processing = if assertive { NotificationProcessing_ImportantMostRecent } else { NotificationProcessing_MostRecent };
        // SAFETY: `hwnd` is the live top-level window of this webview; the provider and the
        // BSTRs outlive the call.
        unsafe {
            let provider = UiaHostProviderFromHwnd(hwnd).map_err(|e| e.to_string())?;
            UiaRaiseNotificationEvent(&provider, NotificationKind_Other, processing, &BSTR::from(text), &BSTR::from("sonarithm.announce"))
                .map_err(|e| e.to_string())
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (window, text, assertive);
        Err("the native route is prototyped on Windows only".into())
    }
}

/// Receives the suite's results, adds host facts, writes them, and exits after an autorun.
#[tauri::command]
fn report(app: AppHandle, opts: State<'_, Opts>, mut results: serde_json::Value) -> Result<(), String> {
    results["host"] = serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "tauri": tauri::VERSION,
        "webview": tauri::webview_version().unwrap_or_default(),
        "gpuRequest": opts.gpu,
    });
    let text = serde_json::to_string_pretty(&results).map_err(|e| e.to_string())?;
    match &opts.out {
        Some(path) => std::fs::write(path, text).map_err(|e| e.to_string())?,
        None => println!("{text}"),
    }
    if opts.autorun.is_some() {
        app.exit(0);
    }
    Ok(())
}

fn main() {
    let result = tauri::Builder::default()
        .manage(opts())
        .setup(|app| {
            let opts = app.state::<Opts>();
            let builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::App(opts.page.clone().into()))
                .title("SONARITHM A02 spike")
                .inner_size(1280.0, 800.0);
            // WebView2 takes Chromium switches; Tauri's defaults must be kept when replacing them.
            #[cfg(windows)]
            let builder = match opts.gpu.as_deref() {
                Some(gpu) => builder.additional_browser_args(&format!(
                    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection {}",
                    if gpu == "high" { "--force_high_performance_gpu" } else { "--force_low_power_gpu" },
                )),
                None => builder,
            };
            builder.build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            autorun,
            bench_ping,
            bench_binary,
            bench_json,
            bench_channel,
            announce_native,
            report
        ])
        .run(tauri::generate_context!());
    if let Err(e) = result {
        eprintln!("tauri: {e}");
        std::process::exit(1);
    }
}

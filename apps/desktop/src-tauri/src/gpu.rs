// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The discrete-GPU option (B04-T02, F-044; the `gpu` setting, approved by the coordinator on
//! 2026-10-30). WebView2 draws on the integrated GPU of a hybrid laptop unless its browser
//! arguments ask for the high-performance one. Passing browser arguments replaces wry's
//! defaults, so they are repeated first. The other webviews take no such argument.

use sonarithm_app::settings::GpuPreference;

/// wry's default WebView2 browser arguments (wry 0.57 `webview2::create_environment`): no
/// "mini menu", no PDF mini menu, no `SmartScreen`. Kept whenever arguments are passed.
pub(crate) const WRY_DEFAULT_ARGS: &str =
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

/// The WebView2 switch that picks the discrete GPU (F-044).
pub(crate) const HIGH_PERFORMANCE_ARG: &str = "--force_high_performance_gpu";

/// The browser arguments for `preference`, or `None` to leave wry's defaults alone.
pub(crate) fn browser_args(preference: GpuPreference) -> Option<String> {
    match preference {
        GpuPreference::HighPerformance => {
            Some(format!("{WRY_DEFAULT_ARGS} {HIGH_PERFORMANCE_ARG}"))
        }
        _ => None,
    }
}

/// What the UI learns at start-up about the GPU setting, through the window's
/// initialization script (as the smoke configuration is passed): the host found WebView2
/// unwilling to take the switch and started without it.
pub(crate) fn start_script(gpu_fallback: bool) -> String {
    format!("window.__SONARITHM_START__ = {{ gpuFallback: {gpu_fallback} }};")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f_044_the_high_performance_switch_keeps_the_default_arguments() {
        let args = browser_args(GpuPreference::HighPerformance);
        assert_eq!(
            args.as_deref(),
            Some(
                "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection \
                 --force_high_performance_gpu"
            )
        );
    }

    #[test]
    fn f_044_the_default_passes_no_arguments() {
        assert_eq!(browser_args(GpuPreference::Default), None);
    }

    #[test]
    fn the_start_script_carries_the_fallback() {
        assert_eq!(
            start_script(true),
            "window.__SONARITHM_START__ = { gpuFallback: true };"
        );
    }
}

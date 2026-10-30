// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Tauri's build step. The app manifest lists the host's commands, so each needs a
//! permission in `capabilities/` (least privilege, 09 §4).

fn main() {
    let manifest = tauri_build::AppManifest::new().commands(&[
        "ipc_dispatch",
        "ipc_binary",
        "ipc_subscribe",
        "ipc_unsubscribe",
        "announce_native",
        "smoke_report",
    ]);
    if let Err(err) = tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
    {
        // A build script reports failure through a panic; there is no other channel.
        #[expect(clippy::panic, reason = "build scripts fail by panicking")]
        {
            panic!("tauri-build failed: {err:#}");
        }
    }
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { invoke } from "@tauri-apps/api/core";

import type { NativeAnnouncer } from "./announcer.ts";

/**
 * The Tauri host's native route (`announce_native`, ADR-0025): a UI Automation notification
 * on Windows. Elsewhere the host refuses, and the routed announcer falls back to live
 * regions. Only the platform call lives in the host; the route is chosen in shared code.
 */
export function tauriNativeAnnouncer(): NativeAnnouncer {
  return {
    async announce(text, assertive) {
      await invoke("announce_native", { text, assertive });
    },
  };
}

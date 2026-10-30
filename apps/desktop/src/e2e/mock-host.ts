// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { MockHandlers, Request } from "@sonarithm/ipc";
import { createMockTransport } from "@sonarithm/ipc";
import { mockIPC } from "@tauri-apps/api/mocks";

/**
 * Tauri's `mockIPC` answering the host's commands from `@sonarithm/ipc`'s in-memory host,
 * so Playwright drives the real UI code, including `createTauriTransport`, in any browser
 * (testing-strategy §12). Flows add handlers here as they need them.
 */
const handlers: MockHandlers = {};
const host = createMockTransport(handlers);

mockIPC(async (command, payload) => {
  const { request } = (payload ?? {}) as { request?: Request };
  switch (command) {
    case "ipc_dispatch":
      return request === undefined ? null : host.invoke(request);
    case "ipc_binary":
      return request === undefined ? null : host.binary(request);
    case "ipc_subscribe":
      return 1;
    default:
      return null;
  }
});

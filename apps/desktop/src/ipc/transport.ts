// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Channel,
  Event,
  IpcError,
  IpcTransport,
  Reply,
  Request,
} from "@sonarithm/ipc";
import { IpcFailure } from "@sonarithm/ipc";

/**
 * The parts of Tauri's JavaScript API the transport uses, injected so tests run without a
 * webview (`tauri-core.ts` binds the real ones).
 */
export interface TauriCore {
  invoke(command: string, args: Record<string, unknown>): Promise<unknown>;
  /** A Tauri `Channel` whose messages go to `onMessage`. */
  channel(onMessage: (message: Event) => void): unknown;
}

/** The host's commands (`src-tauri/src/commands.rs`). */
const COMMANDS = {
  dispatch: "ipc_dispatch",
  binary: "ipc_binary",
  subscribe: "ipc_subscribe",
  unsubscribe: "ipc_unsubscribe",
} as const;

/**
 * The Tauri {@link IpcTransport} (09 §2.3): JSON commands through one generic handler,
 * binary views as raw IPC bodies, events over Tauri channels. It only moves messages; an
 * Electron host would replace this file and the host crate, nothing else (OD-15).
 */
export function createTauriTransport(core: TauriCore): IpcTransport {
  return {
    async invoke(request: Request): Promise<Reply> {
      try {
        return (await core.invoke(COMMANDS.dispatch, { request })) as Reply;
      } catch (error) {
        throw toFailure(error);
      }
    },
    async binary(request: Request): Promise<ArrayBuffer> {
      let body: unknown;
      try {
        body = await core.invoke(COMMANDS.binary, { request });
      } catch (error) {
        throw toFailure(error);
      }
      // F-042: a CSP without the IPC origins silently turns binary replies into JSON
      // arrays. Fail loudly instead of decoding a slow copy.
      if (!(body instanceof ArrayBuffer)) {
        throw new IpcFailure({
          code: "SNR-APP-050",
          message: `The host answered ${request.command} with JSON instead of a binary view; the binary IPC path is not in use.`,
          hint: "Allow the IPC origins in the CSP's connect-src (F-042).",
          pointer: null,
        });
      }
      return body;
    },
    subscribe(channel: Channel, handler: (event: Event) => void): () => void {
      let active = true;
      const onEvent = core.channel((event) => {
        if (active) {
          handler(event);
        }
      });
      const id = core
        .invoke(COMMANDS.subscribe, { channel, onEvent })
        .then((value) => (typeof value === "number" ? value : null))
        .catch(() => null);
      return () => {
        active = false;
        void id.then((value) =>
          value === null
            ? undefined
            : core
                .invoke(COMMANDS.unsubscribe, { id: value })
                .catch(() => undefined),
        );
      };
    },
  };
}

/** The app core's typed error, or the host's own refusal as `SNR-APP-009`. */
export function toFailure(error: unknown): IpcFailure {
  if (isIpcError(error)) {
    return new IpcFailure({
      code: error.code,
      message: error.message,
      hint: error.hint ?? null,
      pointer: error.pointer ?? null,
    });
  }
  return new IpcFailure({
    code: "SNR-APP-009",
    message:
      typeof error === "string" ? error : "The host refused the command.",
    hint: null,
    pointer: null,
  });
}

function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const record = value as Record<string, unknown>;
  return typeof record.code === "string" && typeof record.message === "string";
}

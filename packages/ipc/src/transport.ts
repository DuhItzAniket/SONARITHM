// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Channel,
  Event,
  IpcError,
  Reply,
  Request,
} from "./generated/protocol.ts";
import type { MeterFrame } from "./meters.ts";

/**
 * The only seam between the UI and a host (09 §2.3, ADR-0017, OD-15). The Tauri host
 * implements it over `invoke` and Channels; an Electron host would implement it over its
 * bridge; tests use {@link createMockTransport}. React code never calls a host API directly.
 */
export interface IpcTransport {
  /** Sends a command and resolves with its reply, or rejects with an {@link IpcFailure}. */
  invoke(request: Request): Promise<Reply>;
  /** Calls `handler` for every event on `channel`; returns a function that unsubscribes. */
  subscribe(channel: Channel, handler: (event: Event) => void): () => void;
  /** Sends a command whose reply is a binary view (ADR-0017 §6). */
  binary(request: Request): Promise<ArrayBuffer>;
  /**
   * Calls `handler` for every meter frame (B01-T04). Meters are binary on the `meters`
   * channel, never JSON events (09 §2); returns a function that unsubscribes.
   */
  meters(handler: (frame: MeterFrame) => void): () => void;
}

/** A command the host could not run, carrying the typed error (12 §1). */
export class IpcFailure extends Error {
  readonly code: string;
  readonly hint: string | null;
  readonly pointer: string | null;

  constructor(error: IpcError) {
    super(error.message);
    this.name = "IpcFailure";
    this.code = error.code;
    this.hint = error.hint;
    this.pointer = error.pointer;
  }
}

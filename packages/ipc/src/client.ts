// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Channel,
  BinaryCommand,
  JsonCommand,
  Event,
  ReplyFor,
  RequestFor,
} from "./generated/protocol.ts";
import { PROTOCOL_VERSION, REPLY_KINDS } from "./generated/protocol.ts";
import type { IpcTransport } from "./transport.ts";
import { IpcFailure } from "./transport.ts";
import type { MeterFrame } from "./meters.ts";

/** The UI and the host were built from different commits (09 §2.2, ADR-0017 §5). */
export class ProtocolMismatchError extends Error {
  readonly ui: number;
  readonly host: number | null;

  constructor(host: number | null) {
    const hostText =
      host === null ? "another version" : `version ${String(host)}`;
    super(
      `The SONARITHM user interface speaks IPC protocol version ${String(PROTOCOL_VERSION)}, ` +
        `but the host speaks ${hostText}. Build both from the same commit.`,
    );
    this.name = "ProtocolMismatchError";
    this.ui = PROTOCOL_VERSION;
    this.host = host;
  }
}

/** The typed client: the UI's view of the host, built on an {@link IpcTransport}. */
export interface Client {
  /** Sends a command and resolves with the reply type that command gives. */
  call<C extends JsonCommand>(request: RequestFor<C>): Promise<ReplyFor<C>>;
  /** Calls `handler` for every event on `channel`; returns a function that unsubscribes. */
  subscribe(channel: Channel, handler: (event: Event) => void): () => void;
  /** Sends a command whose reply is a binary view; read it with `readView`. */
  binary(request: RequestFor<BinaryCommand>): Promise<ArrayBuffer>;
  /** Calls `handler` for every meter frame; returns a function that unsubscribes. */
  meters(handler: (frame: MeterFrame) => void): () => void;
}

/**
 * Checks the protocol version with the host (`hello`) and returns the typed client. A
 * mismatch rejects at once with a {@link ProtocolMismatchError}, before any other command.
 */
export async function connect(transport: IpcTransport): Promise<Client> {
  let protocol: number | null = null;
  try {
    const hello = await transport.invoke({
      command: "hello",
      args: { protocol: PROTOCOL_VERSION },
    });
    if (hello.reply === "hello") {
      protocol = hello.protocol;
    }
  } catch (error) {
    if (error instanceof IpcFailure && error.code === "SNR-APP-001") {
      throw new ProtocolMismatchError(null);
    }
    throw error;
  }
  if (protocol !== PROTOCOL_VERSION) {
    throw new ProtocolMismatchError(protocol);
  }
  return {
    async call<C extends JsonCommand>(
      request: RequestFor<C>,
    ): Promise<ReplyFor<C>> {
      const reply = await transport.invoke(request);
      const { command } = request as { command: JsonCommand };
      const expected: string = REPLY_KINDS[command];
      if (reply.reply !== expected) {
        throw new IpcFailure({
          code: "SNR-APP-009",
          message: `The host answered ${command} with a ${reply.reply} reply, not ${expected}.`,
          hint: null,
          pointer: null,
        });
      }
      return reply as ReplyFor<C>;
    },
    subscribe: (channel, handler) => transport.subscribe(channel, handler),
    binary: (request) => transport.binary(request),
    meters: (handler) => transport.meters(handler),
  };
}

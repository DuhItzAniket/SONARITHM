// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Channel,
  BinaryCommand,
  Command,
  JsonCommand,
  Event,
  Reply,
  ReplyFor,
  Request,
  RequestFor,
} from "./generated/protocol.ts";
import { EVENT_CHANNELS, PROTOCOL_VERSION } from "./generated/protocol.ts";
import type { MeterFrame } from "./meters.ts";
import type { IpcTransport } from "./transport.ts";
import { IpcFailure } from "./transport.ts";

/** A handler per command; each returns the reply type its command gives. */
export type MockHandlers = {
  [C in JsonCommand]?: (
    request: RequestFor<C>,
  ) => ReplyFor<C> | Promise<ReplyFor<C>>;
};

/** A transport for tests and Storybook (`mockIPC`). */
export interface MockTransport extends IpcTransport {
  /** Every request sent so far, in order. */
  readonly requests: readonly Request[];
  /** Delivers an event to the subscribers of its channel. */
  emit(event: Event): void;
  /** Delivers a meter frame to the meter subscribers. */
  emitMeters(frame: MeterFrame): void;
}

/** Options for {@link createMockTransport}. */
export interface MockOptions {
  /** The protocol version the mock host speaks; by default the UI's. */
  readonly protocol?: number;
  /** Binary replies by command: a buffer, or a function of the request (a table page). */
  readonly binary?: Partial<
    Record<
      BinaryCommand,
      ArrayBuffer | ((request: RequestFor<BinaryCommand>) => ArrayBuffer)
    >
  >;
}

/**
 * A host in memory. It answers `hello` like the real host (failing with `SNR-APP-001` on a
 * version mismatch) and every other command from `handlers`; a command without a handler
 * fails with `SNR-APP-009`.
 */
export function createMockTransport(
  handlers: MockHandlers = {},
  options: MockOptions = {},
): MockTransport {
  const requests: Request[] = [];
  const subscribers = new Map<Channel, Set<(event: Event) => void>>();
  const meterSubscribers = new Set<(frame: MeterFrame) => void>();
  const hostProtocol = options.protocol ?? PROTOCOL_VERSION;

  function unhandled(command: Command): IpcFailure {
    return new IpcFailure({
      code: "SNR-APP-009",
      message: `The mock host has no handler for ${command}.`,
      hint: null,
      pointer: null,
    });
  }

  async function answer(request: Request): Promise<Reply> {
    if (request.command === "hello") {
      if (request.args.protocol !== hostProtocol) {
        throw new IpcFailure({
          code: "SNR-APP-001",
          message: `The user interface speaks IPC protocol ${String(request.args.protocol)}, but this host speaks protocol ${String(hostProtocol)}.`,
          hint: null,
          pointer: null,
        });
      }
      return { reply: "hello", protocol: hostProtocol, appVersion: "mock" };
    }
    const handler = handlers[request.command as JsonCommand] as
      ((request: Request) => Reply | Promise<Reply>) | undefined;
    if (handler === undefined) {
      throw unhandled(request.command);
    }
    return handler(request);
  }

  return {
    requests,
    async invoke(request) {
      requests.push(request);
      return answer(request);
    },
    subscribe(channel, handler) {
      const set = subscribers.get(channel) ?? new Set();
      set.add(handler);
      subscribers.set(channel, set);
      return () => {
        set.delete(handler);
      };
    },
    binary(request) {
      requests.push(request);
      const reply = options.binary?.[request.command as BinaryCommand];
      const buffer =
        typeof reply === "function"
          ? reply(request as RequestFor<BinaryCommand>)
          : reply;
      return buffer === undefined
        ? Promise.reject(unhandled(request.command))
        : Promise.resolve(buffer);
    },
    meters(handler) {
      meterSubscribers.add(handler);
      return () => {
        meterSubscribers.delete(handler);
      };
    },
    emitMeters(frame) {
      for (const handler of meterSubscribers) {
        handler(frame);
      }
    },
    emit(event) {
      const channel = EVENT_CHANNELS[event.event];
      for (const handler of subscribers.get(channel) ?? []) {
        handler(event);
      }
    },
  };
}

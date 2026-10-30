// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { LoopSpan, Request, TransportState } from "@sonarithm/ipc";
import { assign, fromPromise, setup } from "xstate";

/** What the user asked the transport to do (09 §2.1). */
export type TransportIntent =
  | { readonly kind: "state" }
  | { readonly kind: "play" }
  | { readonly kind: "pause" }
  | { readonly kind: "stop" }
  | { readonly kind: "seek"; readonly seconds: number }
  | { readonly kind: "speed"; readonly speed: number }
  | { readonly kind: "loop"; readonly range: LoopSpan | null };

/** The app-core command for an intent. */
export function requestFor(intent: TransportIntent): Request {
  switch (intent.kind) {
    case "state":
      return { command: "transport_state" };
    case "play":
      return { command: "transport_play" };
    case "pause":
      return { command: "transport_pause" };
    case "stop":
      return { command: "transport_stop" };
    case "seek":
      return { command: "transport_seek", args: { seconds: intent.seconds } };
    case "speed":
      return { command: "transport_set_speed", args: { speed: intent.speed } };
    case "loop":
      return { command: "transport_set_loop", args: { range: intent.range } };
  }
}

/** Sends an intent to the app core and resolves with the authoritative state. */
export type SendIntent = (intent: TransportIntent) => Promise<TransportState>;

export interface TransportInput {
  readonly send: SendIntent;
}

export interface TransportContext {
  /** The last state the app core reported, in a reply or a clock anchor. */
  readonly authority: TransportState | null;
  /** The intent the app core is working on. */
  readonly current: TransportIntent | null;
  /** The newest intent made while another was in flight; it replaces older ones. */
  readonly next: TransportIntent | null;
  /** Why the last intent failed, if it did. */
  readonly failure: string | null;
  readonly send: SendIntent;
}

export type TransportEvent =
  | { readonly type: "intent"; readonly intent: TransportIntent }
  | { readonly type: "authority"; readonly state: TransportState };

const STATUSES = ["stopped", "playing", "paused"] as const;
type Status = (typeof STATUSES)[number];

/**
 * The status region: one node per Rust `PlaybackStatus`, plus `unknown` before the first
 * report. It moves only when the app core's state says so, never on an intent.
 */
function statusStates() {
  const to = (from: Status | "unknown") =>
    STATUSES.filter((status) => status !== from).map((status) => ({
      guard: ({ context }: { context: TransportContext }) =>
        context.authority?.status === status,
      target: status,
    }));
  return {
    unknown: { always: to("unknown") },
    stopped: { always: to("stopped") },
    playing: { always: to("playing") },
    paused: { always: to("paused") },
  };
}

/**
 * The `transport` machine (09 §3, 06 §5): a mirror of the Rust transport, which is the
 * authority. Intents go to the app core one at a time; the machine shows a status only
 * when the app core reports it, in the reply or in a clock anchor, so the Rust state wins
 * over any intent it did not carry out. It never sends a command on its own, so playback
 * starts only on a user command (A11Y-15).
 */
export const transportMachine = setup({
  types: {
    context: {} as TransportContext,
    events: {} as TransportEvent,
    input: {} as TransportInput,
  },
  actors: {
    request: fromPromise<
      TransportState,
      { send: SendIntent; intent: TransportIntent }
    >(({ input }) => input.send(input.intent)),
  },
}).createMachine({
  id: "transport",
  type: "parallel",
  context: ({ input }) => ({
    authority: null,
    current: null,
    next: null,
    failure: null,
    send: input.send,
  }),
  on: {
    authority: {
      actions: assign({ authority: ({ event }) => event.state }),
    },
  },
  states: {
    status: {
      initial: "unknown",
      states: statusStates(),
    },
    request: {
      initial: "idle",
      states: {
        idle: {
          on: {
            intent: {
              target: "busy",
              actions: assign({ current: ({ event }) => event.intent }),
            },
          },
        },
        busy: {
          invoke: {
            src: "request",
            input: ({ context }) => ({
              send: context.send,
              intent: context.current ?? { kind: "state" },
            }),
            onDone: {
              target: "settling",
              actions: assign({
                authority: ({ event }) => event.output,
                current: null,
                failure: null,
              }),
            },
            onError: {
              target: "settling",
              actions: assign({
                current: null,
                failure: ({ event }) =>
                  event.error instanceof Error
                    ? event.error.message
                    : String(event.error),
              }),
            },
          },
          on: {
            intent: {
              actions: assign({ next: ({ event }) => event.intent }),
            },
          },
        },
        settling: {
          always: [
            {
              guard: ({ context }) => context.next !== null,
              target: "busy",
              actions: assign({
                current: ({ context }) => context.next,
                next: null,
              }),
            },
            { target: "idle" },
          ],
        },
      },
    },
  },
});

/** The status the machine shows. */
export type ShownStatus = Status | "unknown";

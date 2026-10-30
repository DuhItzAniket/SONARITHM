// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, RequestFor, TransportState } from "@sonarithm/ipc";
import { useCallback, useEffect, useRef, useState } from "react";
import type { Actor, SnapshotFrom } from "xstate";
import { createActor } from "xstate";

import type { ShownStatus, TransportIntent } from "./machine.ts";
import { requestFor, transportMachine } from "./machine.ts";

/** The app-core commands whose reply is the transport state. */
type TransportCommand =
  | "transport_state"
  | "transport_play"
  | "transport_pause"
  | "transport_stop"
  | "transport_seek"
  | "transport_set_speed"
  | "transport_set_loop";

/** What the transport UI reads and does. */
export interface TransportView {
  /** The status the app core last reported; `unknown` before the first report. */
  readonly status: ShownStatus;
  /** The app core's last state, or `null` before the first report. */
  readonly state: TransportState | null;
  /** Sends an intent; the status changes only when the app core reports it. */
  readonly intend: (intent: TransportIntent) => void;
}

/**
 * The transport machine for one connection: it asks for the state at once, follows the
 * `clock` channel's anchors, and sends the user's intents (B01-T05).
 */
export function useTransport(client: Client | null): TransportView {
  const actor = useRef<Actor<typeof transportMachine> | null>(null);
  const [snapshot, setSnapshot] = useState<SnapshotFrom<
    typeof transportMachine
  > | null>(null);

  useEffect(() => {
    if (client === null) {
      return;
    }
    const running = createActor(transportMachine, {
      input: {
        send: async (intent) =>
          // Every transport command replies with the authoritative state (09 §2.1).
          client.call(
            requestFor(intent) as RequestFor<TransportCommand>,
          ) as Promise<TransportState>,
      },
    });
    const subscription = running.subscribe(setSnapshot);
    running.start();
    actor.current = running;
    running.send({ type: "intent", intent: { kind: "state" } });
    const stopClock = client.subscribe("clock", (event) => {
      if (event.event === "clock") {
        running.send({ type: "authority", state: event.transport });
      }
    });
    return () => {
      stopClock();
      subscription.unsubscribe();
      running.stop();
      actor.current = null;
    };
  }, [client]);

  const intend = useCallback((intent: TransportIntent) => {
    actor.current?.send({ type: "intent", intent });
  }, []);

  const value = snapshot?.value as { status: ShownStatus } | undefined;
  return {
    status: value?.status ?? "unknown",
    state: snapshot?.context.authority ?? null,
    intend,
  };
}

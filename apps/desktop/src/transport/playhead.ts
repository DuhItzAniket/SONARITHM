// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { ClockAnchor, Client } from "@sonarithm/ipc";
import type { PlayheadAnchor } from "@sonarithm/viz/playhead";
import { PlayheadClock } from "@sonarithm/viz/playhead";
import { useEffect, useState } from "react";

/** The clock's view of an anchor: the engine's position, speed and whether it plays. */
export function toPlayheadAnchor(anchor: ClockAnchor): PlayheadAnchor {
  return {
    positionS: anchor.transport.positionS,
    speed: anchor.transport.speed,
    playing: anchor.transport.status === "playing",
  };
}

/**
 * The latency to correct the playhead by (B04-T04, ADR-0017 "Clock anchors"): the
 * anchor's `outputLatencyS`, or no correction at all when the app core knows none. `null`
 * is never replaced by a guess.
 */
export function latencyOf(anchor: ClockAnchor): number {
  return anchor.outputLatencyS ?? 0;
}

/**
 * The playhead clock for one connection: every `clock` anchor updates it at its receive
 * time, and the latency follows the device. The clock itself is read by `Playhead` every
 * frame, outside React; only a latency change re-renders.
 */
export function usePlayheadClock(client: Client | null): {
  readonly clock: PlayheadClock;
  readonly latencyS: number;
} {
  const [clock] = useState(() => new PlayheadClock());
  const [latencyS, setLatencyS] = useState(0);
  useEffect(() => {
    if (client === null) {
      return;
    }
    return client.subscribe("clock", (event) => {
      if (event.event !== "clock") return;
      clock.update(toPlayheadAnchor(event), performance.now() / 1000);
      const next = latencyOf(event);
      setLatencyS((previous) => (previous === next ? previous : next));
    });
  }, [client, clock]);
  return { clock, latencyS };
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, CursorState } from "@sonarithm/ipc";
import { useEffect, useState } from "react";

/** A `cursor` event or reply without its tags: the state itself. */
function stateOf(
  value: CursorState & { readonly event?: string; readonly reply?: string },
): CursorState {
  return {
    seq: value.seq,
    dataset: value.dataset,
    row: value.row,
    x: value.x,
    node: value.node,
    path: value.path,
    excluded: value.excluded,
    origin: value.origin,
    braille: value.braille,
  };
}

/**
 * The one cursor (B04-T06; ADR-0017 "Cursor"), as Rust last reported it: from `cursor_get`
 * when the app connects, then every `cursor` event. Rust owns it (09 §3); views only show it
 * and send `cursor_set`. A late or repeated report (an older `seq`) is ignored.
 */
export function useCursor(client: Client | null): CursorState | null {
  const [cursor, setCursor] = useState<CursorState | null>(null);
  useEffect(() => {
    if (client === null) {
      return;
    }
    let live = true;
    const take = (next: CursorState) => {
      if (live) {
        setCursor((current) =>
          current !== null && current.seq >= next.seq ? current : next,
        );
      }
    };
    const unsubscribe = client.subscribe("cursor", (event) => {
      if (event.event === "cursor") {
        take(stateOf(event));
      }
    });
    client.call({ command: "cursor_get" }).then(
      (reply) => {
        take(stateOf(reply));
      },
      () => undefined,
    );
    return () => {
      live = false;
      unsubscribe();
    };
  }, [client]);
  return cursor;
}

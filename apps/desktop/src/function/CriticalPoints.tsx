// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, CriticalPointInfo, CursorState } from "@sonarithm/ipc";
import { ActionList } from "@sonarithm/ui";
import { useState } from "react";

import type { Announcer } from "../announcer/announcer.ts";
import { useLocalizer } from "../i18n.ts";
import styles from "./function.module.css";
import { currentPoint } from "./readout.ts";

/**
 * A function's critical points as a list (C03-T08b; A11Y-10, A11Y-24, A11Y-28): each row is
 * the text equivalent of a cue, as the app core words it. Enter on a row moves Rust's one
 * cursor there (`cursor_set`, origin `nav`, so its arrival is announced as the explorer's
 * is); the table, the plot and the readout follow. The list keeps no selection: the row
 * the cursor is on is marked current, and the focus never moves when the cursor does. A
 * feature with no row of its own (a horizontal asymptote) says so instead.
 */
export function CriticalPoints({
  client,
  dataset,
  points,
  cursor,
  announcer,
}: {
  readonly client: Client;
  readonly dataset: string;
  readonly points: readonly CriticalPointInfo[];
  readonly cursor: CursorState | null;
  readonly announcer: Announcer;
}) {
  const t = useLocalizer();
  // The point the listener last went to: when two points share a row, that one is current.
  const [entered, setEntered] = useState<number | null>(null);
  if (points.length === 0) {
    return <p className={styles.note}>{t.format("ui-function-points-none")}</p>;
  }
  return (
    <ActionList
      label={t.format("ui-function-points")}
      items={points.map((point, index) => ({ id: index, text: point.text }))}
      current={currentPoint(points, dataset, cursor, entered)}
      onAction={(id) => {
        const point = points[Number(id)];
        if (point === undefined) return;
        if (point.row === null) {
          announcer.announce({
            text: t.format("ui-function-no-row", { name: point.text }),
            assertive: false,
          });
          return;
        }
        setEntered(Number(id));
        void client
          .call({
            command: "cursor_set",
            args: { dataset, row: point.row, origin: "nav" },
          })
          .catch(() => undefined);
      }}
    />
  );
}

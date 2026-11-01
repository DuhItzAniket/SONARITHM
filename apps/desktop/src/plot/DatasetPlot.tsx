// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, CursorState, DatasetSummary } from "@sonarithm/ipc";
import { isValid, readView } from "@sonarithm/ipc";
import type { PlotCursor } from "@sonarithm/viz";
import { Plot } from "@sonarithm/viz";
import { useEffect, useMemo, useRef, useState } from "react";

import { useLocalizer } from "../i18n.ts";
import { headingOf } from "../table/paging.ts";
import { axesOf, nearest } from "./axes.ts";
import styles from "./plot.module.css";

/** The plot's height in CSS pixels. */
const HEIGHT = 240;

/** The tile a plot draws: one point per kept row, in x order. */
interface Tile {
  readonly rows: readonly number[];
  readonly x: Float64Array;
  readonly y: (number | null)[];
}

/** A one-row value of a numeric column, or null. */
function valueAt(
  column: ReturnType<typeof readView>["columns"][number] | undefined,
  row: number,
): number | null {
  if (column === undefined || !isValid(column, row)) return null;
  if (column.dataType === "f64") return column.values[row] ?? null;
  if (column.dataType === "i64") {
    const value = column.values[row];
    return value === undefined ? null : Number(value);
  }
  return null;
}

/**
 * The Plot tab (B04-T06): the dataset's y field against its x field (`axesOf`), drawn from an
 * M4 tile (`view_get`) as wide as the plot, with the shared cursor as a crosshair. A click
 * moves the cursor to the nearest drawn point (`cursor_set`, origin \`plot\`), which Rust
 * announces; the table and the readout follow the same \`cursor\` event. The plot is one
 * image (A11Y-01); the table and the chart explorer are the keyboard ways to move the cursor.
 */
export function DatasetPlot({
  client,
  dataset,
  cursor,
}: {
  readonly client: Client;
  readonly dataset: DatasetSummary;
  readonly cursor: CursorState | null;
}) {
  const t = useLocalizer();
  const axes = useMemo(() => axesOf(dataset), [dataset]);
  const box = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  const [tile, setTile] = useState<Tile | null>(null);
  // The cursor's point when the tile does not hold its row (M4 keeps some rows only).
  const [fetched, setFetched] = useState<{
    readonly row: number;
    readonly point: PlotCursor;
  } | null>(null);

  // The tile is as wide as the plot in device pixels, rounded so small resizes keep it.
  useEffect(() => {
    const element = box.current;
    if (element === null) return;
    const measure = () => {
      const px = Math.round(element.clientWidth * window.devicePixelRatio);
      setWidth(Math.max(64, Math.round(px / 64) * 64));
    };
    measure();
    const resize = new ResizeObserver(measure);
    resize.observe(element);
    return () => {
      resize.disconnect();
    };
  }, []);

  useEffect(() => {
    if (axes === null || width === 0) return;
    let live = true;
    client
      .binary({
        command: "view_get",
        args: {
          dataset: dataset.id,
          x: axes.x.name,
          y: axes.y.name,
          range: null,
          pxWidth: width,
          method: "m4",
        },
      })
      .then(
        (buffer) => {
          if (!live) return;
          const view = readView(buffer);
          const row = view.columns.find((c) => c.name === "row");
          const x = view.columns.find((c) => c.name === "x");
          const y = view.columns.find((c) => c.name === "y");
          const count = view.rows;
          const rows = Array.from({ length: count }, (_, i) =>
            row?.dataType === "i64" ? Number(row.values[i] ?? 0) : i,
          );
          const xs = Float64Array.from(
            { length: count },
            (_, i) => valueAt(x, i) ?? Number.NaN,
          );
          const ys = Array.from({ length: count }, (_, i) => valueAt(y, i));
          setTile({ rows, x: xs, y: ys });
        },
        () => undefined,
      );
    return () => {
      live = false;
    };
  }, [client, dataset.id, axes, width]);

  const cursorRow =
    cursor?.dataset === dataset.id && cursor.row !== null ? cursor.row : null;
  const index =
    cursorRow === null || tile === null ? -1 : tile.rows.indexOf(cursorRow);

  // A cursor row the tile does not hold: read its two values from one table row.
  useEffect(() => {
    if (axes === null || cursorRow === null || index >= 0 || tile === null)
      return;
    if (fetched?.row === cursorRow) return;
    let live = true;
    client
      .binary({
        command: "table_page",
        args: { dataset: dataset.id, offset: cursorRow, limit: 1, sort: null },
      })
      .then(
        (buffer) => {
          if (!live) return;
          const view = readView(buffer);
          const x = valueAt(
            view.columns.find((c) => c.name === axes.x.name),
            0,
          );
          const y = valueAt(
            view.columns.find((c) => c.name === axes.y.name),
            0,
          );
          if (x !== null) setFetched({ row: cursorRow, point: { x, y } });
        },
        () => undefined,
      );
    return () => {
      live = false;
    };
  }, [client, dataset.id, axes, cursorRow, index, tile, fetched?.row]);

  const point: PlotCursor | null =
    cursorRow === null || tile === null
      ? null
      : index >= 0
        ? { x: tile.x[index] ?? 0, y: tile.y[index] ?? null }
        : fetched?.row === cursorRow
          ? fetched.point
          : null;

  if (axes === null) {
    return <p className={styles.status}>{t.format("ui-plot-no-fields")}</p>;
  }
  const label = headingOf(t, axes.y);
  return (
    <div ref={box} className={styles.plot}>
      {tile === null ? (
        <p className={styles.status}>{t.format("ui-table-loading")}</p>
      ) : (
        <Plot
          summary={{
            l1: t.format("ui-plot-label", {
              field: label,
              text: headingOf(t, axes.x),
              name: dataset.name,
            }),
            l2: t.format("ui-plot-description", {
              total: { number: dataset.rows },
            }),
          }}
          x={tile.x}
          series={[{ label, values: tile.y }]}
          height={HEIGHT}
          cursor={point}
          onPick={(value) => {
            const at = nearest(tile.x, value);
            const row = tile.rows[at];
            if (row === undefined) return;
            void client
              .call({
                command: "cursor_set",
                args: { dataset: dataset.id, row, origin: "plot" },
              })
              .catch(() => undefined);
          }}
        />
      )}
    </div>
  );
}

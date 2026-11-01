// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, CursorState, DatasetSummary } from "@sonarithm/ipc";
import { readView } from "@sonarithm/ipc";
import type { DataSort } from "@sonarithm/ui";
import { DataTable } from "@sonarithm/ui";
import type { KeyboardEvent } from "react";
import { useCallback, useEffect, useRef, useState } from "react";

import type { RoutedAnnouncer } from "../announcer/announcer.ts";
import { useCommand, useCommands } from "../commands/context.tsx";
import { useLocalizer } from "../i18n.ts";
import { GoToRow } from "./GoToRow.tsx";
import type { TableWindow } from "./paging.ts";
import {
  WINDOW_ROWS,
  lastWindowStart,
  rangeText,
  windowOf,
  windowStart,
} from "./paging.ts";
import styles from "./table.module.css";

/** Where the focus goes once a window has rendered: a row of the window and a column. */
interface FocusTarget {
  /** 0-based, within the window. */
  readonly row: number | "last";
  /** `aria-colindex`, 1-based. */
  readonly column: number;
}

/** The field a column ID sorts by (`field:<name>`, paging.ts). */
function fieldOf(column: string): string {
  return column.startsWith("field:") ? column.slice("field:".length) : column;
}

/** The focused cell's row (0-based, in the window) and column, if the focus is in a cell. */
function focusedCell(
  container: HTMLElement,
): { readonly row: number; readonly column: number } | null {
  const active = document.activeElement;
  if (!(active instanceof HTMLElement) || !container.contains(active)) {
    return null;
  }
  const row = active.closest<HTMLElement>('[role="row"]');
  const rowIndex = Number(row?.getAttribute("aria-rowindex"));
  if (!Number.isInteger(rowIndex) || rowIndex < 2) {
    return null;
  }
  const cell = active.closest<HTMLElement>("[aria-colindex]");
  const column = Number(cell?.getAttribute("aria-colindex") ?? "1");
  // aria-rowindex 1 is the header row.
  return { row: rowIndex - 2, column: Number.isInteger(column) ? column : 1 };
}

/**
 * Scrolls row `row` (0-based, in a window of `count` rows) near the top of the grid, so the
 * virtualizer renders it. Scrolling never moves the focus.
 */
function scrollToRow(element: HTMLElement, count: number, row: number): void {
  const grid = element.querySelector<HTMLElement>('[role="grid"]');
  const scroller = [grid, element].find(
    (candidate) =>
      candidate !== null && candidate.scrollHeight > candidate.clientHeight,
  );
  if (scroller !== undefined && scroller !== null) {
    // Row height is the compact row (DataTable); the header takes one row.
    const rowPx = scroller.scrollHeight / (count + 1);
    scroller.scrollTop = Math.max(0, (row - 1) * rowPx);
  }
}

/** Waits for the next frame, so the virtualizer has rendered what a scroll exposed. */
function frame(): Promise<void> {
  return new Promise((resolve) => {
    requestAnimationFrame(() => {
      resolve();
    });
  });
}

/**
 * The data table of a dataset (B04-T05; A11Y-03, keyboard-and-navigation §5.6). It holds one
 * window of rows from `table_page` (`WINDOW_ROWS`) in React Aria's Table and moves between
 * windows itself:
 *
 * - ArrowDown or PageDown on the window's last row loads the next window and focuses its
 *   first row; ArrowUp or PageUp on the first row loads the previous one and focuses its
 *   last row. The focus stays in the same column.
 * - Primary+Home and Primary+End go to the first and last cell of the whole table.
 * - "Go to row…" (command palette) loads the window holding a row and focuses it.
 * - Enter on a column header sorts through `table_page`, starting again at row 1.
 *
 * The row header is the true row number, so a screen reader names each cell's position in
 * the whole table; a visible line, also announced politely when the window changes, says
 * which rows are loaded. Nothing here moves the focus unasked.
 *
 * The focused row is the shared cursor (B04-T06, A11Y-11): focusing a row sends
 * `cursor_set` with origin `table`, which Rust does not announce, since the screen reader
 * reads the row. The cursor's row is shown selected. A cursor moved elsewhere (the plot)
 * brings its window in only while the focus is outside the table, so a `cursor` event never
 * moves the focus.
 */
export function DatasetTable({
  client,
  dataset,
  announcer,
  cursor,
}: {
  readonly client: Client;
  readonly dataset: DatasetSummary;
  readonly announcer: RoutedAnnouncer;
  /** The one cursor, as Rust last reported it. */
  readonly cursor: CursorState | null;
}) {
  const t = useLocalizer();
  const platform = useCommands().platform;
  const [window, setWindow] = useState<TableWindow | null>(null);
  const [sort, setSort] = useState<DataSort | null>(null);
  const [goToOpen, setGoToOpen] = useState(false);
  const container = useRef<HTMLDivElement>(null);
  const pendingFocus = useRef<FocusTarget | null>(null);
  const lastColumn = useRef(1);
  // Each load has a number, so a slow reply never replaces a newer one.
  const loadId = useRef(0);
  // The dataset row last sent as the cursor, so focus inside one row sends it once.
  const sentRow = useRef<number | null>(null);
  // Whether the keyboard focus is in the grid: the cursor's window waits until it leaves.
  const [focusInside, setFocusInside] = useState(false);

  const load = useCallback(
    async (start: number, order: DataSort | null): Promise<TableWindow> => {
      const id = ++loadId.current;
      const buffer = await client.binary({
        command: "table_page",
        args: {
          dataset: dataset.id,
          offset: start,
          limit: WINDOW_ROWS,
          sort:
            order === null
              ? null
              : {
                  field: fieldOf(order.column),
                  descending: order.direction === "descending",
                },
        },
      });
      const next = windowOf(t, dataset, readView(buffer), start);
      if (id === loadId.current) {
        setWindow(next);
      }
      return next;
    },
    [client, dataset, t],
  );

  useEffect(() => {
    void load(0, null).catch(() => undefined);
  }, [load]);

  /** Loads the window starting at `start`, says its range, and focuses `target` in it. */
  const moveTo = useCallback(
    (start: number, target: FocusTarget) => {
      pendingFocus.current = target;
      void load(start, sort).then(
        (next) => {
          announcer.announce({
            text: rangeText(t, next.start, next.rows.length, dataset.rows),
            assertive: false,
          });
        },
        () => undefined,
      );
    },
    [announcer, dataset.rows, load, sort, t],
  );

  // Once a window has rendered, put the focus where the move asked for it.
  useEffect(() => {
    const target = pendingFocus.current;
    const element = container.current;
    if (target === null || element === null || window === null) {
      return;
    }
    pendingFocus.current = null;
    const row = target.row === "last" ? window.rows.length - 1 : target.row;
    scrollToRow(element, window.rows.length, row);
    void (async () => {
      for (let attempt = 0; attempt < 5; attempt += 1) {
        await frame();
        const rowElement = element.querySelector<HTMLElement>(
          `[role="row"][aria-rowindex="${String(row + 2)}"]`,
        );
        const cell =
          rowElement?.querySelector<HTMLElement>(
            `[aria-colindex="${String(target.column)}"]`,
          ) ?? rowElement;
        if (cell !== null) {
          cell.focus();
          return;
        }
      }
    })();
  }, [window]);

  const onKeyDownCapture = (event: KeyboardEvent<HTMLDivElement>) => {
    const element = container.current;
    if (element === null || window === null) {
      return;
    }
    const cell = focusedCell(element);
    if (cell === null) {
      return;
    }
    lastColumn.current = cell.column;
    const primary = platform === "mac" ? event.metaKey : event.ctrlKey;
    const lastRow = window.rows.length - 1;
    const last = lastWindowStart(dataset.rows);
    let move: { start: number; target: FocusTarget } | null = null;
    if (primary && event.key === "Home" && window.start !== 0) {
      move = { start: 0, target: { row: 0, column: 1 } };
    } else if (primary && event.key === "End" && window.start !== last) {
      move = {
        start: last,
        target: { row: "last", column: window.columns.length },
      };
    } else if (
      !primary &&
      (event.key === "ArrowDown" || event.key === "PageDown") &&
      cell.row === lastRow &&
      window.start < last
    ) {
      move = {
        start: window.start + WINDOW_ROWS,
        target: { row: 0, column: cell.column },
      };
    } else if (
      !primary &&
      (event.key === "ArrowUp" || event.key === "PageUp") &&
      cell.row === 0 &&
      window.start > 0
    ) {
      move = {
        start: window.start - WINDOW_ROWS,
        target: { row: "last", column: cell.column },
      };
    }
    if (move !== null) {
      event.preventDefault();
      event.stopPropagation();
      moveTo(move.start, move.target);
    }
  };

  // The cursor's row on this dataset, and where it sits in the loaded window.
  const cursorRow =
    cursor?.dataset === dataset.id && cursor.row !== null ? cursor.row : null;
  const selectedPosition =
    cursorRow === null || window === null
      ? -1
      : window.datasetRows.indexOf(cursorRow);
  const selectedKey =
    window === null || selectedPosition < 0
      ? null
      : window.start + selectedPosition;

  // A cursor moved from elsewhere brings its window in, unsorted, while the focus is not in
  // the table: swapping the window under a focused cell would drop the focus.
  useEffect(() => {
    const element = container.current;
    if (
      cursorRow === null ||
      cursor?.origin === "table" ||
      window === null ||
      sort !== null ||
      selectedPosition >= 0 ||
      focusInside ||
      element?.contains(document.activeElement) === true
    ) {
      return;
    }
    void load(windowStart(cursorRow), null).catch(() => undefined);
  }, [
    cursorRow,
    cursor?.origin,
    window,
    sort,
    selectedPosition,
    focusInside,
    load,
  ]);

  // A cursor moved elsewhere scrolls its row into view, while the focus is outside the
  // table; scrolling is not a focus move.
  useEffect(() => {
    const element = container.current;
    if (
      element === null ||
      window === null ||
      selectedPosition < 0 ||
      cursor?.origin === "table" ||
      focusInside
    ) {
      return;
    }
    scrollToRow(element, window.rows.length, selectedPosition);
  }, [window, selectedPosition, cursor?.seq, cursor?.origin, focusInside]);

  // Sends the focused row as the cursor (origin `table`) whenever the focus lands on
  // another row: the focus moves, the cursor follows (keyboard §5.6).
  useEffect(() => {
    const element = container.current;
    if (element === null || window === null) {
      return;
    }
    const onFocusIn = () => {
      const cell = focusedCell(element);
      const row = cell === null ? undefined : window.datasetRows[cell.row];
      if (row === undefined || row === sentRow.current) {
        return;
      }
      sentRow.current = row;
      void client
        .call({
          command: "cursor_set",
          args: { dataset: dataset.id, row, origin: "table" },
        })
        .catch(() => undefined);
    };
    const onFocusOut = (event: FocusEvent) => {
      if (
        !(event.relatedTarget instanceof Node) ||
        !element.contains(event.relatedTarget)
      ) {
        setFocusInside(false);
      }
    };
    const onFocusEnter = () => {
      setFocusInside(true);
      onFocusIn();
    };
    element.addEventListener("focusin", onFocusEnter);
    element.addEventListener("focusout", onFocusOut);
    return () => {
      element.removeEventListener("focusin", onFocusEnter);
      element.removeEventListener("focusout", onFocusOut);
    };
  }, [client, dataset.id, window]);

  useCommand("table.goToRow", () => {
    setGoToOpen(true);
  });

  const onSortChange = (next: DataSort) => {
    setSort(next);
    const header = window?.columns.find((c) => c.id === next.column)?.name;
    void load(0, next).then(
      () => {
        if (header !== undefined) {
          announcer.announce({
            text: t.format(
              next.direction === "ascending"
                ? "ui-table-sorted-ascending"
                : "ui-table-sorted-descending",
              { field: header },
            ),
            assertive: false,
          });
        }
      },
      () => undefined,
    );
  };

  if (dataset.rows === 0) {
    return <p className={styles.status}>{t.format("ui-table-no-rows")}</p>;
  }
  return (
    <div className={styles.table} data-scope="table">
      <p className={styles.status}>
        {window === null
          ? t.format("ui-table-loading")
          : rangeText(t, window.start, window.rows.length, dataset.rows)}
      </p>
      {window !== null && (
        <DataTable
          label={t.format("ui-table-label", { name: dataset.name })}
          columns={window.columns}
          rows={window.rows}
          sort={sort}
          onSortChange={onSortChange}
          onKeyDownCapture={onKeyDownCapture}
          containerRef={container}
          selectedKey={selectedKey}
        />
      )}
      <GoToRow
        isOpen={goToOpen}
        onOpenChange={setGoToOpen}
        total={dataset.rows}
        onGo={(row) => {
          const target = row - 1;
          moveTo(windowStart(target), {
            row: target - windowStart(target),
            column: lastColumn.current,
          });
        }}
      />
    </div>
  );
}

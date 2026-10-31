// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { KeyboardEventHandler, Ref } from "react";
import type { Key, SortDescriptor } from "react-aria-components";
import {
  Cell,
  Column,
  Row,
  Table,
  TableBody,
  TableHeader,
  TableLayout,
  Virtualizer,
} from "react-aria-components";

import { cx } from "./cx.ts";
import styles from "./table.module.css";

/** One column of a {@link DataTable}. */
export interface DataColumn {
  readonly id: string;
  /** The header text: a field name with its unit. */
  readonly name: string;
  /** Whether this column names the row (`rowheader`), such as x or time. One per table. */
  readonly isRowHeader?: boolean;
  /** Right-aligned with tabular figures (DL-P03, DL-P04). */
  readonly numeric?: boolean;
  /** Whether Enter (or a click) on the header sorts by this column. */
  readonly allowsSorting?: boolean;
}

/** The table's order: a column and a direction, as React Aria's `SortDescriptor`. */
export interface DataSort {
  readonly column: string;
  readonly direction: "ascending" | "descending";
}

/** One row: an ID and the text of each column, already formatted with units. */
export interface DataRow {
  readonly id: Key;
  readonly cells: Readonly<Record<string, string>>;
}

/** Props for {@link DataTable}. */
export interface DataTableProps {
  /** The table's accessible name. */
  readonly label: string;
  readonly columns: readonly DataColumn[];
  readonly rows: readonly DataRow[];
  /** The current order, shown with `aria-sort` on its header. */
  readonly sort?: DataSort | null;
  /** Called when a sortable header is pressed; the owner fetches rows in the new order. */
  readonly onSortChange?: (sort: DataSort) => void;
  /** Sees keys before the grid does: paging past the loaded rows (B04-T05). */
  readonly onKeyDownCapture?: KeyboardEventHandler<HTMLDivElement>;
  /** The element around the grid. */
  readonly containerRef?: Ref<HTMLDivElement>;
  /**
   * The row to show as selected (`aria-selected`): the shared cursor (B04-T06). The owner
   * decides it; pressing a row does not change it here.
   */
  readonly selectedKey?: Key | null;
}

/** Rows and the header are the compact row height (design-language §7.2), in pixels. */
const ROW_PX = 24;

/**
 * A data table (APG Grid) for any number of rows: only the visible rows are in the DOM
 * (React Aria `Virtualizer`), while `aria-rowcount` gives the full count. Column headers are
 * `columnheader`s and the row-header column's cells are `rowheader`s, so a screen reader
 * names each value by its row and column (A11Y-03). Arrows move between cells; PageUp and
 * PageDown move a page.
 */
export function DataTable({
  label,
  columns,
  rows,
  sort,
  onSortChange,
  onKeyDownCapture,
  containerRef,
  selectedKey,
}: DataTableProps) {
  const descriptor: SortDescriptor | undefined =
    sort === undefined || sort === null
      ? undefined
      : { column: sort.column, direction: sort.direction };
  return (
    <div
      ref={containerRef}
      className={cx(styles.scroller)}
      onKeyDownCapture={onKeyDownCapture}
    >
      <Virtualizer
        layout={TableLayout}
        layoutOptions={{ rowHeight: ROW_PX, headingHeight: ROW_PX }}
      >
        <Table
          aria-label={label}
          className={cx(styles.table)}
          {...(descriptor === undefined ? {} : { sortDescriptor: descriptor })}
          {...(selectedKey === undefined
            ? {}
            : {
                selectionMode: "single" as const,
                selectedKeys:
                  selectedKey === null
                    ? new Set<Key>()
                    : new Set([selectedKey]),
              })}
          {...(onSortChange === undefined
            ? {}
            : {
                onSortChange: (next: SortDescriptor) => {
                  onSortChange({
                    column: String(next.column),
                    direction: next.direction,
                  });
                },
              })}
        >
          <TableHeader columns={columns} className={cx(styles.header)}>
            {(column) => (
              <Column
                id={column.id}
                isRowHeader={column.isRowHeader ?? false}
                allowsSorting={column.allowsSorting ?? false}
                className={cx(styles.column, column.numeric && styles.numeric)}
              >
                {({ sortDirection }) => (
                  <>
                    {column.name}
                    {sortDirection !== undefined && (
                      // The direction is in aria-sort too; this arrow is for sighted users.
                      <span className={cx(styles.sort)} aria-hidden="true">
                        {sortDirection === "ascending" ? "↑" : "↓"}
                      </span>
                    )}
                  </>
                )}
              </Column>
            )}
          </TableHeader>
          <TableBody items={rows}>
            {(row) => (
              <Row id={row.id} columns={columns} className={cx(styles.row)}>
                {(column) => (
                  <Cell
                    className={cx(
                      styles.cell,
                      column.numeric && styles.numeric,
                    )}
                  >
                    {row.cells[column.id] ?? ""}
                  </Cell>
                )}
              </Row>
            )}
          </TableBody>
        </Table>
      </Virtualizer>
    </div>
  );
}

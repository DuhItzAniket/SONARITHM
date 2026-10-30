// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Key } from "react-aria-components";
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
export function DataTable({ label, columns, rows }: DataTableProps) {
  return (
    <div className={cx(styles.scroller)}>
      <Virtualizer
        layout={TableLayout}
        layoutOptions={{ rowHeight: ROW_PX, headingHeight: ROW_PX }}
      >
        <Table aria-label={label} className={cx(styles.table)}>
          <TableHeader columns={columns} className={cx(styles.header)}>
            {(column) => (
              <Column
                id={column.id}
                isRowHeader={column.isRowHeader ?? false}
                className={cx(styles.column, column.numeric && styles.numeric)}
              >
                {column.name}
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

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Key } from "react-aria-components";
import { GridList, GridListItem } from "react-aria-components";

import type { Severity } from "../icons/Severity.tsx";
import { SeverityIcon } from "../icons/Severity.tsx";
import { cx } from "./cx.ts";
import styles from "./diagnostics.module.css";

/** One row: every part already formatted by the caller (coding-standards §12.7). */
export interface DiagnosticRow {
  readonly id: string;
  readonly severity: Severity;
  /** The severity in words, such as "Error". */
  readonly severityLabel: string;
  /** The stable code, such as `SNR-SPEC-070`. */
  readonly code: string;
  /** The cause and the fix. */
  readonly message: string;
  /** Where, in words, such as "Line 4, column 12". */
  readonly location: string;
}

/** Props of {@link DiagnosticList}. */
export interface DiagnosticListProps {
  /** The list's accessible name, such as "Diagnostics". */
  readonly label: string;
  readonly rows: readonly DiagnosticRow[];
  /** Shown when there are no rows: one line, with the command that resolves it. */
  readonly empty: string;
  /** Enter (or a click) on a row: go to its source (keyboard §5.10). */
  readonly onActivate?: (id: string) => void;
}

/**
 * The diagnostics of the Status region (B05-T03; design-language §11, keyboard §5.10): a
 * React Aria grid list, so arrows, Home and End move through the rows and Enter goes to the
 * source. Each row is named by its severity in words, its code, its message and its place,
 * in that order, and shows a shape for the severity beside the word, never colour alone
 * (DL-X08).
 */
export function DiagnosticList({
  label,
  rows,
  empty,
  onActivate,
}: DiagnosticListProps) {
  return (
    <GridList
      aria-label={label}
      className={cx(styles.list)}
      items={rows}
      renderEmptyState={() => <p className={cx(styles.empty)}>{empty}</p>}
      {...(onActivate === undefined
        ? {}
        : {
            onAction: (key: Key) => {
              onActivate(String(key));
            },
          })}
    >
      {(row) => {
        const text = `${row.severityLabel}. ${row.code}: ${row.message} ${row.location}.`;
        return (
          <GridListItem id={row.id} textValue={text} className={cx(styles.row)}>
            <span className={cx(styles.severity)} data-severity={row.severity}>
              <SeverityIcon severity={row.severity} />
              {row.severityLabel}
            </span>
            <span className={cx(styles.code)}>{row.code}</span>
            <span className={cx(styles.message)}>{row.message}</span>
            <span className={cx(styles.location)}>{row.location}</span>
          </GridListItem>
        );
      }}
    </GridList>
  );
}

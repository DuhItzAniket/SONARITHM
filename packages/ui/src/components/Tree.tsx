// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { ReactNode } from "react";
import type {
  TreeItemProps as AriaTreeItemProps,
  TreeProps as AriaTreeProps,
} from "react-aria-components";
import {
  Button as AriaButton,
  Tree as AriaTree,
  TreeItem as AriaTreeItem,
  TreeItemContent,
} from "react-aria-components";

import { Chevron } from "../icons/Chevron.tsx";
import { cx } from "./cx.ts";
import styles from "./tree.module.css";

/** Props for {@link Tree}. */
export interface TreeProps<T extends object> extends Omit<
  AriaTreeProps<T>,
  "className" | "style" | "aria-label"
> {
  /** The tree's accessible name, such as "Project". */
  readonly label: string;
}

/**
 * The project tree. React Aria exposes it as an APG tree grid (`treegrid`, one cell per
 * row); it has no `tree` role option, and AGENTS.md rule 6 keeps us on its pattern (#383).
 * Arrows move, ArrowRight and ArrowLeft expand and collapse, Home and End jump. Each row's
 * expanded state is on the row (`aria-expanded`).
 */
export function Tree<T extends object>({ label, ...props }: TreeProps<T>) {
  return <AriaTree {...props} aria-label={label} className={cx(styles.tree)} />;
}

/** Props for {@link TreeItem}. */
export interface TreeItemProps extends Omit<
  AriaTreeItemProps,
  "className" | "style" | "children" | "textValue"
> {
  /** The row's text, which is also its accessible name. */
  readonly title: string;
  /** Child rows. */
  readonly children?: ReactNode;
}

/**
 * One row of a {@link Tree}. React Aria's expand button is pointer-only (outside the Tab
 * order; keys expand the row itself), but its label ("Expand" plus the row's name) made
 * every row's cell read its name twice (F-045, #383). The button is hidden from assistive
 * technology, so the cell's name is the title alone.
 */
export function TreeItem({ title, children, ...props }: TreeItemProps) {
  return (
    <AriaTreeItem {...props} textValue={title} className={cx(styles.row)}>
      <TreeItemContent>
        {({ hasChildItems }) => (
          <div className={cx(styles.content)}>
            {hasChildItems ? (
              // React Aria's Button drops `aria-hidden`, so a wrapper hides it; it is
              // outside the Tab order, so nothing focusable by Tab is hidden (axe).
              <span aria-hidden="true" className={cx(styles.chevronSlot)}>
                <AriaButton slot="chevron" className={cx(styles.chevron)}>
                  <Chevron />
                </AriaButton>
              </span>
            ) : (
              <span className={cx(styles.spacer)} />
            )}
            <span className={cx(styles.title)}>{title}</span>
          </div>
        )}
      </TreeItemContent>
      {children}
    </AriaTreeItem>
  );
}

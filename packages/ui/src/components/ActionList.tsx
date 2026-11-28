// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { useCallback } from "react";
import type { Key } from "react-aria-components";
import { GridList, GridListItem } from "react-aria-components";

import { cx } from "./cx.ts";
import list from "./list.module.css";

/** One row of an {@link ActionList}. */
export interface ActionItem {
  readonly id: Key;
  /** The row's text, which is also its accessible name. */
  readonly text: string;
}

/** Props for {@link ActionList}. */
export interface ActionListProps {
  /** The list's accessible name, such as "Critical points". */
  readonly label: string;
  readonly items: readonly ActionItem[];
  /** Called when a row is activated: Enter, or a click. */
  readonly onAction: (id: Key) => void;
  /**
   * The row that stands for where something else is, such as Rust's one cursor; marked
   * `aria-current` and with a visible bar, never by colour alone. The list keeps no
   * selection of its own.
   */
  readonly current?: Key | null;
}

/**
 * A list whose rows act (APG Grid as a list, React Aria's GridList): one Tab stop, the
 * arrows, Home and End move between rows, Enter activates one. There is no selection:
 * a row is only ever "current" because the caller says so, and moving the focus changes
 * nothing.
 */
export function ActionList({
  label,
  items,
  onAction,
  current = null,
}: ActionListProps) {
  return (
    <GridList
      aria-label={label}
      items={items}
      // GridList caches each row by its item; `current` is not part of an item, so the
      // rows must render again when it changes, or the mark stays on the old row.
      dependencies={[current]}
      selectionMode="none"
      onAction={onAction}
      className={cx(list.list)}
    >
      {(item) => <ActionRow item={item} isCurrent={item.id === current} />}
    </GridList>
  );
}

function ActionRow({
  item,
  isCurrent,
}: {
  readonly item: ActionItem;
  readonly isCurrent: boolean;
}) {
  // React Aria passes no `aria-current` to a row (its DOM props filter keeps the global
  // attributes only), so the row gets it here. A callback ref, not an effect: React Aria
  // renders the collection before the rows it attaches, and the ref runs for every row
  // element it attaches, and again whenever `isCurrent` changes.
  const row = useCallback(
    (element: HTMLDivElement | null) => {
      if (isCurrent) {
        element?.setAttribute("aria-current", "true");
      } else {
        element?.removeAttribute("aria-current");
      }
    },
    [isCurrent],
  );
  return (
    <GridListItem
      ref={row}
      id={item.id}
      textValue={item.text}
      data-current={isCurrent ? "" : undefined}
      className={cx(list.item, list.action)}
    >
      <span className={cx(list.name)}>{item.text}</span>
    </GridListItem>
  );
}

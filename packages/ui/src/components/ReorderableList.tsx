// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Key } from "react-aria-components";
import {
  Button as AriaButton,
  GridList,
  GridListItem,
  useDragAndDrop,
} from "react-aria-components";

import styles from "./controls.module.css";
import { cx } from "./cx.ts";
import list from "./list.module.css";

/** One item of a {@link ReorderableList}. */
export interface ReorderableItem {
  readonly id: Key;
  readonly name: string;
}

/** The labels a {@link ReorderableList} needs, formatted by the caller per item. */
export interface ReorderLabels {
  /** The drag handle's name (`ui-reorder`). */
  readonly reorder: (name: string) => string;
  /** `ui-move-up`. */
  readonly moveUp: (name: string) => string;
  /** `ui-move-down`. */
  readonly moveDown: (name: string) => string;
}

/** Props for {@link ReorderableList}. */
export interface ReorderableListProps {
  /** The list's accessible name, such as "Announcement parts". */
  readonly label: string;
  /** The items in their current order. */
  readonly items: readonly ReorderableItem[];
  /** Called with the new order of IDs. */
  readonly onReorder: (order: Key[]) => void;
  readonly labels: ReorderLabels;
}

/** `items` with the item `id` moved by `delta` places, or unchanged at an edge. */
export function moveBy(
  items: readonly ReorderableItem[],
  id: Key,
  delta: number,
): Key[] {
  const order = items.map((item) => item.id);
  const from = order.indexOf(id);
  const to = from + delta;
  if (from < 0 || to < 0 || to >= order.length) {
    return order;
  }
  order.splice(from, 1);
  order.splice(to, 0, id);
  return order;
}

/**
 * An ordered list whose items move by keyboard (A11Y-12): each row has "Move up" and "Move
 * down" buttons, and React Aria's drag handle also supports keyboard drag and drop (Enter
 * picks up, arrows move, Enter drops). The announcement-part order uses it (design-language
 * §11). The first item has no "Move up" and the last no "Move down".
 */
export function ReorderableList({
  label,
  items,
  onReorder,
  labels,
}: ReorderableListProps) {
  const { dragAndDropHooks } = useDragAndDrop({
    getItems: (keys) => [...keys].map((key) => ({ "text/plain": String(key) })),
    onReorder(event) {
      const moving = [...event.keys];
      const order = items
        .map((item) => item.id)
        .filter((id) => !moving.includes(id));
      const at = order.indexOf(event.target.key);
      const index = event.target.dropPosition === "after" ? at + 1 : at;
      order.splice(index, 0, ...moving);
      onReorder(order);
    },
  });
  return (
    <GridList
      aria-label={label}
      items={items}
      dragAndDropHooks={dragAndDropHooks}
      className={cx(list.list)}
    >
      {(item) => {
        const index = items.findIndex((other) => other.id === item.id);
        return (
          <GridListItem
            id={item.id}
            textValue={item.name}
            className={cx(list.item)}
          >
            <AriaButton slot="drag" className={cx(list.handle)}>
              <span className={cx(list.grip)} aria-hidden="true" />
              <span className={cx(list.hidden)}>
                {labels.reorder(item.name)}
              </span>
            </AriaButton>
            <span className={cx(list.name)}>{item.name}</span>
            <span className={cx(list.actions)}>
              {index > 0 && (
                <AriaButton
                  className={cx(styles.control, styles.quiet)}
                  onPress={() => {
                    onReorder(moveBy(items, item.id, -1));
                  }}
                >
                  {labels.moveUp(item.name)}
                </AriaButton>
              )}
              {index < items.length - 1 && (
                <AriaButton
                  className={cx(styles.control, styles.quiet)}
                  onPress={() => {
                    onReorder(moveBy(items, item.id, 1));
                  }}
                >
                  {labels.moveDown(item.name)}
                </AriaButton>
              )}
            </span>
          </GridListItem>
        );
      }}
    </GridList>
  );
}

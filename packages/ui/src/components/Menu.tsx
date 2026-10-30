// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { ReactNode } from "react";
import type {
  MenuItemProps as AriaMenuItemProps,
  MenuProps as AriaMenuProps,
} from "react-aria-components";
import {
  Button as AriaButton,
  Keyboard,
  Menu as AriaMenu,
  MenuItem as AriaMenuItem,
  MenuTrigger,
  Popover,
  Separator,
  SubmenuTrigger,
  Text,
} from "react-aria-components";

import styles from "./controls.module.css";
import overlays from "./overlays.module.css";
import { cx } from "./cx.ts";

/** Props for {@link MenuButton}. */
export interface MenuButtonProps<T extends object> extends Omit<
  AriaMenuProps<T>,
  "className" | "style" | "aria-label"
> {
  /** The trigger's visible label, which also names the menu. */
  readonly label: string;
}

/**
 * A button that opens a menu (APG Menu Button): Enter, Space or ArrowDown opens it with
 * focus on the first item, arrows move, Escape closes it and returns focus to the button.
 */
export function MenuButton<T extends object>({
  label,
  ...menu
}: MenuButtonProps<T>) {
  return (
    <MenuTrigger>
      <AriaButton className={cx(styles.control)}>{label}</AriaButton>
      <Popover className={cx(overlays.popover)} placement="bottom start">
        <AriaMenu {...menu} aria-label={label} className={cx(overlays.menu)} />
      </Popover>
    </MenuTrigger>
  );
}

/** Props for {@link MenuItem}. */
export interface MenuItemProps extends Omit<
  AriaMenuItemProps,
  "className" | "style" | "children"
> {
  /** The command's name, with a verb (design-language §12). */
  readonly children: string;
  /** The current binding, shown as the item's keyboard shortcut. */
  readonly shortcut?: string;
}

/** One command in a menu, with its current binding (keyboard §13 rule 7). */
export function MenuItem({ children, shortcut, ...props }: MenuItemProps) {
  return (
    <AriaMenuItem {...props} textValue={children} className={cx(overlays.item)}>
      {children}
      {shortcut !== undefined && (
        // The shortcut is the item's description (aria-describedby). It is hidden from
        // the name, which comes from the content, so the name is the command alone
        // (WCAG 2.5.3).
        <span aria-hidden="true">
          <Keyboard className={cx(overlays.shortcut)}>{shortcut}</Keyboard>
        </span>
      )}
    </AriaMenuItem>
  );
}

/** Props for {@link Submenu}. */
export interface SubmenuProps {
  /** The item that opens the submenu, which also names it. */
  readonly label: string;
  readonly children: ReactNode;
}

/** A menu item that opens a submenu (ArrowRight opens it, ArrowLeft closes it). */
export function Submenu({ label, children }: SubmenuProps) {
  return (
    <SubmenuTrigger>
      <AriaMenuItem textValue={label} className={cx(overlays.item)}>
        <Text slot="label">{label}</Text>
      </AriaMenuItem>
      <Popover className={cx(overlays.popover)}>
        <AriaMenu aria-label={label} className={cx(overlays.menu)}>
          {children}
        </AriaMenu>
      </Popover>
    </SubmenuTrigger>
  );
}

/** A divider between groups of menu items. */
export function MenuSeparator() {
  return <Separator className={cx(overlays.separator)} />;
}

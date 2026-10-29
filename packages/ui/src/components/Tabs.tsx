// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  TabListProps as AriaTabListProps,
  TabPanelProps as AriaTabPanelProps,
  TabProps as AriaTabProps,
  TabsProps as AriaTabsProps,
} from "react-aria-components";
import {
  Tab as AriaTab,
  TabList as AriaTabList,
  TabPanel as AriaTabPanel,
  Tabs as AriaTabs,
} from "react-aria-components";

import styles from "./tabs.module.css";
import { cx } from "./cx.ts";

/** Props for {@link Tabs}. */
export type TabsProps = Omit<AriaTabsProps, "className" | "style">;

/** Tabs (APG Tabs): Plot · Table · Spec; Announcements · Diagnostics (design-language §11). */
export function Tabs(props: TabsProps) {
  return <AriaTabs {...props} className={cx(styles.tabs)} />;
}

/** Props for {@link TabList}. */
export interface TabListProps<T extends object> extends Omit<
  AriaTabListProps<T>,
  "className" | "style" | "aria-label"
> {
  /** The tab list's accessible name. */
  readonly label: string;
}

/** The row of tabs: one Tab stop; arrow keys move and select. */
export function TabList<T extends object>({
  label,
  ...props
}: TabListProps<T>) {
  return (
    <AriaTabList {...props} aria-label={label} className={cx(styles.list)} />
  );
}

/** Props for {@link Tab}. */
export type TabProps = Omit<AriaTabProps, "className" | "style">;

/** One tab; the selected tab has an accent underline and a heavier weight (DL-X08). */
export function Tab(props: TabProps) {
  return <AriaTab {...props} className={cx(styles.tab)} />;
}

/** Props for {@link TabPanel}. */
export type TabPanelProps = Omit<AriaTabPanelProps, "className" | "style">;

/** The panel of one tab; focusable when it holds no focusable content. */
export function TabPanel(props: TabPanelProps) {
  return <AriaTabPanel {...props} className={cx(styles.panel)} />;
}

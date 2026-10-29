// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { ToolbarProps as AriaToolbarProps } from "react-aria-components";
import { Toolbar as AriaToolbar } from "react-aria-components";

import layout from "./layout.module.css";
import { cx } from "./cx.ts";

/** Props for {@link Toolbar}. */
export interface ToolbarProps extends Omit<
  AriaToolbarProps,
  "className" | "style" | "aria-label"
> {
  /** The toolbar's accessible name, such as "Transport". */
  readonly label: string;
}

/**
 * A toolbar (APG Toolbar): arrow keys move between its controls and Tab leaves it
 * (keyboard-and-navigation §5.3).
 */
export function Toolbar({ label, ...props }: ToolbarProps) {
  return (
    <AriaToolbar {...props} aria-label={label} className={cx(layout.toolbar)} />
  );
}

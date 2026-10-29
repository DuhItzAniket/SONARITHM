// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { TooltipProps as AriaTooltipProps } from "react-aria-components";
import { Tooltip as AriaTooltip, TooltipTrigger } from "react-aria-components";

import overlays from "./overlays.module.css";
import { cx } from "./cx.ts";

export { TooltipTrigger };

/** Props for {@link Tooltip}. */
export interface TooltipProps extends Omit<
  AriaTooltipProps,
  "className" | "style" | "children"
> {
  /** Plain text: a tooltip never holds controls or information found nowhere else. */
  readonly children: string;
}

/**
 * A tooltip (APG Tooltip) for a focusable trigger: it opens on keyboard focus as well as on
 * hover and closes with Escape (DL-X11). Its text also reaches assistive technology through
 * the trigger's name or description (design-language §11).
 */
export function Tooltip(props: TooltipProps) {
  return <AriaTooltip offset={6} {...props} className={cx(overlays.tooltip)} />;
}

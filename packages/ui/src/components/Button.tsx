// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { ReactNode } from "react";
import type { ButtonProps as AriaButtonProps } from "react-aria-components";
import {
  Button as AriaButton,
  Tooltip as AriaTooltip,
  TooltipTrigger,
} from "react-aria-components";

import styles from "./controls.module.css";
import { cx } from "./cx.ts";
import overlays from "./overlays.module.css";

/** Props for {@link Button}. */
export interface ButtonProps extends Omit<
  AriaButtonProps,
  "className" | "style"
> {
  /** `quiet` drops the fill and border, for buttons inside toolbars and rows. */
  readonly variant?: "default" | "quiet";
}

/**
 * A button with a visible text label (React Aria `Button`, APG Button). Labels name the
 * action with a verb (design-language §12).
 */
export function Button({ variant = "default", ...props }: ButtonProps) {
  return (
    <AriaButton
      {...props}
      className={cx(styles.control, variant === "quiet" && styles.quiet)}
    />
  );
}

/** Props for {@link IconButton}. */
export interface IconButtonProps extends Omit<
  AriaButtonProps,
  "className" | "style" | "children" | "aria-label" | "aria-labelledby"
> {
  /**
   * The accessible name, required (DL-X05, A11Y-18). It is also the tooltip unless
   * `tooltip` is given.
   */
  readonly label: string;
  /** The tooltip: the name and the current binding (`ui-name-with-shortcut`). */
  readonly tooltip?: string;
  /** A decorative 16 px icon; hidden from assistive technology. */
  readonly icon: ReactNode;
  readonly variant?: "default" | "quiet";
}

/**
 * A button that shows only an icon (design-language §10). It always has an accessible name
 * and a tooltip that keyboard focus opens too (DL-X11); the tooltip repeats the name and adds
 * the binding, so it holds nothing a screen reader misses.
 */
export function IconButton({
  label,
  tooltip,
  icon,
  variant = "quiet",
  ...props
}: IconButtonProps) {
  return (
    <TooltipTrigger delay={600}>
      <AriaButton
        {...props}
        aria-label={label}
        className={cx(styles.control, variant === "quiet" && styles.quiet)}
      >
        <span className={cx(styles.icon)} aria-hidden="true">
          {icon}
        </span>
      </AriaButton>
      <AriaTooltip className={cx(overlays.tooltip)} offset={6}>
        {tooltip ?? label}
      </AriaTooltip>
    </TooltipTrigger>
  );
}

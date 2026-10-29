// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  ToggleButtonGroupProps as AriaToggleButtonGroupProps,
  ToggleButtonProps as AriaToggleButtonProps,
} from "react-aria-components";
import {
  ToggleButton as AriaToggleButton,
  ToggleButtonGroup as AriaToggleButtonGroup,
} from "react-aria-components";

import styles from "./controls.module.css";
import layout from "./layout.module.css";
import { cx } from "./cx.ts";

/** Props for {@link ToggleButton}. */
export type ToggleButtonProps = Omit<
  AriaToggleButtonProps,
  "className" | "style"
>;

/**
 * A button with a pressed state (APG Button, `aria-pressed`): loop, modality toggles,
 * polarity flip. Selected shows a fill, an accent border and a thicker edge (DL-X08).
 */
export function ToggleButton(props: ToggleButtonProps) {
  return <AriaToggleButton {...props} className={cx(styles.control)} />;
}

/** Props for {@link ToggleButtonGroup}. */
export interface ToggleButtonGroupProps extends Omit<
  AriaToggleButtonGroupProps,
  "className" | "style" | "aria-label"
> {
  /** The group's accessible name. */
  readonly label: string;
}

/** Related toggle buttons; arrow keys move between them (React Aria). */
export function ToggleButtonGroup({ label, ...props }: ToggleButtonGroupProps) {
  return (
    <AriaToggleButtonGroup
      {...props}
      aria-label={label}
      className={cx(layout.row)}
    />
  );
}

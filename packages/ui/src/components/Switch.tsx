// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { SwitchFieldProps } from "react-aria-components";
import { SwitchButton, SwitchField, Text } from "react-aria-components";

import { cx } from "./cx.ts";
import styles from "./switch.module.css";

/** Props for {@link Switch}. */
export interface SwitchProps extends Omit<
  SwitchFieldProps,
  "className" | "style" | "children"
> {
  /** The visible label, which names the switch. */
  readonly children: string;
  /**
   * Visible text the switch is described by: a hint such as "Takes effect after a
   * restart", or why it is disabled. Never only a tooltip (DL-X11).
   */
  readonly description?: string;
}

/**
 * An on/off setting (React Aria `SwitchField`, APG Switch): Space toggles it, and its state
 * is the thumb's position as well as its fill, so it is never shown by colour alone
 * (DL-X08). The description is linked by React Aria (`aria-describedby`).
 */
export function Switch({ children, description, ...props }: SwitchProps) {
  return (
    <SwitchField {...props} className={cx(styles.field)}>
      <SwitchButton className={cx(styles.switch)}>
        <span className={cx(styles.track)} aria-hidden="true">
          <span className={cx(styles.thumb)} />
        </span>
        {children}
      </SwitchButton>
      {description !== undefined && (
        <Text slot="description" className={cx(styles.description)}>
          {description}
        </Text>
      )}
    </SwitchField>
  );
}

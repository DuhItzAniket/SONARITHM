// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { RadioGroupProps as RACRadioGroupProps } from "react-aria-components";
import {
  Label,
  RadioButton,
  RadioField,
  RadioGroup as RACRadioGroup,
  Text,
} from "react-aria-components";

import { cx } from "./cx.ts";
import styles from "./radio.module.css";

/** Props for {@link RadioGroup}. */
export interface RadioGroupProps extends Omit<
  RACRadioGroupProps,
  "className" | "style" | "children"
> {
  /** The visible label, which names the group. */
  readonly label: string;
  /** Visible text the group is described by. Never only a tooltip (DL-X11). */
  readonly description?: string;
  /** The choices, in order. */
  readonly children: React.ReactNode;
}

/**
 * One choice of a few, all in view (React Aria `RadioGroup`, APG Radio Group): Tab reaches
 * the selected radio, the arrows move the choice, and the chosen one is marked by a filled
 * dot as well as its colour (DL-X08).
 */
export function RadioGroup({
  label,
  description,
  children,
  ...props
}: RadioGroupProps) {
  return (
    <RACRadioGroup {...props} className={cx(styles.group)}>
      <Label className={cx(styles.label)}>{label}</Label>
      <div className={cx(styles.choices)}>{children}</div>
      {description !== undefined && (
        <Text slot="description" className={cx(styles.description)}>
          {description}
        </Text>
      )}
    </RACRadioGroup>
  );
}

/** Props for {@link Radio}. */
export interface RadioProps {
  /** The choice's value. */
  readonly value: string;
  /** The visible text, which names the radio. */
  readonly children: string;
  /** Whether the choice cannot be made now; say why in the group's description. */
  readonly isDisabled?: boolean;
}

/** One choice of a {@link RadioGroup} (React Aria `RadioField` and `RadioButton`). */
export function Radio({ value, children, isDisabled = false }: RadioProps) {
  return (
    <RadioField value={value} isDisabled={isDisabled}>
      <RadioButton className={cx(styles.radio)}>
        <span className={cx(styles.dot)} aria-hidden="true" />
        {children}
      </RadioButton>
    </RadioField>
  );
}

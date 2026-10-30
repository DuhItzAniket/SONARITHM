// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { ReactNode } from "react";
import type {
  ComboBoxProps as AriaComboBoxProps,
  ListBoxItemProps as AriaListBoxItemProps,
  ListBoxProps as AriaListBoxProps,
  NumberFieldProps as AriaNumberFieldProps,
  SelectProps as AriaSelectProps,
} from "react-aria-components";
import {
  Button as AriaButton,
  ComboBox as AriaComboBox,
  FieldError,
  Group,
  Input,
  Label,
  ListBox as AriaListBox,
  ListBoxItem as AriaListBoxItem,
  NumberField as AriaNumberField,
  Popover,
  Select as AriaSelect,
  SelectValue,
  Text,
} from "react-aria-components";

import { Chevron } from "../icons/Chevron.tsx";
import { cx } from "./cx.ts";
import styles from "./fields.module.css";
import overlays from "./overlays.module.css";

/** What every labelled field takes besides its own props. */
interface FieldText {
  /** The visible label, which names the field. */
  readonly label: string;
  /** Help text, linked with `aria-describedby`. */
  readonly description?: string;
  /** The error shown and announced when the field is invalid (`isInvalid`). */
  readonly errorMessage?: string;
}

function Messages({
  description,
  errorMessage,
}: {
  readonly description: string | undefined;
  readonly errorMessage: string | undefined;
}) {
  return (
    <>
      {description !== undefined && (
        <Text slot="description" className={cx(styles.description)}>
          {description}
        </Text>
      )}
      <FieldError className={cx(styles.error)}>{errorMessage}</FieldError>
    </>
  );
}

/** Props for {@link NumberField}. */
export interface NumberFieldProps
  extends FieldText, Omit<AriaNumberFieldProps, "className" | "style"> {}

/**
 * A number input (APG Spinbutton) that formats and parses its unit through `formatOptions`
 * (`{ style: "unit", unit: "second" }`): the value reads "2.5 s" and typing "2.5" or "2.5 s"
 * both work. Arrow keys step; PageUp and PageDown take larger steps.
 */
export function NumberField({
  label,
  description,
  errorMessage,
  ...props
}: NumberFieldProps) {
  return (
    <AriaNumberField {...props} className={cx(styles.field)}>
      <Label className={cx(styles.label)}>{label}</Label>
      <Input className={cx(styles.input)} />
      <Messages description={description} errorMessage={errorMessage} />
    </AriaNumberField>
  );
}

/** Props for {@link Select}. */
export interface SelectProps<T extends object>
  extends
    FieldText,
    Omit<AriaSelectProps<T>, "className" | "style" | "children"> {
  /** The options: {@link Option} elements. */
  readonly children: ReactNode;
}

/** A single choice from a short list (APG Listbox in a popover): the scale selector. */
export function Select<T extends object>({
  label,
  description,
  errorMessage,
  children,
  ...props
}: SelectProps<T>) {
  return (
    <AriaSelect {...props} className={cx(styles.field)}>
      <Label className={cx(styles.label)}>{label}</Label>
      <AriaButton className={cx(styles.trigger)}>
        <SelectValue />
        <Chevron className={cx(styles.chevron)} />
      </AriaButton>
      <Messages description={description} errorMessage={errorMessage} />
      <Popover className={cx(overlays.popover)}>
        <AriaListBox className={cx(styles.list)}>{children}</AriaListBox>
      </Popover>
    </AriaSelect>
  );
}

/** Props for {@link ComboBox}. */
export interface ComboBoxProps<T extends object>
  extends
    FieldText,
    Omit<AriaComboBoxProps<T>, "className" | "style" | "children"> {
  /** The options: {@link Option} elements. */
  readonly children: ReactNode;
  /** The name of the button that opens the list. */
  readonly openLabel: string;
}

/** Type to filter a long list (APG Combobox): the concept selector. */
export function ComboBox<T extends object>({
  label,
  description,
  errorMessage,
  openLabel,
  children,
  ...props
}: ComboBoxProps<T>) {
  return (
    <AriaComboBox {...props} className={cx(styles.field)}>
      <Label className={cx(styles.label)}>{label}</Label>
      <Group className={cx(styles.group)}>
        <Input className={cx(styles.input)} />
        <AriaButton aria-label={openLabel} className={cx(styles.trigger)}>
          <Chevron className={cx(styles.chevron)} />
        </AriaButton>
      </Group>
      <Messages description={description} errorMessage={errorMessage} />
      <Popover className={cx(overlays.popover)}>
        <AriaListBox className={cx(styles.list)}>{children}</AriaListBox>
      </Popover>
    </AriaComboBox>
  );
}

/** Props for {@link ListBox}. */
export interface ListBoxProps<T extends object> extends Omit<
  AriaListBoxProps<T>,
  "className" | "style" | "aria-label"
> {
  /** The list's accessible name. */
  readonly label: string;
}

/** A list of choices shown in place (APG Listbox). */
export function ListBox<T extends object>({
  label,
  ...props
}: ListBoxProps<T>) {
  return (
    <AriaListBox {...props} aria-label={label} className={cx(styles.list)} />
  );
}

/** Props for {@link Option}. */
export interface OptionProps extends Omit<
  AriaListBoxItemProps,
  "className" | "style" | "children"
> {
  readonly children: string;
}

/** One option of a {@link Select}, {@link ComboBox} or {@link ListBox}. */
export function Option({ children, ...props }: OptionProps) {
  return (
    <AriaListBoxItem
      {...props}
      textValue={children}
      className={cx(styles.option)}
    >
      {children}
    </AriaListBoxItem>
  );
}

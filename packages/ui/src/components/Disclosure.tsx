// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { ReactNode } from "react";
import type {
  DisclosureGroupProps as AriaDisclosureGroupProps,
  DisclosureProps as AriaDisclosureProps,
} from "react-aria-components";
import {
  Button as AriaButton,
  Disclosure as AriaDisclosure,
  DisclosureGroup as AriaDisclosureGroup,
  DisclosurePanel,
  Heading,
} from "react-aria-components";

import { Chevron } from "../icons/Chevron.tsx";
import styles from "./disclosure.module.css";
import { cx } from "./cx.ts";

/** Props for {@link Disclosure}. */
export interface DisclosureProps extends Omit<
  AriaDisclosureProps,
  "className" | "style" | "children"
> {
  /** The section's title: the trigger's name and a heading. */
  readonly title: string;
  /** The heading level of the title; 3 by default (inspector sections). */
  readonly level?: 2 | 3 | 4;
  readonly children: ReactNode;
}

/**
 * A section that expands and collapses (APG Disclosure): a heading that holds a button with
 * `aria-expanded`; Enter or Space toggles it. The inspector's engine sections use it.
 */
export function Disclosure({
  title,
  level = 3,
  children,
  ...props
}: DisclosureProps) {
  return (
    <AriaDisclosure {...props} className={cx(styles.disclosure)}>
      <Heading level={level} className={cx(styles.heading)}>
        <AriaButton slot="trigger" className={cx(styles.trigger)}>
          <Chevron className={cx(styles.chevron)} />
          {title}
        </AriaButton>
      </Heading>
      <DisclosurePanel className={cx(styles.panel)}>{children}</DisclosurePanel>
    </AriaDisclosure>
  );
}

/** Props for {@link DisclosureGroup}. */
export type DisclosureGroupProps = Omit<
  AriaDisclosureGroupProps,
  "className" | "style"
>;

/** Related sections; unless `allowsMultipleExpanded`, opening one closes the others. */
export function DisclosureGroup(props: DisclosureGroupProps) {
  return <AriaDisclosureGroup {...props} className={cx(styles.group)} />;
}

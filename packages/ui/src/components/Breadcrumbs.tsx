// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import {
  Breadcrumb,
  Breadcrumbs as AriaBreadcrumbs,
  Link,
} from "react-aria-components";

import { cx } from "./cx.ts";
import styles from "./breadcrumbs.module.css";

/** One level of a {@link Breadcrumbs} trail. */
export interface BreadcrumbItem {
  readonly id: string;
  /** The text shown and read: a level word with a data name ("Series temperature"). */
  readonly text: string;
}

/** Props for {@link Breadcrumbs}. */
export interface BreadcrumbsProps {
  /** The trail's accessible name, on its `nav` landmark. */
  readonly label: string;
  readonly items: readonly BreadcrumbItem[];
}

/**
 * A breadcrumb trail (APG Breadcrumb, React Aria `Breadcrumbs`): a `nav` landmark with an
 * ordered list. The last item is the current one, a link with `aria-current="page"` that
 * React Aria disables; the levels before it are text, since nothing moves to them yet.
 * Separators are drawn in CSS, so a screen reader does not read them (B04-T06).
 */
export function Breadcrumbs({ label, items }: BreadcrumbsProps) {
  return (
    <nav aria-label={label} className={cx(styles.trail)}>
      <AriaBreadcrumbs className={cx(styles.list)} items={items}>
        {(item) => (
          <Breadcrumb id={item.id} className={cx(styles.item)}>
            {({ isCurrent }) =>
              isCurrent ? (
                <Link className={cx(styles.current)}>{item.text}</Link>
              ) : (
                <span>{item.text}</span>
              )
            }
          </Breadcrumb>
        )}
      </AriaBreadcrumbs>
    </nav>
  );
}

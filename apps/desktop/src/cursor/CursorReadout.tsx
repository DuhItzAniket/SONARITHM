// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { CursorState } from "@sonarithm/ipc";
import { Breadcrumbs } from "@sonarithm/ui";

import { useLocalizer } from "../i18n.ts";
import styles from "./cursor.module.css";

/**
 * The persistent cursor readout (B04-T06; 08 §3): the cursor's data row and its path from
 * the dataset to the point, as a breadcrumb. It is not a live region: what a move says is
 * the announcement's job, so the readout never speaks by itself (A11Y-17). Level words come
 * from the catalogue; dataset, series and x names are shown as they are.
 */
export function CursorReadout({
  cursor,
}: {
  readonly cursor: CursorState | null;
}) {
  const t = useLocalizer();
  if (cursor?.row === undefined || cursor.row === null) {
    return <p className={styles.readout}>{t.format("ui-cursor-none")}</p>;
  }
  return (
    <div className={styles.readout} data-cursor-readout="">
      <p className={styles.row}>
        {t.format("ui-cursor-row", { index: { number: cursor.row + 1 } })}
      </p>
      <Breadcrumbs
        label={t.format("ui-cursor-trail")}
        items={cursor.path.map((item, index) => ({
          id: `${item.level}-${String(index)}`,
          text: t.format("ui-cursor-step", {
            kind: item.level,
            name: item.label,
          }),
        }))}
      />
    </div>
  );
}

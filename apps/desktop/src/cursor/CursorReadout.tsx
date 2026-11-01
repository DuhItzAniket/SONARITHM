// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { CursorState } from "@sonarithm/ipc";
import { Breadcrumbs } from "@sonarithm/ui";

import { useLocalizer } from "../i18n.ts";
import styles from "./cursor.module.css";

/**
 * The persistent cursor readout (B04-T06; 08 §3): the cursor's data row and its path from
 * the dataset to the point, as a breadcrumb, and the series the explorer's filters took out
 * (B03-T09). It is not a live region: what a move says is the announcement's job, so the
 * readout never speaks by itself (A11Y-17). Level words come from the catalogue; dataset,
 * series and x names are shown as they are.
 */
export function CursorReadout({
  cursor,
}: {
  readonly cursor: CursorState | null;
}) {
  const t = useLocalizer();
  const excluded =
    cursor === null || cursor.excluded.length === 0 ? null : (
      <ul className={styles.excluded}>
        {cursor.excluded.map((series) => (
          <li key={series.node}>
            {t.format("ui-cursor-excluded", { name: series.label })}
          </li>
        ))}
      </ul>
    );
  if (cursor?.row === undefined || cursor.row === null) {
    return (
      <div className={styles.readout}>
        <p className={styles.row}>{t.format("ui-cursor-none")}</p>
        {excluded}
      </div>
    );
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
      {excluded}
    </div>
  );
}

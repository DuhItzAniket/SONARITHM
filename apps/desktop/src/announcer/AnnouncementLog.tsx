// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { AnnouncementInfo } from "@sonarithm/ipc";

import { useLocalizer } from "../i18n.ts";
import styles from "./announcer.module.css";

/**
 * The announcement history: the text record of everything spoken (A11Y-24), with display
 * text (unit symbols). It is a `log`, but not a live one: the announcer already spoke each
 * entry, and a second live region would say it twice.
 */
export function AnnouncementLog({
  entries,
}: {
  readonly entries: readonly AnnouncementInfo[];
}) {
  const t = useLocalizer();
  return (
    <div
      role="log"
      aria-live="off"
      aria-label={t.format("ui-tab-announcements")}
    >
      {entries.length === 0 ? (
        <p className={styles.empty}>{t.format("ui-no-announcements")}</p>
      ) : (
        <ol className={styles.log}>
          {entries.map((entry) => (
            <li key={entry.seq} className={styles.entry}>
              {entry.display}
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}

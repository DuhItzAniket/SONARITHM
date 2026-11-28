// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, CursorState, DatasetSummary } from "@sonarithm/ipc";
import { Disclosure } from "@sonarithm/ui";

import type { Announcer } from "../announcer/announcer.ts";
import { useLocalizer } from "../i18n.ts";
import { AudifyCalculator } from "./AudifyCalculator.tsx";
import { CriticalPoints } from "./CriticalPoints.tsx";
import type { OpenedFunction } from "./FunctionDialog.tsx";
import styles from "./function.module.css";

/**
 * The Inspector's sections for the shown dataset (C03-T08b): "Function", with its summary
 * and critical points, when the dataset is a function this session opened; and the
 * audification calculator for any dataset.
 */
export function FunctionInspector({
  client,
  dataset,
  opened,
  cursor,
  announcer,
}: {
  readonly client: Client;
  readonly dataset: DatasetSummary;
  /** The function shown, if the dataset is one. */
  readonly opened: OpenedFunction | null;
  readonly cursor: CursorState | null;
  readonly announcer: Announcer;
}) {
  const t = useLocalizer();
  const info = opened?.dataset.id === dataset.id ? opened.info : null;
  return (
    <div className={styles.inspector}>
      {info !== null && (
        <Disclosure title={t.format("ui-function-section")} defaultExpanded>
          <p className={styles.summary}>{info.summary}</p>
          <CriticalPoints
            client={client}
            dataset={dataset.id}
            points={info.criticalPoints}
            cursor={cursor}
            announcer={announcer}
          />
        </Disclosure>
      )}
      <Disclosure title={t.format("ui-audify-section")}>
        <AudifyCalculator
          client={client}
          dataset={dataset.id}
          announcer={announcer}
        />
      </Disclosure>
    </div>
  );
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Client,
  CursorState,
  DatasetSummary,
  FunctionInfo,
} from "@sonarithm/ipc";
import { Disclosure } from "@sonarithm/ui";
import { useEffect, useState } from "react";

import type { Announcer } from "../announcer/announcer.ts";
import { useLocalizer } from "../i18n.ts";
import { AudifyCalculator } from "./AudifyCalculator.tsx";
import { CriticalPoints } from "./CriticalPoints.tsx";
import styles from "./function.module.css";

/**
 * The Inspector's sections for the shown dataset (C03-T08b): "Function", with its summary
 * and critical points, when the dataset is a function; and the audification calculator for
 * any dataset. The function comes from the app core by the dataset's ID (`function_info`),
 * so a second function, or one reopened from a project, shows its own points (#498 review).
 */
export function FunctionInspector({
  client,
  dataset,
  cursor,
  announcer,
}: {
  readonly client: Client;
  readonly dataset: DatasetSummary;
  readonly cursor: CursorState | null;
  readonly announcer: Announcer;
}) {
  const t = useLocalizer();
  // The reply for one dataset; a reply for another (or none yet) shows nothing.
  const [details, setDetails] = useState<{
    readonly dataset: string;
    readonly info: FunctionInfo | null;
  } | null>(null);
  useEffect(() => {
    let live = true;
    client
      .call({ command: "function_info", args: { dataset: dataset.id } })
      .then(
        (reply) => {
          if (live)
            setDetails({ dataset: reply.dataset, info: reply.function });
        },
        () => undefined,
      );
    return () => {
      live = false;
    };
  }, [client, dataset.id]);
  const info = details?.dataset === dataset.id ? details.info : null;
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

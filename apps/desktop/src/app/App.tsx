// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, IpcTransport } from "@sonarithm/ipc";
import { connect } from "@sonarithm/ipc";
import { useEffect, useId, useState } from "react";

import { useLocalizer } from "../i18n.ts";
import styles from "./App.module.css";

type Start =
  | { readonly state: "starting" }
  | { readonly state: "ready"; readonly client: Client }
  | { readonly state: "failed"; readonly detail: string };

/**
 * The empty workspace (B01-T03): connects to the app core, which checks the protocol
 * version first (09 §2.2), and shows the workspace region. The application frame with its
 * regions, splitters and F6 order comes with B02-T06.
 */
export function App({ transport }: { readonly transport: IpcTransport }) {
  const t = useLocalizer();
  const titleId = useId();
  const [start, setStart] = useState<Start>({ state: "starting" });

  useEffect(() => {
    let live = true;
    connect(transport).then(
      (client) => {
        if (live) setStart({ state: "ready", client });
      },
      (error: unknown) => {
        if (live) {
          const detail = error instanceof Error ? error.message : "";
          setStart({ state: "failed", detail });
        }
      },
    );
    return () => {
      live = false;
    };
  }, [transport]);

  return (
    <main className={styles.workspace} aria-labelledby={titleId}>
      <h1 id={titleId} className={styles.title}>
        {t.format("ui-region-workspace")}
      </h1>
      {start.state === "failed" ? (
        <p role="alert" className={styles.error}>
          {t.format("ui-start-failed", { detail: start.detail })}
        </p>
      ) : (
        <p className={styles.status} data-state={start.state}>
          {start.state === "ready"
            ? t.format("ui-workspace-empty")
            : t.format("ui-starting")}
        </p>
      )}
    </main>
  );
}

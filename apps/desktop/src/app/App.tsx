// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, DatasetSummary, IpcTransport } from "@sonarithm/ipc";
import { connect } from "@sonarithm/ipc";
import { Tab, TabList, TabPanel, Tabs } from "@sonarithm/ui";
import { useEffect, useState } from "react";

import type { RoutedAnnouncer } from "../announcer/announcer.ts";
import { useAnnouncementFeed } from "../announcer/feed.ts";
import { AnnouncementLog } from "../announcer/AnnouncementLog.tsx";
import { CommandPalette } from "../commands/CommandPalette.tsx";
import { useCommand, useCommands, useKeymap } from "../commands/context.tsx";
import { KeyboardReference } from "../commands/KeyboardReference.tsx";
import type { Scope } from "../commands/scopes.ts";
import { activeScopes } from "../commands/scopes.ts";
import { useLocalizer } from "../i18n.ts";
import { Preferences } from "../preferences/Preferences.tsx";
import { TransportBar } from "../transport/TransportBar.tsx";
import { CursorReadout } from "../cursor/CursorReadout.tsx";
import { Explorer } from "../explorer/Explorer.tsx";
import { DatasetPlot } from "../plot/DatasetPlot.tsx";
import { useCursor } from "../cursor/useCursor.ts";
import { DatasetTable } from "../table/DatasetTable.tsx";
import { useTransport } from "../transport/useTransport.ts";
import styles from "./App.module.css";
import { Frame } from "./frame/Frame.tsx";

/**
 * Start-up facts the host passes before the page loads (`src-tauri/src/gpu.rs`): whether
 * the GPU setting could not be applied this time (B04-T02, F-044).
 */
function startFacts(): { readonly gpuFallback: boolean } {
  const value: unknown = Reflect.get(window, "__SONARITHM_START__");
  const gpuFallback =
    typeof value === "object" &&
    value !== null &&
    Reflect.get(value, "gpuFallback") === true;
  return { gpuFallback };
}

type Start =
  | { readonly state: "starting" }
  | { readonly state: "ready"; readonly client: Client }
  | { readonly state: "failed"; readonly detail: string };

/**
 * The app: connects to the app core, which checks the protocol version first (09 §2.2),
 * inside the application frame (B02-T06). The top bar holds the transport (B01-T05); the
 * other regions hold placeholders until their tasks: project tree (B03), plot and table
 * (B04) and inspector (B05). The Status region shows the announcement log (B03-T04).
 */
export function App({
  transport,
  announcer,
}: {
  readonly transport: IpcTransport;
  readonly announcer: RoutedAnnouncer;
}) {
  const t = useLocalizer();
  const [start, setStart] = useState<Start>({ state: "starting" });
  const client = start.state === "ready" ? start.client : null;
  const log = useAnnouncementFeed(client, announcer);
  const [gpuFallback] = useState(() => startFacts().gpuFallback);
  // Said once the app is ready, and shown in the Status region: the GPU setting applies
  // after a full restart (B04-T02).
  useEffect(() => {
    if (client !== null && gpuFallback) {
      announcer.announce({
        text: t.format("ui-gpu-fallback"),
        assertive: false,
      });
    }
  }, [client, gpuFallback, announcer, t]);
  const transportView = useTransport(client);
  const registry = useCommands();
  useKeymap(registry);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [referenceScopes, setReferenceScopes] = useState<
    readonly Scope[] | null
  >(null);
  useCommand("app.palette", () => {
    setPaletteOpen(true);
  });
  useCommand("help.keyboard", () => {
    setReferenceScopes(activeScopes(document.activeElement).scopes);
  });
  // H in the chart explorer: its own keys first (keyboard §5.4).
  useCommand("help.explorer", () => {
    setReferenceScopes(["explorer", "global"]);
  });
  // Stop all audio from any scope, even in a dialog (A11Y-15, T-13): sonification,
  // earcons and self-voicing, through the app core's `stop_all` (B01-T08).
  useCommand("transport.stopAll", () => {
    void client?.call({ command: "stop_all" }).catch(() => undefined);
  });
  // The dataset the Table tab shows: the first open one, until the project tree chooses
  // (B03). Rust owns the list (09 §3).
  const [dataset, setDataset] = useState<DatasetSummary | null>(null);
  useEffect(() => {
    if (client === null) {
      return;
    }
    let live = true;
    client.call({ command: "dataset_list" }).then(
      (reply) => {
        if (live) setDataset(reply.datasets[0] ?? null);
      },
      () => undefined,
    );
    return () => {
      live = false;
    };
  }, [client]);
  // The one cursor, Rust's (B04-T06): the table and the readout show it.
  const cursor = useCursor(client);
  const [preferencesOpen, setPreferencesOpen] = useState(false);
  useCommand("app.preferences", () => {
    setPreferencesOpen(true);
  });

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

  const startText =
    start.state === "failed" ? (
      <p role="alert" className={styles.error}>
        {t.format("ui-start-failed", { detail: start.detail })}
      </p>
    ) : (
      <p className={styles.status} data-state={start.state}>
        {start.state === "ready"
          ? t.format("ui-workspace-empty")
          : t.format("ui-starting")}
      </p>
    );

  return (
    <>
      <Frame
        startIn={start.state === "ready" ? "project" : null}
        top={<TransportBar transport={transportView} />}
        project={
          <p className={styles.status}>{t.format("ui-workspace-empty")}</p>
        }
        workspace={
          <Tabs>
            <TabList label={t.format("ui-region-workspace")}>
              <Tab id="plot">{t.format("ui-tab-plot")}</Tab>
              <Tab id="table">{t.format("ui-tab-table")}</Tab>
              <Tab id="spec">{t.format("ui-tab-spec")}</Tab>
            </TabList>
            <TabPanel id="plot">
              {client !== null && dataset !== null ? (
                <>
                  <Explorer
                    client={client}
                    dataset={dataset}
                    cursor={cursor}
                    announcer={announcer}
                  />
                  <DatasetPlot
                    client={client}
                    dataset={dataset}
                    cursor={cursor}
                    announcer={announcer}
                  />
                </>
              ) : (
                startText
              )}
            </TabPanel>
            <TabPanel id="table">
              {client !== null && dataset !== null ? (
                <DatasetTable
                  client={client}
                  dataset={dataset}
                  announcer={announcer}
                  cursor={cursor}
                />
              ) : (
                startText
              )}
            </TabPanel>
            <TabPanel id="spec">{startText}</TabPanel>
          </Tabs>
        }
        inspector={
          <p className={styles.status}>{t.format("ui-inspector-empty")}</p>
        }
        status={
          <>
            <CursorReadout cursor={cursor} />
            {gpuFallback && (
              <p className={styles.status}>{t.format("ui-gpu-fallback")}</p>
            )}
            <Tabs>
              <TabList label={t.format("ui-region-status")}>
                <Tab id="announcements">{t.format("ui-tab-announcements")}</Tab>
                <Tab id="diagnostics">{t.format("ui-tab-diagnostics")}</Tab>
              </TabList>
              <TabPanel id="announcements">
                <AnnouncementLog entries={log} />
              </TabPanel>
              <TabPanel id="diagnostics">
                <p className={styles.status}>{t.format("ui-no-diagnostics")}</p>
              </TabPanel>
            </Tabs>
          </>
        }
      />
      <Preferences
        client={client}
        isOpen={preferencesOpen}
        onOpenChange={setPreferencesOpen}
      />
      <CommandPalette
        isOpen={paletteOpen}
        onOpenChange={setPaletteOpen}
        announcer={announcer}
      />
      <KeyboardReference
        scopes={referenceScopes}
        onClose={() => {
          setReferenceScopes(null);
        }}
      />
    </>
  );
}

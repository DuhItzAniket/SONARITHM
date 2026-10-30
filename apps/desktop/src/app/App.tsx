// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, IpcTransport } from "@sonarithm/ipc";
import { connect } from "@sonarithm/ipc";
import { Button, Tab, TabList, TabPanel, Tabs, Toolbar } from "@sonarithm/ui";
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
import styles from "./App.module.css";
import { Frame } from "./frame/Frame.tsx";

type Start =
  | { readonly state: "starting" }
  | { readonly state: "ready"; readonly client: Client }
  | { readonly state: "failed"; readonly detail: string };

/**
 * The app: connects to the app core, which checks the protocol version first (09 §2.2),
 * inside the application frame (B02-T06). The regions hold placeholders until their tasks:
 * transport (B01-T05), project tree (B03), plot and table (B04) and inspector (B05). The
 * Status region shows the announcement log (B03-T04).
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
  // Stop all audio from any scope (A11Y-15); B01-T08 adds earcons and self-voicing.
  useCommand("transport.stopAll", () => {
    void client?.call({ command: "transport_stop" }).catch(() => undefined);
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
        top={
          // The transport commands are wired with the transport machine (B01-T05).
          <Toolbar label={t.format("ui-transport")}>
            <Button variant="quiet" isDisabled>
              {t.format("ui-play")}
            </Button>
            <Button variant="quiet" isDisabled>
              {t.format("ui-stop")}
            </Button>
          </Toolbar>
        }
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
            <TabPanel id="plot">{startText}</TabPanel>
            <TabPanel id="table">{startText}</TabPanel>
            <TabPanel id="spec">{startText}</TabPanel>
          </Tabs>
        }
        inspector={
          <p className={styles.status}>{t.format("ui-inspector-empty")}</p>
        }
        status={
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
        }
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

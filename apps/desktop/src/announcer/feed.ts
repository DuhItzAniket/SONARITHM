// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { AnnouncementInfo, Client, Event } from "@sonarithm/ipc";
import { useEffect, useState } from "react";

import type { RoutedAnnouncer } from "./announcer.ts";

/** The most announcements the log keeps in the UI; the app core keeps the full history. */
export const LOG_LIMIT = 200;

/**
 * Speaks one app-core announcement and returns what the log should show. A self-voiced
 * announcement was already spoken by SONARITHM, and a muted one came while speech was
 * turned off (B03-T09), so both are only logged (A11Y-19).
 */
export function deliver(
  info: AnnouncementInfo,
  announcer: RoutedAnnouncer,
): AnnouncementInfo {
  if (!info.selfVoiced && !info.muted) {
    announcer.announce({ text: info.speech, assertive: info.assertive });
  }
  return info;
}

/**
 * Connects the announcer to the app core: reads the route from `speech.announcer`, then
 * speaks every event on the `announcements` channel and keeps the latest for the log.
 */
export function useAnnouncementFeed(
  client: Client | null,
  announcer: RoutedAnnouncer,
): readonly AnnouncementInfo[] {
  const [log, setLog] = useState<readonly AnnouncementInfo[]>([]);
  useEffect(() => {
    if (client === null) {
      return;
    }
    // A host that cannot read settings keeps the default route (live regions).
    client.call({ command: "settings_get" }).then(
      (settings) => {
        announcer.setRoute(settings.speech.announcer);
      },
      () => undefined,
    );
    return client.subscribe("announcements", (event: Event) => {
      if (event.event === "announcement") {
        const info = deliver(event, announcer);
        setLog((entries) => [...entries.slice(1 - LOG_LIMIT), info]);
      }
    });
  }, [client, announcer]);
  return log;
}

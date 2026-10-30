// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { AnnouncerRoute } from "@sonarithm/ipc";

/** Text for the screen reader (08 §4). */
export interface Announcement {
  readonly text: string;
  /** Errors, device loss and alarms only; everything else is polite (08 §4). */
  readonly assertive: boolean;
}

/**
 * The one announcer interface both OD-05 routes sit behind (B03-T04, OD-15 condition 2).
 * Announcing never moves focus (A11Y-17).
 */
export interface Announcer {
  announce(announcement: Announcement): void;
}

/** A live region's politeness. */
export type Politeness = "polite" | "assertive";

/** Where the live-region route writes: one region per politeness. */
export interface LiveRegionSink {
  set(politeness: Politeness, text: string): void;
}

/** Runs `run` after the clear has reached the accessibility tree; returns a cancel. */
export type Schedule = (run: () => void) => () => void;

/** How long a region stays empty before the new text: long enough for NVDA (A02, F-046). */
export const CLEAR_MS = 50;

const defaultSchedule: Schedule = (run) => {
  const id = setTimeout(run, CLEAR_MS);
  return () => {
    clearTimeout(id);
  };
};

/**
 * The live-region route, the default (OD-05, provisional): a managed polite and assertive
 * pair. Each announcement clears its region and sets the text a moment later, so a
 * repeated message is spoken again (clear-then-set). A newer announcement of the same
 * politeness supersedes one that has not been set yet, so only the latest is spoken.
 */
export function createLiveRegionAnnouncer(
  sink: LiveRegionSink,
  schedule: Schedule = defaultSchedule,
): Announcer {
  const pending = new Map<Politeness, () => void>();
  return {
    announce({ text, assertive }) {
      const politeness: Politeness = assertive ? "assertive" : "polite";
      pending.get(politeness)?.();
      sink.set(politeness, "");
      pending.set(
        politeness,
        schedule(() => {
          pending.delete(politeness);
          sink.set(politeness, text);
        }),
      );
    },
  };
}

/**
 * The host's native notification route: UI Automation on Windows (ADR-0025). It rejects
 * where the platform has none.
 */
export interface NativeAnnouncer {
  announce(text: string, assertive: boolean): Promise<void>;
}

/** An announcer whose route follows the `speech.announcer` setting. */
export interface RoutedAnnouncer extends Announcer {
  setRoute(route: AnnouncerRoute): void;
  readonly route: AnnouncerRoute;
}

/**
 * The announcer the app uses: the route is chosen here, in shared code, and the host only
 * makes the platform call. The native route falls back to live regions when there is no
 * native announcer or the call fails, so nothing is ever lost (A11Y-17).
 */
export function createRoutedAnnouncer(options: {
  readonly live: Announcer;
  readonly native: NativeAnnouncer | null;
  readonly route?: AnnouncerRoute;
  readonly onFallback?: (error: unknown) => void;
}): RoutedAnnouncer {
  let route: AnnouncerRoute = options.route ?? "liveRegion";
  return {
    get route() {
      return route;
    },
    setRoute(next) {
      route = next;
    },
    announce(announcement) {
      const { native } = options;
      if (route !== "native" || native === null) {
        options.live.announce(announcement);
        return;
      }
      native
        .announce(announcement.text, announcement.assertive)
        .catch((error: unknown) => {
          options.live.announce(announcement);
          options.onFallback?.(error);
        });
    },
  };
}

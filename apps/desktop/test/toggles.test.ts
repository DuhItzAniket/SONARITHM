// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The UI's side of the explorer's toggles (B03-T09, keyboard §5.5): a muted announcement
// is logged, not spoken, and the plot's highlight follows the effective visual modality.
import assert from "node:assert/strict";
import { test } from "node:test";

import type { AnnouncementInfo } from "@sonarithm/ipc";

import type { RoutedAnnouncer } from "../src/announcer/announcer.ts";
import { deliver } from "../src/announcer/feed.ts";
import { visualOn } from "../src/explorer/effective.ts";

function recorder(): { said: string[]; announcer: RoutedAnnouncer } {
  const said: string[] = [];
  const announcer: RoutedAnnouncer = {
    announce({ text }) {
      said.push(text);
    },
    setRoute() {
      // One route is enough here.
    },
    route: "liveRegion",
  };
  return { said, announcer };
}

const info = (muted: boolean): AnnouncementInfo => ({
  seq: 1,
  speech: "Row 3",
  display: "Row 3",
  class: "navigation",
  assertive: false,
  interrupt: false,
  selfVoiced: false,
  node: null,
  muted,
});

test("b03_t09_a_muted_announcement_is_logged_not_spoken", () => {
  const { said, announcer } = recorder();
  assert.equal(deliver(info(true), announcer).display, "Row 3");
  assert.deepEqual(said, []);
  deliver(info(false), announcer);
  assert.deepEqual(said, ["Row 3"]);
});

test("b03_t09_the_highlight_follows_the_effective_visual_modality", () => {
  const visual = (value: unknown) => ({
    key: "interaction.modalities.visual",
    value,
    layer: "session" as const,
  });
  assert.equal(visualOn([visual(false)]), false);
  assert.equal(visualOn([visual(true)]), true);
  // Not reported: the default, on (A11Y-10).
  assert.equal(visualOn([]), true);
});

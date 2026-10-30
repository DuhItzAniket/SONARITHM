// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The announcer and its two routes (B03-T04; 08 §4, OD-05).
import assert from "node:assert/strict";
import { test } from "node:test";

import type {
  LiveRegionSink,
  NativeAnnouncer,
  Politeness,
  Schedule,
} from "../src/announcer/announcer.ts";
import {
  createLiveRegionAnnouncer,
  createRoutedAnnouncer,
} from "../src/announcer/announcer.ts";

/** A sink that records writes, and a schedule run by hand. */
function harness() {
  const writes: [Politeness, string][] = [];
  const sink: LiveRegionSink = {
    set(politeness, text) {
      writes.push([politeness, text]);
    },
  };
  const queue = new Set<() => void>();
  const schedule: Schedule = (run) => {
    queue.add(run);
    return () => queue.delete(run);
  };
  const flush = () => {
    for (const run of [...queue]) {
      queue.delete(run);
      run();
    }
  };
  return { writes, sink, schedule, flush };
}

test("a11y_17_live_region_clears_then_sets", () => {
  const h = harness();
  const live = createLiveRegionAnnouncer(h.sink, h.schedule);
  live.announce({ text: "28.7 °C", assertive: false });
  assert.deepEqual(h.writes, [["polite", ""]]);
  h.flush();
  assert.deepEqual(h.writes, [
    ["polite", ""],
    ["polite", "28.7 °C"],
  ]);
});

test("a11y_17_a_newer_announcement_supersedes_one_not_yet_spoken", () => {
  const h = harness();
  const live = createLiveRegionAnnouncer(h.sink, h.schedule);
  live.announce({ text: "first", assertive: false });
  live.announce({ text: "second", assertive: false });
  h.flush();
  assert.deepEqual(
    h.writes.filter(([, text]) => text !== ""),
    [["polite", "second"]],
  );
});

test("assertive and polite use separate regions and do not supersede each other", () => {
  const h = harness();
  const live = createLiveRegionAnnouncer(h.sink, h.schedule);
  live.announce({ text: "Device lost.", assertive: true });
  live.announce({ text: "Paused.", assertive: false });
  h.flush();
  assert.deepEqual(
    h.writes.filter(([, text]) => text !== ""),
    [
      ["assertive", "Device lost."],
      ["polite", "Paused."],
    ],
  );
});

function spyLive() {
  const said: string[] = [];
  return {
    said,
    live: { announce: (a: { text: string }) => said.push(a.text) },
  };
}

test("od_05_live_regions_are_the_default_route", () => {
  const { said, live } = spyLive();
  const native: NativeAnnouncer = {
    announce: () => Promise.reject(new Error("must not be called")),
  };
  const announcer = createRoutedAnnouncer({ live, native });
  assert.equal(announcer.route, "liveRegion");
  announcer.announce({ text: "hello", assertive: false });
  assert.deepEqual(said, ["hello"]);
});

test("od_05_the_native_route_is_used_when_selected", async () => {
  const { said, live } = spyLive();
  const sent: [string, boolean][] = [];
  const native: NativeAnnouncer = {
    announce(text, assertive) {
      sent.push([text, assertive]);
      return Promise.resolve();
    },
  };
  const announcer = createRoutedAnnouncer({ live, native, route: "native" });
  announcer.announce({ text: "Clipped.", assertive: true });
  await Promise.resolve();
  assert.deepEqual(sent, [["Clipped.", true]]);
  assert.deepEqual(said, []);
});

test("od_05_the_native_route_falls_back_to_live_regions", async () => {
  const { said, live } = spyLive();
  let fellBack = false;
  const failing: NativeAnnouncer = {
    announce: () => Promise.reject(new Error("no UI Automation here")),
  };
  const announcer = createRoutedAnnouncer({
    live,
    native: failing,
    route: "native",
    onFallback: () => {
      fellBack = true;
    },
  });
  announcer.announce({ text: "one", assertive: false });
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.deepEqual(said, ["one"]);
  assert.equal(fellBack, true);

  // A host without a native route (Linux, the mock) uses live regions.
  const none = createRoutedAnnouncer({ live, native: null, route: "native" });
  none.announce({ text: "two", assertive: false });
  assert.deepEqual(said, ["one", "two"]);
});

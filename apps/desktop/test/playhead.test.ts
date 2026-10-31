// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// B04-T04: clock anchors feed the playhead; the anchor's output latency corrects it, and a
// latency the app core does not know means no correction (ADR-0017 "Clock anchors").
import assert from "node:assert/strict";
import { test } from "node:test";

import type { ClockAnchor, TransportState } from "@sonarithm/ipc";
import { PlayheadClock } from "@sonarithm/viz/playhead";

import { latencyOf, toPlayheadAnchor } from "../src/transport/playhead.ts";

const PLAYING: TransportState = {
  status: "playing",
  positionS: 2,
  durationS: 10,
  speed: 1.5,
  loopRange: null,
  loaded: true,
  device: "Speakers",
  live: false,
  xruns: 0,
};

const anchor = (outputLatencyS: number | null): ClockAnchor => ({
  transport: PLAYING,
  hostTimeS: null,
  outputLatencyS,
  seq: 1,
});

test("b04_t04 an anchor's latency corrects the playhead", () => {
  const clock = new PlayheadClock();
  const known = anchor(0.08);
  clock.update(toPlayheadAnchor(known), 0);
  assert.ok(
    Math.abs(clock.positionAt(1, latencyOf(known)) - (3.5 - 0.08 * 1.5)) < 1e-9,
  );
});

test("b04_t04 an unknown latency means no correction, not a guess", () => {
  const clock = new PlayheadClock();
  const unknown = anchor(null);
  clock.update(toPlayheadAnchor(unknown), 0);
  assert.equal(latencyOf(unknown), 0);
  assert.equal(clock.positionAt(1, latencyOf(unknown)), 3.5);
});

test("b04_t04 only playing anchors move the playhead", () => {
  const paused = toPlayheadAnchor({
    ...anchor(0.08),
    transport: { ...PLAYING, status: "paused" },
  });
  assert.equal(paused.playing, false);
});

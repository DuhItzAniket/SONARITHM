// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The playhead clock (B04-T04; PB-08, F-033): extrapolation from anchors with IPC jitter,
// the latency correction, and changes (pause, speed, seek).
import assert from "node:assert/strict";
import { test } from "node:test";

import { PlayheadClock } from "./playhead.ts";

/** One frame at 60 Hz, in seconds: the drift budget. */
const FRAME_S = 1 / 60;

/** A seeded uniform generator (xorshift32), for reproducible delays. */
function random(seed: number): () => number {
  let s = seed;
  return () => {
    s ^= s << 13;
    s ^= s >>> 17;
    s ^= s << 5;
    return (s >>> 0) / 2 ** 32;
  };
}

void test("b04_t04 the playhead stays within one frame over a minute of anchors with ipc delay", () => {
  // Anchors every 2 s (06 §9), each arriving 0–8 ms late, at speeds 1 and 2; the true heard
  // position is the engine's minus the latency.
  for (const speed of [1, 2]) {
    const delay = random(1729 + speed);
    const latency = 0.08;
    const clock = new PlayheadClock();
    let worst = 0;
    for (let sent = 0; sent <= 60; sent += 2) {
      const received = sent + delay() * 0.008;
      clock.update({ positionS: sent * speed, speed, playing: true }, received);
      // Every frame until the next anchor arrives.
      for (let now = received; now < sent + 2; now += FRAME_S) {
        const truth = Math.max(0, (now - latency) * speed);
        worst = Math.max(
          worst,
          Math.abs(clock.positionAt(now, latency) - truth),
        );
      }
    }
    assert.ok(
      worst <= FRAME_S * speed,
      `speed ${String(speed)}: ${worst.toFixed(4)} s`,
    );
  }
});

void test("b04_t04 a late anchor does not pull the playhead back", () => {
  const clock = new PlayheadClock();
  clock.update({ positionS: 0, speed: 1, playing: true }, 0.001);
  // The next anchor was sent at 2 s and arrived 30 ms late: the line set by the first
  // (1 ms late) predicts further, so it is kept.
  clock.update({ positionS: 2, speed: 1, playing: true }, 2.03);
  assert.ok(Math.abs(clock.positionAt(3, 0) - 2.999) < 1e-9);
});

void test("b04_t04 the calibration offset is applied", () => {
  // ADR-0016, F-033: the engine runs ahead of what is heard by the output latency; a
  // calibrated offset of 71 ms puts the playhead 71 ms (times the speed) behind the engine.
  const clock = new PlayheadClock();
  clock.update({ positionS: 10, speed: 1.5, playing: true }, 0);
  assert.equal(clock.positionAt(1, 0), 11.5);
  assert.ok(Math.abs(clock.positionAt(1, 0.071) - (11.5 - 0.071 * 1.5)) < 1e-9);
  // Never before the start.
  const start = new PlayheadClock();
  start.update({ positionS: 0, speed: 1, playing: true }, 0);
  assert.equal(start.positionAt(0.01, 0.071), 0);
});

void test("b04_t04 pause, speed changes and backward seeks start a new line", () => {
  const clock = new PlayheadClock();
  clock.update({ positionS: 0, speed: 1, playing: true }, 0);
  // Paused at 1.5 s: the position stays, latency or not.
  clock.update({ positionS: 1.5, speed: 1, playing: false }, 1.5);
  assert.equal(clock.positionAt(5, 0.08), 1.5);
  assert.equal(clock.playing, false);
  // Playing again at double speed.
  clock.update({ positionS: 1.5, speed: 2, playing: true }, 6);
  assert.ok(Math.abs(clock.positionAt(7, 0) - 3.5) < 1e-9);
  // A backward seek of 1 s while playing is taken at once.
  clock.update({ positionS: 2.5, speed: 2, playing: true }, 7);
  assert.ok(Math.abs(clock.positionAt(7, 0) - 2.5) < 1e-9);
});

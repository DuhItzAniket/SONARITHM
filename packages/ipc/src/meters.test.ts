// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import assert from "node:assert/strict";
import { test } from "node:test";

import { SAMPLE_METER_FRAME_BASE64 } from "./generated/fixtures.ts";
import { METERS } from "./generated/protocol.ts";
import { MeterFrameError, readMeterFrame } from "./meters.ts";

function sample(): ArrayBuffer {
  return Uint8Array.from(atob(SAMPLE_METER_FRAME_BASE64), (c) =>
    c.charCodeAt(0),
  ).buffer;
}

void test("b01_t04 a meter frame encoded by Rust reads back", () => {
  const frame = readMeterFrame(sample());
  assert.deepEqual(frame, {
    seq: 7,
    positionS: 1.25,
    peak: [0.5, 0.125],
    limiting: true,
  });
});

void test("b01_t04 the meter reader refuses other lengths, magics and versions", () => {
  assert.throws(() => readMeterFrame(new ArrayBuffer(31)), MeterFrameError);
  const wrongVersion = new Uint8Array(sample().slice(0));
  wrongVersion[METERS.offset.version] = 2;
  assert.throws(() => readMeterFrame(wrongVersion.buffer), MeterFrameError);
  const wrongMagic = new Uint8Array(sample().slice(0));
  wrongMagic[0] = 0x58;
  assert.throws(() => readMeterFrame(wrongMagic.buffer), MeterFrameError);
});

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import assert from "node:assert/strict";
import { test } from "node:test";

import { SAMPLE_METER_FRAME_BASE64 } from "./generated/fixtures.ts";
import { BINARY_VIEW_ERROR, METERS } from "./generated/protocol.ts";
import { MeterFrameError, readMeterFrame } from "./meters.ts";

function sample(): Uint8Array {
  return Uint8Array.from(atob(SAMPLE_METER_FRAME_BASE64), (c) =>
    c.charCodeAt(0),
  );
}

/** A plain `ArrayBuffer` holding `bytes`, as the host delivers it. */
function toBuffer(bytes: Uint8Array): ArrayBuffer {
  const out = new ArrayBuffer(bytes.length);
  new Uint8Array(out).set(bytes);
  return out;
}

function refusedWith050(bytes: Uint8Array): void {
  assert.throws(
    () => readMeterFrame(toBuffer(bytes)),
    (error) =>
      error instanceof MeterFrameError && error.code === BINARY_VIEW_ERROR,
  );
}

void test("b01_t04 a meter frame encoded by Rust reads back", () => {
  assert.deepEqual(readMeterFrame(toBuffer(sample())), {
    seq: 7,
    positionS: 1.25,
    peak: [0.5, 0.125],
    limiting: true,
    last: false,
  });
});

void test("b01_t04 a short buffer, a wrong magic or version is SNR-APP-050", () => {
  refusedWith050(sample().slice(0, 31));
  const wrongVersion = sample();
  wrongVersion[METERS.offset.version] = 2;
  refusedWith050(wrongVersion);
  const wrongMagic = sample();
  wrongMagic[0] = 0x58;
  refusedWith050(wrongMagic);
});

void test("b01_t04 the last flag reads, and unknown flag bits are ignored", () => {
  const bytes = sample();
  bytes[METERS.offset.flags] = METERS.flagLast | 0b1000_0000;
  bytes[METERS.offset.flags + 1] = 0xff;
  const frame = readMeterFrame(toBuffer(bytes));
  assert.equal(frame.last, true);
  assert.equal(frame.limiting, false);
});

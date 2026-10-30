// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { BINARY_VIEW_ERROR, METERS } from "./generated/protocol.ts";

/** One meter reading of the master (B01-T04), decoded from its binary frame. */
export interface MeterFrame {
  /** Counts up with every frame of the session. */
  readonly seq: number;
  /** The transport position of the reading in seconds. */
  readonly positionS: number;
  /** Peak absolute sample, left and right, over the last meter window (linear; 1 is full scale). */
  readonly peak: readonly [number, number];
  /** Whether the master limiter engaged in that window. */
  readonly limiting: boolean;
}

/** The buffer is not a meter frame this version reads. */
export class MeterFrameError extends Error {
  /** `SNR-APP-050`, as for binary views: the reader never guesses (ADR-0017 §6). */
  readonly code: string = BINARY_VIEW_ERROR;

  constructor(reason: string) {
    super(`Not a meter frame this version reads: ${reason}.`);
    this.name = "MeterFrameError";
  }
}

/** Reads a meter frame (`crates/sonarithm-app/src/ipc/meters.rs`); strict, like `readView`. */
export function readMeterFrame(buffer: ArrayBuffer): MeterFrame {
  if (buffer.byteLength !== METERS.bytes) {
    throw new MeterFrameError(`${String(buffer.byteLength)} bytes`);
  }
  const data = new DataView(buffer);
  const magic = String.fromCharCode(
    ...new Uint8Array(buffer, METERS.offset.magic, 4),
  );
  if (magic !== METERS.magic) {
    throw new MeterFrameError("wrong magic bytes");
  }
  if (data.getUint16(METERS.offset.version, true) !== METERS.version) {
    throw new MeterFrameError("unknown version");
  }
  return {
    seq: Number(data.getBigUint64(METERS.offset.seq, true)),
    positionS: data.getFloat64(METERS.offset.position, true),
    peak: [
      data.getFloat32(METERS.offset.peakLeft, true),
      data.getFloat32(METERS.offset.peakRight, true),
    ],
    limiting:
      (data.getUint16(METERS.offset.flags, true) & METERS.flagLimiting) !== 0,
  };
}

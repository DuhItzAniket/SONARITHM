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
  /**
   * Whether the master limiter engaged in that window. A meter shows it as text or an
   * accessible state, never by colour alone (DL-X08).
   */
  readonly limiting: boolean;
  /**
   * The last frame before playback stopped, paused or lost its device: its peaks are
   * zero, so the meter falls to silence instead of freezing.
   */
  readonly last: boolean;
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
  // Unknown flag bits are ignored: a new flag does not change the layout version.
  const flags = data.getUint16(METERS.offset.flags, true);
  return {
    seq: Number(data.getBigUint64(METERS.offset.seq, true)),
    positionS: data.getFloat64(METERS.offset.position, true),
    peak: [
      data.getFloat32(METERS.offset.peakLeft, true),
      data.getFloat32(METERS.offset.peakRight, true),
    ],
    limiting: (flags & METERS.flagLimiting) !== 0,
    last: (flags & METERS.flagLast) !== 0,
  };
}

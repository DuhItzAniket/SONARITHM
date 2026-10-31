// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * The playhead clock (B04-T04; 06 §9): the UI extrapolates the heard position every animation
 * frame from the app core's clock anchors, so no IPC message is needed per frame.
 *
 * An anchor gives the engine's position when it was sent. It arrives a few milliseconds
 * later (F-041: about 2 ms for Channel messages), and a late arrival would make a naive
 * extrapolation lag by the delay. While nothing changes (same speed, still playing, no
 * seek), every anchor describes the same line, so the clock keeps the line that predicts
 * the furthest position: the least-delayed anchor's (a minimum-delay filter). A change (a
 * new status, speed or a jump) starts a new line.
 *
 * The engine's position runs ahead of what is heard by the output latency: the calibrated
 * offset of the device where one exists (ADR-0016, F-033), else the backend's estimate.
 * The heard position is the engine's minus the latency times the speed.
 */

/** What the clock needs from an anchor (the IPC `ClockAnchor`'s transport). */
export interface PlayheadAnchor {
  /** The engine's position in seconds. */
  readonly positionS: number;
  readonly speed: number;
  readonly playing: boolean;
}

/**
 * How far behind the current line a new anchor may fall and still count as a late copy of
 * it, in seconds of position: well above IPC delays (milliseconds, F-041), well below a
 * seek anyone would make. Anything further back is a jump, such as a backward seek.
 */
const JUMP_S = 0.05;

/**
 * Extrapolates the heard position from anchors. Times are in seconds on one monotonic
 * clock of the caller's choice (`performance.now() / 1000` in the UI).
 */
export class PlayheadClock {
  #anchor: PlayheadAnchor | null = null;
  #at = 0;

  /** Takes an anchor received at `receivedAtS`. */
  update(anchor: PlayheadAnchor, receivedAtS: number): void {
    const current = this.#anchor;
    if (
      current !== null &&
      current.playing &&
      anchor.playing &&
      current.speed === anchor.speed
    ) {
      const predicted = this.#engineAt(receivedAtS);
      const lead = anchor.positionS - predicted;
      // Within the same run, an anchor that predicts no further than the current line
      // arrived later than the one that set it: keep the line.
      if (lead <= 0 && lead > -JUMP_S) {
        return;
      }
    }
    this.#anchor = anchor;
    this.#at = receivedAtS;
  }

  /** The engine's position at `nowS`. */
  #engineAt(nowS: number): number {
    const anchor = this.#anchor;
    if (anchor === null) return 0;
    if (!anchor.playing) return anchor.positionS;
    return anchor.positionS + Math.max(0, nowS - this.#at) * anchor.speed;
  }

  /**
   * The heard position at `nowS`, `latencyS` being the device's output latency (its
   * calibrated offset, else the backend's estimate; 0 if unknown). Never below 0.
   */
  positionAt(nowS: number, latencyS: number): number {
    const anchor = this.#anchor;
    if (anchor === null) return 0;
    if (!anchor.playing) return anchor.positionS;
    return Math.max(0, this.#engineAt(nowS) - latencyS * anchor.speed);
  }

  /** Whether the position moves (an anchor said playing). */
  get playing(): boolean {
    return this.#anchor?.playing ?? false;
  }
}

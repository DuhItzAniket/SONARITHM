// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { useEffect, useRef } from "react";

import type { Range } from "./dense/scene.ts";
import type { PlayheadClock } from "./playhead.ts";
import styles from "./playhead.module.css";

/** Props for {@link Playhead}. */
export interface PlayheadProps {
  /** The clock the anchors feed; it is read every frame, never copied into React state. */
  readonly clock: PlayheadClock;
  /** The output latency in seconds (calibrated offset, else the backend's estimate). */
  readonly latencyS: number;
  /** The plot's time range: where the playhead's position maps across the width. */
  readonly range: Range;
}

/**
 * The playhead (B04-T04; 06 §9, notes/05 §6): a line over a dense layer or plot, moved
 * every animation frame with a CSS transform from the clock's extrapolated position. It
 * lives outside React reconciliation (no state, no render per frame) and never redraws
 * the data under it. It is presentational: the cursor readout and announcements say where
 * playback is. It keeps moving with reduced motion, because it is content the user started
 * (design-language §9).
 */
export function Playhead({ clock, latencyS, range }: PlayheadProps) {
  const line = useRef<HTMLDivElement>(null);
  // Read every frame through refs, so a new latency or range does not restart the loop.
  const latest = useRef({ clock, latencyS, range });
  useEffect(() => {
    latest.current = { clock, latencyS, range };
  });

  useEffect(() => {
    let frame = 0;
    const tick = () => {
      const element = line.current;
      const parent = element?.parentElement;
      if (element !== null && parent !== null && parent !== undefined) {
        const {
          clock: source,
          latencyS: latency,
          range: span,
        } = latest.current;
        const position = source.positionAt(performance.now() / 1000, latency);
        const fraction = (position - span.min) / (span.max - span.min);
        const inside = fraction >= 0 && fraction <= 1;
        element.style.visibility = inside ? "visible" : "hidden";
        element.style.transform = `translateX(${String(fraction * parent.clientWidth)}px)`;
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, []);

  return <div ref={line} className={styles.playhead} aria-hidden="true" />;
}

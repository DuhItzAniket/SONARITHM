// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { ReactNode } from "react";
import { useEffect, useId, useRef, useState } from "react";

import styles from "./dense.module.css";
import type { LayerRenderer } from "./dense/renderer.ts";
import { createRenderer } from "./dense/renderer.ts";
import type { DenseScene } from "./dense/scene.ts";
import type { PlotSummary } from "./Plot.tsx";

/** Props for {@link DenseLayer}. */
export interface DenseLayerProps {
  readonly summary: PlotSummary;
  /** What to draw. Pass the same object while the data is unchanged: a new one redraws. */
  readonly scene: DenseScene;
  /** The layer's height in CSS pixels; the width follows the container. */
  readonly height: number;
  /** Draw with Canvas2D even where WebGL2 exists (tests; a setting for broken drivers). */
  readonly forceCanvas?: boolean;
  /**
   * Elements over the data, such as the playhead (B04-T04). They may change every frame;
   * the data layer is not redrawn for them (notes/05 §6, PB-08).
   */
  readonly children?: ReactNode;
  /** Called after each draw with the backend and the draw count (tests, traces). */
  readonly onDraw?: (renderer: LayerRenderer) => void;
}

/**
 * A dense view (B04-T02): waveforms, spectrograms and lines of a million points, drawn
 * with WebGL2, or with Canvas2D where WebGL2 is unavailable. Like {@link Plot} it is one
 * image named by the summary's L1 and described by its L2 (A11Y-01), and not focusable.
 * The data is drawn once per scene, size or theme; the overlay above it is for what moves.
 */
export function DenseLayer({
  summary,
  scene,
  height,
  forceCanvas = false,
  children,
  onDraw,
}: DenseLayerProps) {
  const descriptionId = useId();
  const box = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const renderer = useRef<LayerRenderer | null>(null);
  const [size, setSize] = useState<{ width: number; height: number } | null>(
    null,
  );
  const latestOnDraw = useRef(onDraw);
  useEffect(() => {
    latestOnDraw.current = onDraw;
  });

  // The device-pixel size follows the container.
  useEffect(() => {
    const element = box.current;
    if (element === null) {
      return;
    }
    const measure = () => {
      const ratio = window.devicePixelRatio;
      const width = Math.max(1, Math.round(element.clientWidth * ratio));
      const tall = Math.max(1, Math.round(height * ratio));
      // An unchanged size keeps its object, so it does not redraw the data.
      setSize((previous) =>
        previous?.width === width && previous.height === tall
          ? previous
          : { width, height: tall },
      );
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, [height]);

  // One renderer per canvas.
  useEffect(() => {
    const element = canvas.current;
    if (element === null) {
      return;
    }
    const created = createRenderer(element, { forceCanvas });
    if (created === null) {
      return;
    }
    renderer.current = created;
    return () => {
      created.destroy();
      renderer.current = null;
    };
  }, [forceCanvas]);

  // The data is drawn when the scene or the size changes, never for the overlay.
  useEffect(() => {
    const element = canvas.current;
    const current = renderer.current;
    if (element === null || current === null || size === null) {
      return;
    }
    element.width = size.width;
    element.height = size.height;
    current.draw(scene);
    latestOnDraw.current?.(current);
  }, [scene, size, forceCanvas]);

  return (
    <div
      role="img"
      aria-label={summary.l1}
      aria-describedby={descriptionId}
      className={styles.layer}
      ref={box}
      style={{ blockSize: `${String(height)}px` }}
    >
      <canvas
        key={forceCanvas ? "canvas2d" : "auto"}
        ref={canvas}
        className={styles.data}
      />
      <div className={styles.overlay}>{children}</div>
      <p id={descriptionId} hidden>
        {summary.l2}
      </p>
    </div>
  );
}

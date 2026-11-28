// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { MouseEvent } from "react";

import { cx } from "./cx.ts";
import styles from "./crosshair.module.css";
import { Slider } from "./Slider.tsx";

/** A point marked on the image, such as a calibration anchor. */
export interface ImageMark {
  readonly id: string;
  /** Which axis it belongs to: x marks are squares, y marks circles, never colour alone. */
  readonly axis: "x" | "y";
  /** The pixel it marks (a pixel's index, from 0). */
  readonly at: readonly [number, number];
  /** Its short visible label, such as "x1". */
  readonly label: string;
}

/** A rectangle in pixels, such as the plot area. */
export interface PixelArea {
  readonly min: readonly [number, number];
  readonly max: readonly [number, number];
}

/** Props of {@link ImageCrosshair}. */
export interface ImageCrosshairProps {
  /** The image, as a URL the page may load (a `data:` URL). */
  readonly src: string;
  /** The image's size in pixels. */
  readonly width: number;
  readonly height: number;
  /** The image's text alternative: what it is, and where the crosshair is. */
  readonly imageLabel: string;
  /** The crosshair's pixel, from 0. */
  readonly x: number;
  readonly y: number;
  /** Moves the crosshair: a slider step, or a click on the image. */
  readonly onMove: (x: number, y: number) => void;
  /** The sliders' names, such as "Crosshair x (pixels)". */
  readonly xLabel: string;
  readonly yLabel: string;
  /** Each slider's `aria-valuetext`: the pixel, and its value once the axis is calibrated. */
  readonly formatX: (x: number) => string;
  readonly formatY: (y: number) => string;
  readonly plotArea?: PixelArea | null;
  readonly marks?: readonly ImageMark[];
}

/**
 * A chart image with a crosshair moved by two sliders (C02-T10; APG Slider, A11Y-26): one
 * for the pixel column, one for the row. Arrows move one pixel, Page Up and Page Down take
 * larger steps, Home and End go to the edges. A click on the image moves the crosshair
 * there, the single-pointer alternative. The image itself is not focusable; its text
 * alternative names it, and the sliders say where the crosshair is and what it reads.
 *
 * Marks tell axes apart by shape and label (squares and "x1", circles and "y1") as well as
 * colour (DL-X08), and every stroke follows the theme's tokens, including high contrast.
 */
export function ImageCrosshair({
  src,
  width,
  height,
  imageLabel,
  x,
  y,
  onMove,
  xLabel,
  yLabel,
  formatX,
  formatY,
  plotArea = null,
  marks = [],
}: ImageCrosshairProps) {
  // A mark's size in image pixels, kept readable on small and large images alike.
  const size = Math.max(3, Math.round(Math.min(width, height) / 60));
  const place = (event: MouseEvent<SVGSVGElement>) => {
    const box = event.currentTarget.getBoundingClientRect();
    if (box.width <= 0 || box.height <= 0) return;
    const px = Math.floor(((event.clientX - box.left) / box.width) * width);
    const py = Math.floor(((event.clientY - box.top) / box.height) * height);
    onMove(clamp(px, width), clamp(py, height));
  };
  return (
    <div className={cx(styles.crosshair)}>
      <svg
        className={cx(styles.image)}
        viewBox={`0 0 ${String(width)} ${String(height)}`}
        role="img"
        aria-label={imageLabel}
        onClick={place}
      >
        <image href={src} width={width} height={height} />
        {plotArea !== null && (
          <rect
            className={cx(styles.plot)}
            x={plotArea.min[0]}
            y={plotArea.min[1]}
            width={plotArea.max[0] - plotArea.min[0]}
            height={plotArea.max[1] - plotArea.min[1]}
          />
        )}
        {marks.map((mark) => {
          const [mx, my] = [mark.at[0] + 0.5, mark.at[1] + 0.5];
          return (
            <g key={mark.id} className={cx(styles.mark)} data-axis={mark.axis}>
              {mark.axis === "x" ? (
                <rect
                  x={mx - size / 2}
                  y={my - size / 2}
                  width={size}
                  height={size}
                />
              ) : (
                <circle cx={mx} cy={my} r={size / 2} />
              )}
              <text x={mx + size} y={my - size} fontSize={size * 2.5}>
                {mark.label}
              </text>
            </g>
          );
        })}
        <line
          className={cx(styles.hair)}
          x1={x + 0.5}
          x2={x + 0.5}
          y1={0}
          y2={height}
        />
        <line
          className={cx(styles.hair)}
          x1={0}
          x2={width}
          y1={y + 0.5}
          y2={y + 0.5}
        />
      </svg>
      <div className={cx(styles.sliders)}>
        <Slider
          label={xLabel}
          formatValue={formatX}
          minValue={0}
          maxValue={Math.max(0, width - 1)}
          step={1}
          value={x}
          onChange={(next) => {
            onMove(next, y);
          }}
        />
        <Slider
          label={yLabel}
          formatValue={formatY}
          minValue={0}
          maxValue={Math.max(0, height - 1)}
          step={1}
          value={y}
          onChange={(next) => {
            onMove(x, next);
          }}
        />
      </div>
    </div>
  );
}

function clamp(value: number, size: number): number {
  return Math.min(Math.max(value, 0), Math.max(0, size - 1));
}

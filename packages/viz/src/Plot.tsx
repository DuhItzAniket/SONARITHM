// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { useEffect, useId, useRef, useState } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";

import styles from "./plot.module.css";
import { dashFor, seriesToken } from "./series.ts";

/**
 * The plot's summary from the NavTree (08 §2, A11Y-01): the L1 sentence names the plot, the
 * L2 paragraph describes it. Until the explorer (B03-T09) passes them in, callers build them
 * from the dataset.
 */
export interface PlotSummary {
  /** L1: what the plot is, in one sentence ("Temperature over 2026, 365 days"). */
  readonly l1: string;
  /** L2: the overview a reader needs first: series, ranges, units, trend. */
  readonly l2: string;
}

/** One line on the plot. */
export interface PlotSeries {
  /** The series' name, shown in the legend. */
  readonly label: string;
  /** The y values, one per x value; `null` for a gap. */
  readonly values: ArrayLike<number | null>;
}

/** Props for {@link Plot}. */
export interface PlotProps {
  readonly summary: PlotSummary;
  /** The x values, ascending. */
  readonly x: ArrayLike<number>;
  /** Up to eight series; more wrap the palette, so use small multiples instead (§6). */
  readonly series: readonly PlotSeries[];
  /** The plot area's height in CSS pixels; the width follows the container. */
  readonly height: number;
  /**
   * The shared cursor's point (B04-T06): a crosshair at `x` and a marker at `y` when it is
   * known. Rust owns the cursor; the plot only shows it.
   */
  readonly cursor?: PlotCursor | null;
  /**
   * Called with the x value under a click (B04-T06): the caller moves the cursor to the
   * nearest row. Pointers only; the table and the chart explorer are the keyboard ways.
   */
  readonly onPick?: (x: number) => void;
}

/** Where the cursor is on the plot, in data units. */
export interface PlotCursor {
  readonly x: number;
  readonly y: number | null;
}

/** Changes whenever the theme can change: the `data-theme` setting or the OS preferences. */
function useThemeVersion(): number {
  const [version, setVersion] = useState(0);
  useEffect(() => {
    const bump = () => {
      setVersion((v) => v + 1);
    };
    const observer = new MutationObserver(bump);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    const queries = [
      "(prefers-color-scheme: dark)",
      "(prefers-contrast: more)",
      "(forced-colors: active)",
    ].map((query) => window.matchMedia(query));
    for (const query of queries) {
      query.addEventListener("change", bump);
    }
    return () => {
      observer.disconnect();
      for (const query of queries) {
        query.removeEventListener("change", bump);
      }
    };
  }, []);
  return version;
}

/**
 * A line or time-series plot on uPlot (Canvas2D; 09 §3). The plot is a named, described
 * object (A11Y-01): an image named by the summary's L1 and described by its L2, whose
 * canvas and legend are presentational. It is not focusable; the chart explorer (B03-T09)
 * is the keyboard surface, and its nodes come from the NavTree, never from pixels (08 §5).
 *
 * Every series has its own colour token (`--series-N`, design-language §6) and its own dash
 * pattern, so colour never carries the series alone (DL-X08). There is no hover readout
 * (DL-X11); the linked cursor arrives with B04-T06. Colours are read from the tokens and
 * the plot is redrawn when the theme changes.
 */
export function Plot({
  summary,
  x,
  series,
  height,
  cursor,
  onPick,
}: PlotProps) {
  const descriptionId = useId();
  const area = useRef<HTMLDivElement>(null);
  const theme = useThemeVersion();
  const plotRef = useRef<uPlot | null>(null);
  const overlay = useRef<{
    line: HTMLDivElement;
    marker: HTMLDivElement;
  } | null>(null);
  // Read by uPlot's hooks and the click listener, so a new cursor or callback does not
  // rebuild the plot.
  const latest = useRef({ cursor, onPick });
  useEffect(() => {
    latest.current = { cursor, onPick };
  });

  /** Puts the crosshair and the marker at the cursor, or hides them. */
  const place = (plot: uPlot) => {
    const parts = overlay.current;
    if (parts === null) return;
    const point = latest.current.cursor;
    if (point === null || point === undefined) {
      parts.line.hidden = true;
      parts.marker.hidden = true;
      return;
    }
    const left = plot.valToPos(point.x, "x");
    const inside = left >= 0 && left <= plot.over.clientWidth;
    parts.line.hidden = !inside;
    parts.line.style.transform = `translateX(${String(left)}px)`;
    parts.line.dataset.cursorX = String(point.x);
    const top = point.y === null ? Number.NaN : plot.valToPos(point.y, "y");
    parts.marker.hidden = !inside || Number.isNaN(top);
    parts.marker.style.transform = `translate(${String(left)}px, ${String(top)}px)`;
  };

  useEffect(() => {
    const element = area.current;
    if (element === null) {
      return;
    }
    const style = getComputedStyle(element);
    const token = (name: string) => style.getPropertyValue(name).trim();
    const rem = Number.parseFloat(
      getComputedStyle(document.documentElement).fontSize,
    );
    // The type-12 step (0.75 rem) in the sans family, so axis text follows text scaling.
    const font = `${String(rem * 0.75)}px ${token("--font-sans")}`;
    const axis = {
      stroke: token("--text-2"),
      grid: { stroke: token("--border"), width: 1 },
      ticks: { stroke: token("--border-strong"), width: 1 },
      font,
    };
    const options: uPlot.Options = {
      width: Math.max(element.clientWidth, 1),
      height,
      legend: { show: false },
      cursor: { show: false },
      scales: { x: { time: false } },
      axes: [axis, axis],
      series: [
        {},
        ...series.map((s, i) => ({
          label: s.label,
          stroke: token(seriesToken(i)),
          width: 2,
          dash: [...dashFor(i)],
          points: { show: false },
        })),
      ],
    };
    const data = [
      Array.from(x),
      ...series.map((s) => Array.from(s.values)),
    ] as uPlot.AlignedData;
    const plot = new uPlot(options, data, element);
    plotRef.current = plot;
    // The cursor's crosshair and marker live in uPlot's overlay, over the plotting area.
    const line = document.createElement("div");
    line.className = styles.crosshair ?? "";
    line.dataset.plotCursor = "";
    const marker = document.createElement("div");
    marker.className = styles.marker ?? "";
    plot.over.append(line, marker);
    overlay.current = { line, marker };
    place(plot);
    const pick = (event: MouseEvent) => {
      const value = plot.posToVal(event.offsetX, "x");
      if (Number.isFinite(value)) latest.current.onPick?.(value);
    };
    plot.over.addEventListener("click", pick);
    const resize = new ResizeObserver(() => {
      plot.setSize({ width: Math.max(element.clientWidth, 1), height });
      place(plot);
    });
    resize.observe(element);
    return () => {
      resize.disconnect();
      plot.over.removeEventListener("click", pick);
      overlay.current = null;
      plotRef.current = null;
      plot.destroy();
    };
  }, [x, series, height, theme]);

  // A cursor move repositions the crosshair without rebuilding or redrawing the plot.
  useEffect(() => {
    if (plotRef.current !== null) place(plotRef.current);
  }, [cursor?.x, cursor?.y]);

  return (
    <div
      role="img"
      aria-label={summary.l1}
      aria-describedby={descriptionId}
      className={styles.plot}
    >
      <div ref={area} className={styles.area} />
      <ul className={styles.legend}>
        {series.map((s, i) => (
          <li key={s.label} className={styles.entry}>
            <svg
              className={styles.swatch}
              viewBox="0 0 24 8"
              width="24"
              height="8"
            >
              <line
                x1="0"
                y1="4"
                x2="24"
                y2="4"
                stroke={`var(${seriesToken(i)})`}
                strokeWidth="2"
                strokeDasharray={dashFor(i).join(" ")}
              />
            </svg>
            {s.label}
          </li>
        ))}
      </ul>
      <p id={descriptionId} hidden>
        {summary.l2}
      </p>
    </div>
  );
}

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
export function Plot({ summary, x, series, height }: PlotProps) {
  const descriptionId = useId();
  const area = useRef<HTMLDivElement>(null);
  const theme = useThemeVersion();

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
    const resize = new ResizeObserver(() => {
      plot.setSize({ width: Math.max(element.clientWidth, 1), height });
    });
    resize.observe(element);
    return () => {
      resize.disconnect();
      plot.destroy();
    };
  }, [x, series, height, theme]);

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

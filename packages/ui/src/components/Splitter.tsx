// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { KeyboardEvent, PointerEvent } from "react";
import { useRef } from "react";
import { useFocusRing, useHover } from "react-aria";

import { cx } from "./cx.ts";
import styles from "./splitter.module.css";

/** Props for {@link Splitter}. */
export interface SplitterProps {
  /** The splitter's accessible name: "Resize Project". */
  readonly label: string;
  /** The ID of the region it resizes (`aria-controls`). */
  readonly controls: string;
  /** The region's width in CSS pixels. */
  readonly value: number;
  readonly min: number;
  readonly max: number;
  /** The width in words, with its unit (`ui-width-pixels`). */
  readonly valueText: string;
  /** Called with the new width. */
  readonly onChange: (value: number) => void;
  /** Enter: collapse the region (the frame keeps it in the F6 cycle). */
  readonly onCollapse: () => void;
  /** Which edge of the region it sits on; ArrowRight widens a region on its left. */
  readonly edge: "end" | "start";
  /** Pixels per arrow press; Shift multiplies by four. */
  readonly step?: number;
}

/**
 * A vertical window splitter (APG Window Splitter; React Aria has none). It is a focusable
 * `separator` with `aria-valuenow` from `min` to `max`: ArrowLeft and ArrowRight resize,
 * Home and End go to the limits, Enter collapses the region. Dragging works too, and the
 * keyboard is the non-drag alternative (A11Y-26).
 */
export function Splitter({
  label,
  controls,
  value,
  min,
  max,
  valueText,
  onChange,
  onCollapse,
  edge,
  step = 8,
}: SplitterProps) {
  const { focusProps, isFocusVisible } = useFocusRing();
  const { hoverProps, isHovered } = useHover({});
  const drag = useRef<{ x: number; value: number } | null>(null);
  const clamp = (v: number) => Math.min(max, Math.max(min, Math.round(v)));
  // A splitter on the region's end edge widens it to the right.
  const sign = edge === "end" ? 1 : -1;

  const onKeyDown = (event: KeyboardEvent) => {
    const amount = event.shiftKey ? step * 4 : step;
    const next = new Map<string, number>([
      ["ArrowRight", value + sign * amount],
      ["ArrowLeft", value - sign * amount],
      ["Home", min],
      ["End", max],
    ]).get(event.key);
    if (next !== undefined) {
      event.preventDefault();
      onChange(clamp(next));
    } else if (event.key === "Enter") {
      event.preventDefault();
      onCollapse();
    }
  };

  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    event.currentTarget.setPointerCapture(event.pointerId);
    drag.current = { x: event.clientX, value };
  };
  const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (drag.current !== null) {
      onChange(
        clamp(drag.current.value + sign * (event.clientX - drag.current.x)),
      );
    }
  };
  const onPointerUp = () => {
    drag.current = null;
  };

  return (
    // A focusable separator with a value is an interactive widget in ARIA 1.2 (the APG
    // window splitter); jsx-a11y treats every separator as static.
    // eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions
    <div
      {...focusProps}
      {...hoverProps}
      role="separator"
      // eslint-disable-next-line jsx-a11y/no-noninteractive-tabindex -- see above
      tabIndex={0}
      aria-label={label}
      aria-controls={controls}
      aria-orientation="vertical"
      aria-valuenow={value}
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuetext={valueText}
      className={cx(styles.splitter)}
      data-focus-visible={isFocusVisible || undefined}
      data-hovered={isHovered || undefined}
      onKeyDown={onKeyDown}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
    />
  );
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { useMemo, useRef } from "react";
import {
  mergeProps,
  useFocusRing,
  useHover,
  useSlider,
  useSliderThumb,
  VisuallyHidden,
} from "react-aria";
import { useSliderState } from "react-stately";

import { cx } from "./cx.ts";
import styles from "./slider.module.css";

/** Props for {@link Slider}. */
export interface SliderProps {
  /** The visible label, which names the slider. */
  readonly label: string;
  /**
   * Writes a value with its unit ("440 Hz", "−12.0 dB"), from SONARITHM's shared formatter
   * (DL-P12). It sets both the visible value and `aria-valuetext`, so speech and display agree.
   */
  readonly formatValue: (value: number) => string;
  readonly minValue: number;
  readonly maxValue: number;
  readonly step?: number;
  readonly value?: number;
  readonly defaultValue?: number;
  readonly onChange?: (value: number) => void;
  readonly onChangeEnd?: (value: number) => void;
  readonly isDisabled?: boolean;
}

/**
 * A slider (APG Slider) whose `aria-valuetext` carries the value and its unit, with the range
 * on `aria-valuemin` and `aria-valuemax`. Arrows step, PageUp and PageDown take larger steps,
 * Home and End go to the limits.
 *
 * Built from React Aria's hooks, not React Aria Components' `Slider`, because that formats
 * values only through `Intl.NumberFormat`, whose units have no hertz, decibels, cents or
 * semitones (F-171). The visible value is not a `SliderOutput`: that is a `status` live region
 * that would speak every step a second time (F-045). It is plain text, hidden from assistive
 * technology because the thumb announces the same value.
 */
export function Slider(props: SliderProps) {
  const { label, formatValue, isDisabled = false } = props;
  const trackRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  // React Stately only calls `format`; the shared formatter stands in for Intl's.
  const numberFormatter = useMemo(
    () => ({ format: formatValue }) as unknown as Intl.NumberFormat,
    [formatValue],
  );
  const state = useSliderState({
    ...stateProps(props),
    label,
    numberFormatter,
  });
  const { groupProps, trackProps, labelProps } = useSlider(
    { ...stateProps(props), label },
    state,
    trackRef,
  );
  const { thumbProps, inputProps, isDragging } = useSliderThumb(
    { index: 0, trackRef, inputRef, isDisabled },
    state,
  );
  const { focusProps, isFocusVisible } = useFocusRing();
  const { hoverProps, isHovered } = useHover({ isDisabled });
  return (
    <div
      {...groupProps}
      className={cx(styles.slider)}
      data-disabled={isDisabled || undefined}
    >
      <div className={cx(styles.header)}>
        <label {...labelProps} className={cx(styles.label)}>
          {label}
        </label>
        <span className={cx(styles.value)} aria-hidden="true">
          {state.getThumbValueLabel(0)}
        </span>
      </div>
      <div {...trackProps} ref={trackRef} className={cx(styles.track)}>
        <div
          {...mergeProps(thumbProps, hoverProps)}
          className={cx(styles.thumb)}
          // The thumb's position along the track is layout, not a token.
          style={{
            insetInlineStart: `${String(state.getThumbPercent(0) * 100)}%`,
          }}
          data-focus-visible={isFocusVisible || undefined}
          data-hovered={isHovered || undefined}
          data-dragging={isDragging || undefined}
        >
          <VisuallyHidden>
            <input ref={inputRef} {...mergeProps(inputProps, focusProps)} />
          </VisuallyHidden>
        </div>
      </div>
    </div>
  );
}

/** The props React Stately's slider state takes, without `undefined` values. */
function stateProps(props: SliderProps) {
  return {
    minValue: props.minValue,
    maxValue: props.maxValue,
    isDisabled: props.isDisabled ?? false,
    ...(props.step === undefined ? {} : { step: props.step }),
    ...(props.value === undefined ? {} : { value: props.value }),
    ...(props.defaultValue === undefined
      ? {}
      : { defaultValue: props.defaultValue }),
    ...(props.onChange === undefined
      ? {}
      : {
          onChange: (v: number | number[]) => {
            props.onChange?.(Array.isArray(v) ? (v[0] ?? 0) : v);
          },
        }),
    ...(props.onChangeEnd === undefined
      ? {}
      : {
          onChangeEnd: (v: number | number[]) => {
            props.onChangeEnd?.(Array.isArray(v) ? (v[0] ?? 0) : v);
          },
        }),
  };
}

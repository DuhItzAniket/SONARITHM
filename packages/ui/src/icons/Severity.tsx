// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/** How serious a diagnostic is. */
export type Severity = "error" | "warning" | "info";

/** The 16 px paths: a circle with a cross, a triangle with a bar, a square with an i. */
const PATHS: Record<Severity, string> = {
  error:
    "M8 1.75a6.25 6.25 0 100 12.5 6.25 6.25 0 000-12.5zM5.75 5.75l4.5 4.5M10.25 5.75l-4.5 4.5",
  warning: "M8 2L1.75 13.25h12.5zM8 6.25v3.5M8 11.5v.25",
  info: "M2.25 2.25h11.5v11.5H2.25zM8 7.25v4.25M8 4.75v.25",
};

/**
 * A severity's shape, distinct for each, so the severity never rests on colour (DL-X08).
 * Decorative: it always stands next to the severity in words, so it is hidden from
 * assistive technology (design-language §10).
 */
export function SeverityIcon({
  severity,
  className,
}: {
  readonly severity: Severity;
  readonly className?: string;
}) {
  return (
    <svg
      className={className}
      width="16"
      height="16"
      viewBox="0 0 16 16"
      aria-hidden="true"
      focusable="false"
      data-severity={severity}
    >
      <path
        d={PATHS[severity]}
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Type test (B02-T04, DL-X05): an IconButton without a name does not type-check. `pnpm
// typecheck` fails if any `@ts-expect-error` below stops being an error.
import { IconButton } from "../../src/components/Button.tsx";

const name = "Play";

export const named = <IconButton label={name} icon={null} />;

// @ts-expect-error: an icon-only button needs `label`, its accessible name.
export const unnamed = <IconButton icon={null} />;

// @ts-expect-error: `aria-label` cannot stand in for `label` (the tooltip needs it too).
export const ariaLabelled = <IconButton aria-label={name} icon={null} />;

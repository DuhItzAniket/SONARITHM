// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Chord, Platform } from "./chords.ts";
import { normalize } from "./chords.ts";
import type { CommandSpec } from "./keymap.ts";
import { REGION_COMMANDS } from "./keymap.ts";
import type { BindingProblem } from "./lint.ts";
import { bindingProblems } from "./lint.ts";

/** What the recorder makes of a recorded chord for a command (keyboard §13 rule 5). */
export type Recorded =
  | {
      readonly ok: true;
      readonly chord: Chord;
      readonly shadows: readonly string[];
    }
  | {
      readonly ok: false;
      readonly chord: Chord;
      readonly problems: readonly BindingProblem[];
    };

/**
 * Checks a recorded chord for `spec` (B03-T09; keyboard §13 rules 5 and 6): reserved
 * combinations and screen-reader or layout conflicts are refused, with their reason; an
 * allowed chord lists the commands in the same scope that already use it, which the
 * keymap editor names before saving. The app core refuses reserved chords again on save
 * (`SNR-APP-091`).
 */
export function record(
  chord: Chord,
  spec: CommandSpec,
  platform: Platform,
  bindingsOf: (id: string) => readonly Chord[],
  specs: readonly CommandSpec[],
): Recorded {
  const normal = normalize(chord);
  const problems = bindingProblems(
    normal,
    spec.scope,
    platform,
    REGION_COMMANDS.has(spec.id),
  );
  if (problems.length > 0) {
    return { ok: false, chord: normal, problems };
  }
  const shadows = specs
    .filter(
      (other) =>
        other.id !== spec.id &&
        other.scope === spec.scope &&
        bindingsOf(other.id).some((b) => normalize(b) === normal),
    )
    .map((other) => other.id);
  return { ok: true, chord: normal, shadows };
}

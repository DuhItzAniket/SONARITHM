# 2026-11-01: B03-T10, the braille line in the desktop

> **Date(s):** 2026-11-01. **Phase:** B03. **Author:** Lane U, written by an AI agent (Claude Code), reviewed by the owner.
> **Related:** [worklog conventions](README.md) · [ADR-0017 "Braille"](../../decisions/0017-ipc-protocol-v1.md#braille) · keyboard-and-navigation §10, §14

## Context and phase

- PLOTSONA built the Rust side: AUDAXIS's review line and sparkline, and `CursorState.braille`, `braille_route` and `settings.braille` (`b03-t10-app`). Lane U's branch `feat/b03-t10-braille-field` is the explorer stack with that merged in.

## What was done

- `BrailleField`: a read-only text field showing `BrailleState.text`, named by its mode and described by what it holds, with the caret placed on `caret`. It never takes the focus by itself.
- **Routing.** A screen reader moves the caret on a routing key through the accessibility API, so no key or mouse event arrives and React's `onSelect` misses it. The field listens for `selectionchange` at the field and the document, and sends a caret that it did not place itself as `braille_route`. In the E2E, Home in a read-only field left the caret where it was in Chromium; the test sets the selection as a routing key does.
- `mod.brailleMode` (Shift+B) beside `mod.braille` (B): it saves `settings.braille.mode` and says the new mode.
- Preferences › Braille: the cell count, empty by default, 1 to 160.
- The mock host builds a fake line while braille is on and answers `braille_route`.
- Tests: `e2e/braille.spec.ts` (on and following the cursor, the sparkline and its caret, a silent routing move, the cell count) in Chromium and WebKit, and 49 E2E across the affected specs in each.

## Next steps

- The PR opens after #447, after the explorer, PLOTSONA's braille, point cue, query and remap PRs.
- B07: the walkthrough with a braille display (HP-10), per screen reader [CHECK].

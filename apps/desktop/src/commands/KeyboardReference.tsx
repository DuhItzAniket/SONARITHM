// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { Button, DialogActions, ModalDialog } from "@sonarithm/ui";

import { useLocalizer } from "../i18n.ts";
import { displayChord } from "./chords.ts";
import { useAvailableCommands, useCommands } from "./context.tsx";
import styles from "./commands.module.css";
import type { Scope } from "./scopes.ts";

/** Props for {@link KeyboardReference}. */
export interface KeyboardReferenceProps {
  /** The scopes that were active where Shift+F1 was pressed; `null` when closed. */
  readonly scopes: readonly Scope[] | null;
  readonly onClose: () => void;
}

/**
 * Shift+F1: the commands of the focused scope with their current bindings (KB-09). Only
 * scopes active where it was opened are listed, highest precedence first, so it answers
 * "what do the keys do here".
 */
export function KeyboardReference({ scopes, onClose }: KeyboardReferenceProps) {
  const t = useLocalizer();
  const registry = useCommands();
  const commands = useAvailableCommands();
  const here = (scopes ?? []).flatMap((scope) =>
    commands.filter(({ spec }) => spec.scope === scope),
  );
  return (
    <ModalDialog
      title={t.format("keyboard-reference-title")}
      isOpen={scopes !== null}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      {(close) => (
        <>
          <dl className={styles.reference}>
            {here.map(({ spec, bindings }) => (
              <div key={spec.id} className={styles.row}>
                <dt>{t.format(spec.name)}</dt>
                <dd className={styles.binding}>
                  {bindings.length === 0
                    ? t.format("keyboard-no-binding")
                    : bindings
                        .map((chord) => displayChord(chord, registry.platform))
                        .join(", ")}
                </dd>
              </div>
            ))}
          </dl>
          <DialogActions>
            <Button onPress={close}>{t.format("ui-close-window")}</Button>
          </DialogActions>
        </>
      )}
    </ModalDialog>
  );
}

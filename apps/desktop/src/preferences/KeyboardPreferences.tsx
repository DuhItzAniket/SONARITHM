// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Settings } from "@sonarithm/ipc";
import { Button, DialogActions, ModalDialog, Switch } from "@sonarithm/ui";
import { useState } from "react";

import type { Chord } from "../commands/chords.ts";
import { chordOf, displayChord } from "../commands/chords.ts";
import { useAvailableCommands, useCommands } from "../commands/context.tsx";
import type { CommandSpec } from "../commands/keymap.ts";
import { DEFAULT_KEYMAP } from "../commands/keymap.ts";
import type { Recorded } from "../commands/recorder.ts";
import { record } from "../commands/recorder.ts";
import { useLocalizer } from "../i18n.ts";
import styles from "./preferences.module.css";

/**
 * Preferences › Keyboard (B03-T09; keyboard-and-navigation §13): the group switch for the
 * explorer's single keys, the two presets, and each command's keys with a recorder. Keys
 * are saved in the user's settings (`settings_set`), never in a project; the app core
 * refuses reserved combinations again (`SNR-APP-091`). `onChange` saves, and the app then
 * applies the keymap to the command registry.
 */
export function KeyboardPreferences({
  settings,
  onChange,
}: {
  readonly settings: Settings | null;
  readonly onChange: (next: Settings) => void;
}) {
  const t = useLocalizer();
  const registry = useCommands();
  // Re-render when the registry's keys change (after a save).
  useAvailableCommands();
  const [editing, setEditing] = useState<CommandSpec | null>(null);
  const keymap = settings?.keymap ?? {};
  const shown = (chords: readonly Chord[]) =>
    chords.length === 0
      ? t.format("keyboard-no-binding")
      : chords.map((c) => displayChord(c, registry.platform)).join(", ");
  const save = (change: (current: Settings) => Settings) => {
    if (settings !== null) onChange(change(settings));
  };
  const setKeys = (id: string, keys: string | null) => {
    save((current) => {
      const next = { ...current.keymap };
      if (keys === null) {
        delete next[id];
      } else {
        next[id] = keys;
      }
      return { ...current, keymap: next };
    });
  };
  return (
    <section aria-labelledby="preferences-keyboard" className={styles.section}>
      <h3 id="preferences-keyboard" className={styles.heading}>
        {t.format("ui-keyboard")}
      </h3>
      <Switch
        isSelected={settings?.keyboard.singleKeyCommands ?? true}
        isDisabled={settings === null}
        description={t.format("ui-single-keys-note")}
        onChange={(on) => {
          save((current) => ({
            ...current,
            keyboard: { ...current.keyboard, singleKeyCommands: on },
          }));
        }}
      >
        {t.format("ui-single-keys")}
      </Switch>
      <div className={styles.row}>
        <Button
          isDisabled={settings === null}
          onPress={() => {
            save((current) => ({
              ...current,
              keymap: {},
              keyboard: { ...current.keyboard, singleKeyCommands: true },
            }));
          }}
        >
          {t.format("ui-keys-preset-default")}
        </Button>
        <Button
          isDisabled={settings === null}
          onPress={() => {
            save((current) => ({
              ...current,
              keyboard: { ...current.keyboard, singleKeyCommands: false },
            }));
          }}
        >
          {t.format("ui-keys-preset-no-single")}
        </Button>
      </div>
      <table className={styles.keys}>
        <caption className={styles.note}>{t.format("ui-keys-caption")}</caption>
        <thead>
          <tr>
            <th scope="col">{t.format("ui-keys-command")}</th>
            <th scope="col">{t.format("ui-keys-keys")}</th>
            <th scope="col">{t.format("ui-keys-actions")}</th>
          </tr>
        </thead>
        <tbody>
          {DEFAULT_KEYMAP.map((spec) => {
            const name = t.format(spec.name);
            return (
              <tr key={spec.id}>
                <th scope="row">{name}</th>
                <td>{shown(registry.bindings(spec.id))}</td>
                <td>
                  <Button
                    isDisabled={settings === null}
                    aria-label={t.format("ui-keys-change-for", { name })}
                    onPress={() => {
                      setEditing(spec);
                    }}
                  >
                    {t.format("ui-keys-change")}
                  </Button>
                  {spec.id in keymap && (
                    <Button
                      aria-label={t.format("ui-keys-reset-for", { name })}
                      onPress={() => {
                        setKeys(spec.id, null);
                      }}
                    >
                      {t.format("ui-keys-reset")}
                    </Button>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <KeyRecorder
        spec={editing}
        onClose={() => {
          setEditing(null);
        }}
        onSave={(spec, chord) => {
          setKeys(spec.id, chord);
        }}
      />
    </section>
  );
}

/**
 * The recorder (keyboard §13 rule 5): records a combination on key-down and shows what it
 * means before saving. Reserved combinations and screen-reader conflicts are refused with
 * their reason; a combination another command in the same scope uses is named. Tab still
 * moves the focus (it is reserved), and Escape closes the dialog.
 */
function KeyRecorder({
  spec,
  onClose,
  onSave,
}: {
  readonly spec: CommandSpec | null;
  readonly onClose: () => void;
  readonly onSave: (spec: CommandSpec, chord: Chord) => void;
}) {
  const t = useLocalizer();
  const registry = useCommands();
  const [recorded, setRecorded] = useState<Recorded | null>(null);
  const name = spec === null ? "" : t.format(spec.name);
  const verdict = (() => {
    if (recorded === null) return t.format("ui-keys-press");
    const keys = displayChord(recorded.chord, registry.platform);
    if (!recorded.ok) {
      return t.format("ui-keys-refused", {
        keys,
        rule: recorded.problems[0]?.rule ?? "reserved",
      });
    }
    if (recorded.shadows.length > 0) {
      const other = DEFAULT_KEYMAP.find((s) => s.id === recorded.shadows[0]);
      return t.format("ui-keys-shadows", {
        keys,
        other: other === undefined ? "" : t.format(other.name),
      });
    }
    return t.format("ui-keys-free", { keys });
  })();
  return (
    <ModalDialog
      title={t.format("ui-keys-change-for", { name })}
      isOpen={spec !== null}
      onOpenChange={(open) => {
        if (!open) {
          setRecorded(null);
          onClose();
        }
      }}
    >
      {(close) => (
        <>
          <input
            className={styles.recorder}
            readOnly
            aria-label={t.format("ui-keys-recorder")}
            aria-describedby="key-recorder-verdict"
            value={
              recorded === null
                ? ""
                : displayChord(recorded.chord, registry.platform)
            }
            onKeyDown={(event) => {
              if (spec === null) return;
              // Tab moves on and Escape closes: both are reserved (§13 rule 6).
              if (
                (event.key === "Tab" && !event.ctrlKey && !event.metaKey) ||
                (event.key === "Escape" && !event.shiftKey)
              ) {
                return;
              }
              const chord = chordOf(event);
              if (chord === null) return;
              event.preventDefault();
              event.stopPropagation();
              setRecorded(
                record(
                  chord,
                  spec,
                  registry.platform,
                  (id) => registry.bindings(id),
                  DEFAULT_KEYMAP,
                ),
              );
            }}
          />
          <p
            id="key-recorder-verdict"
            className={styles.note}
            aria-live="polite"
          >
            {verdict}
          </p>
          <DialogActions>
            <Button onPress={close}>{t.format("ui-cancel")}</Button>
            <Button
              isDisabled={recorded === null || !recorded.ok}
              onPress={() => {
                if (spec !== null && recorded?.ok === true) {
                  onSave(spec, recorded.chord);
                  close();
                }
              }}
            >
              {t.format("ui-keys-save")}
            </Button>
          </DialogActions>
        </>
      )}
    </ModalDialog>
  );
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { ModalDialog } from "@sonarithm/ui";
import { useEffect, useState } from "react";
// The palette composes React Aria's Autocomplete, SearchField and Menu directly: no
// @sonarithm/ui component wraps that pattern yet, and the palette is its only user
// (design-language §11, "Command palette").
import {
  Autocomplete,
  Input,
  Keyboard,
  Menu,
  MenuItem,
  SearchField,
  useFilter,
} from "react-aria-components";

import type { Announcer } from "../announcer/announcer.ts";
import { useLocalizer } from "../i18n.ts";
import { displayChord } from "./chords.ts";
import { useAvailableCommands, useCommands } from "./context.tsx";
import styles from "./commands.module.css";

/** Props for {@link CommandPalette}. */
export interface CommandPaletteProps {
  readonly isOpen: boolean;
  readonly onOpenChange: (isOpen: boolean) => void;
  readonly announcer: Announcer;
}

/**
 * The command palette (B02-T07, KB-01): every registered command with its current binding.
 * Typing filters the list while focus stays in the search field; arrows move through the
 * results (`aria-activedescendant`), Enter runs one. Each change of the result count is
 * announced ("3 commands"), so a screen-reader user knows what the filter left.
 */
export function CommandPalette({
  isOpen,
  onOpenChange,
  announcer,
}: CommandPaletteProps) {
  const t = useLocalizer();
  const registry = useCommands();
  const commands = useAvailableCommands();
  const { contains } = useFilter({ sensitivity: "base" });
  const [query, setQuery] = useState("");
  // Autocomplete filters the menu itself (it keeps the active result in step); the same
  // test counts the matches for the announcement.
  const count = commands.filter(({ spec }) =>
    contains(t.format(spec.name), query),
  ).length;

  // Announce the count when the palette opens and whenever the filter changes it.
  useEffect(() => {
    if (isOpen) {
      announcer.announce({
        text: t.format("command-results", { count: { number: count } }),
        assertive: false,
      });
    }
  }, [isOpen, count, announcer, t]);

  const title = t.format("ui-command-palette");
  return (
    <ModalDialog
      title={title}
      isOpen={isOpen}
      onOpenChange={(open) => {
        setQuery("");
        onOpenChange(open);
      }}
    >
      {(close) => (
        <Autocomplete
          inputValue={query}
          onInputChange={setQuery}
          filter={contains}
        >
          {/* The palette exists to be typed into; focus starts in its search field
              (APG combobox in a dialog). */}
          <SearchField
            aria-label={title}
            // eslint-disable-next-line jsx-a11y/no-autofocus -- see above
            autoFocus
            className={styles.search ?? ""}
          >
            <Input className={styles.input ?? ""} />
          </SearchField>
          <Menu
            aria-label={title}
            items={commands}
            className={styles.results ?? ""}
            renderEmptyState={() => (
              <p className={styles.empty ?? ""}>
                {t.format("command-no-results")}
              </p>
            )}
            onAction={(id) => {
              close();
              setQuery("");
              // Run after the dialog has returned focus, so the command decides where
              // focus goes next.
              setTimeout(() => registry.run(String(id)), 0);
            }}
          >
            {({ spec, bindings }) => {
              const name = t.format(spec.name);
              const binding = bindings[0];
              return (
                <MenuItem
                  id={spec.id}
                  textValue={name}
                  className={styles.item ?? ""}
                >
                  {name}
                  {binding !== undefined && (
                    // The binding is the item's description (aria-describedby). It is
                    // hidden from the name, which comes from the content, so the name is
                    // the command's name alone (WCAG 2.5.3).
                    <span aria-hidden="true">
                      <Keyboard className={styles.binding ?? ""}>
                        {displayChord(binding, registry.platform)}
                      </Keyboard>
                    </span>
                  )}
                </MenuItem>
              );
            }}
          </Menu>
        </Autocomplete>
      )}
    </ModalDialog>
  );
}

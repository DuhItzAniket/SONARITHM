// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Meta, StoryObj } from "@storybook/react-vite";
import { useEffect, useRef, useState } from "react";
import { expect, userEvent, waitFor, within } from "storybook/test";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Button } from "../components/Button.tsx";
import type { CompletionContext, CompletionResult } from "./index.ts";
import type { CodeEditorHandle, EditorDiagnostic } from "./CodeEditor.tsx";
import { CodeEditor } from "./CodeEditor.tsx";

const t = storyLocalizer();

const SPEC = `{
  "specVersion": "0.4",
  "mode": "analysis",
  "tracks": [
    { "id": "temperature", "mark": "tone", "channels": { "pitch": { "field": "tempx" } } }
  ]
}`;

const at = (text: string) => SPEC.indexOf(text);

/** One diagnostic of each severity, as the app places them from JSON pointers. */
const DIAGNOSTICS: readonly EditorDiagnostic[] = [
  {
    from: at('"tempx"'),
    to: at('"tempx"') + 7,
    severity: "error",
    message: `${t.format("ui-severity-error")}: SNR-SPEC-070`,
  },
  {
    from: at('"analysis"'),
    to: at('"analysis"') + 10,
    severity: "warning",
    message: `${t.format("ui-severity-warning")}: SNR-SPEC-006`,
  },
  {
    from: at('"0.4"'),
    to: at('"0.4"') + 5,
    severity: "info",
    message: `${t.format("ui-severity-info")}: SNR-SPEC-006`,
  },
];

/** Two property names, offered inside an object. */
function complete(context: CompletionContext): CompletionResult | null {
  const word = context.matchBefore(/"?\w*/);
  if (word === null || (word.from === word.to && !context.explicit)) {
    return null;
  }
  return {
    from: word.from,
    options: [
      { label: '"mode"', type: "property" },
      { label: '"name"', type: "property" },
    ],
  };
}

/**
 * The editor as the Spec tab uses it, with a button after it so Tab has somewhere to go.
 * Primary+I opens completion here, as the app's dispatcher does (keyboard §5.9).
 */
function SpecEditor({
  withDiagnostics,
}: {
  readonly withDiagnostics: boolean;
}) {
  const handle = useRef<CodeEditorHandle>(null);
  const [text, setText] = useState(SPEC);
  const [steps, setSteps] = useState<readonly string[]>([]);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key === "i") {
        event.preventDefault();
        handle.current?.complete();
      }
      if ((event.ctrlKey || event.metaKey) && event.key === "f") {
        event.preventDefault();
        handle.current?.openSearch();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, []);
  return (
    <div>
      <p id="summary">
        {t.format("ui-spec-summary", {
          errors: { number: withDiagnostics ? 1 : 0 },
          warnings: { number: withDiagnostics ? 1 : 0 },
          notes: { number: withDiagnostics ? 1 : 0 },
        })}
      </p>
      <CodeEditor
        label={t.format("ui-spec-editor")}
        descriptionId="summary"
        value={text}
        onChange={setText}
        diagnostics={withDiagnostics ? DIAGNOSTICS : []}
        completion={complete}
        handle={handle}
        onTextHistory={(step) => {
          setSteps((done) => [
            ...done,
            t.format(step === "undo" ? "ui-undo-text" : "ui-redo-text"),
          ]);
        }}
      />
      <output data-testid="history">{steps.join(" ")}</output>
      <Button>{t.format("ui-spec-load-current")}</Button>
    </div>
  );
}

const meta = {
  title: "Components/CodeEditor",
  component: SpecEditor,
  args: { withDiagnostics: true },
} satisfies Meta<typeof SpecEditor>;

export default meta;

const textbox = (canvasElement: HTMLElement) =>
  within(canvasElement).getByRole("textbox", {
    name: t.format("ui-spec-editor"),
  });

/**
 * One Tab stop with a name and a description; Tab leaves it (keyboard §5.9). Each severity
 * has its own underline and gutter mark, so none rests on colour (DL-X08).
 */
export const Diagnostics: StoryObj<typeof meta> = {
  play: async ({ canvasElement }) => {
    const editor = textbox(canvasElement);
    await expect(editor).toHaveAttribute("aria-multiline", "true");
    await expect(editor).toHaveAccessibleDescription(
      t.format("ui-spec-summary", {
        errors: { number: 1 },
        warnings: { number: 1 },
        notes: { number: 1 },
      }),
    );
    await waitFor(() =>
      expect(
        canvasElement.querySelectorAll(".cm-lintRange-error"),
      ).toHaveLength(1),
    );
    for (const severity of ["error", "warning", "info"]) {
      await expect(
        canvasElement.querySelector(`.cm-lintRange-${severity}`),
      ).not.toBeNull();
      await expect(
        canvasElement.querySelector(`.cm-lint-marker-${severity}`),
      ).not.toBeNull();
    }
    const error = canvasElement.querySelector(".cm-lintRange-error");
    if (error === null) throw new Error("no error mark");
    await expect(getComputedStyle(error).textDecorationStyle).toBe("solid");
    const warning = canvasElement.querySelector(".cm-lintRange-warning");
    if (warning === null) throw new Error("no warning mark");
    await expect(getComputedStyle(warning).textDecorationStyle).toBe("dashed");

    await userEvent.tab();
    await expect(editor).toHaveFocus();
    await userEvent.tab();
    await expect(
      within(canvasElement).getByRole("button", {
        name: t.format("ui-spec-load-current"),
      }),
    ).toHaveFocus();
    await userEvent.tab({ shift: true });
    await expect(editor).toHaveFocus();
  },
};

/** Primary+Z undoes typing and says so; with nothing typed, the key is left to the app. */
export const TextUndo: StoryObj<typeof meta> = {
  args: { withDiagnostics: false },
  play: async ({ canvasElement }) => {
    const editor = textbox(canvasElement);
    await userEvent.click(editor);
    await userEvent.keyboard("{Control>}{End}{/Control}");
    await userEvent.keyboard("x");
    await expect(editor.textContent).toContain("}x");
    let reachedPage = false;
    const listen = (event: KeyboardEvent) => {
      if (event.key === "z") reachedPage = true;
    };
    window.addEventListener("keydown", listen);
    await userEvent.keyboard("{Control>}z{/Control}");
    await expect(editor.textContent).not.toContain("}x");
    await expect(
      within(canvasElement).getByTestId("history"),
    ).toHaveTextContent(t.format("ui-undo-text"));
    // The editor handled that key, so the page's dispatcher never saw it.
    await expect(reachedPage).toBe(false);
    // Nothing left to undo: the key goes on to the app (its spec history).
    await userEvent.keyboard("{Control>}z{/Control}");
    window.removeEventListener("keydown", listen);
    await expect(reachedPage).toBe(true);
  },
};

/** Completion opens only on request (Primary+I here), never on its own while typing. */
export const Completion: StoryObj<typeof meta> = {
  args: { withDiagnostics: false },
  play: async ({ canvasElement }) => {
    const editor = textbox(canvasElement);
    await userEvent.click(editor);
    await userEvent.keyboard("{Control>}{Home}{/Control}{ArrowRight}");
    await userEvent.keyboard("{Enter}");
    await userEvent.keyboard('"n');
    // The typed text is in the editor before completion is asked for. On a cold first run
    // in CI the request once came before it, and the list held both names (#505).
    await waitFor(() => expect(editor).toHaveTextContent('"n'));
    // Typing alone opens nothing (INFER, OD-30).
    await expect(
      canvasElement.querySelector(".cm-tooltip-autocomplete"),
    ).toBeNull();
    await userEvent.keyboard("{Control>}i{/Control}");
    // CodeMirror draws its tooltips inside the editor, so the story's canvas holds the one list.
    const list = await within(canvasElement).findByRole("listbox");
    await expect(editor).toHaveAttribute("aria-autocomplete", "list");
    await expect(editor).toHaveAttribute("aria-activedescendant");
    // Filtered by what was typed: only "name" starts with n.
    await waitFor(
      () => expect(within(list).getAllByRole("option")).toHaveLength(1),
      { timeout: 3000 },
    );
    await expect(within(list).getByRole("option")).toHaveTextContent('"name"');
    await userEvent.keyboard("{Escape}");
    await expect(
      canvasElement.querySelector(".cm-tooltip-autocomplete"),
    ).toBeNull();
    await expect(editor).toHaveFocus();
  },
};

/**
 * The search panel (Primary+F in the app, opened here through the handle): F3 or Enter finds
 * the next match, Escape closes the panel from its field and returns to the text with the match
 * selected, so typing replaces it.
 */
export const Search: StoryObj<typeof meta> = {
  args: { withDiagnostics: false },
  play: async ({ canvasElement }) => {
    const editor = textbox(canvasElement);
    await userEvent.click(editor);
    await userEvent.keyboard("{Control>}f{/Control}");
    const field = await within(canvasElement).findByRole("textbox", {
      name: "Find",
    });
    await expect(field).toHaveFocus();
    // F3 finds the next match (Enter does too in a browser; the testing library's key
    // events carry no keyCode, which CodeMirror's search field reads for Enter).
    await userEvent.keyboard("tone{F3}");
    await expect(
      canvasElement.querySelector(".cm-searchMatch-selected")?.textContent,
    ).toBe("tone");
    await userEvent.keyboard("{Escape}");
    await expect(canvasElement.querySelector(".cm-search")).toBeNull();
    await expect(editor).toHaveFocus();
    await userEvent.keyboard("glide");
    await expect(editor.textContent).toContain('"mark": "glide"');
  },
};

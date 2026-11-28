// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  CompletionSource,
  CompletionContext,
} from "@codemirror/autocomplete";
import {
  autocompletion,
  closeBrackets,
  closeBracketsKeymap,
  completionKeymap,
  startCompletion,
} from "@codemirror/autocomplete";
import {
  defaultKeymap,
  history,
  redo,
  redoDepth,
  undo,
  undoDepth,
} from "@codemirror/commands";
import { json } from "@codemirror/lang-json";
import {
  bracketMatching,
  HighlightStyle,
  indentOnInput,
  syntaxHighlighting,
} from "@codemirror/language";
import type { Diagnostic } from "@codemirror/lint";
import { lintGutter, setDiagnostics } from "@codemirror/lint";
import {
  closeSearchPanel,
  findNext,
  findPrevious,
  openSearchPanel,
  search,
} from "@codemirror/search";
import type { Extension } from "@codemirror/state";
import { Compartment, EditorSelection, EditorState } from "@codemirror/state";
import type { KeyBinding } from "@codemirror/view";
import {
  drawSelection,
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
} from "@codemirror/view";
import { tags } from "@lezer/highlight";
import type { Ref } from "react";
import { useEffect, useImperativeHandle, useRef } from "react";

import { cx } from "../components/cx.ts";
import styles from "./code-editor.module.css";

/** How serious a diagnostic is. */
export type EditorSeverity = "error" | "warning" | "info";

/** A diagnostic placed in the text: offsets into the document, from and to. */
export interface EditorDiagnostic {
  readonly from: number;
  readonly to: number;
  readonly severity: EditorSeverity;
  /** The whole message, with its severity and code in words (shown on hover). */
  readonly message: string;
}

/** What the editor's owner can do with it, through `handle`. */
export interface CodeEditorHandle {
  /** Moves the focus into the editor. */
  focus(): void;
  /** Selects `from` to `to` (a caret when they are equal), scrolls there and focuses. */
  select(from: number, to?: number): void;
  /** The text as typed now. */
  text(): string;
  /** The caret's offset. */
  caret(): number;
  /** Opens the completion list at the caret; returns whether it could. */
  complete(): boolean;
  /** Opens the search panel. */
  openSearch(): void;
  /** Replaces the whole text as one edit that Primary+Z undoes (Format). */
  replaceAll(text: string): void;
  /** Forgets the text undo history, so Undo starts from here (after Apply). */
  resetHistory(): void;
}

/** Props of {@link CodeEditor}. */
export interface CodeEditorProps {
  /** The accessible name, such as "Spec (JSON)". */
  readonly label: string;
  /** The ID of an element that describes the editor (a diagnostics summary). */
  readonly descriptionId?: string;
  /**
   * The text. When it differs from what the editor holds, the editor shows it and forgets
   * its undo history; pass back what `onChange` gave to keep typing undisturbed.
   */
  readonly value: string;
  /** Called with the new text on every edit. */
  readonly onChange?: (text: string) => void;
  /** The diagnostics to mark. Never colour alone: each severity has its own line and mark. */
  readonly diagnostics?: readonly EditorDiagnostic[];
  /** Completions at the caret (for example from a JSON Schema). */
  readonly completion?: CompletionSource;
  /**
   * Whether the completion list opens on its own while typing. Off by default: a popup
   * that appears while typing interrupts a screen reader's speech (INFER, OD-30), so
   * completion opens on request through {@link CodeEditorHandle.complete}.
   */
  readonly autoComplete?: boolean;
  /** The page's CSP nonce for the editor's own `<style>` element (09 §4). */
  readonly nonce?: string;
  /** Called after Primary+Z or Primary+Y undid or redid a text edit. */
  readonly onTextHistory?: (step: "undo" | "redo") => void;
  /** Receives the {@link CodeEditorHandle}. */
  readonly handle?: Ref<CodeEditorHandle | null>;
  readonly className?: string;
}

/**
 * CodeMirror keys the app owns instead, so its one dispatcher (keyboard-and-navigation §2)
 * runs them and the palette lists them (KB-01): Primary+Enter applies, Primary+I completes
 * and Ctrl+Space is never bound (KB-04); Primary+Shift+Backslash is the region toggle (§5.2).
 * Ctrl+Alt+letter and Ctrl+Alt+Backslash are never bound (KB-04).
 */
const APP_KEYS = new Set([
  "Mod-Enter",
  "Mod-i",
  "Mod-Shift-\\",
  "Mod-Alt-\\",
  "Ctrl-Space",
  "Alt-`",
  "Mod-z",
  "Mod-y",
  "Mod-Shift-z",
]);

function withoutAppKeys(bindings: readonly KeyBinding[]): KeyBinding[] {
  return bindings.filter(
    (binding) =>
      !APP_KEYS.has(binding.key ?? "") && !APP_KEYS.has(binding.mac ?? ""),
  );
}

/** Text undo and redo, handled only when there is a step to take (see below). */
function historyKeys(
  onStep: (step: "undo" | "redo") => void,
): readonly KeyBinding[] {
  const step =
    (kind: "undo" | "redo") =>
    (view: EditorView): boolean => {
      const depth =
        kind === "undo" ? undoDepth(view.state) : redoDepth(view.state);
      if (depth === 0) {
        // Not handled: the key reaches the app, whose Undo steps the spec history.
        return false;
      }
      if (kind === "undo") undo(view);
      else redo(view);
      onStep(kind);
      return true;
    };
  return [
    { key: "Mod-z", run: step("undo") },
    { key: "Mod-y", mac: "Mod-Shift-z", run: step("redo") },
    { key: "Mod-Shift-z", run: step("redo") },
  ];
}

/** Syntax colours by class, so the stylesheet can use the design tokens. */
const highlight = HighlightStyle.define([
  { tag: tags.propertyName, class: cx(styles.key) },
  { tag: tags.string, class: cx(styles.string) },
  { tag: tags.number, class: cx(styles.number) },
  { tag: [tags.bool, tags.null], class: cx(styles.atom) },
]);

function toLint(
  diagnostics: readonly EditorDiagnostic[],
  length: number,
): Diagnostic[] {
  return diagnostics.map((d) => {
    const from = Math.min(Math.max(d.from, 0), length);
    const to = Math.min(Math.max(d.to, from), length);
    return { from, to, severity: d.severity, message: d.message };
  });
}

/**
 * A CodeMirror 6 text editor (B05-T03; 09 §3, keyboard-and-navigation §5.9). One Tab stop:
 * Tab and Shift+Tab leave it, since Tab is not used for indentation. Its accessible name is
 * `label` and its description the element `descriptionId`. CodeMirror's own editing,
 * selection and search keys work inside it; keys the app owns are left to the app's
 * dispatcher (Primary+Enter, F8, Primary+I and the region keys). Primary+Z and Primary+Y
 * undo and redo typing while there is typing to undo; with none, the key reaches the app.
 *
 * A key the editor handled stops at the editor, so a global command bound to the same key
 * never runs as well (§2: a text field keeps its editing keys).
 */
export function CodeEditor({
  label,
  descriptionId,
  value,
  onChange,
  diagnostics = [],
  completion,
  autoComplete = false,
  nonce,
  onTextHistory,
  handle,
  className,
}: CodeEditorProps) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const latest = useRef({ onChange, onTextHistory, completion });
  useEffect(() => {
    latest.current = { onChange, onTextHistory, completion };
  });
  const historyCompartment = useRef(new Compartment());
  const completionCompartment = useRef(new Compartment());
  const labelCompartment = useRef(new Compartment());

  // Created once; later prop changes are dispatched below.
  useEffect(() => {
    const parent = host.current;
    if (parent === null) {
      return;
    }
    const source: CompletionSource = (context: CompletionContext) =>
      latest.current.completion?.(context) ?? null;
    const extensions: Extension[] = [
      lineNumbers(),
      highlightActiveLineGutter(),
      historyCompartment.current.of(history()),
      drawSelection(),
      indentOnInput(),
      bracketMatching(),
      closeBrackets(),
      highlightActiveLine(),
      json(),
      syntaxHighlighting(highlight),
      search({ top: true }),
      lintGutter(),
      completionCompartment.current.of(
        autocompletion({
          override: [source],
          activateOnTyping: autoComplete,
          defaultKeymap: false,
        }),
      ),
      keymap.of([
        ...historyKeys((step) => latest.current.onTextHistory?.(step)),
        ...withoutAppKeys(completionKeymap),
        ...withoutAppKeys(closeBracketsKeymap),
        // In the editor and in its search panel's fields (CodeMirror's "search-panel" scope),
        // so Escape closes the panel from its search field and returns to the text.
        {
          key: "F3",
          run: findNext,
          shift: findPrevious,
          scope: "editor search-panel",
        },
        { key: "Escape", run: closeSearchPanel, scope: "editor search-panel" },
        ...withoutAppKeys(defaultKeymap),
      ]),
      labelCompartment.current.of(
        EditorView.contentAttributes.of({
          // WebKit leaves a contenteditable without a tabindex out of the Tab order.
          tabindex: "0",
          "aria-label": label,
          ...(descriptionId === undefined
            ? {}
            : { "aria-describedby": descriptionId }),
        }),
      ),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          latest.current.onChange?.(update.state.doc.toString());
        }
      }),
      ...(nonce === undefined ? [] : [EditorView.cspNonce.of(nonce)]),
    ];
    const created = new EditorView({
      parent,
      state: EditorState.create({ doc: value, extensions }),
    });
    view.current = created;
    // CodeMirror prevents the default of a key it ran; stop that key at the editor, so the
    // window's dispatcher does not run a command for it as well. A key the page's
    // accelerator guard prevented before the editor saw it (Primary+F) still reaches the app.
    let preventedBefore = false;
    const capture = (event: globalThis.KeyboardEvent) => {
      preventedBefore = event.defaultPrevented;
    };
    const bubble = (event: globalThis.KeyboardEvent) => {
      if (event.defaultPrevented && !preventedBefore) {
        event.stopPropagation();
      }
    };
    parent.addEventListener("keydown", capture, { capture: true });
    parent.addEventListener("keydown", bubble);
    return () => {
      parent.removeEventListener("keydown", capture, { capture: true });
      parent.removeEventListener("keydown", bubble);
      created.destroy();
      view.current = null;
    };
    // The view is made once per mount; the props below are applied by their own effects.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // A new text from outside replaces the document and starts a fresh undo history.
  useEffect(() => {
    const current = view.current;
    if (current === null || current.state.doc.toString() === value) {
      return;
    }
    current.dispatch({
      changes: { from: 0, to: current.state.doc.length, insert: value },
    });
    resetHistory(current, historyCompartment.current);
  }, [value]);

  useEffect(() => {
    const current = view.current;
    if (current === null) {
      return;
    }
    current.dispatch(
      setDiagnostics(
        current.state,
        toLint(diagnostics, current.state.doc.length),
      ),
    );
  }, [diagnostics]);

  useEffect(() => {
    view.current?.dispatch({
      effects: [
        completionCompartment.current.reconfigure(
          autocompletion({
            override: [
              (context: CompletionContext) =>
                latest.current.completion?.(context) ?? null,
            ],
            activateOnTyping: autoComplete,
            defaultKeymap: false,
          }),
        ),
        labelCompartment.current.reconfigure(
          EditorView.contentAttributes.of({
            tabindex: "0",
            "aria-label": label,
            ...(descriptionId === undefined
              ? {}
              : { "aria-describedby": descriptionId }),
          }),
        ),
      ],
    });
  }, [autoComplete, label, descriptionId]);

  useImperativeHandle(
    handle,
    () => ({
      focus: () => {
        view.current?.focus();
      },
      select: (from, to = from) => {
        const current = view.current;
        if (current === null) return;
        const length = current.state.doc.length;
        const anchor = Math.min(Math.max(from, 0), length);
        const head = Math.min(Math.max(to, anchor), length);
        current.dispatch({
          selection: EditorSelection.single(anchor, head),
          scrollIntoView: true,
        });
        current.focus();
      },
      text: () => view.current?.state.doc.toString() ?? value,
      caret: () => view.current?.state.selection.main.head ?? 0,
      complete: () =>
        view.current === null ? false : startCompletion(view.current),
      openSearch: () => {
        if (view.current !== null) openSearchPanel(view.current);
      },
      replaceAll: (text) => {
        const current = view.current;
        if (current === null) return;
        current.dispatch({
          changes: { from: 0, to: current.state.doc.length, insert: text },
          userEvent: "input.format",
        });
      },
      resetHistory: () => {
        if (view.current !== null) {
          resetHistory(view.current, historyCompartment.current);
        }
      },
    }),
    [value],
  );

  return (
    <div
      ref={host}
      data-scope="editor"
      className={cx(styles.editor, className)}
    />
  );
}

function resetHistory(view: EditorView, compartment: Compartment): void {
  view.dispatch({ effects: compartment.reconfigure([]) });
  view.dispatch({ effects: compartment.reconfigure(history()) });
}

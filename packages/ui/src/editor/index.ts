// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * `@sonarithm/ui/code-editor`: the CodeMirror 6 editor (B05-T03). It is a separate entry
 * point, not part of `@sonarithm/ui`, so an app can load it on demand and keep CodeMirror
 * out of its start-up bundle (PB-07). `@sonarithm/ui/diagnostic-list` is separate for the
 * same reason: React Aria's GridList is loaded only with the Diagnostics tab.
 */
export {
  CodeEditor,
  type CodeEditorHandle,
  type CodeEditorProps,
  type EditorDiagnostic,
  type EditorSeverity,
} from "./CodeEditor.tsx";
export type {
  Completion,
  CompletionContext,
  CompletionResult,
  CompletionSource,
} from "@codemirror/autocomplete";

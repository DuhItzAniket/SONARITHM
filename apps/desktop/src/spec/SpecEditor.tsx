// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { Button } from "@sonarithm/ui";
import type {
  CodeEditorHandle,
  CompletionContext,
  CompletionResult,
  EditorDiagnostic,
} from "@sonarithm/ui/code-editor";
import { CodeEditor } from "@sonarithm/ui/code-editor";
import { useEffect, useMemo, useRef, useState } from "react";

import type { RoutedAnnouncer } from "../announcer/announcer.ts";
import { displayChord } from "../commands/chords.ts";
import { useCommands } from "../commands/context.tsx";
import { useLocalizer } from "../i18n.ts";
import type { Schema } from "./completion.ts";
import type { Coalescer } from "./coalesce.ts";
import { coalesce } from "./coalesce.ts";
import { completeAt } from "./completion.ts";
import { useSpecContext, useSpecSnapshot } from "./context.ts";
import { styleNonce } from "./nonce.ts";
import type { PlacedDiagnostic } from "./place.ts";
import { detailOf, place, severityLabel } from "./place.ts";
import styles from "./spec.module.css";

/** Live validation waits this long after the last keystroke (B05-T03). */
export const VALIDATE_AFTER_MS = 400;

/** Outside changes closer together than this are said once more, not each (B05-T03). */
export const OUTSIDE_QUIET_MS = 1500;

/**
 * The generated schema (`SPEC_SCHEMA_FILE`; a test checks that this path names it), loaded
 * after the editor (it is about 73 kB).
 */
const loadSchema = (): Promise<Schema> =>
  import("../../../../schemas/sonarithm-spec-0.4.schema.json").then(
    (module) => module.default as Schema,
  );

/**
 * The Spec tab (B05-T03; keyboard-and-navigation §5.9, 09 §3): the session's spec in a
 * CodeMirror editor, with live diagnostics from the app core's `spec_validate` and
 * completion from the generated JSON Schema. Typing never speaks: a summary above the
 * editor, which is its description, says how many problems there are, and F8 goes to them.
 * Apply (Primary+Enter) sets the spec and compiles it. When the spec changes elsewhere (the
 * inspector, undo), the editor shows the new text, unless it holds edits not yet applied:
 * then it keeps them and says so.
 *
 * Loaded on demand, the first time the Spec tab opens, so CodeMirror stays out of the
 * start-up bundle (PB-07).
 */
export default function SpecEditor({
  announcer,
  autoComplete,
}: {
  readonly announcer: RoutedAnnouncer;
  readonly autoComplete: boolean;
}) {
  const t = useLocalizer();
  const registry = useCommands();
  const context = useSpecContext();
  const store = context?.store ?? null;
  const bridge = context?.bridge ?? null;
  const snapshot = useSpecSnapshot(store);
  const handle = useRef<CodeEditorHandle | null>(null);
  const [draft, setDraft] = useState(snapshot.text ?? "");
  // The session text the draft started from: the draft has unapplied edits when it differs.
  const [base, setBase] = useState(snapshot.text ?? "");
  // Changes made outside the editor while it holds unapplied edits: each one is said.
  const [outsideChanges, setOutsideChanges] = useState(0);
  const changedOutside = outsideChanges > 0;
  const [schema, setSchema] = useState<Schema | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    let live = true;
    void loadSchema().then((loaded) => {
      if (live) setSchema(loaded);
    });
    return () => {
      live = false;
    };
  }, []);

  // The session's spec changed (any view, or undo): follow it unless edits would be lost.
  const [seenRevision, setSeenRevision] = useState(snapshot.revision);
  if (snapshot.revision !== seenRevision) {
    setSeenRevision(snapshot.revision);
    const text = snapshot.text ?? "";
    if (text === draft || draft === base) {
      setDraft(text);
      setBase(text);
      setOutsideChanges(0);
    } else {
      setOutsideChanges((count) => count + 1);
    }
  }
  // Said for every further change too, politely; a burst of changes is said once more.
  const say = useRef<() => void>(() => undefined);
  useEffect(() => {
    say.current = () => {
      announcer.announce({
        text: t.format("ui-spec-changed-outside"),
        assertive: false,
      });
    };
  });
  const notice = useRef<Coalescer | null>(null);
  useEffect(() => {
    const coalescer = coalesce(OUTSIDE_QUIET_MS, () => {
      say.current();
    });
    notice.current = coalescer;
    return () => {
      coalescer.cancel();
    };
  }, []);
  useEffect(() => {
    if (outsideChanges > 0) notice.current?.trigger();
  }, [outsideChanges]);

  // The Diagnostics tab reads the draft, to show only diagnostics of this text.
  useEffect(() => {
    bridge?.setDraft(draft);
  }, [bridge, draft]);

  // Validate the draft once typing pauses (and once on opening).
  useEffect(() => {
    if (store === null) return;
    if (timer.current !== null) clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      timer.current = null;
      // An empty editor has nothing to check: clear the list rather than keep an old one.
      if (draft.trim() === "") store.clearChecked(draft);
      else void store.validate(draft);
    }, VALIDATE_AFTER_MS);
    return () => {
      if (timer.current !== null) clearTimeout(timer.current);
    };
  }, [draft, store]);

  // Diagnostics belong to the text they were found in; older ones are not shown.
  const checked = snapshot.checked?.text === draft ? snapshot.checked : null;
  const placed: readonly PlacedDiagnostic[] = useMemo(
    () => (checked === null ? [] : place(checked)),
    [checked],
  );
  const marks: readonly EditorDiagnostic[] = useMemo(
    () =>
      placed.flatMap((d) =>
        d.span === null
          ? []
          : [
              {
                from: d.span.from,
                to: d.span.to,
                severity: d.severity,
                message: `${severityLabel(t, d.severity)}. ${detailOf(d.info)}`,
              },
            ],
      ),
    [placed, t],
  );
  useEffect(() => {
    if (bridge !== null) {
      bridge.setDiagnostics(placed);
    }
  }, [bridge, placed]);

  // The editor's handle reaches the app's commands through the bridge.
  useEffect(() => {
    const current = handle.current;
    if (bridge === null || current === null) return;
    bridge.attach(current);
    return () => {
      bridge.detach(current);
    };
  }, [bridge]);

  const completion = useMemo(
    () =>
      (context: CompletionContext): CompletionResult | null => {
        if (schema === null) return null;
        const found = completeAt(
          schema,
          context.state.doc.toString(),
          context.pos,
        );
        if (found === null) return null;
        if (!context.explicit && found.from === found.to) return null;
        return {
          from: found.from,
          to: found.to,
          options: found.options.map((option) => ({
            label: option.label,
            type: option.type,
            ...(option.apply === undefined ? {} : { apply: option.apply }),
            ...(option.info === undefined ? {} : { info: option.info }),
          })),
          validFor: /^"?[\w-]*"?$/,
        };
      },
    [schema],
  );

  const counts = checked?.counts;
  const summary =
    snapshot.checking || (counts === undefined && draft.trim() !== "")
      ? t.format("ui-spec-checking")
      : t.format("ui-spec-summary", {
          errors: { number: counts?.errors ?? 0 },
          warnings: { number: counts?.warnings ?? 0 },
          notes: { number: counts?.info ?? 0 },
        });
  const applyKey = registry.bindings("spec.apply")[0];
  const nonce = styleNonce();
  const nonceProp = nonce === undefined ? {} : { nonce };

  return (
    <div className={styles.spec}>
      {draft.trim() === "" && applyKey !== undefined && (
        <p className={styles.hint}>
          {t.format("ui-spec-empty-hint", {
            key: displayChord(applyKey, registry.platform),
          })}
        </p>
      )}
      <p id="spec-summary" className={styles.summary}>
        {summary}
      </p>
      {changedOutside && (
        <div className={styles.notice}>
          <p>{t.format("ui-spec-changed-outside")}</p>
          <Button
            onPress={() => {
              const text = snapshot.text ?? "";
              setDraft(text);
              setBase(text);
              setOutsideChanges(0);
            }}
          >
            {t.format("ui-spec-load-current")}
          </Button>
        </div>
      )}
      <CodeEditor
        label={t.format("ui-spec-editor")}
        descriptionId="spec-summary"
        value={draft}
        onChange={setDraft}
        diagnostics={marks}
        completion={completion}
        autoComplete={autoComplete}
        handle={handle}
        className={styles.editor ?? ""}
        {...nonceProp}
        onTextHistory={(step) => {
          announcer.announce({
            text: t.format(step === "undo" ? "ui-undo-text" : "ui-redo-text"),
            assertive: false,
          });
        }}
      />
    </div>
  );
}

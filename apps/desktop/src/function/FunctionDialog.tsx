// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, DatasetSummary, FunctionInfo } from "@sonarithm/ipc";
import {
  Button,
  DialogActions,
  ModalDialog,
  NumberField,
  TextField,
} from "@sonarithm/ui";
import { useEffect, useRef, useState } from "react";

import type { Announcer } from "../announcer/announcer.ts";
import { useLocalizer } from "../i18n.ts";
import styles from "./function.module.css";
import type { Message } from "./readout.ts";
import { expressionReason } from "./readout.ts";

/** A function the app core opened as a dataset (`function_open`). */
export interface OpenedFunction {
  readonly dataset: DatasetSummary;
  readonly info: FunctionInfo;
}

/** How the expression reads, from `function_check`. */
type Check =
  | { readonly ok: true; readonly canonical: string }
  | { readonly ok: false; readonly reason: Message };

/** The number fields, which must hold a finite number before Open samples anything. */
type NumberKey = "from" | "to" | "samples";

/** The pause after typing before the expression is checked, so a reader is not interrupted. */
export const CHECK_DELAY_MS = 250;

/** The sample count limits of a function source (ADR-0028). */
const SAMPLES = { min: 2, max: 100_000, default: 1000 } as const;

/**
 * "New function" (C03-T08b; ADR-0017 "Functions"): an expression in x, the x range and the
 * sample count. The expression is checked by the app core (`function_check`) after a
 * pause in typing: a reading expression shows its canonical form, one that does not is
 * marked invalid with the reason and the character where reading stopped. The reason is
 * text beside the field and is not announced while typing; it is announced when Open is
 * pressed with it (keyboard §9). Open samples the function (`function_open`), closes the
 * dialog and announces the function's summary; the focus returns to where the dialog was
 * opened from.
 */
export function FunctionDialog({
  client,
  announcer,
  isOpen,
  onClose,
  onOpened,
}: {
  readonly client: Client | null;
  readonly announcer: Announcer;
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onOpened: (opened: OpenedFunction) => void;
}) {
  const t = useLocalizer();
  const [expr, setExpr] = useState("");
  const [from, setFrom] = useState(-10);
  const [to, setTo] = useState(10);
  const [samples, setSamples] = useState<number>(SAMPLES.default);
  // The last check and the text it was made for: a check of older text is not shown, so
  // a mark never outlives the text it describes.
  const [checked, setChecked] = useState<{
    readonly expr: string;
    readonly result: Check;
  } | null>(null);
  const check = checked?.expr === expr ? checked.result : null;
  const [failure, setFailure] = useState<string | null>(null);
  // NumberField commits a typed number during the Enter keydown that also submits the
  // form, before React renders it (as in "Go to row…").
  const latest = useRef({ from, to, samples });

  useEffect(() => {
    if (!isOpen || client === null || expr.trim() === "") {
      return;
    }
    let live = true;
    const timer = window.setTimeout(() => {
      client
        .call({ command: "function_check", args: { expr, variables: {} } })
        .then(
          (reply) => {
            if (!live) return;
            setChecked({
              expr,
              result:
                reply.ok && reply.canonical !== null
                  ? { ok: true, canonical: reply.canonical }
                  : {
                      ok: false,
                      reason: expressionReason(
                        reply.diagnostics[0]?.message,
                        reply.span,
                      ),
                    },
            });
          },
          () => undefined,
        );
    }, CHECK_DELAY_MS);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
  }, [client, expr, isOpen]);

  const invalid = check !== null && !check.ok;
  const reason =
    check !== null && !check.ok
      ? t.format(check.reason.id, check.reason.args)
      : undefined;
  // The number fields Open found empty: each says so in words until it is edited.
  const [missing, setMissing] = useState<ReadonlySet<NumberKey>>(new Set());
  const names: Record<NumberKey, string> = {
    from: t.format("ui-function-from"),
    to: t.format("ui-function-to"),
    samples: t.format("ui-function-samples"),
  };
  const needed = (key: NumberKey) =>
    missing.has(key)
      ? {
          isInvalid: true,
          errorMessage: t.format("ui-function-number-needed", {
            name: names[key],
          }),
        }
      : {};
  const edited = (key: NumberKey) => {
    if (missing.has(key)) {
      const rest = new Set(missing);
      rest.delete(key);
      setMissing(rest);
    }
  };

  const open = async (close: () => void) => {
    if (client === null) return;
    if (reason !== undefined) {
      announcer.announce({ text: reason, assertive: false });
      return;
    }
    const { from: min, to: max, samples: count } = latest.current;
    // A cleared number field holds NaN, which would reach the app core as `null`.
    const empty = (["from", "to", "samples"] as const).filter(
      (key) => !Number.isFinite({ from: min, to: max, samples: count }[key]),
    );
    if (empty.length > 0) {
      setMissing(new Set(empty));
      announcer.announce({
        text: t.format("ui-function-number-needed", {
          name: names[empty[0] ?? "from"],
        }),
        assertive: false,
      });
      return;
    }
    try {
      const reply = await client.call({
        command: "function_open",
        args: {
          expr,
          domain: [min, max],
          samples: count,
          step: null,
          variables: {},
          name: null,
        },
      });
      setFailure(null);
      close();
      onOpened({ dataset: reply.dataset, info: reply.function });
      announcer.announce({ text: reply.function.summary, assertive: false });
    } catch (error: unknown) {
      setFailure(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <ModalDialog
      title={t.format("ui-function-new")}
      isOpen={isOpen}
      onOpenChange={(next) => {
        if (!next) {
          setFailure(null);
          onClose();
        }
      }}
    >
      {(close) => (
        <form
          className={styles.form}
          onSubmit={(event) => {
            event.preventDefault();
            void open(close);
          }}
        >
          <TextField
            label={t.format("ui-function-expression")}
            description={
              check?.ok === true
                ? t.format("ui-function-reads-as", { text: check.canonical })
                : t.format("ui-function-expression-hint")
            }
            value={expr}
            onChange={setExpr}
            isInvalid={invalid}
            // The app core judges the expression; the browser's own validation would
            // stop Enter from reaching the form, so the reason could not be announced.
            validationBehavior="aria"
            {...(reason === undefined ? {} : { errorMessage: reason })}
            // The dialog exists to type an expression; focus starts in its field (APG
            // Dialog: focus moves to the first field the user acts on).
            // eslint-disable-next-line jsx-a11y/no-autofocus -- see above
            autoFocus
            spellCheck="false"
            autoComplete="off"
          />
          <div className={styles.row}>
            <NumberField
              label={t.format("ui-function-from")}
              description={t.format("ui-function-domain-hint")}
              value={from}
              validationBehavior="aria"
              {...needed("from")}
              onChange={(next) => {
                latest.current = { ...latest.current, from: next };
                setFrom(next);
                edited("from");
              }}
            />
            <NumberField
              label={t.format("ui-function-to")}
              value={to}
              validationBehavior="aria"
              {...needed("to")}
              onChange={(next) => {
                latest.current = { ...latest.current, to: next };
                setTo(next);
                edited("to");
              }}
            />
          </div>
          <NumberField
            label={t.format("ui-function-samples")}
            value={samples}
            minValue={SAMPLES.min}
            maxValue={SAMPLES.max}
            step={1}
            validationBehavior="aria"
            {...needed("samples")}
            onChange={(next) => {
              latest.current = { ...latest.current, samples: next };
              setSamples(next);
              edited("samples");
            }}
          />
          {failure !== null && (
            <p role="alert" className={styles.failure}>
              {failure}
            </p>
          )}
          <DialogActions>
            <Button onPress={close}>{t.format("ui-cancel")}</Button>
            <Button type="submit">{t.format("ui-function-open")}</Button>
          </DialogActions>
        </form>
      )}
    </ModalDialog>
  );
}

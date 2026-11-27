// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  AnchorText,
  AxisFitInfo,
  AxisTable,
  Client,
  DatasetSummary,
  RasterAxisKind,
  ReadArea,
  ReplyFor,
} from "@sonarithm/ipc";
import { IpcFailure } from "@sonarithm/ipc";
import {
  Button,
  DialogActions,
  ImageCrosshair,
  ModalDialog,
  NumberField,
  Option,
  Radio,
  RadioGroup,
  Select,
  TextField,
} from "@sonarithm/ui";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { Announcer } from "../announcer/announcer.ts";
import { useLocalizer } from "../i18n.ts";
import styles from "./calibrate.module.css";
import type { Axis, DialogState } from "./model.ts";
import {
  Latest,
  START,
  anchorsOf,
  checkLines,
  labelOf,
  takesNumbers,
  toCalibration,
  valueAtPixel,
  valueText,
} from "./model.ts";

type RasterChart = ReplyFor<"chart_raster_open">;

/** The code of the extraction summary, which is said when the chart opens (10 §5). */
const SUMMARY_CODE = "SNR-ENG-385";
/** The code of a raster session that has ended (ADR-0017 "Raster calibration"). */
const SESSION_ENDED = "SNR-APP-096";

/** A failure, shown once in a `role="alert"` paragraph: its one route to speech. */
interface Failure {
  readonly text: string;
  /** Changes with every failure, so the same text said twice is announced twice. */
  readonly key: number;
}

/** A failure's text: an ended session says the image must be opened again. */
function failureText(
  t: ReturnType<typeof useLocalizer>,
  error: unknown,
): string {
  if (error instanceof IpcFailure && error.code === SESSION_ENDED) {
    return t.format("ui-calibrate-session-ended");
  }
  return error instanceof Error ? error.message : String(error);
}

/** A failure's state: the text, and a key that tells repeated failures apart. */
function useFailure(): [Failure | null, (text: string | null) => void] {
  const [failure, setFailure] = useState<Failure | null>(null);
  const count = useRef(0);
  const set = useCallback((text: string | null) => {
    count.current += 1;
    setFailure(text === null ? null : { text, key: count.current });
  }, []);
  return [failure, set];
}

const KINDS: Readonly<Record<RasterAxisKind, string>> = {
  linear: "ui-calibrate-scale-linear",
  log: "ui-calibrate-scale-log",
  date: "ui-calibrate-scale-date",
  category: "ui-calibrate-scale-category",
};

/**
 * "Open chart image…" (C02-T10; ADR-0017 "Raster calibration", keyboard-and-navigation
 * §5.11): a PNG or JPEG chart is read through anchors the user enters on each axis.
 *
 * The dialog first asks for the file (the app has no file chooser yet), then shows the image
 * with a crosshair moved by two sliders, adds anchors at the crosshair with the value they
 * stand for, and lets each axis's scale be chosen. The app core checks the anchors as they
 * change (`chart_calibration_check`, at most once per frame, the latest position only) and
 * reads every date and label; its checks are shown, and said only when they change. Read the
 * chart opens it as a dataset and says the extraction summary. Cancel or Escape ends the
 * session; the focus returns to where the dialog was opened from.
 */
export function ChartImageDialog({
  client,
  announcer,
  isOpen,
  onClose,
  onOpened,
}: {
  readonly client: Client;
  readonly announcer: Announcer;
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onOpened: (dataset: DatasetSummary) => void;
}) {
  const t = useLocalizer();
  const [image, setImage] = useState<RasterChart | null>(null);
  const session = useRef<number | null>(null);
  const end = () => {
    const id = session.current;
    session.current = null;
    setImage(null);
    if (id !== null) {
      void client
        .call({ command: "chart_raster_close", args: { id } })
        .catch(() => undefined);
    }
    onClose();
  };
  return (
    <ModalDialog
      title={
        image === null
          ? t.format("ui-chart-image-open-title")
          : t.format("ui-calibrate-title", { name: image.name })
      }
      isOpen={isOpen}
      onOpenChange={(open) => {
        if (!open) end();
      }}
    >
      {(close) =>
        image === null ? (
          <PickImage
            client={client}
            onCancel={close}
            onOpen={(opened) => {
              session.current = opened.id;
              setImage(opened);
            }}
          />
        ) : (
          <Calibrate
            client={client}
            announcer={announcer}
            image={image}
            onCancel={close}
            onRead={(dataset) => {
              session.current = null;
              setImage(null);
              close();
              onOpened(dataset);
              const summary = dataset.diagnostics.find(
                (d) => d.code === SUMMARY_CODE,
              );
              if (summary !== undefined) {
                announcer.announce({ text: summary.message, assertive: false });
              }
            }}
          />
        )
      }
    </ModalDialog>
  );
}

/** The first step: the image's path, checked and decoded by the app core. */
function PickImage({
  client,
  onCancel,
  onOpen,
}: {
  readonly client: Client;
  readonly onCancel: () => void;
  readonly onOpen: (opened: RasterChart) => void;
}) {
  const t = useLocalizer();
  const [path, setPath] = useState("");
  const [failure, setFailure] = useFailure();
  const [busy, setBusy] = useState(false);
  // A reply that arrives after Cancel closes the session it opened, so none is left behind.
  const gone = useRef(false);
  useEffect(
    () => () => {
      gone.current = true;
    },
    [],
  );
  return (
    <form
      className={styles.form}
      onSubmit={(event) => {
        event.preventDefault();
        if (busy) return;
        setBusy(true);
        client.call({ command: "chart_raster_open", args: { path } }).then(
          (opened) => {
            if (gone.current) {
              void client
                .call({
                  command: "chart_raster_close",
                  args: { id: opened.id },
                })
                .catch(() => undefined);
              return;
            }
            setBusy(false);
            setFailure(null);
            onOpen(opened);
          },
          (error: unknown) => {
            if (gone.current) return;
            setBusy(false);
            setFailure(failureText(t, error));
          },
        );
      }}
    >
      <TextField
        label={t.format("ui-chart-image-path")}
        description={t.format("ui-chart-image-path-hint")}
        value={path}
        onChange={setPath}
        isRequired
        // The dialog exists to name the file; focus starts in its field (APG Dialog).
        // eslint-disable-next-line jsx-a11y/no-autofocus -- see above
        autoFocus
        spellCheck="false"
        autoComplete="off"
      />
      {failure !== null && (
        <p key={failure.key} role="alert" className={styles.failure}>
          {failure.text}
        </p>
      )}
      <DialogActions>
        <Button onPress={onCancel}>{t.format("ui-cancel")}</Button>
        <Button type="submit" isDisabled={busy}>
          {t.format("ui-chart-image-open")}
        </Button>
      </DialogActions>
    </form>
  );
}

/** What the last check said, and the anchors it was asked about. */
interface Checked {
  readonly state: DialogState;
  readonly x: AxisFitInfo | null;
  readonly y: AxisFitInfo | null;
  /** Each fitted axis's value at every pixel along it, or `null` while it does not fit. */
  readonly values: Readonly<Record<Axis, readonly AnchorText[] | null>>;
  readonly messages: readonly string[];
}

/** The second step: the crosshair, the anchors, the scales and the checks. */
function Calibrate({
  client,
  announcer,
  image,
  onCancel,
  onRead,
}: {
  readonly client: Client;
  readonly announcer: Announcer;
  readonly image: RasterChart;
  readonly onCancel: () => void;
  readonly onRead: (dataset: DatasetSummary) => void;
}) {
  const t = useLocalizer();
  const start = image.plotArea ?? {
    min: [0, 0],
    max: [image.widthPx, image.heightPx],
  };
  const [at, setAt] = useState<readonly [number, number]>([
    Math.floor((start.min[0] + start.max[0]) / 2),
    Math.floor((start.min[1] + start.max[1]) / 2),
  ]);
  const [state, setState] = useState<DialogState>(START);
  const [axis, setAxis] = useState<Axis>("x");
  const [number, setNumber] = useState(Number.NaN);
  const [text, setText] = useState("");
  const [checked, setChecked] = useState<Checked | null>(null);
  const [failure, setFailure] = useFailure();
  const [reading, setReading] = useState(false);
  const nextId = useRef(1);
  // Where the focus goes once a deleted anchor's button is gone (APG: never lose it).
  const refocus = useRef<number | "add" | null>(null);
  // The file step's field is gone; the focus moves to the crosshair's first slider, the
  // first control of this step (APG Dialog), so it never falls out of the dialog.
  const root = useRef<HTMLDivElement>(null);
  useEffect(() => {
    root.current
      ?.querySelector<HTMLInputElement>('input[type="range"]')
      ?.focus();
  }, []);

  // Ask the app core about the anchors: the newest only, once per frame. The crosshair
  // asks nothing: the check gives every pixel's value, which the sliders look up.
  const latest = useMemo(() => {
    // Each axis's values by fit: the app core sends them once per fit, then the key alone.
    const tables: Record<Axis, AxisTable | null> = { x: null, y: null };
    const resolve = (axis: Axis, table: AxisTable | null) => {
      if (table === null) {
        tables[axis] = null;
      } else if (table.values !== null) {
        tables[axis] = table;
      } else if (tables[axis]?.fit !== table.fit) {
        tables[axis] = null;
      }
      return tables[axis]?.values ?? null;
    };
    return new Latest<DialogState>(
      async (asked) => {
        const reply = await client.call({
          command: "chart_calibration_check",
          args: { id: image.id, calibration: toCalibration(asked), at: null },
        });
        setChecked({
          state: asked,
          x: reply.x,
          y: reply.y,
          values: {
            x: resolve("x", reply.values.x),
            y: resolve("y", reply.values.y),
          },
          messages: reply.diagnostics.map((d) => d.message),
        });
      },
      (run) => {
        window.requestAnimationFrame(run);
      },
      (error) => {
        setFailure(failureText(t, error));
      },
    );
  }, [client, image.id, setFailure, t]);
  useEffect(() => {
    latest.push(state);
  }, [latest, state]);
  useEffect(() => {
    const target = refocus.current;
    if (target === null) return;
    refocus.current = null;
    const scope = root.current;
    const button =
      target === "add"
        ? scope?.querySelector<HTMLElement>("[data-add-anchor]:not([disabled])")
        : scope?.querySelector<HTMLElement>(
            `[data-anchor-delete="${String(target)}"]`,
          );
    // Add anchor is disabled while the value field is empty; the field comes before it.
    (
      button ?? scope?.querySelector<HTMLElement>("[data-anchor-value] input")
    )?.focus();
  }, [state.anchors]);

  // The checks in words; said only when a new check changes them, never on every crosshair
  // step. The live region speaks only the latest polite message, so an anchor added or
  // deleted since the last check is said again at the start of the checks it changed.
  // The lines describe one state: the anchors the last check was asked about, not ones
  // entered since, which the next check describes.
  const lines = checkLines(
    t,
    checked?.state ?? state,
    { x: checked?.x ?? null, y: checked?.y ?? null },
    checked?.messages ?? [],
  );
  const summary = lines.join(" ");
  const said = useRef<string | null>(null);
  const seen = useRef<typeof checked>(null);
  const acted = useRef<string | null>(null);
  useEffect(() => {
    if (checked === null || checked === seen.current) return;
    seen.current = checked;
    if (said.current !== null && said.current !== summary) {
      const action = acted.current;
      announcer.announce({
        text: action === null ? summary : `${action} ${summary}`,
        assertive: false,
      });
    }
    said.current = summary;
    acted.current = null;
  }, [checked, summary, announcer]);
  /** Says an anchor action now, and keeps it for the checks it changes. */
  const sayAction = (text: string) => {
    acted.current = text;
    announcer.announce({ text, assertive: false });
  };

  // The value under the crosshair, looked up from the last check in the same render as
  // the step, so a slider's text never changes after it (the #517 review).
  const current = (which: Axis, pixel: number): string | null =>
    valueText(t, valueAtPixel(checked?.values[which], pixel));
  const pixelText = (pixel: number, value: string | null) =>
    value === null
      ? t.format("ui-calibrate-pixel", { index: { number: pixel } })
      : t.format("ui-calibrate-pixel-value", {
          index: { number: pixel },
          value,
        });

  const kind = state.kinds[axis];
  const value: AnchorText | null = takesNumbers(kind)
    ? Number.isFinite(number)
      ? number
      : null
    : text.trim() === ""
      ? null
      : text.trim();

  const add = () => {
    if (value === null) return;
    const anchor = { id: nextId.current, axis, at, value };
    nextId.current += 1;
    const next = { ...state, anchors: [...state.anchors, anchor] };
    setState(next);
    setNumber(Number.NaN);
    setText("");
    sayAction(
      t.format("ui-calibrate-anchor-added", { label: labelOf(next, anchor) }),
    );
  };

  // A failure is said once, by its alert.
  const read = () => {
    if (reading) return;
    if (anchorsOf(state, "x").length < 2 || anchorsOf(state, "y").length < 2) {
      setFailure(t.format("ui-calibrate-not-ready"));
      return;
    }
    setReading(true);
    client
      .call({
        command: "chart_calibration_apply",
        args: { id: image.id, calibration: toCalibration(state) },
      })
      .then(
        (dataset) => {
          setFailure(null);
          onRead(dataset);
        },
        (error: unknown) => {
          setReading(false);
          setFailure(failureText(t, error));
        },
      );
  };

  return (
    <div ref={root} className={styles.calibrate}>
      <ImageCrosshair
        src={image.image}
        width={image.widthPx}
        height={image.heightPx}
        imageLabel={t.format("ui-calibrate-image", {
          name: image.name,
          x: { number: image.widthPx },
          y: { number: image.heightPx },
        })}
        x={at[0]}
        y={at[1]}
        onMove={(x, y) => {
          setAt([x, y]);
        }}
        xLabel={t.format("ui-calibrate-crosshair-x")}
        yLabel={t.format("ui-calibrate-crosshair-y")}
        formatX={(x) => pixelText(x, current("x", x))}
        formatY={(y) => pixelText(y, current("y", y))}
        plotArea={state.area === "detected" ? image.plotArea : null}
        marks={state.anchors.map((anchor) => ({
          id: String(anchor.id),
          axis: anchor.axis,
          at: anchor.at,
          label: labelOf(state, anchor),
        }))}
      />
      <div className={styles.row}>
        <NumberField
          label={t.format("ui-calibrate-pixel-x")}
          value={at[0]}
          minValue={0}
          maxValue={image.widthPx - 1}
          step={1}
          onChange={(x) => {
            if (Number.isFinite(x)) setAt([x, at[1]]);
          }}
        />
        <NumberField
          label={t.format("ui-calibrate-pixel-y")}
          value={at[1]}
          minValue={0}
          maxValue={image.heightPx - 1}
          step={1}
          onChange={(y) => {
            if (Number.isFinite(y)) setAt([at[0], y]);
          }}
        />
      </div>
      <div className={styles.row}>
        {(["x", "y"] as const).map((which) => (
          <Select
            key={which}
            label={t.format("ui-calibrate-scale-of", {
              axisLabel: t.format(
                which === "x" ? "ui-calibrate-axis-x" : "ui-calibrate-axis-y",
              ),
            })}
            value={state.kinds[which]}
            onChange={(key) => {
              if (key === null) return;
              setState({
                ...state,
                kinds: { ...state.kinds, [which]: key as RasterAxisKind },
              });
            }}
          >
            {(Object.keys(KINDS) as RasterAxisKind[])
              .filter((k) => which === "x" || k !== "category")
              .map((k) => (
                <Option key={k} id={k}>
                  {t.format(KINDS[k])}
                </Option>
              ))}
          </Select>
        ))}
      </div>
      <RadioGroup
        label={t.format("ui-calibrate-area")}
        description={t.format(
          image.plotArea === null
            ? "ui-calibrate-area-none-hint"
            : "ui-calibrate-area-hint",
        )}
        value={state.area}
        onChange={(area) => {
          setState({ ...state, area: area as ReadArea });
        }}
      >
        <Radio value="anchors">{t.format("ui-calibrate-area-anchors")}</Radio>
        <Radio value="detected" isDisabled={image.plotArea === null}>
          {t.format("ui-calibrate-area-detected")}
        </Radio>
        <Radio value="image">{t.format("ui-calibrate-area-image")}</Radio>
      </RadioGroup>
      <form
        className={styles.row}
        onSubmit={(event) => {
          event.preventDefault();
          add();
        }}
      >
        <Select
          label={t.format("ui-calibrate-axis")}
          value={axis}
          onChange={(key) => {
            if (key === "x" || key === "y") setAxis(key);
          }}
        >
          <Option id="x">{t.format("ui-calibrate-axis-x")}</Option>
          <Option id="y">{t.format("ui-calibrate-axis-y")}</Option>
        </Select>
        <div data-anchor-value className={styles.value}>
          {takesNumbers(kind) ? (
            <NumberField
              label={t.format("ui-calibrate-value")}
              description={t.format("ui-calibrate-value-number-hint")}
              value={number}
              onChange={setNumber}
            />
          ) : (
            <TextField
              label={t.format("ui-calibrate-value")}
              description={t.format(
                kind === "date"
                  ? "ui-calibrate-value-date-hint"
                  : "ui-calibrate-value-category-hint",
              )}
              value={text}
              onChange={setText}
            />
          )}
        </div>
        <Button type="submit" isDisabled={value === null} data-add-anchor>
          {t.format("ui-calibrate-add")}
        </Button>
      </form>
      {(["x", "y"] as const).map((which) => {
        const anchors = anchorsOf(state, which);
        const heading = t.format(
          which === "x" ? "ui-calibrate-axis-x" : "ui-calibrate-axis-y",
        );
        return (
          <section key={which} aria-label={heading} className={styles.anchors}>
            <h3 className={styles.heading}>{heading}</h3>
            {anchors.length === 0 ? (
              <p className={styles.note}>
                {t.format("ui-calibrate-anchors-none")}
              </p>
            ) : (
              <ul className={styles.list}>
                {anchors.map((anchor) => {
                  const label = labelOf(state, anchor);
                  return (
                    <li key={anchor.id} className={styles.item}>
                      <span>
                        {t.format("ui-calibrate-anchor", {
                          label,
                          x: { number: anchor.at[0] },
                          y: { number: anchor.at[1] },
                          value: valueText(t, anchor.value) ?? "",
                        })}
                      </span>
                      <Button
                        data-anchor-delete={anchor.id}
                        onPress={() => {
                          // The next anchor's Delete, else the previous one's, else Add.
                          const place = anchors.indexOf(anchor);
                          refocus.current =
                            anchors[place + 1]?.id ??
                            anchors[place - 1]?.id ??
                            "add";
                          setState({
                            ...state,
                            anchors: state.anchors.filter(
                              (a) => a.id !== anchor.id,
                            ),
                          });
                          sayAction(
                            t.format("ui-calibrate-anchor-deleted", { label }),
                          );
                        }}
                      >
                        {t.format("ui-calibrate-anchor-delete", { label })}
                      </Button>
                    </li>
                  );
                })}
              </ul>
            )}
          </section>
        );
      })}
      <section
        aria-label={t.format("ui-calibrate-checks")}
        className={styles.checks}
      >
        <h3 className={styles.heading}>{t.format("ui-calibrate-checks")}</h3>
        <ul className={styles.list}>
          {lines.map((line) => (
            <li key={line}>{line}</li>
          ))}
        </ul>
      </section>
      {failure !== null && (
        <p key={failure.key} role="alert" className={styles.failure}>
          {failure.text}
        </p>
      )}
      <DialogActions>
        <Button onPress={onCancel}>{t.format("ui-cancel")}</Button>
        <Button onPress={read} isDisabled={reading}>
          {t.format("ui-calibrate-apply")}
        </Button>
      </DialogActions>
    </div>
  );
}

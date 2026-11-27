// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  AnchorText,
  AxisFitInfo,
  Client,
  DatasetSummary,
  RasterAxisKind,
  ReplyFor,
  ValueAt,
} from "@sonarithm/ipc";
import {
  Button,
  DialogActions,
  ImageCrosshair,
  ModalDialog,
  NumberField,
  Option,
  Select,
  TextField,
} from "@sonarithm/ui";
import { useEffect, useMemo, useRef, useState } from "react";

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
  valueText,
} from "./model.ts";

type RasterChart = ReplyFor<"chart_raster_open">;

/** The code of the extraction summary, which is said when the chart opens (10 §5). */
const SUMMARY_CODE = "SNR-ENG-385";

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
  const [failure, setFailure] = useState<string | null>(null);
  return (
    <form
      className={styles.form}
      onSubmit={(event) => {
        event.preventDefault();
        client.call({ command: "chart_raster_open", args: { path } }).then(
          (opened) => {
            setFailure(null);
            onOpen(opened);
          },
          (error: unknown) => {
            setFailure(error instanceof Error ? error.message : String(error));
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
        <p role="alert" className={styles.failure}>
          {failure}
        </p>
      )}
      <DialogActions>
        <Button onPress={onCancel}>{t.format("ui-cancel")}</Button>
        <Button type="submit">{t.format("ui-chart-image-open")}</Button>
      </DialogActions>
    </form>
  );
}

/** What the last check said, and the crosshair position it was made at. */
interface Checked {
  readonly at: readonly [number, number];
  readonly x: AxisFitInfo | null;
  readonly y: AxisFitInfo | null;
  readonly valueAt: ValueAt | null;
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
  const [failure, setFailure] = useState<string | null>(null);
  const nextId = useRef(1);
  // The file step's field is gone; the focus moves to the crosshair's first slider, the
  // first control of this step (APG Dialog), so it never falls out of the dialog.
  const root = useRef<HTMLDivElement>(null);
  useEffect(() => {
    root.current
      ?.querySelector<HTMLInputElement>('input[type="range"]')
      ?.focus();
  }, []);

  // Ask the app core about the anchors and the crosshair: the newest only, once per frame.
  const latest = useMemo(
    () =>
      new Latest<{ state: DialogState; at: readonly [number, number] }>(
        async ({ state: asked, at: where }) => {
          const reply = await client.call({
            command: "chart_calibration_check",
            args: {
              id: image.id,
              calibration: toCalibration(asked),
              at: [where[0] + 0.5, where[1] + 0.5],
            },
          });
          setChecked({
            at: where,
            x: reply.x,
            y: reply.y,
            valueAt: reply.valueAt,
            messages: reply.diagnostics.map((d) => d.message),
          });
        },
        (run) => {
          window.requestAnimationFrame(run);
        },
      ),
    [client, image.id],
  );
  useEffect(() => {
    latest.push({ state, at });
  }, [latest, state, at]);

  // The checks in words; said only when they change, never on every crosshair step.
  const lines = checkLines(
    t,
    state,
    { x: checked?.x ?? null, y: checked?.y ?? null },
    checked?.messages ?? [],
  );
  const summary = lines.join(" ");
  const said = useRef<string | null>(null);
  useEffect(() => {
    if (checked === null) return;
    if (said.current !== null && said.current !== summary) {
      announcer.announce({ text: summary, assertive: false });
    }
    said.current = summary;
  }, [checked, summary, announcer]);

  const current = (which: Axis): string | null => {
    if (checked?.at[0] !== at[0] || checked.at[1] !== at[1]) {
      return null;
    }
    return valueText(t, checked.valueAt?.[which] ?? null);
  };
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
    announcer.announce({
      text: t.format("ui-calibrate-anchor-added", {
        label: labelOf(next, anchor),
      }),
      assertive: false,
    });
  };

  const read = () => {
    if (anchorsOf(state, "x").length < 2 || anchorsOf(state, "y").length < 2) {
      const message = t.format("ui-calibrate-not-ready");
      setFailure(message);
      announcer.announce({ text: message, assertive: false });
      return;
    }
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
          const message =
            error instanceof Error ? error.message : String(error);
          setFailure(message);
          announcer.announce({ text: message, assertive: true });
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
        formatX={(x) => pixelText(x, x === at[0] ? current("x") : null)}
        formatY={(y) => pixelText(y, y === at[1] ? current("y") : null)}
        plotArea={image.plotArea}
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
        <Button type="submit" isDisabled={value === null}>
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
                        onPress={() => {
                          setState({
                            ...state,
                            anchors: state.anchors.filter(
                              (a) => a.id !== anchor.id,
                            ),
                          });
                          announcer.announce({
                            text: t.format("ui-calibrate-anchor-deleted", {
                              label,
                            }),
                            assertive: false,
                          });
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
        <p role="alert" className={styles.failure}>
          {failure}
        </p>
      )}
      <DialogActions>
        <Button onPress={onCancel}>{t.format("ui-cancel")}</Button>
        <Button onPress={read}>{t.format("ui-calibrate-apply")}</Button>
      </DialogActions>
    </div>
  );
}

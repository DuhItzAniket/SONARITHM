// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, ReplyFor } from "@sonarithm/ipc";
import { Button, NumberField } from "@sonarithm/ui";
import { useEffect, useRef, useState } from "react";

import type { Announcer } from "../announcer/announcer.ts";
import { useLocalizer } from "../i18n.ts";
import styles from "./function.module.css";
import { audifySummary, bandState, significant } from "./readout.ts";

type Readout = ReplyFor<"audify_calculate">;

/** Dombois's seismic speed-up (SDR-31): CURVASONA's `SEISMIC_K`. */
export const SEISMIC_K = 2200;

/** The speed-up range the `audify` encoder accepts (`SNR-ENG-463` outside it). */
const K_RANGE = { min: 0.001, max: 1_000_000 } as const;

/**
 * The audification calculator (C03-T08b; SDR-31): for the dataset's series at speed-up k,
 * how long it plays and the band it sounds in, from the app core's `audify_calculate`, which
 * uses the `audify` encoder's own grid, so the readout and a compile agree. Leaving k empty
 * uses the app core's default (SDR-44). When the whole band is outside hearing
 * (`SNR-ENG-462`), the readout is an error, "will not compile", with its reason, and is
 * announced: a compile of such a track refuses it. Applying k to a track waits for the
 * spec editor (C03-T08c).
 */
export function AudifyCalculator({
  client,
  dataset,
  announcer,
}: {
  readonly client: Client;
  readonly dataset: string;
  readonly announcer: Announcer;
}) {
  const t = useLocalizer();
  const [k, setK] = useState(Number.NaN);
  const [readout, setReadout] = useState<Readout | null>(null);
  const [failure, setFailure] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    client
      .call({
        command: "audify_calculate",
        args: {
          dataset,
          time: null,
          k: Number.isFinite(k) ? k : null,
          topHz: null,
        },
      })
      .then(
        (reply) => {
          if (!live) return;
          setReadout(reply);
          setFailure(null);
        },
        (error: unknown) => {
          if (!live) return;
          setReadout(null);
          setFailure(error instanceof Error ? error.message : String(error));
        },
      );
    return () => {
      live = false;
    };
  }, [client, dataset, k]);

  // Set when the listener commits a k (the field commits on Enter or leaving it, not per
  // keystroke, and the seismic button commits at once), so the readout that k produces is
  // summarised once, politely; the first readout, for the default k, is not (#498 review).
  const summarise = useRef(false);
  const commit = (next: number) => {
    summarise.current = true;
    setK(next);
  };
  useEffect(() => {
    if (readout === null || !summarise.current) return;
    summarise.current = false;
    const summary = audifySummary(readout);
    announcer.announce({
      text: t.format(summary.id, summary.args),
      assertive: false,
    });
  }, [readout, announcer, t]);

  const state = readout === null ? null : bandState(readout.codes);
  // Said once each time the readout becomes "will not compile", not on every change.
  const wasBlocked = useRef(false);
  useEffect(() => {
    const blocked = state === "blocked";
    if (blocked && !wasBlocked.current) {
      announcer.announce({
        text: t.format("ui-audify-will-not-compile"),
        assertive: false,
      });
    }
    wasBlocked.current = blocked;
  }, [state, announcer, t]);

  const hertz = ([low, high]: readonly [number, number]) =>
    t.format("ui-audify-hertz-range", {
      low: significant(low),
      high: significant(high),
    });

  return (
    <div className={styles.calculator}>
      <div className={styles.row}>
        <NumberField
          label={t.format("ui-audify-k")}
          description={t.format("ui-audify-k-hint")}
          value={Number.isFinite(k) ? k : (readout?.k ?? Number.NaN)}
          minValue={K_RANGE.min}
          maxValue={K_RANGE.max}
          formatOptions={{ maximumFractionDigits: 3 }}
          onChange={commit}
        />
        <Button
          onPress={() => {
            commit(SEISMIC_K);
          }}
        >
          {t.format("ui-audify-seismic")}
        </Button>
      </div>
      {failure !== null && (
        <p className={styles.failure}>
          {t.format("ui-audify-unavailable", { detail: failure })}
        </p>
      )}
      {readout !== null && (
        <>
          <dl
            className={styles.readout}
            aria-label={t.format("ui-audify-readout")}
            data-state={state}
          >
            <dt>{t.format("ui-audify-duration")}</dt>
            <dd>
              {t.format("ui-audify-seconds", {
                value: { number: readout.durationS, maximumFractionDigits: 3 },
              })}
            </dd>
            <dt>{t.format("ui-audify-band")}</dt>
            <dd>{hertz(readout.bandHz)}</dd>
            <dt>{t.format("ui-audify-audible")}</dt>
            <dd>
              {readout.audibleHz === null
                ? t.format("ui-audify-audible-none")
                : hertz(readout.audibleHz)}
            </dd>
          </dl>
          {state === "partly" && (
            <p className={styles.note}>{t.format("ui-audify-partly")}</p>
          )}
          {state === "blocked" && (
            <p className={styles.failure} data-audify-blocked="">
              {t.format("ui-audify-will-not-compile")}
            </p>
          )}
        </>
      )}
    </div>
  );
}

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client, DeviceSummary, Settings } from "@sonarithm/ipc";
import {
  Button,
  DialogActions,
  ModalDialog,
  NumberField,
  Option,
  Select,
  Slider,
  Switch,
} from "@sonarithm/ui";
import { useEffect, useState } from "react";

import { useLocalizer } from "../i18n.ts";
import styles from "./preferences.module.css";
import { settingsOf } from "./settings.ts";

/** The key of the "System default" device option. */
const SYSTEM_DEFAULT = "system-default";

/** The sonification volume range in dB (settings.rs: −60 to 0 dB, A11Y-15). */
const VOLUME = { min: -60, max: 0 } as const;

/** The calibrated latency range in milliseconds (settings.rs `LATENCY_RANGE_S`: 0 to 1 s). */
const LATENCY_MS = { min: 0, max: 1000 } as const;

/**
 * Only WebView2 (Windows) can be asked for the discrete GPU (F-044); the host passes the
 * switch at start-up. WebView2's user agent names Windows.
 */
const GPU_CHOOSABLE = navigator.userAgent.includes("Windows");

/**
 * Preferences (B01-T08), opened by `app.preferences` (Ctrl+Comma, Cmd+Comma). The Audio
 * section picks the output device (`devices_list`, `device_select`) and sets the
 * sonification volume, which is SONARITHM's own level and never the system volume (A11Y-15).
 * Rust owns both values (09 §3): the dialog shows what the app core reports.
 */
export function Preferences({
  client,
  isOpen,
  onOpenChange,
}: {
  readonly client: Client | null;
  readonly isOpen: boolean;
  readonly onOpenChange: (isOpen: boolean) => void;
}) {
  const t = useLocalizer();
  const [settings, setSettings] = useState<Settings | null>(null);
  const [devices, setDevices] = useState<readonly DeviceSummary[]>([]);
  const [volume, setVolume] = useState<number | null>(null);

  useEffect(() => {
    if (!isOpen || client === null) {
      return;
    }
    let live = true;
    Promise.all([
      client.call({ command: "settings_get" }),
      client.call({ command: "devices_list" }),
    ]).then(
      ([current, list]) => {
        if (live) {
          setSettings(settingsOf(current));
          setVolume(current.audio.volumeDb);
          setDevices(list.devices);
        }
      },
      () => undefined,
    );
    return () => {
      live = false;
    };
  }, [isOpen, client]);

  const selectDevice = (key: string) => {
    if (client === null) {
      return;
    }
    const id = key === SYSTEM_DEFAULT ? null : key;
    client
      .call({ command: "device_select", args: { id } })
      .then(() => client.call({ command: "settings_get" }))
      .then(
        (current) => {
          setSettings(settingsOf(current));
        },
        () => undefined,
      );
  };

  const saveVolume = (volumeDb: number) => {
    if (client === null || settings === null) {
      return;
    }
    const next = { ...settings, audio: { ...settings.audio, volumeDb } };
    client.call({ command: "settings_set", args: { settings: next } }).then(
      (saved) => {
        setSettings(settingsOf(saved));
        setVolume(saved.audio.volumeDb);
      },
      () => undefined,
    );
  };

  // The device the latency belongs to: the chosen one, or the system default's ID.
  const latencyDevice =
    settings?.audio.device ??
    devices.find((device) => device.isDefault)?.id ??
    null;
  const storedLatencyS =
    latencyDevice === null
      ? undefined
      : settings?.audio.calibratedLatencyS[latencyDevice];

  /**
   * Stores the measured latency of the current device (ADR-0016; B04-T04), or removes it
   * when the field is emptied, so the playhead goes back to the system's estimate. The
   * device ID stays in the local settings file only (rule 9).
   */
  const saveLatency = (milliseconds: number) => {
    if (client === null || settings === null || latencyDevice === null) {
      return;
    }
    const latencies = { ...settings.audio.calibratedLatencyS };
    if (Number.isNaN(milliseconds)) {
      Reflect.deleteProperty(latencies, latencyDevice);
    } else {
      latencies[latencyDevice] = milliseconds / 1000;
    }
    const next: Settings = {
      ...settings,
      audio: { ...settings.audio, calibratedLatencyS: latencies },
    };
    client.call({ command: "settings_set", args: { settings: next } }).then(
      (saved) => {
        setSettings(settingsOf(saved));
      },
      () => undefined,
    );
  };

  const saveGpu = (highPerformance: boolean) => {
    if (client === null || settings === null) {
      return;
    }
    const next: Settings = {
      ...settings,
      gpu: highPerformance ? "highPerformance" : "default",
    };
    client.call({ command: "settings_set", args: { settings: next } }).then(
      (saved) => {
        setSettings(settingsOf(saved));
      },
      () => undefined,
    );
  };

  return (
    <ModalDialog
      title={t.format("ui-preferences")}
      isOpen={isOpen}
      onOpenChange={onOpenChange}
    >
      {(close) => (
        <>
          <section
            aria-labelledby="preferences-audio"
            className={styles.section}
          >
            <h3 id="preferences-audio" className={styles.heading}>
              {t.format("ui-audio")}
            </h3>
            <Select
              label={t.format("ui-output-device")}
              value={settings?.audio.device ?? SYSTEM_DEFAULT}
              onChange={(key) => {
                if (key !== null) selectDevice(String(key));
              }}
              isDisabled={settings === null}
            >
              <Option id={SYSTEM_DEFAULT}>
                {t.format("ui-system-default-device")}
              </Option>
              {devices.map((device) => (
                <Option key={device.id} id={device.id}>
                  {device.name}
                </Option>
              ))}
            </Select>
            <Slider
              label={t.format("ui-sonification-volume")}
              minValue={VOLUME.min}
              maxValue={VOLUME.max}
              step={1}
              value={volume ?? VOLUME.max}
              isDisabled={volume === null}
              formatValue={(value) =>
                t.format("unit-decibel", {
                  style: "symbol",
                  value: { number: value },
                })
              }
              onChange={setVolume}
              onChangeEnd={saveVolume}
            />
            <p className={styles.note}>{t.format("ui-volume-note")}</p>
            <NumberField
              label={t.format("ui-output-latency")}
              description={t.format("ui-output-latency-note")}
              value={
                storedLatencyS === undefined
                  ? Number.NaN
                  : Math.round(storedLatencyS * 1000)
              }
              minValue={LATENCY_MS.min}
              maxValue={LATENCY_MS.max}
              step={1}
              formatOptions={{ style: "unit", unit: "millisecond" }}
              isDisabled={latencyDevice === null}
              onChange={saveLatency}
            />
          </section>
          <section
            aria-labelledby="preferences-display"
            className={styles.section}
          >
            <h3 id="preferences-display" className={styles.heading}>
              {t.format("ui-display")}
            </h3>
            <Switch
              isSelected={settings?.gpu === "highPerformance"}
              isDisabled={!GPU_CHOOSABLE || settings === null}
              description={t.format(
                GPU_CHOOSABLE ? "ui-gpu-restart-hint" : "ui-gpu-unavailable",
              )}
              onChange={saveGpu}
            >
              {t.format("ui-gpu-high-performance")}
            </Switch>
          </section>
          <DialogActions>
            <Button onPress={close}>{t.format("ui-close-window")}</Button>
          </DialogActions>
        </>
      )}
    </ModalDialog>
  );
}

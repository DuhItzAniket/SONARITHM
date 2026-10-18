// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// A02-T07: React Aria Tree, Toolbar and Slider, a scoped role="application" chart explorer,
// and the announcement routes, for the NVDA, Narrator, VoiceOver and Orca walkthroughs
// (WALKTHROUGH.md; 08 §4–5).

import { type KeyboardEvent, useEffect, useState } from "react";
import {
  Button,
  Label,
  Radio,
  RadioGroup,
  Separator,
  Slider,
  SliderOutput,
  SliderThumb,
  SliderTrack,
  ToggleButton,
  Toolbar,
  Tree,
  TreeItem,
  TreeItemContent,
} from "react-aria-components";
import { type LogEntry, type Route, announce, onAnnounce } from "./announcer";

const DATA = [12.1, 13.4, 15.0, 14.2, 16.8, 19.5, 21.4, 22.0, 20.7, 18.3, 17.9, 16.2, 15.5, 14.0, 12.8, 11.9, 13.3, 15.7, 18.1, 19.9];
const day = (i: number) => new Date(Date.UTC(2026, 2, i + 1)).toLocaleDateString("en-GB", { day: "numeric", month: "long", timeZone: "UTC" });

function Item({ id, label, children }: { id: string; label: string; children?: React.ReactNode }) {
  return (
    <TreeItem id={id} textValue={label}>
      <TreeItemContent>
        {({ hasChildItems }) => (
          <>
            {hasChildItems ? <Button slot="chevron" className="chevron" /> : <span className="chevron-space" />}
            {label}
          </>
        )}
      </TreeItemContent>
      {children}
    </TreeItem>
  );
}

export function App() {
  const [route, setRoute] = useState<Route>("live");
  const [playing, setPlaying] = useState(false);
  const [speed, setSpeed] = useState(1);
  const [cursor, setCursor] = useState(0);
  const [log, setLog] = useState<LogEntry[]>([]);
  useEffect(() => onAnnounce((e) => setLog((l) => [e, ...l].slice(0, 30))), []);
  const say = (text: string, priority: "polite" | "assertive" = "polite") => void announce(text, route, priority);
  const point = (i: number) => `${day(i)}, ${DATA[i].toFixed(1)} degrees Celsius, point ${i + 1} of ${DATA.length}`;

  const explore = (e: KeyboardEvent<HTMLDivElement>) => {
    const next = { ArrowRight: cursor + 1, ArrowLeft: cursor - 1, Home: 0, End: DATA.length - 1 }[e.key];
    if (next !== undefined) {
      e.preventDefault();
      const i = Math.max(0, Math.min(DATA.length - 1, next));
      setCursor(i);
      say(i === cursor ? `${i === 0 ? "Start" : "End"} of data. ${point(i)}` : point(i));
    } else if (e.key === "h" || e.key === "H") {
      e.preventDefault();
      say("Chart explorer. Left and Right arrows move by one point, Home and End go to the first and last point. Press Tab to leave.");
    }
  };

  return (
    <>
      <header>
        <h1>A02 accessibility prototype</h1>
        <p>Throwaway prototype for the screen-reader walkthroughs. Nothing here is product UI.</p>
      </header>
      <main>
        <section aria-labelledby="route-h">
          <h2 id="route-h">Announcement route</h2>
          <RadioGroup value={route} onChange={(v) => setRoute(v as Route)} aria-labelledby="route-h">
            <Radio value="live">ARIA live region</Radio>
            <Radio value="native">Native notification (Windows UI Automation)</Radio>
            <Radio value="both">Both</Radio>
          </RadioGroup>
        </section>

        <section aria-labelledby="dataset-h">
          <h2 id="dataset-h">Dataset</h2>
          <Tree aria-labelledby="dataset-h" selectionMode="single" defaultExpandedKeys={["dataset"]} onAction={(key) => say(`Opened ${String(key)}`)}>
            <Item id="dataset" label="temps.csv, 20 rows">
              <Item id="fields" label="Fields">
                <Item id="date" label="date, date" />
                <Item id="temperature" label="temperature, number, degrees Celsius" />
                <Item id="station" label="station, text" />
              </Item>
              <Item id="tracks" label="Tracks">
                <Item id="pitch" label="Temperature mapped to pitch" />
                <Item id="reference" label="Reference tone" />
              </Item>
            </Item>
          </Tree>
        </section>

        <section aria-labelledby="transport-h">
          <h2 id="transport-h">Transport</h2>
          <Toolbar aria-labelledby="transport-h">
            <Button onPress={() => (setPlaying(!playing), say(playing ? "Paused" : "Playing"))}>{playing ? "Pause" : "Play"}</Button>
            <Button onPress={() => (setPlaying(false), say("Stopped"))}>Stop</Button>
            <Separator orientation="vertical" />
            <ToggleButton onChange={(on) => say(on ? "Loop on" : "Loop off")}>Loop</ToggleButton>
            <Button onPress={() => say(`At ${point(cursor)}. Speed ${speed} times.`)}>Where am I</Button>
          </Toolbar>
          <Slider value={speed} onChange={setSpeed} minValue={0.25} maxValue={4} step={0.25} formatOptions={{ maximumFractionDigits: 2 }} className="slider">
            <Label>Playback speed, times real time</Label>
            <SliderOutput />
            <SliderTrack className="track">
              <SliderThumb className="thumb" />
            </SliderTrack>
          </Slider>
        </section>

        <section aria-labelledby="tests-h">
          <h2 id="tests-h">Announcement tests</h2>
          <div className="row">
            <Button onPress={() => say("Compiled 4 tracks in 120 milliseconds")}>Announce once</Button>
            <Button onPress={() => DATA.slice(0, 10).forEach((_, i) => say(`Rapid update ${i + 1} of 10`))}>Announce 10 rapid updates</Button>
            <Button onPress={() => say("Audio device lost. Playback stopped.", "assertive")}>Assertive alert</Button>
          </div>
        </section>

        <section aria-labelledby="chart-h">
          <h2 id="chart-h">Chart explorer</h2>
          <p id="chart-help">Temperature in March, 20 points. Focus the chart, then use Left and Right arrows, Home and End. Press H for help.</p>
          <div className="chart" role="application" aria-label="Chart explorer: temperature over time" aria-describedby="chart-help" tabIndex={0} onKeyDown={explore}>
            {DATA.map((v, i) => (
              <span key={i} aria-hidden="true" className={i === cursor ? "bar current" : "bar"} style={{ height: `${v * 4}px` }} />
            ))}
          </div>
        </section>

        <section aria-labelledby="log-h">
          <h2 id="log-h">Announcement log</h2>
          <p>What the page sent, newest first, to compare with what was spoken (for example in NVDA's Speech Viewer).</p>
          <ol className="log">
            {log.map((e) => (
              <li key={`${e.at}-${e.text}`}>
                {(e.at / 1000).toFixed(3)} s, {e.priority}, {e.result}: {e.text}
              </li>
            ))}
          </ol>
        </section>
      </main>
    </>
  );
}

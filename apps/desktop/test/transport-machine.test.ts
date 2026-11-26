// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The transport machine (B01-T05) mirrors the Rust transport: model-based tests against a
// fake app core, the Rust state winning over a conflicting intent, and playback starting
// only on a user command (A11Y-15).
import assert from "node:assert/strict";
import { test } from "node:test";

import type { TransportState } from "@sonarithm/ipc";
import { createActor, waitFor } from "xstate";

import type { TransportIntent } from "../src/transport/machine.ts";
import { requestFor, transportMachine } from "../src/transport/machine.ts";

const STOPPED: TransportState = {
  status: "stopped",
  positionS: 0,
  durationS: 4,
  speed: 1,
  loopRange: null,
  loaded: true,
  device: null,
  live: false,
  xruns: 0,
  muted: false,
  excludedTracks: [],
};

/** A small model of the Rust transport (06 §5): what each command does to the state. */
function rust(state: TransportState, intent: TransportIntent): TransportState {
  switch (intent.kind) {
    case "state":
      return state;
    case "play":
      return { ...state, status: "playing", device: "Null output" };
    case "pause":
      return state.status === "playing"
        ? { ...state, status: "paused" }
        : state;
    case "stop":
      return { ...state, status: "stopped", positionS: 0 };
    case "seek":
      return {
        ...state,
        positionS: intent.seconds,
        status: state.status === "stopped" ? "paused" : state.status,
      };
    case "speed":
      return { ...state, speed: intent.speed };
    case "loop":
      return { ...state, loopRange: intent.range };
  }
}

/** A fake app core: applies intents after a tick, records every command it receives. */
function fakeHost(initial: TransportState = STOPPED) {
  let state = initial;
  const commands: string[] = [];
  const reported = new Set<string>();
  return {
    commands,
    reported,
    get state() {
      return state;
    },
    async send(intent: TransportIntent): Promise<TransportState> {
      commands.push(requestFor(intent).command);
      await new Promise((resolve) => setTimeout(resolve, 0));
      state = rust(state, intent);
      reported.add(state.status);
      return state;
    },
  };
}

function start(host: ReturnType<typeof fakeHost>) {
  const actor = createActor(transportMachine, {
    input: { send: (intent) => host.send(intent) },
  });
  const shown = new Set<string>();
  actor.subscribe((snapshot) => {
    const value = snapshot.value as { status: string };
    shown.add(value.status);
  });
  actor.start();
  return { actor, shown };
}

const settled = (actor: ReturnType<typeof start>["actor"]) =>
  waitFor(actor, (snapshot) => snapshot.matches({ request: "idle" }));

const status = (actor: ReturnType<typeof start>["actor"]) =>
  (actor.getSnapshot().value as { status: string }).status;

const INTENTS: readonly TransportIntent[] = [
  { kind: "play" },
  { kind: "pause" },
  { kind: "stop" },
  { kind: "seek", seconds: 1.5 },
  { kind: "speed", speed: 2 },
  { kind: "loop", range: { startS: 0, endS: 2 } },
  { kind: "loop", range: null },
];

/** Every sequence of `length` intents. */
function sequences(length: number): TransportIntent[][] {
  if (length === 0) {
    return [[]];
  }
  return sequences(length - 1).flatMap((head) =>
    INTENTS.map((intent) => [...head, intent]),
  );
}

void test("b01_t05 the machine mirrors the app core for every sequence of three intents", async () => {
  for (const sequence of sequences(3)) {
    const host = fakeHost();
    const { actor, shown } = start(host);
    for (const intent of sequence) {
      actor.send({ type: "intent", intent });
      await settled(actor);
      // After each intent settles, the machine shows exactly what Rust reports.
      assert.equal(status(actor), host.state.status, JSON.stringify(sequence));
      assert.deepEqual(actor.getSnapshot().context.authority, host.state);
    }
    // One command per intent, in order, and nothing else.
    assert.deepEqual(
      host.commands,
      sequence.map((intent) => requestFor(intent).command),
    );
    // No optimistic status: the machine never showed one Rust did not report.
    for (const value of shown) {
      assert.ok(
        value === "unknown" || host.reported.has(value),
        `${value} in ${JSON.stringify(sequence)}`,
      );
    }
    actor.stop();
  }
});

void test("b01_t05 the rust state wins after a conflicting intent", async () => {
  // The user pauses, but the app core keeps playing (for example in live mode) and says so
  // in a clock anchor and in its reply: the machine shows playing.
  let release: (state: TransportState) => void = () => undefined;
  const actor = createActor(transportMachine, {
    input: {
      send: () =>
        new Promise<TransportState>((resolve) => {
          release = resolve;
        }),
    },
  }).start();
  const playing = { ...STOPPED, status: "playing" as const };
  actor.send({ type: "authority", state: playing });
  actor.send({ type: "intent", intent: { kind: "pause" } });
  assert.equal(status(actor), "playing", "an intent alone changes nothing");
  actor.send({ type: "authority", state: playing });
  release(playing);
  await settled(actor);
  assert.equal(status(actor), "playing");

  // A later anchor after the reply wins too: a device loss pauses playback.
  actor.send({ type: "authority", state: { ...playing, status: "paused" } });
  assert.equal(status(actor), "paused");
  actor.stop();
});

void test("a11y_15 playback starts only on a user command", async () => {
  const host = fakeHost();
  const { actor } = start(host);
  // State reports, including a loaded Score and a device, never make the machine play.
  actor.send({ type: "authority", state: STOPPED });
  actor.send({
    type: "authority",
    state: { ...STOPPED, status: "paused", positionS: 1 },
  });
  await new Promise((resolve) => setTimeout(resolve, 10));
  assert.deepEqual(host.commands, []);
  actor.send({ type: "intent", intent: { kind: "play" } });
  await settled(actor);
  assert.deepEqual(host.commands, ["transport_play"]);
  actor.stop();
});

void test("b01_t05 a newer intent replaces one waiting behind the request in flight", async () => {
  const host = fakeHost();
  const { actor } = start(host);
  actor.send({ type: "intent", intent: { kind: "play" } });
  actor.send({ type: "intent", intent: { kind: "pause" } });
  actor.send({ type: "intent", intent: { kind: "stop" } });
  await waitFor(
    actor,
    (s) => s.matches({ request: "idle" }) && host.commands.length === 2,
  );
  assert.deepEqual(host.commands, ["transport_play", "transport_stop"]);
  assert.equal(status(actor), "stopped");
  actor.stop();
});

void test("b01_t05 a failed intent keeps the last authoritative state", async () => {
  const actor = createActor(transportMachine, {
    input: {
      send: () => Promise.reject(new Error("No Score is loaded.")),
    },
  }).start();
  actor.send({ type: "authority", state: { ...STOPPED, loaded: false } });
  actor.send({ type: "intent", intent: { kind: "play" } });
  await waitFor(actor, (s) => s.context.failure !== null);
  assert.equal(status(actor), "stopped");
  assert.equal(actor.getSnapshot().context.failure, "No Score is loaded.");
  actor.stop();
});

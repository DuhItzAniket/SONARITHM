// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The Tauri transport only moves messages (OD-15): requests reach the host's generic
// commands unchanged, errors keep the app core's shape, and a JSON body where a binary
// view was expected fails loudly (F-042).
import assert from "node:assert/strict";
import { test } from "node:test";

import type { Event, MeterFrame } from "@sonarithm/ipc";
import { IpcFailure, METERS } from "@sonarithm/ipc";

import type { TauriCore } from "../src/ipc/transport.ts";
import { createTauriTransport } from "../src/ipc/transport.ts";

interface Call {
  readonly command: string;
  readonly args: Record<string, unknown>;
}

function fakeCore(answer: (call: Call) => unknown): TauriCore & {
  calls: Call[];
  channels: ((message: unknown) => void)[];
} {
  const calls: Call[] = [];
  const channels: ((message: unknown) => void)[] = [];
  return {
    calls,
    channels,
    // An async function, so a throwing answer becomes a rejection, as in Tauri.
    // eslint-disable-next-line @typescript-eslint/require-await
    async invoke(command, args) {
      calls.push({ command, args });
      return answer({ command, args });
    },
    channel(onMessage) {
      channels.push(onMessage);
      return { id: channels.length };
    },
  };
}

test("a command goes to ipc_dispatch unchanged", async () => {
  const core = fakeCore(() => ({
    reply: "hello",
    protocol: 1,
    appVersion: "t",
  }));
  const transport = createTauriTransport(core);
  const request = { command: "hello", args: { protocol: 1 } } as const;
  const reply = await transport.invoke(request);
  assert.deepEqual(core.calls, [
    { command: "ipc_dispatch", args: { request } },
  ]);
  assert.equal(reply.reply, "hello");
});

test("the app core's error keeps its code, hint and pointer", async () => {
  const core = fakeCore(() => {
    // Tauri rejects with the serialized IpcError object, not an Error.
    // eslint-disable-next-line @typescript-eslint/only-throw-error
    throw {
      code: "SNR-APP-002",
      message: "No dataset d7.",
      hint: "Open one.",
      pointer: null,
    };
  });
  const transport = createTauriTransport(core);
  await assert.rejects(
    transport.invoke({ command: "transport_play" }),
    (error) => {
      assert.ok(error instanceof IpcFailure);
      assert.equal(error.code, "SNR-APP-002");
      assert.equal(error.hint, "Open one.");
      return true;
    },
  );
});

test("a host refusal becomes SNR-APP-009", async () => {
  const core = fakeCore(() => {
    // Tauri's own refusals are plain strings.
    // eslint-disable-next-line @typescript-eslint/only-throw-error
    throw "Command ipc_dispatch not allowed by ACL";
  });
  await assert.rejects(
    createTauriTransport(core).invoke({ command: "transport_play" }),
    (error) => error instanceof IpcFailure && error.code === "SNR-APP-009",
  );
});

test("f_042_a_binary_view_must_arrive_as_an_array_buffer", async () => {
  const request = {
    command: "table_page",
    args: { dataset: "d1", offset: 0, limit: 10, sort: null },
  } as const;
  const binary = createTauriTransport(fakeCore(() => new ArrayBuffer(64)));
  assert.equal((await binary.binary(request)).byteLength, 64);
  const json = createTauriTransport(fakeCore(() => [83, 78, 82, 86]));
  await assert.rejects(
    json.binary(request),
    (error) => error instanceof IpcFailure && error.code === "SNR-APP-050",
  );
});

test("events reach the handler until unsubscribed, then the host is told", async () => {
  const core = fakeCore((call) =>
    call.command === "ipc_subscribe" ? 7 : null,
  );
  const seen: Event[] = [];
  const unsubscribe = createTauriTransport(core).subscribe("app", (event) =>
    seen.push(event),
  );
  const event: Event = { event: "playback_finished", durationS: 2 };
  core.channels[0]?.(event);
  unsubscribe();
  core.channels[0]?.(event);
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(seen.length, 1);
  assert.deepEqual(
    core.calls.map((call) => call.command),
    ["ipc_subscribe", "ipc_unsubscribe"],
  );
  assert.equal(core.calls.at(0)?.args.channel, "app");
  assert.deepEqual(core.calls.at(1)?.args, { id: 7 });
});

/** A meter frame in the layout of `crates/sonarithm-app/src/ipc/meters.rs`. */
function meterBytes(
  seq: number,
  version: number = METERS.version,
): ArrayBuffer {
  const buffer = new ArrayBuffer(METERS.bytes);
  const data = new DataView(buffer);
  for (let i = 0; i < METERS.magic.length; i += 1) {
    data.setUint8(METERS.offset.magic + i, METERS.magic.charCodeAt(i));
  }
  data.setUint16(METERS.offset.version, version, true);
  data.setBigUint64(METERS.offset.seq, BigInt(seq), true);
  data.setFloat32(METERS.offset.peakLeft, 0.5, true);
  return buffer;
}

test("b01_t04 meters arrive as binary frames on the meters channel", () => {
  const core = fakeCore(() => 1);
  const transport = createTauriTransport(core);
  const frames: MeterFrame[] = [];
  const events: Event[] = [];
  transport.meters((frame) => frames.push(frame));
  transport.subscribe("app", (event) => events.push(event));
  assert.equal(core.calls.at(0)?.args.channel, "meters");
  const [meters, app] = core.channels;
  meters?.(meterBytes(1));
  // A frame of another layout version is dropped; JSON never reaches the meter handler.
  meters?.(meterBytes(2, 2));
  meters?.({ event: "playback_finished", durationS: 1 });
  // A binary frame never reaches a JSON event handler.
  app?.(meterBytes(3));
  assert.deepEqual(
    frames.map((frame) => [frame.seq, frame.peak[0]]),
    [[1, 0.5]],
  );
  assert.equal(events.length, 0);
});

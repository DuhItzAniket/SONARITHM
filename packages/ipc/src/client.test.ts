// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import assert from "node:assert/strict";
import { test } from "node:test";

import { ProtocolMismatchError, connect } from "./client.ts";
import type { Event, TransportState } from "./generated/protocol.ts";
import { PROTOCOL_VERSION } from "./generated/protocol.ts";
import { createMockTransport } from "./mock.ts";
import { IpcFailure } from "./transport.ts";

const stopped: TransportState = {
  status: "stopped",
  positionS: 0,
  durationS: 4,
  speed: 1,
  loopRange: null,
  loaded: true,
  device: null,
};

void test("a version mismatch fails fast with a clear error", async () => {
  const transport = createMockTransport({}, { protocol: PROTOCOL_VERSION + 1 });
  await assert.rejects(connect(transport), (error: unknown) => {
    assert.ok(error instanceof ProtocolMismatchError);
    assert.match(error.message, /IPC protocol version 1/);
    assert.match(error.message, /same commit/);
    return true;
  });
  // Nothing but the handshake was sent.
  assert.deepEqual(
    transport.requests.map((request) => request.command),
    ["hello"],
  );
});

void test("calls resolve with the reply type of their command", async () => {
  const transport = createMockTransport({
    transport_seek: (request) => ({
      reply: "transport",
      ...stopped,
      status: "paused",
      positionS: request.args.seconds,
    }),
  });
  const client = await connect(transport);
  const state = await client.call({
    command: "transport_seek",
    args: { seconds: 2.5 },
  });
  assert.equal(state.reply, "transport");
  assert.equal(state.positionS, 2.5);
});

void test("host errors arrive typed", async () => {
  const client = await connect(createMockTransport());
  await assert.rejects(
    client.call({ command: "transport_play" }),
    (error: unknown) => {
      assert.ok(error instanceof IpcFailure);
      assert.equal(error.code, "SNR-APP-009");
      return true;
    },
  );
});

void test("a reply of the wrong kind is refused", async () => {
  const transport = createMockTransport({
    transport_state: () => ({ reply: "jobs", jobs: [] }) as never,
  });
  const client = await connect(transport);
  await assert.rejects(client.call({ command: "transport_state" }), IpcFailure);
});

void test("events reach the subscribers of their channel only", async () => {
  const transport = createMockTransport();
  const client = await connect(transport);
  const clock: Event[] = [];
  const jobs: Event[] = [];
  const stop = client.subscribe("clock", (event) => clock.push(event));
  client.subscribe("jobs", (event) => jobs.push(event));
  transport.emit({
    event: "clock",
    transport: stopped,
    hostTimeS: null,
    seq: 1,
  });
  transport.emit({ event: "job_progress", id: 1, progress: 0.5 });
  stop();
  transport.emit({
    event: "clock",
    transport: stopped,
    hostTimeS: null,
    seq: 2,
  });
  assert.equal(clock.length, 1);
  assert.equal(jobs.length, 1);
});

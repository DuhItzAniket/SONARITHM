// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * `@sonarithm/ipc`: the TypeScript types generated from Rust by ts-rs (in `src/generated/`,
 * never edited by hand) and the typed client the UI calls instead of a host API
 * (ADR-0011, ADR-0017, coding-standards §12.1). The UI imports protocol types only from here.
 */
export * from "./generated/protocol.ts";
export {
  type BinaryView,
  BinaryViewError,
  type ViewColumn,
  isValid,
  readView,
} from "./binary.ts";
export { type Client, ProtocolMismatchError, connect } from "./client.ts";
export { type EncodeColumn, type EncodeView, encodeView } from "./encode.ts";
export { type MeterFrame, MeterFrameError, readMeterFrame } from "./meters.ts";
export {
  type MockHandlers,
  type MockOptions,
  type MockTransport,
  createMockTransport,
} from "./mock.ts";
export { IpcFailure, type IpcTransport } from "./transport.ts";

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Tauri serves the built `dist/`; `tauri dev` uses the fixed port below (`tauri.conf.json`).
// The UI reads the Fluent catalogues from the Rust crate that owns them (12 §5), so the dev
// server may read the repository root.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    fs: { allow: ["../.."] },
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: { target: "es2023" },
});

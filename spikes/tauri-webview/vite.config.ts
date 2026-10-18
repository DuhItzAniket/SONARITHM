// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Tauri serves the built `dist/`; `tauri dev` uses the fixed port below. Two pages: the
// benchmark runner (index.html) and the accessibility prototype (a11y.html, A02-T07).
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true, watch: { ignored: ["**/src-tauri/**"] } },
  build: { target: "es2022", rollupOptions: { input: { main: "index.html", a11y: "a11y.html" } } },
});

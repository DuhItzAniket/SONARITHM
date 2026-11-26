// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * The Tauri host replaces this token in the page with a fresh nonce on every load and adds
 * that nonce to the CSP's `style-src` (tauri-utils `STYLE_NONCE_TOKEN`). Unreplaced, the page
 * is not served by the host (Vite, tests), which has no CSP.
 */
const TOKEN = "__TAURI_STYLE_NONCE__";

/**
 * The page's style nonce, from `<meta property="csp-nonce">` in index.html, for the
 * `<style>` element CodeMirror adds (09 §4). The CSP keeps `style-src 'self'` without
 * `'unsafe-inline'`; only that element gets the nonce. `undefined` outside the host.
 */
export function styleNonce(): string | undefined {
  const meta = document.querySelector<HTMLMetaElement>(
    'meta[property="csp-nonce"]',
  );
  // Browsers hide a nonce attribute's value once the page has a CSP; the property keeps it.
  const property = meta?.nonce ?? "";
  const nonce =
    property !== "" ? property : (meta?.getAttribute("nonce") ?? "");
  return nonce === "" || nonce === TOKEN ? undefined : nonce;
}

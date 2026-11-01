// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The smoke run's guard (B01-T03 smoke test; #436). A classic script, loaded before the
// app's modules: if a module fails to load or to run, no module code runs at all, and a
// smoke run would wait until CI's step limit with nothing to debug. In a smoke run only
// (the host sets `__SONARITHM_SMOKE__`), this sends a failing report instead:
//
// - a page error before the smoke run starts, with its message;
// - or no start within 60 s.
//
// Once `runSmoke` starts (`__SONARITHM_SMOKE_STARTED__`), its own watchdog takes over.
/* global window, setTimeout */
(function () {
  "use strict";
  if (
    typeof window.__SONARITHM_SMOKE__ !== "object" ||
    window.__SONARITHM_SMOKE__ === null
  ) {
    return;
  }
  var NEVER_STARTED_MS = 60000;
  var reported = false;

  function report(detail) {
    if (reported || window.__SONARITHM_SMOKE_STARTED__ === true) {
      return;
    }
    reported = true;
    var internals = window.__TAURI_INTERNALS__;
    if (internals === undefined || typeof internals.invoke !== "function") {
      return;
    }
    void internals.invoke("smoke_report", {
      results: {
        ok: false,
        checks: [{ name: "page-load", ok: false, detail: String(detail) }],
      },
    });
  }

  window.addEventListener("error", function (event) {
    var where = event.filename
      ? " (" + event.filename + ":" + String(event.lineno) + ")"
      : "";
    report(
      "page error before the smoke run started: " +
        String(event.message) +
        where,
    );
  });
  window.addEventListener("unhandledrejection", function (event) {
    report(
      "unhandled rejection before the smoke run started: " +
        String(event.reason),
    );
  });
  setTimeout(function () {
    report(
      "the smoke run did not start within 60 s: the app's modules did not load or run",
    );
  }, NEVER_STARTED_MS);
})();

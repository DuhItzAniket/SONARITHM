// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The `sonarithm` ESLint plugin: rules that keep UI text in the Fluent catalogues
// (coding-standards §12.7). The root eslint.config.js enables it for JSX and TSX files.
import noLiteralUiString from "./no-literal-ui-string.js";

export default {
  meta: { name: "sonarithm" },
  rules: { "no-literal-ui-string": noLiteralUiString },
};

// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// ESLint flat config for the frontend workspace (docs/engineering/coding-standards.md §12.2):
// typescript-eslint type-checked rule sets, jsx-a11y (strict), react-hooks. Prettier owns
// formatting. Packages run `eslint --max-warnings 0 .`, which finds this file.
import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import jsxA11y from "eslint-plugin-jsx-a11y";
import reactHooks from "eslint-plugin-react-hooks";
import tseslint from "typescript-eslint";

import sonarithm from "./packages/i18n/eslint/plugin.js";

export default defineConfig([
  globalIgnores(["**/dist/", "**/storybook-static/", "**/src/generated/"]),
  js.configs.recommended,
  tseslint.configs.strictTypeChecked,
  tseslint.configs.stylisticTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  jsxA11y.flatConfigs.strict,
  reactHooks.configs.flat.recommended,
  {
    rules: {
      // Required in addition to the strict preset (coding-standards §12.2).
      "jsx-a11y/control-has-associated-label": "error",
    },
  },
  {
    // UI text comes from the Fluent catalogues (coding-standards §12.7, NFR-09).
    files: ["**/*.{jsx,tsx}"],
    plugins: { sonarithm },
    rules: { "sonarithm/no-literal-ui-string": "error" },
  },
  {
    // node:test's test() and describe() return promises the runner awaits itself.
    files: ["**/test/**/*.ts"],
    rules: {
      "@typescript-eslint/no-floating-promises": [
        "error",
        {
          allowForKnownSafeCalls: [
            {
              from: "package",
              name: ["describe", "it", "suite", "test"],
              package: "node:test",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["**/*.{js,mjs,cjs}"],
    extends: [tseslint.configs.disableTypeChecked],
  },
]);

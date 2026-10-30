// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import "../src/generated/tokens.css";
import "../src/generated/fonts.css";

import type { Decorator, Preview } from "@storybook/react-vite";

import { THEMES, type Theme } from "../src/foundations/themes.ts";

/** Sets `data-theme` on the root, as the desktop host does from the theme setting. */
const withTheme: Decorator = (Story, context) => {
  const theme = context.globals.theme as Theme | undefined;
  document.documentElement.dataset.theme = theme ?? "dark";
  return <Story />;
};

const preview: Preview = {
  decorators: [withTheme],
  globalTypes: {
    theme: {
      description: "Theme (design-language §5.4)",
      toolbar: { icon: "contrast", items: [...THEMES], dynamicTitle: true },
    },
  },
  initialGlobals: { theme: "dark" },
  parameters: {
    // axe runs on every story; a violation fails its Vitest test (A11Y-05, A11Y-31).
    a11y: { test: "error" },
    layout: "padded",
  },
};

export default preview;

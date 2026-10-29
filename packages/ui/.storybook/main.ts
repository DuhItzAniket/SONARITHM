// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { StorybookConfig } from "@storybook/react-vite";

// The component workbench (B02-T03; testing-strategy §12): every component story, in every
// state and theme, runs as a Vitest browser test with an axe check (`vitest.config.ts`).
const config: StorybookConfig = {
  framework: "@storybook/react-vite",
  stories: ["../src/**/*.stories.tsx", "../test/seeded/*.stories.tsx"],
  addons: ["@storybook/addon-a11y", "@storybook/addon-vitest"],
  core: { disableTelemetry: true, disableWhatsNewNotifications: true },
};

export default config;

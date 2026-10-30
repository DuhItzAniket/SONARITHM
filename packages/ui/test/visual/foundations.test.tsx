// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Screenshot baselines of the tokens in every theme (B02-T03, testing-strategy §12). The
// references are taken in CI's pinned Linux Chromium image only; `__screenshots__` holds them.
import "../../src/generated/tokens.css";
import "../../src/generated/fonts.css";

import { expect, test } from "vitest";
import { render } from "vitest-browser-react";

import { storyLocalizer } from "../../.storybook/localizer.ts";
import { Foundations } from "../../src/foundations/Foundations.tsx";
import { THEMES } from "../../src/foundations/themes.ts";

const t = storyLocalizer();

for (const theme of THEMES) {
  test(`foundations in the ${theme} theme`, async () => {
    document.documentElement.dataset.theme = theme;
    const screen = await render(
      <Foundations
        sample={t.format("ui-workspace-empty")}
        readout={t.format("unit-hertz", {
          style: "symbol",
          value: { number: 440.25, minimumFractionDigits: 2 },
        })}
      />,
    );
    await document.fonts.ready;
    await expect
      .element(screen.getByTestId("foundations"))
      .toMatchScreenshot(`foundations-${theme}`);
  });
}

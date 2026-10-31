// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Prints the derived series colours (tokens/series.ts) as recipe lines, with their rendered
// colours and the closest pair under any dichromacy: `node tokens/series-print.ts`.
import { buildThemes } from "./build.ts";
import { resolve } from "./build.ts";
import { closestSeries, deriveAll, seriesThemes } from "./series.ts";

const derived = deriveAll(seriesThemes(buildThemes()));
for (const [theme, colors] of Object.entries(derived)) {
  const hexes = colors.map((color) => resolve(color).hex);
  console.log(
    `${theme}: closest pair ${closestSeries(hexes).distance.toFixed(3)} (${hexes.join(" ")})`,
  );
  colors.forEach((c, i) => {
    console.log(
      `  series-${String(i + 1)}: fixed(${String(c.l)}, ${c.c.toFixed(4)}, ${c.h.toFixed(2)})`,
    );
  });
}

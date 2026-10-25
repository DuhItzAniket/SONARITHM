// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Renders D3 charts to SVG for PLOTSONA's corpus (DS-007, C01-T07), with d3 7.9.0 (ISC)
// and jsdom 24.1.3 (MIT): dev-only tools from the C01-T02 table, never shipped.
// Usage: node d3_charts.mjs <node_modules folder> <input JSON> <output folder>
// The input lists charts: {name, kind: "line" | "bar" | "scatter", x, series: [{name, y}]}.

import { createRequire } from "node:module";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const [modules, input, out] = process.argv.slice(2);
const require = createRequire(join(modules, "noop.js"));
const d3 = await import(pathToFileURL(require.resolve("d3")).href);
const { JSDOM } = require("jsdom");

const WIDTH = 960;
const HEIGHT = 600;
const MARGIN = { top: 40, right: 160, bottom: 50, left: 70 };
const COLORS = ["#1f77b4", "#ff7f0e", "#2ca02c", "#d62728"];

for (const chart of JSON.parse(readFileSync(input, "utf8"))) {
  const dom = new JSDOM("<!DOCTYPE html><body></body>");
  const body = d3.select(dom.window.document.body);
  const svg = body
    .append("svg")
    .attr("xmlns", "http://www.w3.org/2000/svg")
    .attr("width", WIDTH)
    .attr("height", HEIGHT)
    .attr("viewBox", `0 0 ${WIDTH} ${HEIGHT}`);
  const g = svg.append("g").attr("transform", `translate(${MARGIN.left},${MARGIN.top})`);
  const w = WIDTH - MARGIN.left - MARGIN.right;
  const h = HEIGHT - MARGIN.top - MARGIN.bottom;
  const all = chart.series.flatMap((s) => s.y);
  const y = d3.scaleLinear().domain([Math.min(0, d3.min(all)), d3.max(all)]).nice().range([h, 0]);
  let x;
  if (chart.kind === "scatter") {
    x = d3.scaleLinear().domain(d3.extent(chart.x)).nice().range([0, w]);
  } else if (chart.kind === "bar") {
    x = d3.scaleBand().domain(chart.x).range([0, w]).padding(0.2);
  } else {
    x = d3.scalePoint().domain(chart.x).range([0, w]);
  }
  g.append("g").attr("transform", `translate(0,${h})`).call(d3.axisBottom(x));
  g.append("g").call(d3.axisLeft(y));
  svg
    .append("text")
    .attr("x", WIDTH / 2)
    .attr("y", 24)
    .attr("text-anchor", "middle")
    .attr("font-size", 18)
    .text(chart.title);
  chart.series.forEach((s, i) => {
    const color = COLORS[i % COLORS.length];
    if (chart.kind === "line") {
      const line = d3.line().x((d, j) => x(chart.x[j])).y((d) => y(d));
      g.append("path").attr("fill", "none").attr("stroke", color).attr("stroke-width", 2).attr("d", line(s.y));
    } else if (chart.kind === "bar") {
      const band = x.bandwidth() / chart.series.length;
      g.append("g")
        .attr("fill", color)
        .selectAll("rect")
        .data(s.y)
        .join("rect")
        .attr("x", (d, j) => x(chart.x[j]) + band * i)
        .attr("y", (d) => y(Math.max(0, d)))
        .attr("width", band)
        .attr("height", (d) => Math.abs(y(d) - y(0)));
    } else {
      g.append("g")
        .attr("fill", color)
        .selectAll("circle")
        .data(s.y)
        .join("circle")
        .attr("cx", (d, j) => x(chart.x[j]))
        .attr("cy", (d) => y(d))
        .attr("r", 3);
    }
    // Legend: a swatch and the series name, right of the plot.
    const legend = svg.append("g").attr("transform", `translate(${WIDTH - MARGIN.right + 20},${MARGIN.top + 20 * i})`);
    legend.append("rect").attr("width", 12).attr("height", 12).attr("fill", color);
    legend.append("text").attr("x", 18).attr("y", 10).text(s.name);
  });
  writeFileSync(join(out, `${chart.name}.svg`), body.html() + "\n");
}

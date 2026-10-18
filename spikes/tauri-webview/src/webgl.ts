// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// A02-T06: WebGL2 frame times for a 1M-point waveform and a scrolling spectrogram
// (PB-08: 60 fps playhead, ≤ 8 ms script per frame with a 1M-point series; notes/05 §6).

import { summarize } from "./stats";

const POINTS = 1_000_000;
const FRAMES = 600;
const SPEC_W = 2048;
const SPEC_H = 512;

function compile(gl: WebGL2RenderingContext, vs: string, fs: string): WebGLProgram {
  const prog = gl.createProgram();
  for (const [type, src] of [[gl.VERTEX_SHADER, vs], [gl.FRAGMENT_SHADER, fs]] as const) {
    const sh = gl.createShader(type);
    if (!sh) throw new Error("createShader failed");
    gl.shaderSource(sh, src);
    gl.compileShader(sh);
    if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(sh) ?? "shader");
    gl.attachShader(prog, sh);
  }
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(prog) ?? "link");
  return prog;
}

/** A deterministic test signal: two sines plus seeded noise (xorshift32). */
function signal(n: number): Float32Array {
  const y = new Float32Array(n);
  let s = 1729;
  for (let i = 0; i < n; i++) {
    s ^= s << 13;
    s ^= s >>> 17;
    s ^= s << 5;
    y[i] = 0.6 * Math.sin(i / 900) + 0.25 * Math.sin(i / 37) + 0.1 * ((s >>> 0) / 2 ** 32 - 0.5);
  }
  return y;
}

/** M4 aggregation (first, min, max, last per pixel column; Jugel et al. 2014). */
function m4(y: Float32Array, columns: number): Float32Array {
  const out = new Float32Array(columns * 4);
  const per = y.length / columns;
  for (let c = 0; c < columns; c++) {
    const a = Math.floor(c * per);
    const b = Math.max(a + 1, Math.floor((c + 1) * per));
    let lo = Infinity;
    let hi = -Infinity;
    for (let i = a; i < b; i++) {
      lo = Math.min(lo, y[i]);
      hi = Math.max(hi, y[i]);
    }
    out.set([y[a], lo, hi, y[b - 1]], c * 4);
  }
  return out;
}

const LINE_VS = `#version 300 es
in float y;
uniform float n, zoom, pan;
void main() {
  float x = (float(gl_VertexID) / (n - 1.0) - pan) * zoom * 2.0 - 1.0;
  gl_Position = vec4(x, y * 0.9, 0.0, 1.0);
}`;
const LINE_FS = `#version 300 es
precision mediump float;
out vec4 color;
void main() { color = vec4(0.84, 0.87, 0.9, 1.0); }`;
const QUAD_VS = `#version 300 es
out vec2 uv;
void main() {
  uv = vec2(float(gl_VertexID & 1), float(gl_VertexID >> 1));
  gl_Position = vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}`;
// Magnitudes are dB-scaled bytes in a ring buffer; a 256-entry LUT maps them to colour.
const SPEC_FS = `#version 300 es
precision mediump float;
in vec2 uv;
uniform sampler2D mag, lut;
uniform float head;
out vec4 color;
void main() {
  float m = texture(mag, vec2(fract(uv.x + head), uv.y)).r;
  color = texture(lut, vec2(m, 0.5));
}`;

interface FrameStats {
  frameMs: ReturnType<typeof summarize>;
  scriptMs: ReturnType<typeof summarize>;
  /** GPU time of the frame's draw calls (EXT_disjoint_timer_query_webgl2), if available. */
  gpuMs: ReturnType<typeof summarize> | null;
  fps: number;
  droppedFrames: number;
}

/** Runs `draw` once per animation frame for FRAMES frames and measures the frame interval,
 *  the script time and, where the timer-query extension exists, the GPU time. */
async function measure(gl: WebGL2RenderingContext, playhead: HTMLElement, draw: (t: number) => void): Promise<FrameStats> {
  const ext = gl.getExtension("EXT_disjoint_timer_query_webgl2");
  const frame: number[] = [];
  const script: number[] = [];
  const gpu: number[] = [];
  const pending: WebGLQuery[] = [];
  const poll = () => {
    while (ext && pending.length > 0 && gl.getQueryParameter(pending[0], gl.QUERY_RESULT_AVAILABLE)) {
      const q = pending.shift() as WebGLQuery;
      if (!gl.getParameter(ext.GPU_DISJOINT_EXT)) gpu.push(gl.getQueryParameter(q, gl.QUERY_RESULT) / 1e6);
      gl.deleteQuery(q);
    }
  };
  await new Promise<void>((done) => {
    let last = 0;
    let i = 0;
    const tick = (now: number) => {
      const t0 = performance.now();
      const q = ext ? gl.createQuery() : null;
      if (ext && q) gl.beginQuery(ext.TIME_ELAPSED_EXT, q);
      draw(i);
      if (ext && q) {
        gl.endQuery(ext.TIME_ELAPSED_EXT);
        pending.push(q);
      }
      // The playhead is a transformed element, so moving it never redraws the data layer.
      playhead.style.transform = `translateX(${(i % 600) * 2}px)`;
      script.push(performance.now() - t0);
      if (i > 0) frame.push(now - last);
      last = now;
      poll();
      i += 1;
      if (i < FRAMES || (pending.length > 0 && i < FRAMES + 60)) requestAnimationFrame(tick);
      else done();
    };
    requestAnimationFrame(tick);
  });
  const f = summarize(frame);
  return {
    frameMs: f,
    scriptMs: summarize(script),
    gpuMs: gpu.length > 0 ? summarize(gpu) : null,
    fps: Math.round(10_000 / f.mean) / 10,
    droppedFrames: frame.filter((x) => x > f.p50 * 1.5).length,
  };
}

export async function runWebgl(progress: (s: string) => void): Promise<Record<string, unknown>> {
  const canvas = document.querySelector<HTMLCanvasElement>("#plot");
  const playhead = document.querySelector<HTMLElement>("#playhead");
  if (!canvas || !playhead) throw new Error("page is missing #plot or #playhead");
  const dpr = window.devicePixelRatio;
  canvas.width = Math.round(canvas.clientWidth * dpr);
  canvas.height = Math.round(canvas.clientHeight * dpr);
  const gl = canvas.getContext("webgl2", { antialias: false, powerPreference: "high-performance" });
  if (!gl) throw new Error("WebGL2 is not available");
  const dbg = gl.getExtension("WEBGL_debug_renderer_info");
  const r: Record<string, unknown> = {
    renderer: dbg ? gl.getParameter(dbg.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER),
    vendor: dbg ? gl.getParameter(dbg.UNMASKED_VENDOR_WEBGL) : gl.getParameter(gl.VENDOR),
    canvas: [canvas.width, canvas.height],
    devicePixelRatio: dpr,
    gpuTimerQuery: gl.getExtension("EXT_disjoint_timer_query_webgl2") !== null,
  };
  gl.viewport(0, 0, canvas.width, canvas.height);

  const line = compile(gl, LINE_VS, LINE_FS);
  const y = signal(POINTS);
  const lineCase = (data: Float32Array, label: string) => {
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
    const loc = gl.getAttribLocation(line, "y");
    gl.enableVertexAttribArray(loc);
    gl.vertexAttribPointer(loc, 1, gl.FLOAT, false, 0, 0);
    progress(label);
    return (i: number) => {
      gl.useProgram(line);
      gl.bindVertexArray(vao);
      gl.uniform1f(gl.getUniformLocation(line, "n"), data.length);
      // Slow zoom and pan, so every frame redraws the whole series (the worst case).
      gl.uniform1f(gl.getUniformLocation(line, "zoom"), 1 + 0.5 * Math.sin(i / 90));
      gl.uniform1f(gl.getUniformLocation(line, "pan"), 0.1 * Math.sin(i / 70));
      gl.clearColor(0.11, 0.12, 0.14, 1);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.drawArrays(gl.LINE_STRIP, 0, data.length);
    };
  };
  const full = lineCase(y, "Waveform: 1M points, full redraw every frame");
  r.waveform1M = await measure(gl, playhead, full);
  // The recommended path (notes/05 §6): the data layer is drawn once; only the playhead moves.
  full(0);
  progress("Playhead over a static 1M-point waveform");
  r.playheadOver1M = await measure(gl, playhead, () => {});
  const t0 = performance.now();
  const reduced = m4(y, canvas.width);
  r.m4DownsampleMs = Math.round((performance.now() - t0) * 100) / 100;
  r.waveformM4 = await measure(gl, playhead, lineCase(reduced, "Waveform: M4 to pixel columns"));

  progress("Spectrogram: 2048 × 512, one new column per frame");
  const spec = compile(gl, QUAD_VS, SPEC_FS);
  const tex = (unit: number, w: number, h: number, fmt: number, ifmt: number, data: Uint8Array) => {
    const t = gl.createTexture();
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, t);
    gl.texImage2D(gl.TEXTURE_2D, 0, ifmt, w, h, 0, fmt, gl.UNSIGNED_BYTE, data);
    for (const p of [gl.TEXTURE_MIN_FILTER, gl.TEXTURE_MAG_FILTER]) gl.texParameteri(gl.TEXTURE_2D, p, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
    return t;
  };
  gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
  const lut = new Uint8Array(256 * 4).map((_, k) => (k % 4 === 3 ? 255 : Math.round((k >> 2) * [0.9, 0.6, 1.0][k % 4])));
  tex(1, 256, 1, gl.RGBA, gl.RGBA8, lut);
  tex(0, SPEC_W, SPEC_H, gl.RED, gl.R8, new Uint8Array(SPEC_W * SPEC_H));
  const column = new Uint8Array(SPEC_H);
  r.spectrogram = await measure(gl, playhead, (i) => {
    gl.bindVertexArray(null);
    for (let k = 0; k < SPEC_H; k++) column[k] = (128 + 120 * Math.sin(k / 9 + i / 15)) & 255;
    gl.activeTexture(gl.TEXTURE0);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, i % SPEC_W, 0, 1, SPEC_H, gl.RED, gl.UNSIGNED_BYTE, column);
    gl.useProgram(spec);
    gl.uniform1i(gl.getUniformLocation(spec, "mag"), 0);
    gl.uniform1i(gl.getUniformLocation(spec, "lut"), 1);
    gl.uniform1f(gl.getUniformLocation(spec, "head"), (i % SPEC_W) / SPEC_W);
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
  });
  return r;
}

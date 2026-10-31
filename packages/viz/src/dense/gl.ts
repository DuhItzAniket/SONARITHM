// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { LayerRenderer } from "./renderer.ts";
import type { DenseScene } from "./scene.ts";
import { normalize } from "./scene.ts";

// Positions arrive normalized to [0, 1] (`normalize`), so 32-bit floats keep full precision
// whatever the data's magnitude; the shaders only map them to clip space.
const LINE_VS = `#version 300 es
in float x;
in float y;
void main() {
  gl_Position = vec4(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
}`;

// A waveform column is two vertices, its minimum and maximum, drawn as one line; x comes
// from the vertex index, so only the values are uploaded.
const SPAN_VS = `#version 300 es
in float y;
uniform float columns;
void main() {
  float column = float(gl_VertexID / 2);
  float x = (column + 0.5) / columns;
  gl_Position = vec4(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
}`;

const COLOR_FS = `#version 300 es
precision mediump float;
uniform vec3 color;
out vec4 fragment;
void main() { fragment = vec4(color, 1.0); }`;

// A spectrogram is one quad; each fragment reads its magnitude byte and looks it up in the
// colormap (a 256 × 1 RGB texture).
const QUAD_VS = `#version 300 es
out vec2 uv;
void main() {
  uv = vec2(float(gl_VertexID & 1), float(gl_VertexID >> 1));
  gl_Position = vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}`;

const SPECTROGRAM_FS = `#version 300 es
precision mediump float;
in vec2 uv;
uniform sampler2D magnitudes;
uniform sampler2D lut;
out vec4 fragment;
void main() {
  float value = texture(magnitudes, vec2(uv.y, uv.x)).r;
  fragment = vec4(texture(lut, vec2(value * (255.0 / 256.0) + 0.5 / 256.0, 0.5)).rgb, 1.0);
}`;

function compile(
  gl: WebGL2RenderingContext,
  vertex: string,
  fragment: string,
): WebGLProgram {
  const program = gl.createProgram();
  for (const [type, source] of [
    [gl.VERTEX_SHADER, vertex],
    [gl.FRAGMENT_SHADER, fragment],
  ] as const) {
    const shader = gl.createShader(type);
    if (shader === null) {
      throw new Error("WebGL2 could not create a shader.");
    }
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (gl.getShaderParameter(shader, gl.COMPILE_STATUS) !== true) {
      throw new Error(
        gl.getShaderInfoLog(shader) ?? "shader compilation failed",
      );
    }
    gl.attachShader(program, shader);
  }
  gl.linkProgram(program);
  if (gl.getProgramParameter(program, gl.LINK_STATUS) !== true) {
    throw new Error(gl.getProgramInfoLog(program) ?? "program link failed");
  }
  return program;
}

/** Uploads `data` as the float attribute `name` of `program`. */
function attribute(
  gl: WebGL2RenderingContext,
  program: WebGLProgram,
  name: string,
  data: Float32Array,
  buffers: WebGLBuffer[],
): void {
  const location = gl.getAttribLocation(program, name);
  if (location < 0) return;
  const buffer = gl.createBuffer();
  buffers.push(buffer);
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
  gl.enableVertexAttribArray(location);
  gl.vertexAttribPointer(location, 1, gl.FLOAT, false, 0, 0);
}

/**
 * The WebGL2 renderer (B04-T02): one draw call per series, waveform or spectrogram. Data is
 * uploaded and drawn when the scene changes, never per frame; the playhead lives on its own
 * layer (notes/05 §6).
 */
export class GlRenderer implements LayerRenderer {
  readonly backend = "webgl2" as const;
  draws = 0;
  readonly #gl: WebGL2RenderingContext;
  readonly #line: WebGLProgram;
  readonly #span: WebGLProgram;
  readonly #spectrogram: WebGLProgram;
  #buffers: WebGLBuffer[] = [];
  #textures: WebGLTexture[] = [];

  constructor(gl: WebGL2RenderingContext) {
    this.#gl = gl;
    this.#line = compile(gl, LINE_VS, COLOR_FS);
    this.#span = compile(gl, SPAN_VS, COLOR_FS);
    this.#spectrogram = compile(gl, QUAD_VS, SPECTROGRAM_FS);
  }

  draw(scene: DenseScene): void {
    const gl = this.#gl;
    this.#release();
    gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);
    switch (scene.kind) {
      case "lines":
        gl.useProgram(this.#line);
        for (const series of scene.series) {
          attribute(
            gl,
            this.#line,
            "x",
            normalize(series.x, scene.x),
            this.#buffers,
          );
          attribute(
            gl,
            this.#line,
            "y",
            normalize(series.y, scene.y),
            this.#buffers,
          );
          gl.uniform3fv(
            gl.getUniformLocation(this.#line, "color"),
            series.color,
          );
          gl.drawArrays(gl.LINE_STRIP, 0, series.x.length);
        }
        break;
      case "waveform": {
        gl.useProgram(this.#span);
        const columns = Math.min(scene.min.length, scene.max.length);
        const ends = new Float32Array(columns * 2);
        const lows = normalize(scene.min, scene.y);
        const highs = normalize(scene.max, scene.y);
        for (let i = 0; i < columns; i += 1) {
          ends[i * 2] = lows[i] ?? 0;
          ends[i * 2 + 1] = highs[i] ?? 0;
        }
        attribute(gl, this.#span, "y", ends, this.#buffers);
        gl.uniform1f(gl.getUniformLocation(this.#span, "columns"), columns);
        gl.uniform3fv(gl.getUniformLocation(this.#span, "color"), scene.color);
        gl.drawArrays(gl.LINES, 0, columns * 2);
        break;
      }
      case "spectrogram": {
        gl.useProgram(this.#spectrogram);
        gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
        // The magnitudes as an R8 texture of `rows` × `columns` texels: each column of the
        // spectrogram is one texture row, so the bytes upload as they are.
        this.#texture(
          0,
          gl.R8,
          gl.RED,
          scene.rows,
          scene.columns,
          scene.magnitudes,
        );
        this.#texture(1, gl.RGB8, gl.RGB, 256, 1, scene.lut);
        gl.uniform1i(gl.getUniformLocation(this.#spectrogram, "magnitudes"), 0);
        gl.uniform1i(gl.getUniformLocation(this.#spectrogram, "lut"), 1);
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
        break;
      }
    }
    gl.bindVertexArray(null);
    gl.deleteVertexArray(vao);
    this.draws += 1;
  }

  #texture(
    unit: number,
    internal: number,
    format: number,
    width: number,
    height: number,
    data: Uint8Array,
  ): void {
    const gl = this.#gl;
    const texture = gl.createTexture();
    this.#textures.push(texture);
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      internal,
      width,
      height,
      0,
      format,
      gl.UNSIGNED_BYTE,
      data,
    );
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  }

  /** Frees the previous scene's buffers and textures. */
  #release(): void {
    for (const buffer of this.#buffers) this.#gl.deleteBuffer(buffer);
    for (const texture of this.#textures) this.#gl.deleteTexture(texture);
    this.#buffers = [];
    this.#textures = [];
  }

  destroy(): void {
    this.#release();
    for (const program of [this.#line, this.#span, this.#spectrogram]) {
      this.#gl.deleteProgram(program);
    }
  }
}

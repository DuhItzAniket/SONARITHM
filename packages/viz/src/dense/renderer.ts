// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { CanvasRenderer } from "./canvas.ts";
import { GlRenderer } from "./gl.ts";
import type { DenseScene, Rgb } from "./scene.ts";

/** Draws a {@link DenseScene} into a canvas. */
export interface LayerRenderer {
  /** Which API draws: WebGL2, or the Canvas2D fallback. */
  readonly backend: "webgl2" | "canvas2d";
  /** Scenes drawn so far; the data layer redraws only when its scene or size changes. */
  readonly draws: number;
  draw(scene: DenseScene): void;
  destroy(): void;
}

/**
 * A renderer for `canvas`: WebGL2 where the webview has it, otherwise the Canvas2D fallback
 * (B04-T02). `forceCanvas` takes the fallback anyway, for tests and as a setting if a GPU
 * driver misbehaves. Returns `null` only if the canvas has neither context.
 */
export function createRenderer(
  canvas: HTMLCanvasElement,
  options: { readonly forceCanvas?: boolean } = {},
): LayerRenderer | null {
  if (options.forceCanvas !== true && webgl2Works()) {
    const gl = canvas.getContext("webgl2", GL_OPTIONS);
    if (gl !== null) {
      return new GlRenderer(gl);
    }
  }
  const context = canvas.getContext("2d");
  return context === null ? null : new CanvasRenderer(context);
}

const GL_OPTIONS: WebGLContextAttributes = {
  antialias: false,
  premultipliedAlpha: true,
  // Kept, so the canvas can be read back (tests, screenshots) after the frame.
  preserveDrawingBuffer: true,
};

let probed: boolean | null = null;

/**
 * Whether WebGL2 works here: a context opens and the shaders compile, tried once on a
 * throwaway canvas. A canvas that took a WebGL2 context cannot give a 2D one, so the real
 * canvas asks for the context that will work.
 */
export function webgl2Works(): boolean {
  if (probed === null) {
    const gl = document
      .createElement("canvas")
      .getContext("webgl2", GL_OPTIONS);
    try {
      // Compiling the programs is the test; the renderer is then discarded.
      if (gl !== null) new GlRenderer(gl).destroy();
      probed = gl !== null;
    } catch {
      probed = false;
    }
    gl?.getExtension("WEBGL_lose_context")?.loseContext();
  }
  return probed;
}

/**
 * A CSS colour (a token's computed value, in any syntax the webview supports, such as
 * `oklch()`) as sRGB channels in [0, 1], by drawing it into a 1 × 1 canvas.
 */
export function cssColor(value: string): Rgb {
  const canvas = document.createElement("canvas");
  canvas.width = 1;
  canvas.height = 1;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  if (ctx === null) {
    return [0, 0, 0];
  }
  ctx.fillStyle = value;
  ctx.fillRect(0, 0, 1, 1);
  const [r = 0, g = 0, b = 0] = ctx.getImageData(0, 0, 1, 1).data;
  return [r / 255, g / 255, b / 255];
}

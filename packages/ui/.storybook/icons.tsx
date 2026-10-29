// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Stand-in 16 px glyphs for stories, drawn after Lucide's play, square and repeat
// (design-language §10). The app takes Lucide itself with its ISC notice.

function Glyph({ d }: { readonly d: string }) {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" focusable="false">
      <path
        d={d}
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export const PlayGlyph = () => <Glyph d="M5 3l8 5-8 5z" />;
export const StopGlyph = () => <Glyph d="M4 4h8v8H4z" />;
export const LoopGlyph = () => (
  <Glyph d="M11 2l2 2-2 2M3 8V6a2 2 0 012-2h8M5 14l-2-2 2-2M13 8v2a2 2 0 01-2 2H3" />
);

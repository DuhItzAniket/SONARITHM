# Bundled fonts

IBM Plex Sans and IBM Plex Mono at 400, 500 and 600 ([design-language §7.1](../../../docs/design/design-language.md#71-typography)), bundled so that no font loads from the network and every platform renders the same glyphs.

| Files                                         | Source                                                                              | Licence |
| --------------------------------------------- | ----------------------------------------------------------------------------------- | ------- |
| `IBMPlexSans-{Regular,Medium,SemiBold}.woff2` | npm `@ibm/plex-sans` 1.1.0, `fonts/complete/woff2/` (font version 3.005)            | OFL-1.1 |
| `IBMPlexMono-{Regular,Medium,SemiBold}.woff2` | npm `@ibm/plex-mono` 2.5.0, `fonts/complete/woff2/` (font version 2.005)            | OFL-1.1 |
| `LICENSE.txt`                                 | Both packages (identical): IBM's copyright notice and the SIL Open Font License 1.1 | OFL-1.1 |

**The files are IBM's, unmodified.** They are not subset, renamed or converted, so the OFL's Reserved Font Name "Plex" stays with IBM's own font. `.gitattributes` keeps their exact bytes. The package tests check each file's SHA-256 against the upstream value in `../tokens/fonts.ts`, and check that the font names and versions are still IBM's. `REUSE.toml` licenses these files, and only these, as OFL-1.1, as the [supply-chain policy](../../../docs/engineering/supply-chain-and-licensing.md#assets) requires for fonts.

The IBM packages are not dependencies of the workspace: their `postinstall` script sends install telemetry to IBM (`@ibm/telemetry-js`), which the privacy rule does not allow ([ADR-0012](../../../docs/decisions/0012-privacy-and-telemetry.md)).

## Updating

1. Download the tarballs without running their scripts: `npm pack @ibm/plex-sans@<version>` and `npm pack @ibm/plex-mono@<version>`. `npm pack` runs no install scripts.
2. Extract them, and copy the six files from `package/fonts/complete/woff2/` and `package/LICENSE.txt` here, unchanged.
3. In `../tokens/fonts.ts`, update `FONT_SOURCES` (package version, tarball integrity, font version) and each file's SHA-256.
4. Run `pnpm -C packages/ui tokens`, then `pnpm -C packages/ui test`.

# Development environment

> **Status:** Baseline v1 (2026-10-18). **Owner:** Architecture lead.
> **Related:** [CONTRIBUTING.md](../../CONTRIBUTING.md) · [AGENTS.md](../../AGENTS.md) · [tech-stack](tech-stack.md) · [coding-standards](coding-standards.md) · [testing-strategy](testing-strategy.md) · [ci-cd](ci-cd.md) · [02-repository-and-crate-layout](../architecture/02-repository-and-crate-layout.md)

This page lists what to install to build, test and debug SONARITHM on Windows, macOS and Linux. The Cargo workspace, `rust-toolchain.toml`, the pnpm workspace and `cargo xtask` are created in phase A01. Commands that need them are marked with the phase from which they work.

## What you need for which work

| Work | Rust toolchain | C/C++ build tools | Node.js + pnpm | Python + maturin | Webview runtime and system libraries | Nightly Rust |
|---|---|---|---|---|---|---|
| Core crates, CLI | Yes | Yes | — | — | Audio headers (Linux) | — |
| Desktop app (B01 onward) | Yes | Yes | Yes | — | Yes | — |
| Python bindings (A07 onward) | Yes | Yes | — | Yes | — | — |
| Fuzzing, Miri | Yes | Yes | — | — | — | Yes |
| Accessibility testing | — | — | — | — | Screen readers per OS | — |

## Prerequisites on every OS

| Tool | Version | Notes |
|---|---|---|
| Git | Recent | Git LFS is needed only if A05 confirms LFS for golden audio ([testing-strategy §5.3](testing-strategy.md#53-golden-file-policy)). |
| rustup | Recent | Do not install a toolchain by hand. `rust-toolchain.toml` (A01) pins the stable release used by CI; the first `cargo` command in the repository installs it with the components and targets it lists (rustfmt, clippy, `wasm32-unknown-unknown`). Core crates must also build on the MSRV (1.88), which CI checks for you ([coding-standards §1](coding-standards.md#1-rust-toolchain-edition-and-msrv)). |
| Nightly Rust | Any recent nightly | Only for Miri and cargo-fuzz: `rustup toolchain install nightly --component miri`. Never used for shipped builds. |
| Node.js | Range in the root `package.json` `engines` field (A01) | The baseline development machine runs Node 25, a Current line. Odd-numbered Node lines never become LTS; CI should use an Active LTS line within the accepted range ([tech-stack](tech-stack.md)). |
| pnpm | Version pinned by the root `package.json` `packageManager` field (A01) | Install with the official standalone installer or `npm install -g pnpm`. |
| Python | 3.10 or newer | The development floor and the abi3 minimum of released wheels (A07-T03). Build locally with `maturin develop --uv` in `bindings/python` inside a virtual environment, then `pytest bindings/python/tests` ([coding-standards §13](coding-standards.md#13-python-bindings)). |
| uv | Recent | Creates virtual environments and installs Python tools: `uv venv`, `uv tool install maturin`. |
| maturin | Version pinned in `bindings/python/pyproject.toml` (A07) | Builds the Python extension. `maturin develop` requires an activated virtual environment. |

## Windows

Supported: Windows 10 and 11 (cpal requires Windows 10 or later; Tauri 2.12 dropped Windows 7).

1. **Microsoft C++ Build Tools** with the "Desktop development with C++" workload (MSVC and the Windows SDK). The Rust MSVC toolchain and Tauri both need it. Use the `x86_64-pc-windows-msvc` host toolchain (rustup's default on Windows).
2. **WebView2 Runtime** (Evergreen). Included with Windows 11 and current Windows 10 builds; otherwise install the Evergreen runtime from Microsoft. Needed to run the desktop app.
3. **Git for Windows** with LF line endings preserved: `git config --global core.autocrlf false`. If you hit path-length errors in `target/` or `node_modules/`, enable `core.longpaths`.
4. **Optional: ASIO.** The `asio` feature of `sonarithm-audio` is off by default ([02 §4](../architecture/02-repository-and-crate-layout.md#4-feature-flags-planned)). Building it needs the Steinberg ASIO SDK and LLVM/Clang (for bindgen); follow cpal's ASIO instructions for pointing the build at the SDK. Never commit the SDK to the repository. Known issue: a failed ASIO load can crash the build even though the feature should be optional ([cpal #1083](https://github.com/RustAudio/cpal/issues/1083)).

Build speed: a Dev Drive or an antivirus exclusion for the repository's `target/` directory can shorten builds considerably. Follow your organization's security policy before changing antivirus settings.

## macOS

1. **Xcode Command Line Tools**: `xcode-select --install`. Full Xcode is not needed for desktop development.
2. WKWebView ships with macOS; there is nothing else to install for the webview.
3. **Microphone permission.** The latency harness records from an input device ([testing-strategy §14](testing-strategy.md#14-latency-measurement)). macOS asks for microphone access for the terminal or app running it; grant it in System Settings → Privacy & Security → Microphone.

## Linux

Package names below are for Debian and Ubuntu. For other distributions, use the [Tauri prerequisites page](https://v2.tauri.app/start/prerequisites/) for the webview stack; the Fedora equivalents of the audio and speech packages are `alsa-lib-devel`, `pipewire-devel` and `speech-dispatcher-devel`.

| Purpose | Packages | Needed when |
|---|---|---|
| Build basics | `build-essential pkg-config curl wget file` | Always |
| Tauri webview stack (per Tauri prerequisites; check the current page) | `libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev` | Desktop app (B01) |
| OpenSSL headers | `libssl-dev` | Only if a dependency links OpenSSL. Tauri's list includes it; the project prefers rustls where there is a choice |
| ALSA headers | `libasound2-dev` | Any crate that builds cpal (`sonarithm-audio`, A05) |
| PipeWire headers and libclang | `libpipewire-0.3-dev libclang-dev` | If the cpal PipeWire host is compiled in; confirmed when A05 fixes the cpal feature set |
| JACK headers | `libjack-jackd2-dev` | Only with the `jack` feature |
| Speech | `libspeechd-dev speech-dispatcher espeak-ng` | Self-voicing through `sonarithm-speech` (B03) |
| Screen reader | `orca` | Accessibility testing |

Notes:

- cpal 0.18 prefers PipeWire, then PulseAudio, then ALSA on Linux ([04-rust-core-stack §1](../research/notes/04-rust-core-stack.md#1-audio-io-cpal-status-backends-latency-higher-layers-rodio-kira-oddio-firewheel-interflow-others-which-suits-low-latency-interactive-playback-plus-offline-rendering)).
- Keep both an X11 and a Wayland session available if you test with Orca; the release matrix covers both.

## Recommended tools

We use **`cargo xtask`** as the task runner: it is a plain Rust binary in `xtask/`, so it behaves the same on every OS. There is no `just`, `make` or shell-script layer for Rust tasks.

Install Cargo tools with `cargo install --locked <tool>`, which builds from source with the published lockfile.

| Tool | Purpose | Needed for |
|---|---|---|
| cargo-nextest | Test runner used locally and in CI | All Rust work |
| cargo-insta | Reviewing Score IR and CLI snapshot changes (`cargo insta review`) | Compiler, engine and CLI work |
| cargo-deny | Licence, advisory, ban and source checks | Adding or upgrading dependencies |
| cargo-machete | Finding unused dependencies | Dependency hygiene |
| cargo-semver-checks | Detecting public API breaks | Changes to published crates |
| cargo-fuzz | Fuzzing with libFuzzer (nightly; use Linux or macOS) | Parsers and importers |
| Miri (nightly component) | Undefined-behaviour checks | Unsafe and lock-free code |
| maturin (via `uv tool install maturin`) | Building the Python extension | Python bindings |
| Tauri CLI | Desktop dev server and bundling | Installed as a dev dependency of `apps/desktop` through pnpm; no global install |

## Editor setup

Any editor with rust-analyzer works. For VS Code, these user or workspace settings match the project's tooling:

```jsonc
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.allTargets": true,
  "rust-analyzer.cargo.targetDir": true,        // separate target dir: no lock contention with your own cargo runs
  // Keep default features. Enabling all features pulls in `asio`, which needs the ASIO SDK.
  "editor.formatOnSave": true,
  "files.eol": "\n",
  "[rust]": { "editor.defaultFormatter": "rust-lang.rust-analyzer" },
  "[typescript][typescriptreact][css][json][jsonc][yaml][markdown]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[python]": { "editor.defaultFormatter": "charliermarsh.ruff" }
}
```

Recommended VS Code extensions:

| Extension ID | Purpose |
|---|---|
| `rust-lang.rust-analyzer` | Rust language server |
| `tamasfe.even-better-toml` | `Cargo.toml` and other TOML files |
| `vadimcn.vscode-lldb` | Debugging on macOS and Linux (on Windows use `ms-vscode.cpptools` for MSVC builds) |
| `dbaeumer.vscode-eslint` | ESLint |
| `esbenp.prettier-vscode` | Prettier |
| `stylelint.vscode-stylelint` | Stylelint for CSS Modules |
| `vitest.explorer` | Vitest |
| `ms-playwright.playwright` | Playwright |
| `tauri-apps.tauri-vscode` | Tauri configuration |
| `ms-python.python`, `charliermarsh.ruff` | Python and ruff |
| `EditorConfig.EditorConfig` | `.editorconfig` support |
| `DavidAnson.vscode-markdownlint`, `bierner.markdown-mermaid` | Docs linting and Mermaid preview |

The repository does not include a `.vscode/` directory. Adding one (for example `extensions.json`) is a new top-level directory and needs a PR that updates [02 §1](../architecture/02-repository-and-crate-layout.md#1-repository-tree-target).

## Common commands

All commands run from the repository root unless stated. Available after A01; the "Works from" column gives the phase in which each command has something to act on.

| Command | What it does | Works from |
|---|---|---|
| `cargo xtask ci` | Runs the local equivalent of the required CI checks (format, clippy, tests, doctests, cargo-deny, dependency rules) | A01 |
| `cargo nextest run` | Runs the Rust test suite (add `-p <crate>` to narrow) | A01 |
| `cargo test --doc` | Runs doctests, which nextest does not execute | A01 |
| `cargo xtask check-deps` | Checks the crate dependency rules in [02 §3](../architecture/02-repository-and-crate-layout.md#3-dependency-rules-enforced) | A01 |
| `pnpm install` | Installs the JavaScript workspace | A01 |
| `cargo insta review` | Reviews pending snapshot changes interactively | A03 |
| `cargo xtask codegen` | Regenerates all generated artifacts: JSON Schemas in `schemas/` (A03) and TypeScript IPC types in `packages/ipc/src/generated/` (B01) | A03 |
| `cargo xtask codes` | Checks the diagnostic code tables and writes `docs/reference/error-codes.md` (also run by `codegen`) | A03 |
| `cargo +nightly fuzz run <target>` | Runs a fuzz target (from the owning crate's directory) | A03 |
| `cargo xtask golden --update` | Re-renders golden references; state the reason in the PR ([testing-strategy §5.3](testing-strategy.md#53-golden-file-policy)) | A05 |
| `cargo +nightly miri test -p <crate>` | Runs a crate's tests under Miri | A05 |
| `maturin develop -m bindings/python/Cargo.toml` | Builds the Python module and installs it into the active virtual environment | A07 |
| `pytest` (from `bindings/python`) | Runs the Python tests against the installed module | A07 |
| `pnpm -C apps/desktop tauri dev` | Runs the desktop app with the frontend dev server and the Rust host | B01 |

## Disk usage

Rust build output in `target/` grows quickly. Every worktree has its own, and cargo keeps outdated artifacts after each rebase. On 2026-10-19, four agent worktrees held about 70 GB, most of it debug info and incremental caches ([F-090](../research-paper/findings-log.md)).

- **Debug info is trimmed by default.** The root `Cargo.toml` builds our crates with line tables only, so backtraces still show file and line, and builds dependencies without debug info. This roughly halves a clean test build (982 MB to 528 MB at the time of the change). For full debug info in a debugger session, build with `--config 'profile.dev.debug="full"' --config 'profile.dev.package."*".debug="full"'`.
- **Incremental caches are disposable.** Delete `target/*/incremental` (and `target/<triple>/*/incremental`) at any time. The next build is slower once.
- **Clean up after a branch or phase lands.** Run `cargo clean` in its worktree. Fuzz targets keep their own target directory: `cargo clean --manifest-path crates/<crate>/fuzz/Cargo.toml`.

## Audio devices during development

- **Use headphones.** Closed-back headphones avoid feedback during loopback tests, keep the sound to yourself and make clicks audible (every note needs onset and offset ramps, SDR-22).
- **Start at low gain.** Before playing output from changed DSP, gain or limiter code, turn the OS volume and the app's sonification volume down. The engine starts the master at −20 dB with a ramp and limits at −1 dBTP ([06 §10](../architecture/06-audio-engine.md#10-hearing-safety-and-levels)), but a bug in the code under test can bypass both.
- **Match the device rate.** Set the output device to 48 kHz, the render default, so the OS does not resample. On Windows, turn off audio enhancements and spatial sound for the device used in measurements.
- **Use wired outputs for latency work.** Bluetooth adds large and variable latency.
- **Virtual loopback devices** let the harness capture output in software: VB-Audio Virtual Cable (Windows), BlackHole (macOS), a PipeWire loopback or null sink, or the ALSA `snd-aloop` module (Linux). They measure only the software path. Budget sign-off uses a physical loopback cable on an audio interface ([testing-strategy §14](testing-strategy.md#14-latency-measurement)). These are local test tools, not project dependencies.
- **Test the budgeted buffer sizes.** 256 frames for interactive latency and 128 frames for soak tests ([01 §7](../architecture/01-overview.md#7-quality-attributes-and-budgets)).
- **Check mono.** Turn on the OS mono-audio accessibility setting now and then. Essential information must survive mono output (A11Y-14).
- **Real-time scheduling on Linux.** If cpal reports `RealtimeDenied`, check that rtkit is running or that your user has real-time limits configured.

## Screen readers for testing

| Screen reader | OS | Cost | Toggle | Notes |
|---|---|---|---|---|
| NVDA | Windows | Free | Ctrl+Alt+N (installed copy); NVDA+Q to quit | Use an installed copy: audio ducking does not work in portable mode. Audio settings: NVDA+Ctrl+U. Sound split: NVDA+Alt+S. Tools → Speech viewer shows spoken text, which helps when recording results |
| JAWS | Windows | Commercial; runs in time-limited demo sessions without a licence | Desktop shortcut | Part of the release matrix |
| Narrator | Windows | Built in | Win+Ctrl+Enter | |
| VoiceOver | macOS | Built in | Cmd+F5 | VoiceOver Utility controls verbosity |
| Orca | Linux (GNOME) | Free | Super+Alt+S | Test in both X11 and Wayland sessions |

Braille: NVDA, JAWS and VoiceOver drive braille displays directly; Orca uses BRLTTY. The release matrix needs one refreshable display ([testing-strategy §13.2](testing-strategy.md#132-manual-screen-reader-matrix-every-release-a11y-32)).

Learn the basic navigation commands of a screen reader before judging an interaction with it. A failure caused by not knowing the screen reader is not a product finding.

## Troubleshooting

Add an entry when a setup problem costs someone more than 30 minutes.

| Symptom | Likely cause | Fix |
|---|---|---|
| `link.exe` not found (Windows) | MSVC Build Tools missing or incomplete | Install the "Desktop development with C++" workload |
| Blank or very slow desktop window on Linux with NVIDIA | WebKitGTK DMABUF renderer and driver conflict | Set `WEBKIT_DISABLE_DMABUF_RENDERER=1`; this trades speed for stability ([Tauri Linux graphics](https://v2.tauri.app/develop/debug/linux-graphics/)) |
| "Could not determine the accessibility bus address" warning (Linux) | No AT-SPI bus, common in containers and minimal sessions | Usually harmless ([tauri #4315](https://github.com/tauri-apps/tauri/issues/4315)); for Orca testing use a full desktop session |
| Build fails with the `asio` feature | ASIO SDK or LLVM not found | To be documented in A05 |
| No audio device found (VM, container, CI) | No output device available | Use the null or offline backend; device tests run only on the dedicated jobs |
| `maturin develop` refuses to install | No active virtual environment | `uv venv`, activate it, rerun |
| `cargo fuzz` fails on Windows | libFuzzer tooling targets Linux and macOS | Fuzz on Linux, macOS or WSL |
| *(placeholder)* | | |

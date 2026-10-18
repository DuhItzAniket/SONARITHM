---
status: proposed
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0009 · WebAssembly component plugins, compile time only

## Context and problem statement

Researchers will want custom mappings, analyzers and sources without forking SONARITHM. How should third-party engines be loaded safely?

## Decision drivers

* Safety: plugins handle user data and must not compromise the machine.
* Real-time safety: nothing third-party on the audio thread.
* Language neutrality: researchers use Rust, C, Python and others.
* ABI stability across SONARITHM versions.

## Considered options

* WebAssembly components (wasmtime + WIT; WASI 0.3 released June 2026)
* Native dynamic libraries (`libloading`, `abi_stable`)
* Embedded scripting (rhai, Lua) for plugins
* Extism

## Decision outcome

Proposed option: **WebAssembly components via wasmtime with a versioned WIT world**, running **only at compile time and in control-thread live encoding**, sandboxed (no FS or network), with fuel and epoch limits ([11 §5](../architecture/11-bindings-cli-plugins.md#5-plugin-system-d05)). Rhai remains for small user expressions inside specs.

### Consequences

* Good: sandboxed, typed and language-agnostic, with a stable interface.
* Bad: wasmtime has a heavy dependency footprint (MSRV 1.96), so it lives in the leaf crate `sonarithm-plugin-host`.
* Bad: plugins cannot provide DSP voices in v1.

### Confirmation

Decided in D05 after a prototype plugin (a custom Encoder) builds from Rust and one other language and passes the sandbox tests. Status moves to `accepted` or is revised then.

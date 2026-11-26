# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Decides which CI jobs a change needs, and checks the result (docs/engineering/ci-cd.md).

Two commands, standard library only (Python 3.11 or later, for tomllib):

``changes --event EVENT``
    Reads the changed paths, one per line, on stdin (``git diff --name-only --no-renames``
    of the pull request's merge commit against its base). Prints ``key=value`` lines for
    ``$GITHUB_OUTPUT``:

    - ``kind``: ``everything``, ``rust``, ``frontend`` or ``docs``, for the job summary;
    - ``rust``: ``true`` when the Rust jobs run;
    - ``code``: ``true`` when the frontend and desktop jobs run (everything but docs-only);
    - ``codegen``: ``true`` when ``docs / schema-diff`` regenerates the generated files;
    - ``fuzz``: the JSON matrix of fuzz targets to run, ``[]`` for none.

    Every event other than ``pull_request`` (pushes to ``main``, the merge queue, schedules,
    manual runs) runs everything, and so does a pull request with no changed paths.

``gate``
    Reads ``toJSON(needs)`` of ``ci / all-checks-pass`` from the ``NEEDS`` environment
    variable and exits 1 unless every job succeeded or was skipped by design: skipped while
    the ``changes`` output its scope depends on was ``false``.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path, PurePosixPath

#: Paths that make every job run: build configuration, CI itself, contracts and generated
#: files (any ``Cargo.toml`` or ``Cargo.lock``, at any depth, also counts).
EVERYTHING_PREFIXES = (".github/", "schemas/", "xtask/", "packages/ipc/src/generated/")
EVERYTHING_NAMES = ("Cargo.toml", "Cargo.lock")

#: Frontend files: no Rust code reads them, so the Rust jobs are skipped. Markdown inside
#: these trees is frontend too, because each package's `prettier --check .` reads it.
FRONTEND_PREFIXES = ("apps/desktop/", "packages/")
FRONTEND_EXCLUDED_PREFIXES = ("apps/desktop/src-tauri/",)
FRONTEND_FILES = (
    "package.json",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "eslint.config.js",
    "prettier.config.js",
    "stylelint.config.js",
    "tsconfig.base.json",
)

#: Documentation, and the configuration only the `docs.yml` checks read.
DOCS_PREFIXES = ("docs/", "LICENSES/")
DOCS_SUFFIXES = (".md",)
DOCS_FILES = ("REUSE.toml", "lychee.toml", ".markdownlint-cli2.jsonc")

#: Hand-edited copies of these `cargo xtask codegen` outputs fail `docs / schema-diff`, so
#: a change to one runs it even in a docs-only pull request. The others count as everything.
CODEGEN_OUTPUTS = ("docs/reference/error-codes.md",)

#: Changes that smoke-fuzz every target on a pull request: the fuzz workflow and this
#: script; any `Cargo.toml` or `Cargo.lock`, because a dependency bump is when a parser's
#: behaviour changes under it; and `sonarithm-core`, whose types flow into every parser.
FUZZ_EVERYTHING = (".github/workflows/nightly.yml", ".github/scripts/ci_scope.py")
FUZZ_EVERYTHING_NAMES = ("Cargo.toml", "Cargo.lock")
FUZZ_EVERYTHING_PREFIXES = ("crates/sonarithm-core/",)

#: `ci.yml` jobs and the `changes` output that decides whether they run. `changes` itself
#: and `all-checks-pass` always run.
SCOPED_JOBS = {
    "fmt": "rust",
    "clippy-test": "rust",
    "doc": "rust",
    "deny": "rust",
    "check-deps": "rust",
    "msrv": "rust",
    "wasm32": "rust",
    "golden": "rust",
    "golden-compare": "rust",
    "no-alloc": "rust",
    "gallery": "rust",
    "network-deny": "rust",
    "chart-accuracy": "rust",
    "frontend": "code",
    "licenses-npm": "code",
    "frontend-browser": "code",
    "desktop": "code",
}

_INCLUDE = re.compile(r'include_(?:str|bytes)!\(\s*"([^"]+)"\s*\)')


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def rust_includes(root: Path) -> set[str]:
    """Repository paths that Rust code compiles in with `include_str!` or `include_bytes!`.

    Such a file is a test input or data even under `docs/` (the NUMISONA conformance tests
    read docs/research/design-rules.md), so changing it runs the Rust jobs. Only literal
    paths are found; a file read at run time must be listed in a prefix above instead.
    """
    listed = subprocess.run(
        ["git", "ls-files", "-z", "*.rs"], cwd=root, capture_output=True, check=True
    ).stdout.decode()
    found: set[str] = set()
    for source in filter(None, listed.split("\0")):
        text = (root / source).read_text(encoding="utf-8", errors="replace")
        for match in _INCLUDE.finditer(text):
            joined = PurePosixPath(source).parent / match.group(1)
            parts: list[str] = []
            for part in joined.parts:
                if part == "..":
                    if parts:
                        parts.pop()
                elif part != ".":
                    parts.append(part)
            found.add("/".join(parts))
    return found


def classify(path: str, included: set[str]) -> str:
    """One path's class: ``everything``, ``rust``, ``frontend`` or ``docs``."""
    name = PurePosixPath(path).name
    if path.startswith(EVERYTHING_PREFIXES) or name in EVERYTHING_NAMES:
        return "everything"
    if path in included:
        return "rust"
    if path in FRONTEND_FILES or (
        path.startswith(FRONTEND_PREFIXES) and not path.startswith(FRONTEND_EXCLUDED_PREFIXES)
    ):
        return "frontend"
    if path.startswith(DOCS_PREFIXES) or path.endswith(DOCS_SUFFIXES) or path in DOCS_FILES:
        return "docs"
    return "rust"


def fuzz_targets(root: Path) -> list[dict[str, str]]:
    """Every cargo-fuzz target: the `[[bin]]` entries of each `crates/**/fuzz/Cargo.toml`."""
    targets = []
    for manifest in sorted(root.glob("crates/**/fuzz/Cargo.toml")):
        crate = manifest.parent.parent.relative_to(root / "crates").as_posix()
        with manifest.open("rb") as file:
            bins = tomllib.load(file).get("bin", [])
        targets.extend({"crate": crate, "target": b["name"]} for b in bins)
    return targets


def fuzzes_everything(path: str) -> bool:
    """Whether a pull request that changes this path smoke-fuzzes every target."""
    return (
        path in FUZZ_EVERYTHING
        or PurePosixPath(path).name in FUZZ_EVERYTHING_NAMES
        or path.startswith(FUZZ_EVERYTHING_PREFIXES)
    )


def scope(event: str, paths: list[str], included: set[str], targets: list[dict[str, str]]):
    """The `changes` outputs for these changed paths."""
    paths = [p for p in paths if p]
    classes = {classify(p, included) for p in paths}
    everything = event != "pull_request" or not paths or "everything" in classes
    if everything:
        kind = "everything"
    elif "rust" in classes:
        kind = "rust"
    elif "frontend" in classes:
        kind = "frontend"
    else:
        kind = "docs"
    rust = kind in ("everything", "rust")
    if event != "pull_request" or any(fuzzes_everything(p) for p in paths):
        fuzz = targets
    else:
        fuzz = [t for t in targets if any(p.startswith(f"crates/{t['crate']}/") for p in paths)]
    return {
        "kind": kind,
        "rust": rust,
        "code": kind != "docs",
        "codegen": rust or any(p in CODEGEN_OUTPUTS for p in paths),
        "fuzz": fuzz,
    }


def gate(needs: dict) -> list[str]:
    """Problems that fail `all-checks-pass`; empty when every job passed or was skipped by design."""
    changes = needs.get("changes", {})
    if changes.get("result") != "success":
        return [f"changes: {changes.get('result', 'missing')}"]
    outputs = changes.get("outputs", {})
    problems = []
    for job, info in sorted(needs.items()):
        result = info.get("result")
        if result == "success":
            continue
        key = SCOPED_JOBS.get(job)
        if result == "skipped" and key is not None and outputs.get(key) == "false":
            continue
        reason = " (not skipped by design)" if result == "skipped" else ""
        problems.append(f"{job}: {result}{reason}")
    return problems


def _format(outputs: dict) -> str:
    lines = []
    for key, value in outputs.items():
        if isinstance(value, bool):
            value = "true" if value else "false"
        elif not isinstance(value, str):
            value = json.dumps(value, separators=(",", ":"))
        lines.append(f"{key}={value}")
    return "\n".join(lines) + "\n"


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    commands = parser.add_subparsers(dest="command", required=True)
    changes = commands.add_parser("changes")
    changes.add_argument("--event", required=True)
    commands.add_parser("gate")
    args = parser.parse_args(argv)

    if args.command == "changes":
        root = repo_root()
        paths = sys.stdin.read().splitlines()
        result = scope(args.event, paths, rust_includes(root), fuzz_targets(root))
        sys.stdout.write(_format(result))
        summary = os.environ.get("GITHUB_STEP_SUMMARY")
        if summary:
            with open(summary, "a", encoding="utf-8") as file:
                file.write(
                    f"### Change scope: {result['kind']}\n\n"
                    f"{len(paths)} changed path(s); Rust jobs: {result['rust']}, frontend and "
                    f"desktop jobs: {result['code']}, schema-diff: {result['codegen']}, "
                    f"fuzz targets: {len(result['fuzz'])}.\n"
                )
        return 0

    problems = gate(json.loads(os.environ["NEEDS"]))
    for problem in problems:
        print(f"::error::{problem}")
    if not problems:
        print("Every job passed or was skipped by design.")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Checks the pnpm workspace's licences against the allowlist (supply-chain-and-licensing.md).

    pnpm -r licenses list --json > all.json
    pnpm -r licenses list --json --prod > prod.json
    python3 .github/scripts/npm_licences.py all.json prod.json

A package in ``prod.json`` ships in the desktop app's bundle and must use a licence of
SHIPPED; every other package is development-only tooling and may also use DEV_ONLY.
Licences are SPDX expressions: ``OR`` needs one allowed side, ``AND`` needs both. Exits 1
and lists each package whose licence is not allowed. Standard library only.
"""

from __future__ import annotations

import json
import re
import sys
import tomllib
from pathlib import Path

#: Licences any package may use: the Rust allowlist of deny.toml (a test keeps the two
#: equal) plus `OFL-1.1`, which the policy allows for font packages.
SHIPPED_EXTRA = ("OFL-1.1",)

#: Licences only development-only packages may use (never in the app, the CLI, the wheel
#: or the WASM build). `BlueOak-1.0.0`: the owner's decision of 2026-11-26. `CC-BY-4.0`:
#: data licences are allowed for build-time tools (caniuse-lite's browser data).
DEV_ONLY = ("BlueOak-1.0.0", "CC-BY-4.0")

_TOKEN = re.compile(r"\(|\)|[^\s()]+")


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def shipped(root: Path) -> set[str]:
    """The licences a shipped package may use."""
    with (root / "deny.toml").open("rb") as file:
        rust = tomllib.load(file)["licenses"]["allow"]
    return set(rust) | set(SHIPPED_EXTRA)


def allowed(expression: str, licences: set[str]) -> bool:
    """Whether an SPDX expression is satisfied by the allowed licences.

    ``A WITH B`` is one licence. Anything that does not parse (pnpm's ``Unknown``, a
    ``SEE LICENSE IN`` text, an unbalanced parenthesis) is not allowed.
    """
    folded = {licence.casefold() for licence in licences}
    tokens = _TOKEN.findall(expression.strip())
    position = 0

    def peek() -> str | None:
        return tokens[position] if position < len(tokens) else None

    def take() -> str:
        nonlocal position
        position += 1
        return tokens[position - 1]

    def term() -> bool:
        token = peek()
        if token is None or token in (")", "AND", "OR", "WITH"):
            raise ValueError(expression)
        take()
        if token == "(":
            value = either()
            if peek() != ")":
                raise ValueError(expression)
            take()
            return value
        licence = token
        if peek() == "WITH":
            take()
            exception = peek()
            if exception is None or exception in ("(", ")", "AND", "OR", "WITH"):
                raise ValueError(expression)
            licence = f"{licence} WITH {take()}"
        return licence.casefold() in folded

    def both() -> bool:
        value = term()
        while peek() == "AND":
            take()
            value = term() and value
        return value

    def either() -> bool:
        value = both()
        while peek() == "OR":
            take()
            value = both() or value
        return value

    try:
        value = either()
    except ValueError:
        return False
    return value and position == len(tokens)


def packages(report: dict) -> dict[tuple[str, str], str]:
    """`pnpm licenses list --json` as {(name, version): licence}."""
    found = {}
    for licence, entries in report.items():
        for entry in entries:
            for version in entry.get("versions", []):
                found[(entry["name"], version)] = entry.get("license", licence)
    return found


def problems(all_report: dict, prod_report: dict, shipped_licences: set[str]) -> list[str]:
    """One line per package whose licence is not allowed for how it is used."""
    production = packages(prod_report)
    development = set(shipped_licences) | set(DEV_ONLY)
    out = []
    for (name, version), licence in sorted(packages(all_report).items()):
        ships = (name, version) in production
        if not allowed(licence, shipped_licences if ships else development):
            use = "shipped" if ships else "development-only"
            out.append(f"{name}@{version}: {licence} ({use})")
    return out


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    reports = [json.loads(Path(path).read_text(encoding="utf-8")) for path in argv]
    found = problems(reports[0], reports[1], shipped(repo_root()))
    total = len(packages(reports[0]))
    for line in found:
        print(f"::error title=npm licence::{line} is not on the allowlist")
    if found:
        print(
            f"{len(found)} of {total} packages have a licence outside the allowlist "
            "(docs/engineering/supply-chain-and-licensing.md#npm-licence-checking)."
        )
        return 1
    print(f"All {total} packages use allowed licences ({len(packages(reports[1]))} shipped).")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

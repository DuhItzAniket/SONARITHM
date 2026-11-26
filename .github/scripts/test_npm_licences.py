# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Tests of npm_licences.py; run by the `ci / changes` job with the other script tests."""

from __future__ import annotations

import re
import unittest

import npm_licences
from npm_licences import DEV_ONLY, allowed, problems

ROOT = npm_licences.repo_root()
SHIPPED = npm_licences.shipped(ROOT)
DEVELOPMENT = SHIPPED | set(DEV_ONLY)


def report(*entries: tuple[str, str, str]) -> dict:
    """A `pnpm licenses list --json` report of (name, version, licence) entries."""
    out: dict = {}
    for name, version, licence in entries:
        out.setdefault(licence, []).append(
            {"name": name, "versions": [version], "license": licence, "paths": ["/x"]}
        )
    return out


class Expressions(unittest.TestCase):
    def test_single_licences(self):
        self.assertTrue(allowed("MIT", SHIPPED))
        self.assertTrue(allowed("mit", SHIPPED))
        self.assertFalse(allowed("GPL-3.0-only", SHIPPED))
        self.assertFalse(allowed("BlueOak-1.0.0", SHIPPED))
        self.assertTrue(allowed("BlueOak-1.0.0", DEVELOPMENT))

    def test_or_needs_one_allowed_side_and_needs_both(self):
        self.assertTrue(allowed("Apache-2.0 OR MIT", SHIPPED))
        self.assertTrue(allowed("(GPL-2.0-only OR MIT)", SHIPPED))
        self.assertFalse(allowed("MIT AND GPL-2.0-only", SHIPPED))
        self.assertTrue(allowed("(MIT AND Zlib) OR GPL-3.0-only", SHIPPED))
        self.assertFalse(allowed("MIT AND (GPL-3.0-only OR AGPL-3.0-only)", SHIPPED))

    def test_with_names_one_licence(self):
        self.assertTrue(allowed("Apache-2.0 WITH LLVM-exception", SHIPPED))
        self.assertFalse(allowed("GPL-2.0-only WITH Classpath-exception-2.0", SHIPPED))

    def test_what_does_not_parse_is_not_allowed(self):
        for text in ("", "Unknown", "SEE LICENSE IN LICENSE.md", "(MIT", "MIT)", "MIT OR", "MIT MIT"):
            self.assertFalse(allowed(text, SHIPPED), text)


class Packages(unittest.TestCase):
    def test_dev_only_licences_pass_only_outside_the_shipped_set(self):
        everything = report(("glob", "13.0.6", "BlueOak-1.0.0"), ("react", "19.2.0", "MIT"))
        self.assertEqual(problems(everything, report(("react", "19.2.0", "MIT")), SHIPPED), [])
        self.assertEqual(
            problems(everything, report(("glob", "13.0.6", "BlueOak-1.0.0")), SHIPPED),
            ["glob@13.0.6: BlueOak-1.0.0 (shipped)"],
        )

    def test_a_version_is_shipped_only_if_that_version_is(self):
        everything = report(("lru-cache", "11.5.3", "BlueOak-1.0.0"), ("lru-cache", "5.1.1", "ISC"))
        shipped = report(("lru-cache", "5.1.1", "ISC"))
        self.assertEqual(problems(everything, shipped, SHIPPED), [])

    def test_denied_licences_fail_even_for_development(self):
        everything = report(("a", "1.0.0", "GPL-3.0-or-later"), ("b", "1.0.0", "AGPL-3.0-only"))
        self.assertEqual(
            problems(everything, {}, SHIPPED),
            ["a@1.0.0: GPL-3.0-or-later (development-only)", "b@1.0.0: AGPL-3.0-only (development-only)"],
        )


class Policy(unittest.TestCase):
    def test_the_shipped_list_is_deny_toml_plus_fonts(self):
        self.assertIn("MPL-2.0", SHIPPED)
        self.assertIn("OFL-1.1", SHIPPED)
        self.assertNotIn("BlueOak-1.0.0", SHIPPED)

    def test_the_policy_table_lists_every_allowed_licence(self):
        doc = (ROOT / "docs/engineering/supply-chain-and-licensing.md").read_text(encoding="utf-8")
        table = doc[doc.index("### Allowed") : doc.index("### Denied")]
        listed = set(re.findall(r"`([A-Za-z0-9.\- ]+?)`", table))
        for licence in SHIPPED | set(DEV_ONLY):
            self.assertIn(licence, listed, licence)


if __name__ == "__main__":
    unittest.main()

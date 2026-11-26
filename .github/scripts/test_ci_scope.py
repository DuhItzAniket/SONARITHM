# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Tests of ci_scope.py; run by the `ci / changes` and `nightly / plan` jobs.

    python3 -m unittest discover --start-directory .github/scripts --verbose
"""

from __future__ import annotations

import re
import unittest

import ci_scope
from ci_scope import SCOPED_JOBS, classify, gate, scope

ROOT = ci_scope.repo_root()
INCLUDED = ci_scope.rust_includes(ROOT)
TARGETS = ci_scope.fuzz_targets(ROOT)


def pr(*paths: str) -> dict:
    return scope("pull_request", list(paths), INCLUDED, TARGETS)


def fuzzed(*paths: str) -> set[str]:
    return {t["target"] for t in pr(*paths)["fuzz"]}


class Classify(unittest.TestCase):
    def test_build_configuration_ci_contracts_and_generated_files_are_everything(self):
        for path in (
            "Cargo.toml",
            "Cargo.lock",
            "crates/sonarithm-data/Cargo.toml",
            "crates/sonarithm-data/fuzz/Cargo.lock",
            ".github/workflows/ci.yml",
            ".github/dependabot.yml",
            "schemas/sonarithm-spec-0.4.schema.json",
            "xtask/src/main.rs",
            "packages/ipc/src/generated/protocol.ts",
        ):
            self.assertEqual(classify(path, INCLUDED), "everything", path)

    def test_documentation_and_the_doc_check_configuration_are_docs(self):
        for path in (
            "docs/engineering/ci-cd.md",
            "docs/research-paper/figures/plot.svg",
            "README.md",
            "CHANGELOG.md",
            "crates/sonarithm-data/README.md",
            "LICENSES/MIT.txt",
            "REUSE.toml",
            "lychee.toml",
            ".markdownlint-cli2.jsonc",
        ):
            self.assertEqual(classify(path, INCLUDED), "docs", path)

    def test_docs_that_rust_compiles_in_are_rust(self):
        # include_str! in the NUMISONA conformance tests and the spec's doc-example test.
        for path in (
            "docs/research/design-rules.md",
            "docs/architecture/04-sonification-spec.md",
        ):
            self.assertIn(path, INCLUDED)
            self.assertEqual(classify(path, INCLUDED), "rust", path)

    def test_the_frontend_trees_and_root_frontend_files_are_frontend(self):
        for path in (
            "apps/desktop/src/app/App.tsx",
            "apps/desktop/e2e/audio.spec.ts",
            "apps/desktop/package.json",
            "packages/ui/src/Button.tsx",
            # Each package's `prettier --check .` reads its Markdown.
            "packages/ui/fonts/README.md",
            "pnpm-lock.yaml",
            "eslint.config.js",
        ):
            self.assertEqual(classify(path, INCLUDED), "frontend", path)

    def test_the_desktop_host_and_everything_else_are_rust(self):
        for path in (
            "apps/desktop/src-tauri/src/main.rs",
            "apps/desktop/src-tauri/tauri.conf.json",
            "crates/sonarithm-core/src/lib.rs",
            "tests/golden/cases/sine.json",
            "examples/specs/line.spec.json",
            "rust-toolchain.toml",
            "deny.toml",
            "CITATION.cff",
        ):
            self.assertEqual(classify(path, INCLUDED), "rust", path)


class Scope(unittest.TestCase):
    def test_docs_only_runs_no_rust_frontend_or_codegen(self):
        out = pr("docs/engineering/ci-cd.md", "README.md")
        self.assertEqual(out["kind"], "docs")
        self.assertEqual((out["rust"], out["code"], out["codegen"]), (False, False, False))
        self.assertEqual(out["fuzz"], [])

    def test_a_hand_edited_error_code_table_runs_schema_diff_only(self):
        out = pr("docs/reference/error-codes.md")
        self.assertEqual(out["kind"], "docs")
        self.assertEqual((out["rust"], out["code"], out["codegen"]), (False, False, True))

    def test_frontend_only_runs_the_frontend_and_desktop_jobs(self):
        out = pr("apps/desktop/src/app/App.tsx", "packages/ui/src/Button.tsx", "docs/x.md")
        self.assertEqual(out["kind"], "frontend")
        self.assertEqual((out["rust"], out["code"], out["codegen"]), (False, True, False))

    def test_frontend_with_a_generated_ipc_change_runs_everything(self):
        out = pr("apps/desktop/src/app/App.tsx", "packages/ipc/src/generated/protocol.ts")
        self.assertEqual(out["kind"], "everything")
        self.assertTrue(out["rust"] and out["code"] and out["codegen"])

    def test_any_rust_change_runs_everything_in_ci(self):
        out = pr("docs/x.md", "crates/sonarithm-core/src/lib.rs")
        self.assertEqual(out["kind"], "rust")
        self.assertTrue(out["rust"] and out["code"] and out["codegen"])

    def test_pushes_the_merge_queue_schedules_and_empty_diffs_run_everything(self):
        for event in ("push", "merge_group", "schedule", "workflow_dispatch"):
            out = scope(event, ["docs/x.md"], INCLUDED, TARGETS)
            self.assertEqual(out["kind"], "everything", event)
            self.assertEqual(out["fuzz"], TARGETS, event)
        self.assertEqual(pr()["kind"], "everything")

    def test_outputs_are_strings_for_github_output(self):
        text = ci_scope._format(pr("crates/sonarithm-spec/src/lib.rs"))
        self.assertIn("rust=true\n", text)
        self.assertIn('fuzz=[{"crate":"sonarithm-spec","target":"spec_parse"}', text)
        self.assertIn("\nkind=rust\n", "\n" + text)


class Fuzz(unittest.TestCase):
    def test_every_fuzz_target_of_the_repository_is_found(self):
        self.assertEqual(len(TARGETS), len({t["target"] for t in TARGETS}))
        files = {p.stem for p in ROOT.glob("crates/**/fuzz/fuzz_targets/*.rs")}
        self.assertTrue({t["target"] for t in TARGETS} <= files)
        self.assertIn({"crate": "engines/sonarithm-plotsona", "target": "pdf_chart"}, TARGETS)

    def test_a_pull_request_fuzzes_the_targets_of_the_crates_it_changes(self):
        data = {t["target"] for t in TARGETS if t["crate"] == "sonarithm-data"}
        self.assertEqual(fuzzed("crates/sonarithm-data/src/source/csv.rs"), data)
        self.assertEqual(fuzzed("crates/sonarithm-data/fuzz/fuzz_targets/csv_read.rs"), data)
        self.assertEqual(
            fuzzed("crates/engines/sonarithm-curvasona/src/expr.rs"), {"expr_parse"}
        )

    def test_crates_without_targets_and_other_changes_fuzz_nothing(self):
        self.assertEqual(fuzzed("crates/sonarithm-render/src/lib.rs"), set())
        self.assertEqual(fuzzed("crates/sonarithm-data-extra/src/lib.rs"), set())
        self.assertEqual(fuzzed(".github/workflows/ci.yml", "docs/x.md"), set())
        self.assertEqual(fuzzed("apps/desktop/src/app/App.tsx"), set())

    def test_a_cargo_lock_only_pull_request_fuzzes_every_target(self):
        # A dependency bump (calamine 0.37 to 0.38) changes parsers without touching them.
        self.assertEqual(pr("Cargo.lock")["fuzz"], TARGETS)

    def test_any_cargo_manifest_change_fuzzes_every_target(self):
        for path in (
            "Cargo.toml",
            "crates/sonarithm-render/Cargo.toml",
            "crates/sonarithm-data/fuzz/Cargo.lock",
        ):
            self.assertEqual(pr(path)["fuzz"], TARGETS, path)

    def test_a_core_only_pull_request_fuzzes_every_target(self):
        # Core types flow into every parser.
        self.assertEqual(pr("crates/sonarithm-core/src/lib.rs")["fuzz"], TARGETS)

    def test_changing_the_fuzz_workflow_or_this_script_fuzzes_every_target(self):
        for path in ci_scope.FUZZ_EVERYTHING:
            self.assertEqual(pr(path)["fuzz"], TARGETS, path)


def needs(changes: dict, **results: str) -> dict:
    out = {"changes": {"result": "success", "outputs": changes}}
    out.update({job: {"result": result, "outputs": {}} for job, result in results.items()})
    return out


DOCS_ONLY = {"rust": "false", "code": "false"}
FRONTEND_ONLY = {"rust": "false", "code": "true"}
EVERYTHING = {"rust": "true", "code": "true"}


class Gate(unittest.TestCase):
    """The four outcomes of `all-checks-pass` (ci-cd.md#change-scoped-jobs)."""

    def test_1_success_passes(self):
        self.assertEqual(gate(needs(EVERYTHING, **{j: "success" for j in SCOPED_JOBS})), [])

    def test_2_skipped_by_design_passes(self):
        self.assertEqual(gate(needs(DOCS_ONLY, **{j: "skipped" for j in SCOPED_JOBS})), [])
        frontend = {j: "success" if k == "code" else "skipped" for j, k in SCOPED_JOBS.items()}
        self.assertEqual(gate(needs(FRONTEND_ONLY, **frontend)), [])

    def test_3_failure_fails(self):
        self.assertEqual(
            gate(needs(EVERYTHING, fmt="success", golden="failure")), ["golden: failure"]
        )
        # Even in a job that this change could have skipped.
        self.assertEqual(gate(needs(DOCS_ONLY, frontend="failure")), ["frontend: failure"])

    def test_4_cancelled_fails(self):
        self.assertEqual(gate(needs(FRONTEND_ONLY, desktop="cancelled")), ["desktop: cancelled"])

    def test_a_skip_that_the_scope_did_not_ask_for_fails(self):
        # golden-compare is skipped when golden fails; on main every job must run.
        self.assertEqual(
            gate(needs(EVERYTHING, golden="failure", **{"golden-compare": "skipped"})),
            ["golden: failure", "golden-compare: skipped (not skipped by design)"],
        )
        self.assertEqual(
            gate(needs(FRONTEND_ONLY, desktop="skipped")),
            ["desktop: skipped (not skipped by design)"],
        )

    def test_the_npm_licence_check_runs_with_the_frontend(self):
        # Skipped only on docs-only PRs; on a frontend-only PR or on main it must run and pass.
        self.assertEqual(SCOPED_JOBS["licenses-npm"], "code")
        self.assertEqual(gate(needs(DOCS_ONLY, **{"licenses-npm": "skipped"})), [])
        self.assertEqual(gate(needs(FRONTEND_ONLY, **{"licenses-npm": "success"})), [])
        self.assertEqual(
            gate(needs(FRONTEND_ONLY, **{"licenses-npm": "skipped"})),
            ["licenses-npm: skipped (not skipped by design)"],
        )
        self.assertEqual(
            gate(needs(EVERYTHING, **{"licenses-npm": "failure"})), ["licenses-npm: failure"]
        )
        self.assertTrue(pr("pnpm-lock.yaml")["code"])

    def test_a_job_outside_the_scope_table_may_not_be_skipped(self):
        self.assertEqual(
            gate(needs(DOCS_ONLY, **{"new-job": "skipped"})),
            ["new-job: skipped (not skipped by design)"],
        )

    def test_a_failed_or_cancelled_changes_job_fails(self):
        for result in ("failure", "cancelled", "skipped"):
            broken = {"changes": {"result": result, "outputs": {}}, "fmt": {"result": "skipped"}}
            self.assertEqual(gate(broken), [f"changes: {result}"])
        self.assertEqual(gate({}), ["changes: missing"])


def jobs(workflow: str) -> dict[str, str]:
    """Each job id of a workflow file and the text of its block (two-space-indented keys)."""
    text = (ROOT / ".github/workflows" / workflow).read_text(encoding="utf-8")
    body = text[text.index("\njobs:\n") :]
    parts = re.split(r"^  ([a-z0-9-]+):\n", body, flags=re.MULTILINE)
    return dict(zip(parts[1::2], parts[2::2]))


class Workflows(unittest.TestCase):
    def test_every_ci_job_is_scoped_by_the_changes_job(self):
        ci = jobs("ci.yml")
        self.assertEqual(set(ci) - {"changes", "all-checks-pass"}, set(SCOPED_JOBS))
        for job, key in SCOPED_JOBS.items():
            block = ci[job]
            self.assertRegex(block, r"(?m)^    needs: (changes|\[changes, .*\])$", job)
            self.assertIn(f"\n    if: needs.changes.outputs.{key} == 'true'\n", "\n" + block, job)
        outputs = re.findall(r"(?m)^      ([a-z]+): \$\{\{ steps\.scope", ci["changes"])
        self.assertEqual(set(outputs), set(SCOPED_JOBS.values()))

    def test_the_gate_needs_every_job_and_always_runs(self):
        ci = jobs("ci.yml")
        gate_block = ci["all-checks-pass"]
        self.assertIn("    if: always()\n", gate_block)
        listed = re.search(r"(?m)^    needs: \[(.*)\]$", gate_block).group(1).split(", ")
        self.assertEqual(sorted(listed), sorted(set(ci) - {"all-checks-pass"}))
        self.assertIn("ci_scope.py gate", gate_block)

    def test_schema_diff_and_fuzz_read_their_scope_outputs(self):
        self.assertIn("steps.scope.outputs.codegen == 'true'", jobs("docs.yml")["schema-diff"])
        nightly = jobs("nightly.yml")
        self.assertIn("fromJSON(needs.plan.outputs.fuzz)", nightly["fuzz"])
        self.assertIn("if: needs.plan.outputs.fuzz != '[]'", nightly["fuzz"])


if __name__ == "__main__":
    unittest.main()

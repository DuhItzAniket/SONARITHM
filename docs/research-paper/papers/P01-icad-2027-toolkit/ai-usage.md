# P01 AI-usage record

> **Status:** Draft (2026-10-19). **Owner:** Paper lead.
> **Related:** [publication plan, AI-usage disclosure](../../publication-plan.md#ai-usage-disclosure) · [draft](src/paper.md)

This file records how AI tools contributed to P01 and its software, so that the final statement in the form the ICAD 2027 call asks for can be written from facts. The call has not been published (checked 2026-10-19); ICAD's AI policy is `TODO(verify)`.

## Record

| What | Tool | Human check |
|---|---|---|
| Software: most SONARITHM code, tests and documentation were written by AI coding agents (Claude Code) working from the project's documented plan, one task per pull request | Claude Code | Continuous integration on three operating systems; the project lead decides on contract changes (RFCs, ADRs) and on every research decision. TODO: describe the owner's review practice under the fast-track rule honestly |
| Source verification: the published stimulus parameters (F-110 to F-113) and every citation in the draft were checked against the primary sources by an agent, which recorded locators for each value | Claude Code subagent | TODO: the paper lead spot-checks the locators before submission |
| Drafting: the first draft of this paper (2026-10-19) | Claude Code (A08 session) | TODO: the paper lead rewrites and approves every section |
| Data, results, participant responses | none | No AI tool generated data or results. Every number comes from a logged measurement or a pinned test |

## Draft statement

> TODO(verify) against the ICAD 2027 call. Draft: "The SONARITHM software, including the code measured in this paper, was largely written with AI coding agents (Claude Code) under the author's direction, and every change passed automated tests on three operating systems. An AI agent drafted parts of this manuscript and checked the bibliographic records and published stimulus parameters against their sources; the author verified every number, citation and claim and takes responsibility for them. No AI tool generated data or results."

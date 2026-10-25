---
status: proposed
date: 2026-10-26
decision-makers: ["@DuhItzAniket"]
consulted: []
informed: []
---

# 0022 · Clean-room re-implementation of the raster digitizer

## Context and problem statement

PLOTSONA's raster tier (C02) reads values from chart images: axis calibration, colour masks, line, bar and point extraction, and manual correction ([10 §3](../architecture/10-ingestion-and-extraction.md#3-the-bring-any-chart-pipeline-plotsona)). The best-documented design for this is WebPlotDigitizer (WPD). Its frontend code is AGPL-3.0, and its cloud parts are closed. Engauge Digitizer, the other well-known open tool, is GPL ([notes 06 §1](../research/notes/06-chart-extraction-ai.md#1-classical-and-semi-automatic-digitizers-webplotdigitizer-engauge-others-status-licensing-algorithms-accuracy)). SONARITHM is `MIT OR Apache-2.0` ([ADR-0010](0010-licensing.md)). Copying, porting or translating either code base would bring copyleft obligations into the combined work (AGENTS.md rule 8). How do we use the published knowledge about these tools without their code, and show that we did?

## Decision drivers

* AGENTS.md rule 8 and the supply-chain policy: no AGPL or GPL code, and WPD's code is never copied.
* Evidence a reviewer and an outside reader can check: which sources informed which code.
* The project is one maintainer plus AI coding agents. The procedure must work without a second human team.
* AI agents may have seen WPD's code in training. The procedure cannot rely on what a model has not seen; it relies on what a session is given and what review checks.
* Low cost: C02 has eight Lane R tasks, and the procedure must not slow them much.

## Considered options

* **A. Documentation-only clean room with a sources register.** Nobody working on SONARITHM reads WPD's or Engauge's source. Implementers work from WPD's user documentation, published papers and general image-processing references. Every source is recorded in a register, and every C02 PR lists the register entries it used.
* **B. Two-team clean room.** One person reads WPD's code and writes a functional specification free of code; a second team that never saw the code implements it.
* **C. Black-box study of the hosted WPD application** in addition to its documentation: run project-made test charts through the web app and compare behaviour.
* **D. No formal procedure**, relying on the general rule "do not copy".

## Decision outcome

Chosen option: **A**, because it gives the same protection as B for the parts of WPD that are documented (its calibration types, masks, colour-distance filter, averaging-window, X-step and bar and blob algorithms are all described in its user documentation), without a second team the project does not have. C adds little that the documentation does not already say, sends files to a third-party service, and invites inspecting the served JavaScript; it is not allowed for now. D leaves no evidence.

The procedure itself (roles, allowed and forbidden sources, the register, the PR line, review checks and what to do after an exposure) is in the [supply-chain policy](../engineering/supply-chain-and-licensing.md#clean-room-procedure-raster-digitizer). The register is [docs/research/clean-room/raster-digitizer-sources.md](../research/clean-room/raster-digitizer-sources.md).

### Consequences

* Good, because every C02 algorithm can be traced to public documentation or a paper, and a reviewer can check each PR against the register.
* Good, because the register doubles as the citation list for the raster tier in P05 and in the docs.
* Neutral, because algorithm details that WPD's documentation does not give (thresholds, tie-breaking, interpolation choices) must be designed by us and tested against the corpus (C02-T08). That is more work, and it is the point.
* Bad, because a model's training data cannot be audited. Review checks (no WPD identifiers or comments, structure that follows our own design notes) reduce but do not remove the risk.
* Bad, because a contributor who has read WPD's or Engauge's code (for example while using it elsewhere) cannot write C02 raster code. With one maintainer this could stall C02; the fallback is a fresh agent session working only from the register.

### Confirmation

* The PR template has a clean-room sources line, and every C02 PR that adds raster code fills it in (C02-T02 to C02-T05, C02-T08).
* The register lists every source a C02 PR cites; a reviewer checks the PR against it before approving.
* The C06 security review (C06-T06) re-checks the raster crate's history against the register.
* Revisit if WPD's licence changes, if a permissively licensed digitizer with equivalent algorithms appears, or if the project gains a second team that makes option B practical.

## Pros and cons of the options

### A. Documentation-only clean room

* Good, because it needs one team and leaves a written trail.
* Good, because WPD's user documentation covers the workflow C02 needs ([notes 06 §1](../research/notes/06-chart-extraction-ai.md#1-classical-and-semi-automatic-digitizers-webplotdigitizer-engauge-others-status-licensing-algorithms-accuracy)).
* Bad, because undocumented details must be designed from scratch.

### B. Two-team clean room

* Good, because it is the classic, well-understood defence.
* Bad, because the project has one maintainer; the reading team would have to be an AI session whose output the implementing session then reads, which proves little.

### C. Black-box study of the hosted application

* Good, because behaviour could be compared on the same charts.
* Bad, because it sends files to a third party, is bound by that service's terms (not checked), and puts the served AGPL JavaScript one keystroke away in the browser's developer tools.

### D. No formal procedure

* Good, because it costs nothing.
* Bad, because nothing shows, later, that the code is independent.

## More information

* [Implementation plan §4.4](../planning/implementation-plan.md#44-licence-traps-and-provenance-rules) lists WPD and Engauge among Stage C's licence traps; C02-T01 (issue [#303](https://github.com/DuhItzAniket/SONARITHM/issues/303)) writes this procedure.
* This ADR is accepted by the coordinator under the owner's delegation before C02-T03's code (the first code that re-implements WPD's documented algorithms).
* Not legal advice. The procedure follows the common clean-room practice of separating knowledge of an implementation from the people who write a new one; whether it is enough in a given jurisdiction is TODO(verify) if the project ever seeks legal review.

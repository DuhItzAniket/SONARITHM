# Clean-room sources register: raster digitizer (PLOTSONA C02)

> **Status:** Living register, started 2026-10-26 (C02-T01). **Owner:** Project lead.
> **Related:** [ADR-0022](../../decisions/0022-clean-room-raster-digitizer.md) · [clean-room procedure](../../engineering/supply-chain-and-licensing.md#clean-room-procedure-raster-digitizer) · [notes 06 §1](../notes/06-chart-extraction-ai.md#1-classical-and-semi-automatic-digitizers-webplotdigitizer-engauge-others-status-licensing-algorithms-accuracy) · [10 §3](../../architecture/10-ingestion-and-extraction.md#3-the-bring-any-chart-pipeline-plotsona)

Every source that informs PLOTSONA's raster code (calibration from anchors, masks, colour filtering, line, bar and point extraction, confidence and correction) is listed here before a PR cites it. A C02 PR names the entries it used on its "Clean-room sources" line. A source that is not in this register cannot be cited; add it in the same PR, with its kind, so the reviewer can check it against the [allowed sources](../../engineering/supply-chain-and-licensing.md#clean-room-procedure-raster-digitizer).

## Allowed kinds

| Kind | Meaning |
|---|---|
| `user-docs` | A tool's user documentation: pages that explain how to use it, not its source, build files or developer notes |
| `paper` | A published paper, preprint or thesis |
| `reference` | A textbook, standard or library documentation on general image processing or numerical methods, unrelated to WPD or Engauge |
| `licence-fact` | A page read only for a licence or status fact, never as a design source (for example a repository's front-page README). Cannot be cited for code |

## Register

Fields: ID; source and link; kind; what it may inform; when it was read and by whom (a GitHub handle or "agent session", never a name; ADR-0012).

| ID | Source | Kind | May inform | Read |
|---|---|---|---|---|
| CR-01 | [WebPlotDigitizer documentation: Digitize](https://automeris.io/docs/digitize/) | `user-docs` | Axis calibration types (2D XY with linear, log and date axes; bar; polar and ternary later), box, pen and erase masks, foreground and background colour modes with a colour-distance threshold, the averaging-window, X-step, X-step-with-interpolation, custom-independents, bar and blob-detector algorithms as described, manual add, adjust and delete of points | Kickoff research for notes 06 (October 2026), agent session; read again 2026-10-30 by the C02 agent session for C02-T02 (calibration entry, scale kinds, value formats) |
| CR-02 | [WebPlotDigitizer documentation index](https://automeris.io/docs/) | `user-docs` | Product overview and the citation version (5.2) only | As CR-01 |
| CR-03 | Masson, Malacria, Vogel, Lank and Casiez, "ChartDetective" (CHI 2023): [HAL record](https://hal-lara.archives-ouvertes.fr/CRISTAL-LOKI/hal-04017638v1), [PDF](https://cs.uwaterloo.ca/~dmasson/pdfs/chartdetective.pdf) | `paper` | Comparison of vector and raster extraction (78% lower relative error for vector); interactive selection and filtering of marks | As CR-01 |
| CR-04 | Microsoft Research, ChartSense ([project page](https://www.microsoft.com/en-us/research/?p=365924)) | `paper` | Chart-type classification followed by per-type extraction with user interaction | As CR-01 |
| CR-05 | Drevon et al. 2017, *Behavior Modification* ([PubMed 27760807](https://www.ncbi.nlm.nih.gov/pubmed/27760807)) | `paper` | Reliability of manual digitizing; framing of extraction as an estimate | As CR-01 |
| CR-06 | EpiCurveBench (2026, preprint, [arXiv 2605.27195](https://arxiv.org/html/2605.27195)) | `paper` | Human inter-annotator agreement on dense charts (mean ECS 90.9%); why extraction is shown as an editable estimate | As CR-01 |
| CR-07 | Natural cubic spline interpolation, as in numerical-analysis textbooks (for example Burden and Faires, *Numerical Analysis*, chapter 3, "Cubic Spline Interpolation"; Press et al., *Numerical Recipes*, §3.3), solved with the tridiagonal (Thomas) algorithm | `reference` | X step with interpolation: the spline through found points, zero second derivative at both ends | 2026-10-31, C02 agent session, from general knowledge of the textbook method for C02-T03 |
| CR-L1 | [automeris-io/WebPlotDigitizer](https://github.com/automeris-io/WebPlotDigitizer) front-page README | `licence-fact` | Licence and status only: frontend AGPL-3.0, cloud parts closed. **Not a design source** | As CR-01 (notes 06 quotes the README only) |
| CR-L2 | Engauge Digitizer packaging records ([Zenodo 3558440](https://zenodo.org/records/3558440)) | `licence-fact` | Licence (GPL) and dormant status only. **Not a design source** | As CR-01 |

## Exposures

Record here any time someone working on C02 saw WPD's or Engauge's source, what they saw, and what was done about it (the procedure's "after an exposure" step). None recorded.

| Date | Who (handle or "agent session") | What was seen | Action |
|---|---|---|---|
| — | — | — | — |

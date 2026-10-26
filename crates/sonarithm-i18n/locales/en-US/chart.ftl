# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0

### Chart extraction (docs/architecture/10-ingestion-and-extraction.md §5; C01-T12).
### Spoken when a chart is opened and printed by `sonarithm inspect`. $points counts every
### value; $exact were read exactly, $calibrated were measured from a vector drawing against
### its axes, and $estimated were estimated (from an image or a model).

chart-extraction-summary =
    { $points ->
        [one] { $points } point
       *[other] { $points } points
    }: { $exact } exact, { $calibrated } calibrated, { $estimated } estimated

# As chart-extraction-summary, when some values have a confidence: $confidence is the
# median confidence in [0, 1], with two decimals.
chart-extraction-summary-confidence =
    { $points ->
        [one] { $points } point
       *[other] { $points } points
    }: { $exact } exact, { $calibrated } calibrated, { $estimated } estimated, median confidence { $confidence }

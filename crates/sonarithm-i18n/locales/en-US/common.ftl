# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0

### Shared phrases (docs/architecture/12-cross-cutting-concerns.md §5).
### Message IDs are shared by the Rust crates (sonarithm-i18n) and the desktop UI
### (@sonarithm/i18n). Never build a sentence by joining messages: add a message with
### arguments instead (coding-standards §12.7).

-product-name = SONARITHM

# A number on its own, formatted for the locale.
common-number = { $value }

# A position among siblings: "195 of 365".
common-position = { $index } of { $total }

# A count of data points.
common-point-count =
    { $count ->
        [one] { $count } point
       *[other] { $count } points
    }

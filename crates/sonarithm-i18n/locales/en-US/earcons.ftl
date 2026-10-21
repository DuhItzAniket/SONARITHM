# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0

### Earcon text equivalents (keyboard-and-navigation §11; A11Y-08, A11Y-24). E-BOUNDARY,
### E-INVALID, E-LEVEL and E-GAP use the announce- messages for edges, reasons, levels and
### missing values; these cover the rest.

# E-LANDMARK.
earcon-landmark = Insight: { $name }
# E-THRESHOLD: the value with its unit, already formatted.
earcon-threshold = Crossed { $value }
# E-MODE.
earcon-mode =
    { $state ->
        [on] { $name } on
       *[off] { $name } off
    }
# E-STREAM.
earcon-stream = { $stream }: { $event }
# E-ALARM.
earcon-alarm = Alarm: { $condition }

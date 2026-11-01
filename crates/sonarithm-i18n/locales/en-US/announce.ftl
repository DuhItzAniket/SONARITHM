# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0

### AUDAXIS announcements (keyboard-and-navigation §8, §11; 08 §4).
### An announcement is a list of parts in the listener's chosen order (A11Y-12). Each
### part is one message; announce-join puts two parts together, so the order stays
### configurable and every sentence still comes from the catalogue. Data names (series,
### categories, insight labels) and formatted values arrive as arguments.

# Joins two parts of one announcement: "28.7 degrees C. July 14".
announce-join = { $first }. { $rest }

## Parts

# A data name or a formatted value used as a part on its own.
announce-text = { $text }

# A range of values or x positions: "24.9 to 28.7 degrees Celsius".
announce-range = { $from } to { $to }

# The verbose value range of a series or segment.
announce-range-verbose = Range { $from } to { $to }

# A position among peers. Verbose names the level.
announce-position = { $index } of { $total }
announce-position-verbose =
    { $kind ->
        [series] Series { $index } of { $total }
        [segment] Segment { $index } of { $total }
        [point] Point { $index } of { $total }
        [insight] Insight { $index } of { $total }
        [axis] Axis { $index } of { $total }
       *[other] { $index } of { $total }
    }

# The level, spoken when it changes.
announce-level =
    { $kind ->
        [dataset] dataset
        [series] series
        [segment] segment
        [point] point
        [axes] axes
        [axis] axis
        [insights] insights
        [insight-group] insight type
        [insight] insight
        [filters] filters
        [filter] filter
       *[other] level
    }

# A point without a value (03 §5).
announce-missing = no value

# The lateral or go-to move landed on the nearest x, not the same x.
announce-nearest = nearest

# Values that are not stated plainly carry a qualifier (03 §4 rule 3, AGENTS rule 7).
announce-provenance =
    { $class ->
        [derived] derived from other values
        [synthetic] synthetic
        [cv_estimate] estimated from a chart image
        [ml_estimate] estimated by a machine-learning model
        [cloud_estimate] estimated by a cloud model
       *[other] estimated
    }
announce-provenance-terse =
    { $class ->
        [derived] derived
        [synthetic] synthetic
       *[other] estimated
    }
announce-provenance-confidence = { $qualifier }, confidence { $confidence }

# The estimates in a series or segment (C02-T09, 03 §4 rule 3): terse says whether all or
# some points are estimates; verbose says how many, from what, and the median confidence.
announce-estimated-terse =
    { $share ->
        [all] estimated
       *[some] partly estimated
    }
announce-estimated = { $estimated } of { $total } points { $qualifier }
announce-estimated-confidence = { $mix }, median confidence { $confidence }
# Where a series' reading is least certain, relative to the series (OD-07: no absolute cut-off).
announce-lowest-confidence = lowest confidence around { $x }, median { $confidence }

# A count of points.
announce-points =
    { $count ->
        [one] { $count } point
       *[other] { $count } points
    }

# The dataset summary (A11Y-01).
announce-series-count =
    { $count ->
        [one] { $count } series
       *[other] { $count } series
    }

announce-axis =
    { $axis ->
        [x] X axis
       *[y] Y axis
    }
announce-axes = Axes
announce-insight-count =
    { $count ->
        [0] No insights
        [one] { $count } insight
       *[other] { $count } insights
    }
announce-insight = Insight
announce-filters = Filters
announce-filter-state =
    { $included ->
        [yes] included
       *[no] excluded
    }

# Hints, verbose only, on the first visit to a kind of node.
announce-hint-explore = Press { $key } to explore
announce-hint-points = Press { $key } to explore points
announce-hint-insight = Press { $key } to go to the point

## Earcon text equivalents (keyboard §11, A11Y-08, A11Y-24)

announce-boundary =
    { $edge ->
        [first] First
        [last] Last
        [top] Top level
       *[bottom] No lower level
    }
announce-boundary-verbose =
    { $edge ->
        [first] First. Nothing further to the left
        [last] Last. Nothing further to the right
        [top] Top level. Nothing above
       *[bottom] No lower level
    }
announce-invalid =
    { $reason ->
        [one-series] No other series
        [not-in-series] Not in a series
        [no-values] No values
        [no-insights] No insights
        [not-found] Not found
        [no-landmark] No such landmark
       *[other] Not available
    }
announce-invalid-verbose =
    { $reason ->
        [one-series] Not available: this dataset has one series
        [not-in-series] Not available: move into a series first
        [no-values] Not available: this series has no values
        [no-insights] Not available: this series has no insights
        [not-found] Not available: no point matches
        [no-landmark] Not available: this chart has no such landmark
       *[other] Not available here
    }

## Responses

# The current value on pause (A11Y-09).
announce-paused = Paused

# "Where am I" neighbours (keyboard §6).
announce-previous = Previous { $text }
announce-next = Next { $text }
announce-excluded =
    { $count ->
        [0] All series included
        [one] { $count } series excluded
       *[other] { $count } series excluded
    }

## Times

# A time of day from its zero-padded fields: "14:02:11".
announce-time = { $hours }:{ $minutes }:{ $seconds }
# A date with its time of day.
announce-date-time = { $date }, { $time }

## The spoken legend (SDR-18; sound-design §7.3)

announce-legend-x = X axis { $field }, { $range }
announce-legend-y = { $series }, { $range }
announce-legend-mapping =
    { $polarity ->
        [positive] Pitch from { $low } to { $high } hertz; higher values sound higher
        [negative] Pitch from { $low } to { $high } hertz; higher values sound lower
       *[other] Pitch from { $low } to { $high } hertz
    }
announce-legend-mode =
    { $mode ->
        [presentation] Presentation mode
       *[analysis] Analysis mode
    }
announce-legend-cues =
    { $ticks ->
        [yes]
            { $reference ->
                [yes] Ticks and reference tone on
               *[no] Ticks on
            }
       *[no]
            { $reference ->
                [yes] Reference tone on
               *[no] No context cues
            }
    }

## Queries (B03-T09; keyboard-and-navigation §7): answers about the series under the
## cursor. $series is its name, $value and $x a point's value and position, $from, $to and
## $span quantities with units.

announce-query-minimum = Minimum of { $series }: { $value } at { $x }
announce-query-maximum = Maximum of { $series }: { $value } at { $x }
announce-query-range = Range of { $series }: { $from } to { $to }, span { $span }
announce-query-value-at = At { $x }: { $value }

## Export (B03-T09): said when an export starts while the explorer's filters exclude a
## series, because exports render every series.

announce-export-all-series = Export includes all series; exclusions apply to playback only.

## Audio output (B01-T08, HS-05): the device changed or failed. $device is the new device's
## name; $detail is the system's message; $paused is yes when playback was paused by it.

announce-audio-device-changed =
    { $paused ->
        [yes] Audio device changed to { $device }. Playback is paused.
       *[no] Audio device changed to { $device }.
    }
announce-audio-lost =
    { $paused ->
        [yes] Audio output stopped: { $detail }. Playback is paused.
       *[no] Audio output stopped: { $detail }.
    }

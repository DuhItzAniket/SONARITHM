# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0

### Functions (CURVASONA; docs/architecture/07-engines.md, C03-T07 and C03-T08; F-191).
### A function's critical points as text, the text equivalent of its cues (A11Y-24,
### A11Y-28), and its summary. Numbers arrive formatted, without grouping separators;
### $expr is the expression as written (data, read as it is).

## The name of each kind of critical point or span, as an entry of the list names it.
function-kind-zero = zero
function-kind-local-maximum = local maximum
function-kind-local-minimum = local minimum
function-kind-inflection-point = inflection point
function-kind-vertical-asymptote = vertical asymptote
function-kind-horizontal-asymptote-to-the-left = horizontal asymptote to the left
function-kind-horizontal-asymptote-to-the-right = horizontal asymptote to the right
function-kind-hole = hole
function-kind-jump = jump
function-kind-undefined = undefined
function-kind-above-the-window = above the window
function-kind-below-the-window = below the window

## How many of one kind, in a summary. $count is a number.
function-count-zero =
    { $count ->
        [one] { $count } zero
       *[other] { $count } zeros
    }
function-count-local-maximum =
    { $count ->
        [one] { $count } local maximum
       *[other] { $count } local maxima
    }
function-count-local-minimum =
    { $count ->
        [one] { $count } local minimum
       *[other] { $count } local minima
    }
function-count-inflection-point =
    { $count ->
        [one] { $count } inflection point
       *[other] { $count } inflection points
    }
function-count-vertical-asymptote =
    { $count ->
        [one] { $count } vertical asymptote
       *[other] { $count } vertical asymptotes
    }
function-count-horizontal-asymptote-to-the-left =
    { $count ->
        [one] { $count } horizontal asymptote to the left
       *[other] { $count } horizontal asymptotes to the left
    }
function-count-horizontal-asymptote-to-the-right =
    { $count ->
        [one] { $count } horizontal asymptote to the right
       *[other] { $count } horizontal asymptotes to the right
    }
function-count-hole =
    { $count ->
        [one] { $count } hole
       *[other] { $count } holes
    }
function-count-jump =
    { $count ->
        [one] { $count } jump
       *[other] { $count } jumps
    }
function-count-undefined =
    { $count ->
        [one] { $count } undefined span
       *[other] { $count } undefined spans
    }
function-count-above-the-window =
    { $count ->
        [one] { $count } span above the window
       *[other] { $count } spans above the window
    }
function-count-below-the-window =
    { $count ->
        [one] { $count } span below the window
       *[other] { $count } spans below the window
    }

## An entry of the critical-point list. $label is a function-kind message; $x, $y and $end
## are numbers.
function-point = { $label } at x = { $x }
function-point-value = { $label } at x = { $x }, y = { $y }
function-span = { $label } from x = { $x } to { $end }
function-asymptote-value = { $label }, y = { $y }

## The summary: the expression, the domain, the grid and the counts. $grid is
## function-grid-samples or function-grid-step; $counts is function-count messages joined
## with function-list-separator.
function-summary = f(x) = { $expr } from x = { $min } to { $max }, { $grid }: { $counts }
function-summary-none = f(x) = { $expr } from x = { $min } to { $max }, { $grid }: no critical points
function-grid-samples =
    { $count ->
        [one] { $count } sample
       *[other] { $count } samples
    }
function-grid-step = a step of { $step }
function-list-separator = {", "}

## Insight labels in announcements (AUDAXIS): for an insight of analyzer A with label L,
## AUDAXIS speaks insight-A-L when the catalogue has it, else L as written (F-191).
insight-critical-points-zero = { function-kind-zero }
insight-critical-points-local-maximum = { function-kind-local-maximum }
insight-critical-points-local-minimum = { function-kind-local-minimum }
insight-critical-points-inflection-point = { function-kind-inflection-point }
insight-critical-points-vertical-asymptote = { function-kind-vertical-asymptote }
insight-critical-points-horizontal-asymptote-to-the-left = { function-kind-horizontal-asymptote-to-the-left }
insight-critical-points-horizontal-asymptote-to-the-right = { function-kind-horizontal-asymptote-to-the-right }
insight-critical-points-hole = { function-kind-hole }
insight-critical-points-jump = { function-kind-jump }
insight-critical-points-undefined = { function-kind-undefined }
insight-critical-points-above-the-window = { function-kind-above-the-window }
insight-critical-points-below-the-window = { function-kind-below-the-window }

# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0

### Desktop UI text. Components never contain user-facing literals; the
### sonarithm/no-literal-ui-string lint enforces it (coding-standards §12.7).

ui-product-name = { -product-name }
# The global command that silences all output (keyboard-and-navigation §5.1).
ui-stop-all-audio = Stop all audio
ui-command-palette = Command palette

## Application frame (B01-T03; the full frame comes with B02-T06).

ui-region-workspace = Workspace
ui-workspace-empty = No dataset is open.
ui-starting = Starting SONARITHM…
# The app core could not be reached; $detail is the app core's message.
ui-start-failed = SONARITHM could not start. { $detail }

## Commands and controls (B02-T04). Labels name the thing or the action with a verb, in
## sentence case (design-language §12).

ui-transport = Transport
ui-play = Play
ui-pause = Pause
ui-stop = Stop
ui-loop = Loop
# A control's name with its shortcut, for tooltips; $name is the name, $key the binding.
ui-name-with-shortcut = { $name } ({ $key })
ui-menu-file = File
ui-open-file = Open file…
ui-export-audio = Export audio…
ui-recent-files = Recent files
ui-close-window = Close window
ui-modalities = Modalities
ui-modality-sonification = Sonification
ui-modality-speech = Speech
ui-modality-braille = Braille
ui-tab-plot = Plot
ui-tab-table = Table
ui-tab-spec = Spec
# Confirmation dialog; buttons are verbs, never Yes and No (design-language §12 rule 5).
ui-discard-title = Discard unsaved changes?
ui-discard-body = { $name } has unsaved changes. They are lost if you discard them.
ui-discard = Discard
ui-cancel = Cancel
ui-section-channels = Channels
ui-section-context = Context cues

## Data components (B02-T05).

ui-project = Project
ui-tree-fields = Fields
ui-tree-tracks = Tracks
ui-data-table = Data table
ui-reference-frequency = Reference frequency
ui-duration = Duration
ui-scale = Scale
ui-scale-linear = Linear
ui-scale-log = Logarithmic
ui-concept = Concept
ui-concept-temperature = Temperature
ui-concept-price = Price
ui-concept-count = Count
ui-announcement-parts = Announcement parts
ui-part-value = Value
ui-part-position = Position
ui-part-series = Series
# Reordering a list item without dragging (design-language §11); $name is the item.
ui-move-up = Move { $name } up
ui-move-down = Move { $name } down
ui-reorder = Reorder { $name }
# A number outside its field's range; $low and $high are the limits with their unit.
ui-error-out-of-range = Enter a value from { $low } to { $high }.

## Data table (B04-T05; keyboard-and-navigation §5.6). The table holds a window of rows at a
## time; the range says where those rows sit in the whole table.

# The table's name; $name is the dataset.
ui-table-label = Rows of { $name }
# The loaded rows; $from, $to and $total are row numbers.
ui-table-rows = Rows { $from } to { $to } of { $total }
ui-table-loading = Loading rows…
ui-table-no-rows = The dataset has no rows.
ui-table-row-number = Row
ui-table-provenance = Provenance
# A column header with its unit; $name is the field and $unit its symbol.
ui-table-heading-unit = { $name } ({ $unit })
# Values stated plainly (03 §4 rule 3); estimates use announce-provenance, as speech does.
ui-table-provenance-plain =
    { $class ->
        [vector] calibrated
        [user_edited] edited
       *[other] exact
    }
ui-table-no-value = no value
ui-table-yes = yes
ui-table-no = no
# Said after a sort; $field is the column header.
ui-table-sorted-ascending = Sorted by { $field }, ascending
ui-table-sorted-descending = Sorted by { $field }, descending
ui-go-to-row = Go to row
ui-go-to-row-number = Row number
ui-go-to-row-action = Go

## The cursor (B04-T06, B03-T09; 08 §1). Level words name the breadcrumb's levels; $name is
## a data name, shown as it is, and empty for a group (axes, insights, filters). A level
## this build does not know shows its name alone.

ui-cursor-trail = Cursor position
ui-cursor-step =
    { $kind ->
        [dataset] Dataset { $name }
        [series] Series { $name }
        [segment] Segment { $name }
        [point] Point { $name }
        [axes] Axes
        [axis] Axis { $name }
        [insights] Insights
        [insightGroup] Insights { $name }
        [insight] Insight { $name }
        [filters] Filters
        [filter] Filter { $name }
       *[other] { $name }
    }
# A series the explorer's filters took out (B03-T09; keyboard-and-navigation §3); $name is
# its data name.
ui-cursor-excluded = { $name } excluded from playback and view

## Preferences › Keyboard (B03-T09; keyboard-and-navigation §13). $name is a command's
## name, $keys a key combination as shown, $other another command's name.

ui-braille = Braille
ui-braille-cells = Braille display cells
ui-braille-cells-note = A web page cannot read your display's size. Leave this empty and the screen reader pans the line; set it and the line is fitted to the display, whole parts only.

## The braille line (B03-T10; keyboard-and-navigation §10, §14). $mode is review or
## sparkline.

ui-braille-line =
    { $mode ->
        [sparkline] Braille sparkline
       *[review] Braille line
    }
ui-braille-hint =
    { $mode ->
        [sparkline] One cell per point of the segment; a routing key moves the cursor to that point. Shift+B in the chart explorer shows the review line.
       *[review] The cursor's value and position. Shift+B in the chart explorer shows a sparkline of the segment.
    }
# Said after Shift+B.
# The braille field with no line (#513 review): it stays, so the focus never falls to the
# page.
ui-braille-off = Braille output is off. Press B in the chart explorer to turn it on.
ui-braille-waiting = The braille line shows the cursor's point once the cursor is on the chart.
ui-braille-mode-changed =
    { $mode ->
        [sparkline] Braille sparkline
       *[review] Braille review line
    }

ui-keyboard = Keyboard
ui-single-keys = Single-key commands in the chart explorer
ui-single-keys-note = Off: letters do nothing in the explorer. Arrows, Home, End and the Page keys still work, and every command stays in the menus and the palette.
ui-keys-preset-default = Use the default keys
ui-keys-preset-no-single = No single-key commands
ui-keys-caption = Every command and its keys. Keys are saved in your settings, never in a project.
ui-keys-command = Command
ui-keys-keys = Keys
ui-keys-actions = Change
ui-keys-change = Change…
ui-keys-change-for = Change keys for { $name }
ui-keys-reset = Reset
ui-keys-reset-for = Reset keys for { $name }
ui-keys-recorder = New keys
ui-keys-press = Press the new keys. Tab moves on and Escape cancels.
ui-keys-refused =
    { $keys } cannot be used: { $rule ->
        [KB-02] single keys work only in the chart explorer
        [KB-03] a screen reader uses it
        [KB-04] it conflicts with a keyboard layout or a screen reader
       *[reserved] it is reserved
    }.
ui-keys-shadows = { $keys } is already used by { $other }. Save to give it to both; the one listed first runs.
ui-keys-free = { $keys } is free.
ui-keys-save = Save

## The chart explorer (B03-T09; 08 §5, keyboard-and-navigation §4, §5.5). $name is the
## dataset's name, shown as it is.

ui-explorer-label = Chart explorer, { $name }
ui-explorer-hint = Arrow keys move through the chart. Press H for its keys.
ui-explorer-start = At the dataset
ui-explorer-go-to = Go to an x value
ui-explorer-go-to-value = X value
ui-explorer-go-to-action = Go
ui-explorer-find = Find a value
ui-explorer-find-value = Value
ui-explorer-find-action = Find
# The query menu (Q; keyboard §7): answers are announced and never move the cursor.
ui-explorer-query = Ask
ui-query-summary = Summary
ui-query-minimum = Minimum
ui-query-maximum = Maximum
ui-query-range = Range
ui-query-value-at = Value at x…
ui-explorer-value-at = Value at an x value
ui-explorer-value-at-action = Ask
# The cursor's row in the dataset's own order, from 1; $index is a number.
ui-cursor-row = Data row { $index }
ui-cursor-none = No row is selected.
# The Plot tab's image name and description (A11Y-01): $field is the y column's header and $text
# the x column's; $name is the dataset and $total its row count.
ui-plot-label = { $field } against { $text } in { $name }
ui-plot-description = { $total } rows. The Table tab lists every value, and the cursor readout gives the selected row.
ui-plot-no-fields = This dataset has no two numeric fields to plot.
# The plot's zoom and pan toolbar (B04-T07), and the x range it shows: $field is the x
# column's header, $from and $to are numbers.
ui-plot-view = Plot view
ui-plot-range = Showing { $field } from { $from } to { $to }

## Application frame (B02-T06; design-language §4, keyboard-and-navigation §3).

ui-region-inspector = Inspector
ui-region-status = Status
ui-region-switcher = Regions
# A collapsed region's only control, kept in the F6 cycle; $name is the region.
ui-expand-region = Expand { $name }
# A window splitter's name; $name is the region it resizes.
ui-resize-region = Resize { $name }
# A splitter's value; $value is a width in CSS pixels.
ui-width-pixels = { $value } pixels
ui-tab-announcements = Announcements
ui-tab-diagnostics = Diagnostics
ui-no-announcements = No announcements yet.
ui-no-diagnostics = No diagnostics.
ui-inspector-empty = Nothing is selected.

## Preferences (B01-T08). The volume note says in words what the slider does not do.

ui-preferences = Preferences
ui-audio = Audio
ui-output-device = Output device
ui-system-default-device = System default
ui-sonification-volume = Sonification volume
ui-volume-note = Sets the level of SONARITHM's sound only; the system volume does not change.
# ADR-0016, B04-T04: a latency measured for the selected device, used for the playhead.
ui-output-latency = Measured output latency
ui-output-latency-note = For the selected device, from a loopback or cable test. The playhead uses it instead of the system's estimate; leave it empty to use the estimate.
ui-display = Display
ui-gpu-high-performance = Use the high-performance GPU for plots
ui-gpu-restart-hint = Takes effect after a restart.
ui-gpu-unavailable = Only Windows lets SONARITHM choose the GPU; this system makes its own choice.
# Said and shown at start-up when the GPU setting could not be applied (B04-T02).
ui-gpu-fallback = The high-performance GPU setting will apply after SONARITHM is fully closed and started again.

## Functions (C03-T08b; ADR-0017 "Functions"). Expressions and numbers are data, shown as
## they are; the critical points' own text comes from the app core.

ui-function-new = New function
ui-function-expression = Expression in x
ui-function-expression-hint = For example x^2 - 1 or sin(x) / x.
# $text is the expression as the app core reads it (its canonical form).
ui-function-reads-as = Reads as: { $text }
# Why the expression cannot be used, for an error with no place in the text, such as a
# budget or a variable count. (An error with a place is the app core's own message, which
# already says where reading stopped.)
ui-function-expression-refused = The expression cannot be used: { $detail }
# When the app core gives no reason.
ui-function-expression-unreadable = The expression does not read.
# A number field left empty when Open is pressed; $name is the field's label.
ui-function-number-needed = Enter a number for { $name }.
ui-function-from = From x
ui-function-to = To x
ui-function-domain-hint = The x range the function is sampled over.
ui-function-samples = Samples
ui-function-open = Open the function
ui-function-section = Function
ui-function-points = Critical points
ui-function-points-none = No critical points in this range.
# Said when a listed feature covers no single row, such as a horizontal asymptote; $name is
# its text.
ui-function-no-row = { $name } has no row to go to.
ui-audify-section = Audification calculator
ui-audify-k = Speed-up k
ui-audify-k-hint = How many times faster than the data's own rate the series plays.
ui-audify-seismic = Use the seismic speed-up (k = 2200)
ui-audify-readout = Audification readout
ui-audify-duration = Duration
ui-audify-band = Band
ui-audify-audible = Audible part
# $value is a duration in seconds.
ui-audify-seconds = { $value } s
# $low and $high are frequencies in hertz.
ui-audify-hertz-range = { $low } to { $high } Hz
ui-audify-audible-none = None
# Said once when k is committed, not while typing: $k the speed-up, $seconds how long the
# series plays, $low to $high its band, and $audibleLow to $audibleHigh the audible part.
ui-audify-summary = k { $k }: { $seconds } s, { $low } to { $high } Hz, audible { $audibleLow } to { $audibleHigh } Hz
# The same when none of the band is audible.
ui-audify-summary-inaudible = k { $k }: { $seconds } s, { $low } to { $high } Hz, none of it audible
ui-audify-partly = Part of the band is outside 20 Hz to 20 kHz, so not all of it will be heard.
ui-audify-will-not-compile = Will not compile: the whole band is outside 20 Hz to 20 kHz. Choose another speed-up.
# $detail is the app core's message.
ui-audify-unavailable = The calculator cannot read this dataset. { $detail }

## Commands (B02-T07; keyboard-and-navigation §5). Names say what the command does.

command-app-palette = Show the command palette
command-file-new = New project
command-file-open = Open a project or data file
command-file-save = Save the project
command-file-save-as = Save the project as
command-file-export = Export audio
command-data-new-function = New function…
command-edit-undo = Undo
command-edit-redo = Redo
command-edit-find = Find in this region
command-app-preferences = Preferences
command-help-context = Help for the focused element
command-help-keyboard = Keyboard shortcuts here
command-view-zoom-in = Zoom in
command-view-zoom-out = Zoom out
command-view-zoom-reset = Reset zoom
command-view-pan-left = Pan left
command-view-pan-right = Pan right
command-view-focus-explorer = Go to the chart explorer
command-view-announcements = Show the announcement history
command-transport-play-pause = Play or pause
command-transport-stop = Stop
command-transport-loop = Loop on or off
command-transport-stop-all = Stop all audio
command-view-next-region = Next region
command-view-prev-region = Previous region
command-view-menu-bar = Go to the menu bar
command-view-context-menu = Context menu
command-view-toggle-region = Collapse or expand this region
command-table-go-to-row = Go to row…
# The chart explorer (B03-T09; keyboard-and-navigation §4.2, §5.4, §5.5).
command-nav-next = Next at this level
command-nav-previous = Previous at this level
command-nav-jump-forward = Jump forward 10 percent
command-nav-jump-back = Jump back 10 percent
command-nav-first = First at this level
command-nav-last = Last at this level
command-nav-into = Into the next level
command-nav-activate = Describe, toggle or go into
command-nav-out = Out to the level above
command-nav-next-series = Next series
command-nav-previous-series = Previous series
command-nav-maximum = Go to the maximum
command-nav-minimum = Go to the minimum
command-nav-next-insight = Next insight
command-nav-previous-insight = Previous insight
command-nav-x-axis = Go to the x axis
command-nav-y-axis = Go to the y axis
command-nav-insights = Go to the insights
command-nav-filters = Go to the filters
command-nav-dataset = Go to the dataset
command-nav-go-to = Go to an x value…
command-nav-find-value = Find a value…
command-nav-where-am-i = Where am I
command-nav-repeat = Repeat the last announcement
command-query-menu = Ask about this series…
command-help-explorer = Explorer keys
command-mod-speech = Speech on or off
command-mod-verbosity = Terse or verbose announcements
command-mod-sonification = Sonification on or off
command-mod-braille = Braille on or off
command-mod-braille-mode = Braille review line or sparkline
command-mod-visual = Visual highlight on or off
command-mod-reset = Reset the toggles to the profile's defaults
# The palette's result count, announced as it filters; $count is a number.
command-results =
    { $count ->
        [one] { $count } command
       *[other] { $count } commands
    }
command-no-results = No commands match.
keyboard-reference-title = Keyboard shortcuts
keyboard-no-binding = No shortcut

## Chart image calibration (C02-T10; 10 §3.1, §5; keyboard-and-navigation §5.11). A PNG or
## JPEG chart is read through anchors the user enters on each axis: a pixel and the value it
## stands for. $name is the image's file name, shown as it is.

command-chart-open-image = Open chart image…
ui-chart-image-open-title = Open chart image
ui-chart-image-path = Chart image file
ui-chart-image-path-hint = The full path of a PNG or JPEG chart image.
ui-chart-image-open = Open
ui-calibrate-title = Calibrate { $name }
# The image's text alternative. $x and $y are its width and height in pixels.
ui-calibrate-image = { $name }, a chart image of { $x } by { $y } pixels
ui-calibrate-crosshair-x = Crosshair x (pixels)
ui-calibrate-crosshair-y = Crosshair y (pixels)
# A crosshair position. $index is a pixel number; $value is the value there once the axis is
# calibrated: a number, a date or a category label.
ui-calibrate-pixel = Pixel { $index }
ui-calibrate-pixel-value = Pixel { $index }, value { $value }
ui-calibrate-pixel-x = Pixel x
ui-calibrate-pixel-y = Pixel y
ui-calibrate-axis-x = x axis
ui-calibrate-axis-y = y axis
# An axis's scale picker. $axisLabel is ui-calibrate-axis-x or ui-calibrate-axis-y.
ui-calibrate-scale-of = { $axisLabel } scale
ui-calibrate-axis = Axis of the new anchor
ui-calibrate-scale-linear = Linear
ui-calibrate-scale-log = Logarithmic
ui-calibrate-scale-date = Date
ui-calibrate-scale-category = Category
ui-calibrate-value = Value at the crosshair
ui-calibrate-value-number-hint = The axis value at the crosshair, as its tick label reads.
ui-calibrate-value-date-hint = A date at the crosshair, such as 2024-01-31.
ui-calibrate-value-category-hint = The category's label at the crosshair, as the chart shows it.
ui-calibrate-add = Add anchor at the crosshair
# One anchor. $label is its short name (x1, y2); $x and $y its pixel; $value as entered.
ui-calibrate-anchor = { $label }: pixel { $x }, { $y }, value { $value }
ui-calibrate-anchor-delete = Delete { $label }
ui-calibrate-anchors-none = No anchors yet. Add at least two.
ui-calibrate-anchor-added = { $label } added.
ui-calibrate-anchor-deleted = { $label } deleted.
ui-calibrate-checks = Checks
# How each axis stands. $axisLabel is ui-calibrate-axis-x or ui-calibrate-axis-y; $count is a
# number of anchors; $value is the fit's residual as a percentage of the axis.
ui-calibrate-axis-pending =
    { $count ->
        [one] { $axisLabel }: { $count } anchor, at least 2 needed
       *[other] { $axisLabel }: { $count } anchors, at least 2 needed
    }
ui-calibrate-axis-fits = { $axisLabel }: { $count } anchors fit within { $value }
ui-calibrate-apply = Read the chart
ui-calibrate-not-ready = Add at least two anchors on each axis, then read the chart.
# The area of the image that is read (#516).
ui-calibrate-area = Area to read
ui-calibrate-area-anchors = The anchors' area
ui-calibrate-area-detected = The plot area found from the axis lines
ui-calibrate-area-image = The whole image
ui-calibrate-area-hint = The anchors' area spans the outermost anchors on each axis.
ui-calibrate-area-none-hint = The anchors' area spans the outermost anchors on each axis. No plot area was found from the axis lines.
# A check or read whose image session has ended (SNR-APP-096).
ui-calibrate-session-ended = The image is no longer open. Cancel, then open the image again.

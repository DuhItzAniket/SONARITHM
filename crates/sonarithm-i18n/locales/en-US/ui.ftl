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

## Commands (B02-T07; keyboard-and-navigation §5). Names say what the command does.

command-app-palette = Show the command palette
command-file-new = New project
command-file-open = Open a project or data file
command-file-save = Save the project
command-file-save-as = Save the project as
command-file-export = Export audio
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
command-help-explorer = Explorer keys
# The palette's result count, announced as it filters; $count is a number.
command-results =
    { $count ->
        [one] { $count } command
       *[other] { $count } commands
    }
command-no-results = No commands match.
keyboard-reference-title = Keyboard shortcuts
keyboard-no-binding = No shortcut

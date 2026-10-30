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
command-view-focus-explorer = Go to the chart explorer
command-view-announcements = Show the announcement history
command-transport-play-pause = Play or pause
command-transport-stop-all = Stop all audio
command-view-next-region = Next region
command-view-prev-region = Previous region
command-view-menu-bar = Go to the menu bar
command-view-context-menu = Context menu
command-view-toggle-region = Collapse or expand this region
# The palette's result count, announced as it filters; $count is a number.
command-results =
    { $count ->
        [one] { $count } command
       *[other] { $count } commands
    }
command-no-results = No commands match.
keyboard-reference-title = Keyboard shortcuts
keyboard-no-binding = No shortcut

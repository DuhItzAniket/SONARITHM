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

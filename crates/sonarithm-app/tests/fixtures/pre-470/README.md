# Pre-#470 project fixture

`pre-470.sonarithm` was saved by the build of `main` at `cf0f03ee`, the last commit before #470 moved arrow from 60.0 to 60.2 (F-222). Its fingerprints were computed by that build. It embeds `d1` (`temps.csv`, three rows) and links `d2` to `more.csv` next to it.

It was written by a one-off test run on that commit through the app's own commands (`dataset_open`, `spec_set`, `project_save_as` with data embedded, then `project_open`, `dataset_open` of `more.csv` and `project_save_as` without embedding). The test was not committed. Do not regenerate the fixture with a current build: its purpose is to hold fingerprints that this build computes differently (RFC-0002 §13, #476).

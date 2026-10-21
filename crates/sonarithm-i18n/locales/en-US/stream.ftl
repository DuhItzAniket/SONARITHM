# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0

### Live stream notices (keyboard-and-navigation §9.1, A11Y-16; B06-T05). The app core
### speaks at most one notice per stream every few seconds and sums up the events since
### the last one. $stream is the stream's name as the user knows it (its file or address);
### $detail is the stream's own words for an error.

## One event

stream-connected = { $stream } connected
stream-disconnected = { $stream } disconnected, reconnecting
stream-error = { $stream } error: { $detail }
stream-alarm = Alarm: { $stream }, { $detail }
stream-event = { $stream }: { $detail }

## A summary of several events of one kind within $seconds seconds. $kind selects the
## kind: threshold, anomaly, connected, disconnected, error or alarm.

stream-summary = { $stream }: { $count } { $kind ->
        [threshold] { $count ->
            [one] threshold event
           *[other] threshold events
        }
        [anomaly] { $count ->
            [one] anomaly
           *[other] anomalies
        }
        [connected] { $count ->
            [one] connection
           *[other] connections
        }
        [disconnected] { $count ->
            [one] disconnection
           *[other] disconnections
        }
        [error] { $count ->
            [one] error
           *[other] errors
        }
       *[other] { $count ->
            [one] alarm
           *[other] alarms
        }
    } in { $seconds ->
        [one] { $seconds } second
       *[other] { $seconds } seconds
    }

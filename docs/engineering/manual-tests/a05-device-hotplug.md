# A05-T06: device hot-plug keeps the transport position

> **Task:** A05-T06 (issue #60). **Acceptance:** "Device unplug/replug keeps the transport position (manual test script)". **Design:** [06 §11](../../architecture/06-audio-engine.md#11-failure-handling), [sound-design §12](../../design/sound-design.md#12-hearing-safety) HS-05. **Program:** `crates/sonarithm-audio/examples/hotplug.rs`.

## What it checks

When the output device disappears or the system default changes, playback continues on the new default device at the same transport position, and it is paused until you resume it (HS-05). The program reports "Audio device changed to …", which is the text the app will announce.

## You need

- The development laptop (HW-01) or any machine with a working output device.
- A second output device that you can plug and unplug: a USB headset, a USB audio interface or a Bluetooth headset.
- About 10 minutes.

## Steps

1. Set the system volume to about 20%. Put headphones on only after checking the level at arm's length. The pulses are very quiet on purpose (about −50 dBFS); if you cannot hear them, raise the volume in small steps.
2. Plug in the second device and make it the system default output.
3. In the repository, run:

   ```bash
   cargo run -p sonarithm-audio --example hotplug --release
   ```

   You hear a soft pulse once a second, rising in pitch over each 10 s. The console prints the position once a second, for example `position   12.00 s  playing true  late callbacks 0  xruns 0`.
4. Wait until the position passes 20 s. Note the position.
5. **Unplug** the second device (or switch it off for Bluetooth).
6. Check the console: within about a second it prints `Audio device changed to <laptop speakers>; paused. Press Enter to resume.` The position stops advancing, and it is within 0.1 s of the position at the moment you unplugged.
7. Press Enter. The pulses resume on the laptop speakers, and the position continues from where it stopped (the next pulse has the pitch the 10 s pattern predicts).
8. **Plug the device back in** and make it the default again, if the OS does not do so itself. If the program prints another device change, check the position again as in step 6, then press Enter.
9. Repeat steps 5 to 8 twice.
10. Let the program run to the end (3 minutes) or stop it with Ctrl+C. Note the late-callback and xrun counts it printed last.

## Record

In issue #60, or in the A05-T06 PR:

| Item | Result |
|---|---|
| OS, device names, how the second device connects | |
| Position before unplugging and after the change (each of the 3 runs) | |
| Did playback pause after every change? | |
| Did resuming continue from the same position? | |
| Late callbacks and xruns at the end | |
| Anything unexpected (clicks, a loud onset, a missed change) | |

Pass: in all three runs the position after the change is within 0.1 s of the position at the change, playback paused, and resuming continued from there.

## Notes

- cpal follows a change of the default device by itself on some systems. The program then reports the change without rebuilding the stream; the pause and the position rules are the same.
- A device that disappears while it was opened by name (not the default) triggers a rebuild on the default device. The example opens the default device; the null-device test `unplugging_keeps_the_position_and_pauses` covers the rebuild path automatically.

# A08-T03: `sonarithm study run` with a screen reader

> **Status:** v1 (2026-10-21). **Owner:** project lead (runs it); A08 session (wrote it).
> **Related:** [manual tests](README.md) · [07, ECHOMETRIC, study runner](../../architecture/07-engines.md) · [pilot protocol](../../../examples/studies/pilot-001/README.md) · [keyboard and navigation](../../design/keyboard-and-navigation.md)

This checks that the study runner's dialogue works with a screen reader and the keyboard alone. It is a software check, run by the developer on themself: it is **not** a study, collects no participant data, and its log is not research data (OD-04). Delete the log afterwards.

## Prepare

- Windows with NVDA (record its version), or another screen reader; record which.
- A terminal: Windows Terminal or the classic console. Speech viewer on (NVDA menu → Tools → Speech Viewer) to copy what NVDA says.
- System volume low; headphones only after a level check at arm's length (see the hearing-safety note in the [manual tests README](README.md)).
- A build of the CLI: `cargo build --release -p sonarithm-cli`.

## Do

1. In the repository root, run:

   ```bash
   target/release/sonarithm study check examples/studies/pilot-001/protocol.toml
   ```

   Listen: the summary is four plain lines.
2. Run the pilot on the audio device:

   ```bash
   target/release/sonarithm study run examples/studies/pilot-001/protocol.toml --participant P000 --log pilot-check.jsonl
   ```

3. Using only the keyboard:
   - At trial 1, press Enter to listen, then type `h` and Enter: the help is read.
   - Type `r` and Enter: the sound plays again.
   - Type `abc` and Enter: the hint asks for a number from 0 to 120 dollars.
   - Type a number and Enter: in the practice block, the correct answer is read.
   - At trial 3, type `p` and Enter, then Enter at the pause prompt: trial 3 starts again.
   - At trial 4, type `q` and Enter: the "Saved after …" line is read.
4. Run the same command with `--resume` and finish the session; the last line read is "The session is complete. Thank you."
5. Delete `pilot-check.jsonl`.

## Record

In the A08-T03 pull request or issue #87:

- Screen reader and version, terminal, Windows build.
- Whether every prompt was read once, in order, without re-reading earlier lines, and whether typing echoed as expected.
- Whether any line was hard to understand by ear. Quote it from the speech viewer.
- Whether the sound and the speech overlapped. The question is read before the sound, and the answer prompt after it.

Pass: every step above can be done without sight, and nothing is read out of order or twice.

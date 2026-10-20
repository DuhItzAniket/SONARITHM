# PILOT-001: study runner pilot

A pilot protocol for `sonarithm study run` (A08-T03). It exercises the runner end to end: a practice block with feedback and a main block, two conditions after Smith & Walker (2005, Study 1), point estimation scored against the stimulus's ground truth, a break, pause and resume, and the JSONL log.

**It is a software pilot, not a study.** SONARITHM has no ethics review route yet, so no one may run it as research with participants (OD-04). Its automated test uses scripted answers and the silent audio output; the developer's own screen-reader check is in [docs/engineering/manual-tests/a08-study-run-screen-reader.md](../../../docs/engineering/manual-tests/a08-study-run-screen-reader.md).

| File | What it is |
|---|---|
| [protocol.toml](protocol.toml) | The protocol (`sonarithm-study/0`): instructions, the task, two conditions, a practice and a main block, GEN-002 stimuli |
| [none.json](none.json) | Condition `none`: the price series as 20 sine tones, no context (as `replication/smith-walker-2005/none.json`) |
| [dynamic.json](dynamic.json) | Condition `dynamic`: the same with the dynamic reference tone (as `replication/smith-walker-2005/dynamic.json`) |

The condition specs read their data from the dataset `stimulus`, which the runner fills with each trial's stimulus. Every trial uses the reconstructed Smith & Walker series and asks for the price at one hour, drawn from the trial's seed.

```bash
sonarithm study check examples/studies/pilot-001/protocol.toml
```

```bash
sonarithm study run examples/studies/pilot-001/protocol.toml --participant P000 --audio null
```

`--audio null` plays nothing and runs as fast as it can; leave it out to hear the sounds. Every prompt is one line: press Enter to listen, type an answer and press Enter; `r` listens again, `p` pauses, `q` saves and quits (continue with `--resume`), `h` repeats the help.

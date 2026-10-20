# Manual tests

Some acceptance checks need hardware, a person or both: a device that is unplugged, a cable from an output to an input, or a listener. Each script below says what to prepare, what to do and what to record. Record the results in the PR or issue named in the script, and log any measurement in the [findings log](../../research-paper/findings-log.md) and the [benchmarks log](../../research-paper/benchmarks-log.md).

**Hearing safety for every script.** Start with the system volume low, and use headphones only after a level check at arm's length. On Windows laptops with OEM audio processing (such as Nahimic or other "audio enhancement" software), the processing can add about 20 dB of gain after SONARITHM's limiter (F-032). The test programs play at −30 dBFS or lower and open streams 20 dB down.

| Script | Task | Needs |
|---|---|---|
| [a05-device-hotplug.md](a05-device-hotplug.md) | A05-T06: unplugging and replugging keeps the transport position | A second output device (USB or Bluetooth headset, or a USB audio interface) |
| [a05-cable-latency.md](a05-cable-latency.md) | A05-T13: key-to-onset latency by cable; ADR-0016 confirmation | A USB audio interface, one cable, NVDA |

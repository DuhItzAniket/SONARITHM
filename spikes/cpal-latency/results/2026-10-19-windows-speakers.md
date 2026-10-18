host Wasapi; device Speakers (Realtek(R) Audio); capture loopback; default config SupportedStreamConfig { channels: 2, sample_rate: 48000, buffer_size: Range { min: 480, max: 480 }, sample_format: F32 }; 200 trials per mode

| Mode | Period | Callback frames | Trials ok | Stray | Errors | Loopback xruns | Total p50 | p95 | p99 | max | Wait p50 | Path p50 | Path p95 | Reported p50 | Loopback peak | Max gap | Clock check p50 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cpal:128 | 480 | 480–1056 | 200/200 | 0 | 0 | 9 | 116.8 | 123.7 | 125.9 | 126.1 | 5.4 | 111.7 | 116.6 | 31.8 | -24.4 dBFS | 14.2 | -0.00 |
| cpal:256 | 480 | 480–1056 | 200/200 | 0 | 0 | 9 | 116.4 | 123.1 | 123.8 | 124.6 | 5.3 | 112.0 | 116.0 | 31.9 | -24.4 dBFS | 11.9 | -0.00 |
| cpal:512 | 480 | 480–1056 | 200/200 | 0 | 0 | 5 | 117.2 | 124.0 | 125.2 | 126.3 | 5.7 | 112.5 | 116.2 | 31.8 | -24.4 dBFS | 11.2 | -0.00 |
| cpal:default | 480 | 480–1056 | 200/200 | 0 | 0 | 8 | 116.9 | 124.0 | 125.5 | 126.3 | 5.3 | 112.1 | 116.4 | 31.7 | -24.4 dBFS | 16.2 | -0.00 |
| raw | 480 | 480–1056 | 200/200 | 0 | 0 | 19 | 79.2 | 83.7 | 84.2 | 84.4 | 4.9 | 74.3 | 74.7 | 12.0 | -45.0 dBFS | 12.1 | -0.00 |

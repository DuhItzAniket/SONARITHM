# Golden files

Checked-in expectations that more than one test suite reads.

| Directory | What | Checked by | Regenerate |
|---|---|---|---|
| `parity/` | A spec, its CSV, the Score the facade compiles for it (engine versions written as `*`, since they change with every release) and the SHA-256 of its default WAV render | `crates/sonarithm/tests/parity.rs` (Rust facade) and `bindings/python/tests/test_parity.py` (Python package): both front ends must produce this Score, value for value | `UPDATE_PARITY=1 cargo test -p sonarithm --test parity`, with the reason in the PR |

Golden audio and Score snapshots of the render engine join here with A05-T11.

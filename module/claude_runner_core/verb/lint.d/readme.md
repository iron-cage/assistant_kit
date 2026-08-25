# lint.d/

Layer scripts for the `lint` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l0` | Disabled: blocks host-native clippy; prints error and exits 1; reachable only by running it directly — the host entry rejects any `VERB_LAYER`. |
| `l1` | Container-internal: `cargo clippy -p claude_runner_core --all-targets --all-features -- -D warnings`; payload of `runbox .live`. |

# run.d/

Layer scripts for the `run` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l0` | Disabled: blocks host-native execution; prints error and exits 1; entered via `VERB_LAYER=l0`. |
| `l1` | Container-internal: `cargo run -p claude_storage --bin clg`, cwd-scoped to the module; payload of `runbox .live`. |

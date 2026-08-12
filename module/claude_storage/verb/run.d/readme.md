# run.d/

Layer scripts for the `run` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l0` | Disabled: blocks host-native execution; prints error and exits 1; reachable only via direct invocation (`./verb/run.d/l0`) — the top-level `verb/run` rejects any `VERB_LAYER` override outright. |
| `l1` | Container-internal: `cargo run -p claude_storage --bin clg`, cwd-scoped to the module; payload of `runbox .live`. |

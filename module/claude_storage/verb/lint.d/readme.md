# lint.d/

Layer scripts for the `lint` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l0` | Disabled: blocks host-native clippy; prints error and exits 1; entered via `VERB_LAYER=l0`. |
| `l1` | Container-internal: `cargo clippy -p claude_storage --all-targets --all-features -- -D warnings`; payload of `runbox .live`. |

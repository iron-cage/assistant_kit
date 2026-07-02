# test.d/

Layer scripts for the `test` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l0` | Disabled: blocks host-native execution; prints error and exits 1; entered via `VERB_LAYER=l0`. |
| `l1` | Container-internal: `RUSTFLAGS="-D warnings" cargo nextest run --all-features` inside Docker; entered via `VERB_LAYER=l1`. |

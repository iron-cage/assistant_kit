# test.d/

Layer scripts for the `test` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l0` | Host-native: `RUSTFLAGS="-D warnings" cargo nextest run --all-features` on host; no Docker; entered via `VERB_LAYER=l0`. |
| `l1` | Container-internal: `RUSTFLAGS="-D warnings" cargo nextest run --all-features` inside Docker; entered via `VERB_LAYER=l1` (set by runbox-run). |

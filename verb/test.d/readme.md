# test.d/

Layer scripts for the `test` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l0` | Disabled: blocks host-native execution; prints error and exits 1; entered via `VERB_LAYER=l0`. |
| `l1` | Container-internal: `RUSTFLAGS="-D warnings" cargo nextest run --all-features` inside Docker; entered via `VERB_LAYER=l1`. |
| `l1_filter` | Container-internal targeted: `cargo nextest run --all-features -E "$NEXTEST_FILTER"` inside Docker; entered by `runbox-run cmd_test1()` with `NEXTEST_FILTER` set. |

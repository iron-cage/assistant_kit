# test1.d/

Layer scripts for the `test1` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l1` | Container-internal: targeted `cargo nextest run -E "$VERB_FILTER"` with no level-3 overhead; entered via `cmd_test1` in runbox-run. |

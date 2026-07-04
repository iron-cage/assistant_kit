# test_only.d/

Layer scripts for the `test_only` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l1` | Container-internal: targeted `cargo nextest run -E "$NEXTEST_FILTER"` with no level-3 overhead; entered via `.test` when `runbox/plugins.sh` detects `NEXTEST_FILTER`. |

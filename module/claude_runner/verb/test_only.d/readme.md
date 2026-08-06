# test_only.d/

Layer scripts for the `test_only` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l1` | Container-internal: targeted `cargo nextest run --all-features --no-fail-fast "$NEXTEST_FILTER"` with no level-3 overhead; entered via the `.live` payload (`verb/test_only`) with `NEXTEST_FILTER` set. |

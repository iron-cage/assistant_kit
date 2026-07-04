# test_only.d/

Layer scripts for the `test_only` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l1` | Container-internal: `cargo nextest run --all-features $NEXTEST_FILTER`; entered via `VERB_LAYER=l1`. |

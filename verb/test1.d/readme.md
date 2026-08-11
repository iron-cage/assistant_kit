# test1.d/

Layer scripts for the `test1` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l1` | Container-internal: targeted `cargo nextest run --workspace --all-features --no-fail-fast -E "<filter>"` (positional arg); payload of `runbox .live` via `verb/test1`. |

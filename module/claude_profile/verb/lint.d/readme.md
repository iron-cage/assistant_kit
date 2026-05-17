# lint.d/

Layer scripts for the `lint` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l1` | Direct execution: `cargo clippy -p <module> --all-features -- -D warnings`; default when no `VERB_LAYER` set. |

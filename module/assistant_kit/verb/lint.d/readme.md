# lint.d/

Layer scripts for the `lint` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l1` | Direct execution: `cargo clippy -p assistant_kit --all-features -- -D warnings`; default when no `VERB_LAYER` set. |

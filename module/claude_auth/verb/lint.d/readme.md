# lint.d/

Layer scripts for the `lint` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l1` | Direct execution: `cargo clippy -p claude_auth --all-features -- -D warnings`; entered via `VERB_LAYER=l1`. |

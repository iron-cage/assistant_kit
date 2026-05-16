# lint.d/

Layer scripts for the `lint` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l1` | Direct execution: `cargo clippy -p claude_auth --all-features -- -D warnings`; entered via `VERB_LAYER=l1`. |
| `l2` | Host orchestration: `./run/runbox .lint`; default when no `VERB_LAYER` set. |

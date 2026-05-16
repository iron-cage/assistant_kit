# run.d/

Layer scripts for the `run` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l1` | Direct execution: `cargo run -p <module> --bin <binary>`; entered via `VERB_LAYER=l1`. |
| `l2` | Host orchestration: `./run/runbox .run`; default when no `VERB_LAYER` set. |

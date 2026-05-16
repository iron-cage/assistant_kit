# test.d/

Layer scripts for the `test` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l1` | Direct execution: `w3 .test level::3`; entered via `VERB_LAYER=l1`. |
| `l2` | Host orchestration: `./run/runbox .test`; default when no `VERB_LAYER` set. |

# test.d/

Layer scripts for the `test` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l0` | Host-native: `w3 .test level::3` on host; no Docker; default when no `VERB_LAYER` set. |
| `l1` | Container-internal: `w3 .test level::3` inside Docker; entered via `VERB_LAYER=l1` (set by runbox-run). |

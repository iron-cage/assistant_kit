# test.d/

Layer scripts for the `test` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l0` | Host-native: `w3 .test level::3` on host; no Docker; entered via `VERB_LAYER=l0`. |
| `l1` | Container-internal: `w3 .test level::3` inside Docker; entered via `VERB_LAYER=l1`. |

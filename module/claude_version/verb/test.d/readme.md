# test.d/

Layer scripts for the `test` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `docker` | Default: run tests in Docker via `verb-run` + `verb.yml`; no `VERB_LAYER` needed. |
| `l0` | Host-native: `w3 .test level::3` on host; no Docker; entered via `VERB_LAYER=l0`. |
| `l1` | Container-internal: `w3 .test level::3` inside Docker; entered via `VERB_LAYER=l1` (set by both verb-run and runbox-run). |

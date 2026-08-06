# test.d/

Layer scripts for the `test` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l0` | Disabled: blocks host-native execution; prints error and exits 1; entered via `VERB_LAYER=l0`. |
| `l1` | Container-internal: workspace-wide nextest + doc tests + clippy (`-D warnings`); default payload of `runbox .live` (config `script:`). |

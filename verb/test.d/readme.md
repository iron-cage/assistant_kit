# test.d/

Layer scripts for the `test` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l0` | Disabled: blocks host-native execution; prints error and exits 1; reachable only by running it directly — the host entry rejects any `VERB_LAYER`. |
| `l1` | Container-internal: workspace-wide nextest + doc tests + clippy (`-D warnings`); default payload of `runbox .live` (config `script:`). |

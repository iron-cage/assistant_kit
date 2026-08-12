# test.d/

Layer scripts for the `test` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l0` | Disabled: blocks host-native execution; prints error and exits 1; reachable only via direct invocation (`./verb/test.d/l0`) — the top-level `verb/test` rejects any `VERB_LAYER` override outright. |
| `l1` | Container-internal: nextest + doc tests + clippy (`-D warnings`), cwd-scoped to the module; payload of `runbox .live`. |

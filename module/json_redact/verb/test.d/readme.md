# test.d/

Layer scripts for the `test` verb dispatcher.

| File | Responsibility |
|------|----------------|
| `l1` | Container-internal: nextest + doc tests + clippy (`-D warnings`), cwd-scoped to the module; payload of `runbox .live`. |

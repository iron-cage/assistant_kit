# verb

Universal Action Protocol implementation across workspace modules.

| File/Directory | Responsibility |
|----------------|----------------|
| `docs/` | Per-verb reference for all 8 `do` protocol verbs |
| `test` | Dispatcher: run workspace test suite; delegates to `test.d/` layer by `VERB_LAYER`. |
| `test.d/` | Layer directory: `l0` (host-native, default), `l1` (container-internal). |

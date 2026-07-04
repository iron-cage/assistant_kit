# verb

Universal Action Protocol implementation at workspace scope.

| File/Directory | Responsibility |
|----------------|----------------|
| `docs/` | Per-verb reference for all 8 `do` protocol verbs |
| `build` | Compile all workspace crates |
| `test` | Dispatcher: run workspace test suite; delegates to `test.d/` layer by `VERB_LAYER` |
| `test1` | Dispatcher: run a single targeted test in container; requires a nextest filter arg |
| `test.d/` | Layer directory: `l0` (host-native), `l1` (container-internal), `l1_filter` (container-targeted) |
| `clean` | Remove generated artifacts for entire workspace |
| `lint` | Static analysis across all workspace crates |
| `run` | Unavailable at workspace scope (exit 3) |
| `verify` | Full checks: container test suite + udeps + audit |
| `verbs` | List available workspace-level verbs |
| `package_info` | Report workspace metadata as JSON |

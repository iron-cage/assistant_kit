# verb

Universal Action Protocol implementation at workspace scope.

| File/Directory | Responsibility |
|----------------|----------------|
| `docs/` | Per-verb reference for all 8 `do` protocol verbs |
| `build` | Compile all workspace crates |
| `test` | Dispatcher: run workspace test suite; delegates to `test.d/` layer by `VERB_LAYER` |
| `test.d/` | Layer directory: `l0` (host-native), `l1` (container-internal) |
| `test1` | Targeted test dispatcher: run one nextest filter in container; no level-3 overhead |
| `test1.d/` | Layer directory: `l1` (container-internal targeted nextest) |
| `clean` | Remove generated artifacts for entire workspace |
| `lint` | Static analysis across all workspace crates |
| `run` | Unavailable at workspace scope (exit 3) |
| `verify` | Full checks: container test suite + udeps + audit |
| `verbs` | List available workspace-level verbs |
| `package_info` | Report workspace metadata as JSON |

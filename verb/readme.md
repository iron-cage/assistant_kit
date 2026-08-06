# verb

Universal Action Protocol implementation at workspace scope.

| File/Directory | Responsibility |
|----------------|----------------|
| `docs/` | Per-verb reference for the workspace `do` protocol verbs |
| `build` | Compile all workspace crates |
| `test` | Run workspace suite in container: `runbox .live` with `test.d/l1` as payload |
| `test1` | Run a targeted nextest filter in container; requires a filter expression arg |
| `test.d/` | Layer directory: `l0` (disabled hard-error stub), `l1` (container-internal) |
| `test1.d/` | Layer directory: `l1` (container-internal targeted run) |
| `clean` | Remove generated artifacts for entire workspace |
| `lint` | Static analysis across all workspace crates |
| `run` | Unavailable at workspace scope (exit 3) |
| `verify` | Full checks: container test suite + udeps + audit |
| `verbs` | List available workspace-level verbs |
| `package_info` | Report workspace metadata as JSON |

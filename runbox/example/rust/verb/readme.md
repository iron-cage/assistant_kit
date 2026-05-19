# verb/

Shell scripts implementing the `do` protocol verbs for `rust_example` (runbox/container ecosystem).

| File | Responsibility |
|------|----------------|
| `build` | Build the container image via `./runbox/runbox .build`. |
| `test` | Dispatcher: run full test suite; delegates to `test.d/` layer by `VERB_LAYER`. |
| `test.d/` | Layer directory: `l1` (container), `l2` (runbox orchestration). |
| `clean` | Remove container image and named `target` volume. |
| `run` | Dispatcher: execute entry point; delegates to `run.d/` layer by `VERB_LAYER`. |
| `run.d/` | Layer directory: `l1` (container), `l2` (runbox orchestration). |
| `lint` | Dispatcher: run linter; delegates to `lint.d/` layer by `VERB_LAYER`. |
| `lint.d/` | Layer directory: `l1` (container), `l2` (runbox orchestration). |
| `verify` | Run offline tests via `./runbox/runbox .test.offline`. |
| `verbs` | List all available verbs and their availability (meta). |
| `package_info` | Report deterministic package metadata as JSON (meta). |

Canonical verbs support `--dry-run`: prints the delegated command without executing it. Meta verbs (`verbs`, `package_info`) do not.

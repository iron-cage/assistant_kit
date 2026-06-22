# verb/

Shell scripts implementing the `do` protocol verbs for `nodejs` example (container ecosystem).

| File | Responsibility |
|------|----------------|
| `build` | Build the container image via `./verb/.build`. |
| `test` | Dispatcher: run full test suite; delegates to `test.d/` layer by `VERB_LAYER`. |
| `test.d/` | Layer directory: `l1` (container), `l2` (container orchestration). |
| `clean` | Remove container image and named `node_modules` volume. |
| `run` | Dispatcher: execute entry point; delegates to `run.d/` layer by `VERB_LAYER`. |
| `run.d/` | Layer directory: `l1` (container), `l2` (container orchestration). |
| `lint` | Dispatcher: run linter; delegates to `lint.d/` layer by `VERB_LAYER`. |
| `lint.d/` | Layer directory: `l1` (container), `l2` (container orchestration). |
| `verify` | Run offline tests via `./verb/.test.offline`. |
| `verbs` | List all available verbs and their availability (meta). |
| `package_info` | Report deterministic package metadata as JSON (meta). |

Canonical verbs support `--dry-run`: prints the delegated command without executing it. Meta verbs (`verbs`, `package_info`) do not.

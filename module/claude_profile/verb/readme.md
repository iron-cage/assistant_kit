# verb/

Shell scripts implementing the `do` protocol verbs for `claude_profile` (cargo ecosystem).

| File | Responsibility |
|------|---------------|
| `build` | Compile project artifacts via `cargo build`. |
| `test` | Dispatcher: run full test suite; delegates to `test.d/` layer by `VERB_LAYER`. |
| `test.d/` | Layer directory: `l1` (container), `l2` (runbox orchestration). |
| `clean` | Remove generated artifacts and caches via `cargo clean`. |
| `run` | Dispatcher: execute entry point; delegates to `run.d/` layer by `VERB_LAYER`. |
| `run.d/` | Layer directory: `l1` (cargo run). |
| `lint` | Dispatcher: run linter; delegates to `lint.d/` layer by `VERB_LAYER`. |
| `lint.d/` | Layer directory: `l1` (cargo clippy). |
| `verify` | Run full pre-push gate: tests, deps analysis, audit. |
| `verbs` | List all available verbs and their availability (meta). |
| `package_info` | Report deterministic package metadata as JSON (meta). |

Canonical verbs support `--dry-run`: prints the delegated command without executing it. Meta verbs (`verbs`, `package_info`) do not.

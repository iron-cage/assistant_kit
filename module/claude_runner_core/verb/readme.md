# verb/

Shell scripts implementing the `do` protocol verbs for `claude_runner_core` (cargo ecosystem).

| File | Responsibility |
|------|---------------|
| `build` | Compile project artifacts via `cargo build`. |
| `test/` | Run full test suite; directory form (`default` → `l1` → `w3 .test level::3`). |
| `clean` | Remove generated artifacts and caches via `cargo clean`. |
| `run` | Execute entry point binary — unavailable for this library crate. |
| `lint` | Run static analysis and style checks via `cargo clippy`. |
| `verify` | Run full pre-push gate: tests, deps analysis, audit. |
| `verbs` | List all available verbs and their availability (meta). |
| `package_info` | Report deterministic package metadata as JSON (meta). |

Canonical verbs support `--dry-run`: prints the delegated command without executing it. Meta verbs (`verbs`, `package_info`) do not.

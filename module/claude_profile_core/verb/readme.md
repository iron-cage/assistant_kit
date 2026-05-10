# verb/

Shell scripts implementing the `do` protocol verbs for `claude_profile_core` (cargo ecosystem).

| File | Responsibility |
|------|---------------|
| `build` | Compile project artifacts via `cargo build`. |
| `test` | Run full test suite via `w3 .test level::3`. |
| `clean` | Remove generated artifacts and caches via `cargo clean`. |
| `run` | Execute entry point binary — unavailable for this library crate. |
| `lint` | Run static analysis and style checks via `cargo clippy`. |
| `verbs` | List all available verbs and their availability (meta). |
| `detect` | Report the detected ecosystem and signal confidence (meta). |

All action scripts support `--dry-run`: prints the delegated command without executing it.

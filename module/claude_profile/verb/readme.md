# verb/

Shell scripts implementing the `do` protocol verbs for `claude_profile` (cargo ecosystem).

| File | Responsibility |
|------|---------------|
| `build` | Compile project artifacts via `cargo build`. |
| `test` | Run full test suite via `w3 .test level::3`. |
| `clean` | Remove generated artifacts and caches via `cargo clean`. |
| `run` | Execute the `clp` binary entry point via `cargo run`. |
| `lint` | Run static analysis and style checks via `cargo clippy`. |
| `verbs` | List all available verbs and their availability (meta). |
| `detect` | Report the detected ecosystem and signal confidence (meta). |

All scripts support `--dry-run`: prints the delegated command without executing it.

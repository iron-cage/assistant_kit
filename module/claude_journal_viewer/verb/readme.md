# verb/

### Scope

**Responsibilities:** Shell scripts implementing the `do` protocol verbs for `claude_journal_viewer` (cargo ecosystem).
**In Scope:** Canonical verbs (`build`, `test`, `test_only`, `clean`, `lint`, `verify`, `install`, `run`, `package_info`), layer dispatchers (`*.d/`), and the `verbs` meta verb.
**Out of Scope:** Source code (→ `src/`), test logic (→ `tests/`), documentation (→ `docs/`).

### Responsibility Table

| File | Responsibility |
|------|---------------|
| `build` | Compile the crate: `cargo build -p claude_journal_viewer`. |
| `clean` | Remove build artifacts: `cargo clean -p claude_journal_viewer`. |
| `lint` | Run clippy via layer dispatcher (`lint.d/`). |
| `lint.d/` | Layer directory: `l1` (direct clippy). |
| `verify` | Full checks: `will .test level::4`. |
| `run` | Run the `clj` binary via layer dispatcher (`run.d/`). |
| `run.d/` | Layer directory: `l1` (direct `cargo run`). |
| `package_info` | Report package metadata as JSON (meta). |
| `test` | Run module suite in container: `runbox .live` with `test.d/l1` as payload. |
| `test.d/` | Layer directory: `l1` (container-internal). |
| `test_only` | Run tests matching a filter in container: `runbox .live` with `test_only.d/l1 <filter>` as payload. |
| `test_only.d/` | Layer directory: `l1` (container-internal targeted run). |
| `install` | Install the `clj` binary to `~/.cargo/bin`: `cargo install --path .`. |
| `verbs` | List all available verbs and their availability (meta). |

Canonical verbs support `--dry-run`: prints the delegated command without executing it. The `verbs` meta verb does not.

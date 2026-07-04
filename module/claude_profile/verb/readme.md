# verb/

### Scope

**Responsibilities:** Shell scripts implementing the `do` protocol verbs for `claude_profile` (cargo ecosystem).
**In Scope:** Canonical verbs (`build`, `test`, `test_only`, `lint`, `run`, `clean`, `verify`), layer dispatchers (`*.d/`), and meta verbs (`verbs`, `package_info`).
**Out of Scope:** Source code (→ `src/`), test logic (→ `tests/`), documentation (→ `docs/`).

### Responsibility Table

| File | Responsibility |
|------|---------------|
| `build` | Compile project artifacts via `cargo build`. |
| `test` | Dispatcher: run full test suite; delegates to `test.d/` layer by `VERB_LAYER`. |
| `test.d/` | Layer directory: `l0` (host-native), `l1` (container-internal). |
| `test_only` | Dispatcher: run single test by nextest filter inside container; sets `NEXTEST_FILTER`. |
| `test_only.d/` | Layer directory: `l1` (container-internal targeted run). |
| `clean` | Remove generated artifacts and caches via `cargo clean`. |
| `install` | Install crate binaries to `~/.cargo/bin` via `cargo install`. |
| `run` | Dispatcher: execute entry point; delegates to `run.d/` layer by `VERB_LAYER`. |
| `run.d/` | Layer directory: `l1` (direct; default). |
| `lint` | Dispatcher: run linter; delegates to `lint.d/` layer by `VERB_LAYER`. |
| `lint.d/` | Layer directory: `l1` (direct; default). |
| `verify` | Run full pre-push gate: tests, deps analysis, audit. |
| `verbs` | List all available verbs and their availability (meta). |
| `package_info` | Report deterministic package metadata as JSON (meta). |

Canonical verbs support `--dry-run`: prints the delegated command without executing it. Meta verbs (`verbs`, `package_info`) do not.

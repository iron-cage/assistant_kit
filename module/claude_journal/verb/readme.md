# verb/

### Scope

**Responsibilities:** Shell scripts implementing the `do` protocol verbs for `claude_journal` (cargo ecosystem).
**In Scope:** Canonical verbs (`test`, `test_only`, `lint`, `clean`, `install`, `verify`), layer dispatchers (`*.d/`), and meta verbs (`verbs`, `package_info`).
**Out of Scope:** Source code (→ `src/`), test logic (→ `tests/`), documentation (→ `docs/`).

### Responsibility Table

| File | Responsibility |
|------|---------------|
| `test` | Dispatcher: run full test suite; delegates to `test.d/` layer by `VERB_LAYER`. |
| `test.d/` | Layer directory: `l0` (host-native), `l1` (container-internal). |
| `test_only` | Dispatcher: run single test by nextest filter inside container; sets `NEXTEST_FILTER`. |
| `test_only.d/` | Layer directory: `l1` (container-internal targeted run). |
| `clean` | Remove generated artifacts and caches via `cargo clean`. |
| `install` | Install crate binary — unavailable for this library crate. |
| `run` | Execute entry point binary — unavailable for this library crate. |
| `lint` | Dispatcher: run linter; delegates to `lint.d/` layer by `VERB_LAYER`. |
| `lint.d/` | Layer directory: `l1` (direct; default). |
| `verify` | Run full pre-push gate: tests, deps analysis, audit. |
| `verbs` | List all available verbs and their availability (meta). |
| `package_info` | Report deterministic package metadata as JSON (meta). |

Canonical verbs support `--dry-run`: prints the delegated command without executing it. Meta verbs (`verbs`, `package_info`) do not.

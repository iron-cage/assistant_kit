# verb/

### Scope

**Responsibilities:** Shell scripts implementing the `do` protocol verbs for `claude_auth` (cargo ecosystem).
**In Scope:** Canonical verbs (`build`, `test`, `lint`, `run`, `clean`, `verify`), layer dispatchers (`*.d/`), and meta verbs (`verbs`, `package_info`).
**Out of Scope:** Source code (→ `src/`), test logic (→ `tests/`), documentation (→ `docs/`).

### Responsibility Table

| File | Responsibility |
|------|---------------|
| `build` | Compile project artifacts via `cargo build`. |
| `test` | Dispatcher: run full test suite; delegates to `test.d/` layer by `VERB_LAYER`. |
| `test.d/` | Layer directory: `l1` (direct, `VERB_LAYER=l1`), `l2` (runbox, default). |
| `clean` | Remove generated artifacts and caches via `cargo clean`. |
| `run` | Execute entry point binary — unavailable for this library crate. |
| `lint` | Dispatcher: run linter; delegates to `lint.d/` layer by `VERB_LAYER`. |
| `lint.d/` | Layer directory: `l1` (direct, `VERB_LAYER=l1`), `l2` (runbox, default). |
| `verify` | Run full pre-push gate: tests, deps analysis, audit. |
| `verbs` | List all available verbs and their availability (meta). |
| `package_info` | Report deterministic package metadata as JSON (meta). |

Canonical verbs support `--dry-run`: prints the delegated command without executing it. Meta verbs (`verbs`, `package_info`) do not.

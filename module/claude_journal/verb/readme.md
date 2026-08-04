# verb/

### Scope

**Responsibilities:** Shell scripts implementing the `do` protocol verbs for `claude_journal` (cargo ecosystem).
**In Scope:** Canonical verbs (`test`, `test_only`, `install`), layer dispatchers (`*.d/`), and the `verbs` meta verb.
**Out of Scope:** Source code (→ `src/`), test logic (→ `tests/`), documentation (→ `docs/`).

### Responsibility Table

| File | Responsibility |
|------|---------------|
| `test` | Run module suite in container: `runbox .live` with `test.d/l1` as payload. |
| `test.d/` | Layer directory: `l1` (container-internal). |
| `test_only` | Run tests matching a filter in container: `runbox .live` with `test_only.d/l1 <filter>` as payload. |
| `test_only.d/` | Layer directory: `l1` (container-internal targeted run). |
| `install` | Install crate binary — unavailable for this library crate. |
| `verbs` | List all available verbs and their availability (meta). |

Canonical verbs support `--dry-run`: prints the delegated command without executing it. The `verbs` meta verb does not.

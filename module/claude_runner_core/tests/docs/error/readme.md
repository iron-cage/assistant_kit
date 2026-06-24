# Error Classification Tests

### Scope

- **Purpose**: Document behavioral cases for `ErrorKind` classification via `ExecutionOutput::classify_error()`.
- **Responsibility**: Index of per-error test spec files covering subprocess failure mode classification.
- **In Scope**: `ErrorKind` variant selection from stderr/stdout patterns and exit codes.
- **Out of Scope**: Consumer-side error label formatting (→ `claude_runner/tests/`), CLI exit code propagation (→ `claude_runner/tests/docs/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 001_classify_error.md | Behavioral cases for `classify_error()` pattern matching and exit code fallbacks |

### Index

| Error Doc | File | Tests | Status |
|-----------|------|-------|--------|
| [Rate Limit](../../../../docs/error/001_rate_limit_reached.md) + [Quota Exhausted](../../../../docs/error/006_quota_exhausted.md) | [001_classify_error.md](001_classify_error.md) | 19 FT | ✅ |

# tests/

Unit tests for the `claude_version_core` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `config_resolve_test.rs` | 4-layer resolution algorithm: env → project → user → catalog default |
| `settings_io_test.rs` | `infer_type`, `json_escape`, and `set`/`get`/`read_all` round-trip |
| `version_test.rs` | Semver extraction, alias resolution, and version spec validation |

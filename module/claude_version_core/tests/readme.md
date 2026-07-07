# tests/

Unit tests for the `claude_version_core` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `config_resolve_test.rs` | 4-layer resolution algorithm: env → project → user → catalog default |
| `params_catalog_test.rs` | Catalog structural integrity: no dupes, sort order, lookup |
| `settings_io_test.rs` | `infer_type`, `json_escape`, and `set`/`get`/`read_all`/`remove` round-trip |
| `version_test.rs` | Semver extraction, alias resolution, and version spec validation |

# claude_version_core

Layer 1 domain helpers for Claude Code version management and settings. Depends only on `claude_common`.

## Responsibility Table

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Crate manifest: depends on `claude_common` + `error_tools` |
| `src/` | Version detection, settings I/O domain logic |
| `tests/` | Unit tests for domain helpers |

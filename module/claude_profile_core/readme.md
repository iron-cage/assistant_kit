# claude_profile_core

Layer 1 domain logic for Claude Code account and token management. Depends only on `claude_core`.

## Responsibility Table

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Crate manifest: depends on `claude_core` + `error_tools` |
| `src/` | Token status detection and account CRUD domain logic |
| `tests/` | Unit tests for token classification and account lifecycle |
| `verb/` | Shell scripts for each `do` protocol verb. |
| `run/` | Shell scripts for container-orchestrated operations. |

# claude_core

Layer 0 shared primitives for the assistant workspace. Zero workspace dependencies.

## Responsibility Table

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Crate manifest: zero workspace deps, stdlib only |
| `src/` | `ClaudePaths`, process scanner/signal utilities, atomic settings I/O |
| `docs/` | Public API contract for `settings_io` |
| `tests/` | Unit tests for path construction and process scanning |
| `verb/` | Shell scripts for each `do` protocol verb. |

# claude_core

Layer 0 shared primitives for the assistant workspace. Zero workspace dependencies.

## Responsibility Table

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Crate manifest: zero workspace deps, stdlib only |
| `src/` | `ClaudePaths`, process scanner/signal utilities, atomic file I/O with trace redaction, atomic settings/config I/O (JSON + TOML) |
| `docs/` | Public API contracts for `settings_io`, `toml_io`, and `file_io` |
| `tests/` | Unit tests for paths, process scanning, file I/O, settings/TOML I/O |
| `verb/` | Shell scripts for each `do` protocol verb. |

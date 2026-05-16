# claude_version

Claude Code version manager: install, upgrade, and session lifecycle.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `docs/` | Design and CLI documentation (feature/, invariant/, algorithm/, etc.) |
| `src/` | Binary and library source code |
| `tests/` | Unit and integration test suite |
| `Cargo.toml` | Crate manifest |
| `unilang.commands.yaml` | YAML command metadata for all 11 manager commands (not aggregated by build.rs) |
| `changelog.md` | Notable changes by version |
| `verb/` | Shell scripts for each `do` protocol verb. |
| `run/` | Shell scripts for container-orchestrated operations. |

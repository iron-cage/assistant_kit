# src/

Source code for the `claude_common` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root; module declarations and `ClaudePaths` re-export |
| `paths.rs` | `ClaudePaths`: all canonical `~/.claude/` paths derived from `HOME` |
| `process.rs` | `/proc`-based Claude process enumeration and signal sending |

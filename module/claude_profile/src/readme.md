# src/

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root: module declarations, re-exports, and `register_commands()`. |
| `paths.rs` | ClaudePaths — all `~/.claude/` canonical paths from HOME. |
| `token.rs` | TokenStatus — read expiresAt, classify Valid/ExpiringSoon/Expired. |
| `account.rs` | Account CRUD: save, list, switch, delete, auto_rotate; _active marker. |
| `main.rs` | CLI binary entry point, 5-phase unilang pipeline. |
| `adapter.rs` | Argv-to-unilang token conversion, alias expansion, validation. |
| `output.rs` | Output format extraction and JSON string escaping. |
| `commands.rs` | CLI command handler functions for 9 named commands and the dot fallback. |
| `usage.rs` | `.usage` command — parse stats-cache.json, format 7-day token report. |
| `persist.rs` | PersistPaths — persistent user storage path from $PRO/$HOME. |

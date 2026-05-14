# src/

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root: module declarations, re-exports, and `register_commands()`. |
| `paths.rs` | ClaudePaths — all `~/.claude/` canonical paths from HOME. |
| `token.rs` | TokenStatus — read expiresAt, classify Valid/ExpiringSoon/Expired. |
| `account.rs` | Account CRUD: save, list, switch, delete, auto_rotate; _active marker. |
| `main.rs` | CLI binary entry point, 5-phase unilang pipeline. |
| `adapter.rs` | Argv-to-unilang token conversion, alias expansion, validation. |
| `output.rs` | Output format extraction, JSON string escaping, and duration display. |
| `commands.rs` | CLI command handler routines for 11 commands (10 named + dot fallback). |
| `usage.rs` | `.usage` command — fetch live rate-limit quota for all saved accounts. |
| `persist.rs` | PersistPaths — persistent user storage path from $PRO/$HOME. |
| `bin/` | Separate Cargo compilation units for each binary target. |
| `bin/clp.rs` | `clp` short-alias binary entry point; delegates to `run_cli()`. |

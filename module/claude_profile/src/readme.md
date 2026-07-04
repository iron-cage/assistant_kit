# src/

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root: module declarations and re-exports. |
| `registry.rs` | Command registration: argument definitions and routines for 18 commands. |
| `cli.rs` | CLI pipeline: adapter → parser → semantic analysis → execution. |
| `paths.rs` | ClaudePaths — all `~/.claude/` canonical paths from HOME. |
| `token.rs` | TokenStatus — read expiresAt, classify Valid/ExpiringSoon/Expired. |
| `account.rs` | Account CRUD: save, list, switch, delete; _active marker. |
| `main.rs` | CLI binary entry point; delegates to `run_cli()`. |
| `adapter.rs` | Argv-to-unilang token conversion, alias expansion, validation. |
| `output.rs` | Output format extraction, JSON string escaping, and duration display. |
| `commands/` | CLI command handler routines: one module per command group (accounts, account_ops, credentials, limits, token_paths, dot) plus cmd_args (argument/error helpers) and cmd_context (environment/credentials context). |
| `usage/` | `.usage` command modules — quota fetch, render, sort, refresh, and live loop. |
| `owner_dispatch.rs` | Shared owner batch-clear and named-dispatch logic for `.accounts` and `.usage`. |
| `persist.rs` | PersistPaths — persistent user storage path from $PRO/$HOME. |
| `bin/` | Separate Cargo compilation units for each binary target. |
| `bin/clp.rs` | `clp` short-alias binary entry point; delegates to `run_cli()`. |

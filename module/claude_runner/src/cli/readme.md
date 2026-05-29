# cli/

| File | Responsibility |
|------|----------------|
| `mod.rs` | Coordinator: help printing, dry-run, run modes, subcommand dispatch. |
| `parse.rs` | `CliArgs` struct, `parse_args`, `apply_env_vars`, shared env helpers. |
| `cred_parse.rs` | `IsolatedArgs`, `RefreshArgs`, their parsers and env-var fallbacks. |
| `builder.rs` | Session continuity check and `ClaudeCommand` construction. |
| `fence.rs` | `strip_fences` utility and unit tests. |
| `credential.rs` | `run_isolated_command`, `run_refresh_command`, credential trace emission. |

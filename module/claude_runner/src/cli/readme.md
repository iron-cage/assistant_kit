# cli/

| File | Responsibility |
|------|----------------|
| `mod.rs` | Subcommand dispatch, execution modes (run/interactive), dry-run, and guard. |
| `parse.rs` | `CliArgs` struct, `parse_args`, `apply_env_vars`, shared env helpers. |
| `cred_parse.rs` | `IsolatedArgs`, `RefreshArgs`, their parsers and env-var fallbacks. |
| `builder.rs` | Session continuity check and `ClaudeCommand` construction. |
| `fence.rs` | `strip_fences` utility — outermost code-fence stripping for `--strip-fences`. |
| `credential.rs` | `run_isolated_command`, `run_refresh_command`, credential trace emission. |
| `gate.rs` | Session count check and blocking wait for concurrent-session limit. |
| `help.rs` | Help text printing for all subcommands (clr, ask, isolated, refresh). |

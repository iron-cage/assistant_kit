# cli/

| File | Responsibility |
|------|----------------|
| `mod.rs` | Subcommand dispatch, execution modes (run/interactive), dry-run, and guard. |
| `parse.rs` | `CliArgs` struct, `ExpectStrategy`, `parse_args`, `parse_value_flag`. |
| `cred_parse.rs` | `IsolatedArgs`, `RefreshArgs`, their parsers and env-var fallbacks. |
| `builder.rs` | Session continuity check and `ClaudeCommand` construction. |
| `fence.rs` | `strip_fences` utility — outermost code-fence stripping for `--strip-fences`. |
| `credential.rs` | `run_isolated_command`, `run_refresh_command`, credential trace emission. |
| `gate.rs` | Session count check and blocking wait for concurrent-session limit. |
| `ps.rs` | `clr ps` dispatch — active sessions and queued waiters in two plain-style tables. |
| `help.rs` | Help text printing for all subcommands (clr, ask, isolated, refresh). |
| `execution.rs` | `run_print_mode`, `run_interactive`, timeout watchdog, expect validation. |
| `env.rs` | `env_bool`, `env_str`, `apply_env_vars` — CLR_* env-variable fallbacks. |
| `kill.rs` | `dispatch_kill`, `print_kill_help` — SIGTERM delivery to a validated claude PID. |

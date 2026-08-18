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
| `gate_limits.rs` | Gate knob resolution and external deadline-budget clamping. |
| `gate_slot.rs` | Atomic slot reservation, dead-owner reclaim, and denial causes. |
| `gate_liveness.rs` | PID liveness and start-time incarnation checks via `/proc`. |
| `ps.rs` | `clr ps` dispatch — active sessions and queued waiters in two plain-style tables. |
| `help.rs` | Help text printing for all subcommands (clr, ask, isolated, refresh, scope). |
| `execution.rs` | `run_print_mode`, `run_interactive`, timeout watchdog, expect validation, 3-tier retry resolution. |
| `retry_classify.rs` | `ErrorClass`, `ClassAttempts` — error-class taxonomy and 3-tier retry count/delay resolution used by `execution.rs`. |
| `env.rs` | `env_bool`, `env_str`, `apply_env_vars` — CLR_* env-variable fallbacks. |
| `kill.rs` | `dispatch_kill`, `print_kill_help` — SIGTERM delivery to a validated claude PID. |
| `tools.rs` | `dispatch_tools` — list all 26 Claude Code built-in tools in a plain-style table. |
| `scope.rs` | `dispatch_scope` — print all 6 CLAUDE_* path variables for a directory. |
| `summary.rs` | `render_summary` — parse CLR result envelope, render key:val header + text body for `--output-style summary`. |
| `json_config.rs` | JSON config loading: `load_json_source`, `parse_json_object`, `apply_json_config`, `load_and_apply`. |
| `json_config_isolated.rs` | JSON config application for `isolated`/`refresh` subcommands: `apply_json_config_isolated`, `apply_json_config_refresh`, `load_and_apply_isolated`, `load_and_apply_refresh`. |
| `column_validate.rs` | `validate_columns` — shared comma-separated column-key validation used by `ps` and `tools`. |
| `query.rs` | `clr query` — PID-addressed control-session dispatch (start daemon / dispatch method). |
| `config.rs` | Config-file parameter tier: `~/.clr/config.toml` and project-level `.clr.toml` loading. |

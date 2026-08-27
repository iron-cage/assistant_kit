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
| `ps.rs` | `clr ps` dispatch — active sessions and queued waiters in two plain-style tables. |
| `ps_snapshot.rs` | `clr ps` temporal-diff state: read/write `~/.clr/ps` snapshot, build the "Ended Since Last Check" table. |
| `help.rs` | Help text printing for all subcommands (clr, ask, isolated, refresh, scope). |
| `execution.rs` | `run_print_mode`, `run_interactive`, timeout watchdog, expect validation, retry orchestration loop (classification/resolution delegated to `retry_classify.rs`). |
| `retry_classify.rs` | `ErrorClass`, `ClassAttempts` — error-class taxonomy and 3-tier retry count/delay resolution used by `execution.rs`. |
| `env.rs` | `env_bool`, `env_str`, `apply_env_vars` — CLR_* env-variable fallbacks. |
| `kill.rs` | `dispatch_kill`, `print_kill_help` — SIGTERM delivery to a validated claude PID. |
| `tools.rs` | `dispatch_tools` — list all 26 Claude Code built-in tools in a plain-style table. |
| `scope.rs` | `dispatch_scope` — print all 6 CLAUDE_* path variables for a directory. |
| `topic.rs` | `clr topic` — derive a `--topic` slug from the message, then delegate to `run`. |
| `topics.rs` | `clr topics` — list existing topics or resolve one topic's directory/session path. |
| `forward.rs` | `clr delegate` / `clr broadcast` — send one prompt to one topic or to every live one. |
| `summary.rs` | `render_summary` — parse CLR result envelope, render key:val header + text body for `--output-style summary`. |
| `json_config.rs` | JSON config loading: `load_json_source`, `parse_json_object`, `apply_json_config`, `load_and_apply`. |
| `json_config_isolated.rs` | JSON config application for `isolated`/`refresh` subcommands: `apply_json_config_isolated`, `apply_json_config_refresh`, `load_and_apply_isolated`, `load_and_apply_refresh`. |
| `column_validate.rs` | `validate_columns` — shared comma-separated column-key validation used by `ps` and `tools`. |
| `query.rs` | `clr query` — PID-addressed control-session dispatch (start daemon / dispatch method). |
| `daemon.rs` | `clr daemon` — session-daemon lifecycle, and the hidden serve entry point. |
| `chat.rs` | `clr chat` — one prompt to a hosted session, and knowing when the answer ended. |
| `sessions.rs` | `clr sessions` — one row per session the daemon is hosting. |
| `config.rs` | Config-file parameter tier: `~/.clr/config.toml` and project-level `.clr.toml` loading. |

## Not here

Reading a turn's answer out of the session transcript — what `chat.rs` prints
instead of the terminal dump. It is a fact about how Claude Code writes storage,
not about this CLI, so it lives in `claude_storage_core::transcript_answer_since`
along with the parser it depends on.

What a topic name resolves to, which topics exist, and which one a prompt should
go to. `topic.rs` and `topics.rs` are the CLI surface over it, but the formula
(`<base>/-<name>`), the base precedence, `TopicMode` selection, the fork-topic
name registry, and the enumerate/select/pool/lock logic all live in
`claude_topic_core` — computation over paths and a process list, with nothing
CLI-shaped about it, and needed by more than one caller.

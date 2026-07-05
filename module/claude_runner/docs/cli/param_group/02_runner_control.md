# CLI Parameter Group: Runner Control

**Pattern:** Consumed by the runner before subprocess launch; may suppress, override, or replace automatic flag injections; never forwarded directly.

**Purpose:** Control runner execution behavior — before, during, or instead of subprocess invocation.
**Order:** 2

### Semantic Coherence Test

"Is this flag consumed by the runner, not Claude?" — YES for all 48.

### Why NOT X

- `--print`: forwarded to claude subprocess as `--print`
- `--model`: forwarded to claude subprocess as `--model`
- `--verbose`: forwarded to claude subprocess as `--verbose`
- `--effort`: forwarded to claude subprocess as `--effort <level>`
- `--dangerously-skip-permissions`: not a user flag — injected automatically by the runner (default-on)

### Invariants

No parameter is forwarded to the subprocess unchanged. Each is fully consumed by runner logic before subprocess construction.

### Notes

`[MESSAGE]` is not in any group — it is a positional argument serving as input content, not a control flag. `--help` is handled separately as a universal override.

### Typical Patterns

```sh
clr --dir /project --max-tokens 50000 "test"
clr --interactive "Continue this work" --dir /project
clr --new-session --dry-run "check command"
clr --trace "Fix bug" --dir /project
```

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|------------|-----------------|-------|
| 1 | [`run`](../command/01_run.md) | Full | — | All 47 params apply; default command |
| 5 | [`ask`](../command/05_ask.md) | Full | — | All 47 params apply; identical behavior — pure alias for run |

### Referenced Parameters

| Parameter | Type | Default | Role in Group | Description |
|-----------|------|---------|---------------|-------------|
| [`--no-skip-permissions`](../param/005_no_skip_permissions.md) | bool | false | Injection suppressor | Disable automatic permission bypass |
| [`--interactive`](../param/006_interactive.md) | bool | false | Mode selector | Interactive TTY passthrough when message given |
| [`--new-session`](../param/007_new_session.md) | bool | false | Session mode | Start fresh session (disable default continuation) |
| [`--dir`](../param/008_dir.md) | [`DirectoryPath`](../type/02_directory_path.md) | cwd | Working directory | Working directory for subprocess (alias: `--to`) |
| [`--subdir`](../param/028_subdir.md) | string | `.` | Named workspace | Named subdirectory appended to `--dir` (`/-NAME`); `.` = identity |
| [`--max-tokens`](../param/009_max_tokens.md) | [`TokenLimit`](../type/03_token_limit.md) | 200000 | Token cap | Max output tokens |
| [`--session-dir`](../param/010_session_dir.md) | [`DirectoryPath`](../type/02_directory_path.md) | — | Session storage | Session storage directory |
| [`--dry-run`](../param/011_dry_run.md) | bool | false | Execution gate | Preview without executing |
| [`--quiet`](../param/074_quiet.md) | bool | false | Diagnostic suppressor | Suppress non-fatal runner diagnostics |
| [`--trace`](../param/013_trace.md) | bool | false | Trace mode | Print env+command to stderr then execute |
| [`--no-ultrathink`](../param/014_no_ultrathink.md) | bool | false | Injection suppressor | Disable default ultrathink message suffix |
| [`--no-effort-max`](../param/018_no_effort_max.md) | bool | false | Injection suppressor | Suppress default `--effort max` injection |
| [`--no-chrome`](../param/021_no_chrome.md) | bool | false | Injection suppressor | Suppress default `--chrome` injection |
| [`--no-persist`](../param/022_no_persist.md) | bool | false | Injection suppressor | Disable session persistence injection |
| [`--file`](../param/025_file.md) | [`FilePath`](../type/12_file_path.md) | — | Stdin source | File content piped as subprocess stdin |
| [`--strip-fences`](../param/026_strip_fences.md) | bool | false | Output processor | Strip outermost markdown code fences from stdout |
| [`--keep-claudecode`](../param/027_keep_claudecode.md) | bool | false | Env filter | Preserve `CLAUDECODE` env var in subprocess (default: removed) |
| [`--output-file`](../param/029_output_file.md) | string | — | Output sink | Write captured stdout to a file (tee behavior) |
| [`--expect`](../param/030_expect.md) | string | — | Output validator | Pipe-separated enum values; stdout must match one after trim+lowercase |
| [`--expect-strategy`](../param/031_expect_strategy.md) | enum | `fail` | Mismatch handler | Mismatch handling: exit 3, retry N times, or output fallback value |
| [`--max-sessions`](../param/033_max_sessions.md) | u32 | 10 | Concurrency gate | Max concurrent non-interactive Claude Code sessions before blocking; 0 = unlimited; interactive exempt |
| [`--retry-on-transient`](../param/034_retry_on_transient.md) | u8 | auto | Retry (Tier 2) | Transient class retry count; effective default = 2 via fallback |
| [`--transient-delay`](../param/035_transient_delay.md) | u32 | auto | Retry delay (Tier 2) | Transient class delay; effective default = 30 via fallback |
| [`--timeout`](../param/036_timeout.md) | u32 | `0` | Execution watchdog | Seconds before watchdog kills subprocess; 0 = unlimited (run/ask only) |
| [`--retry-on-account`](../param/040_retry_on_account.md) | u8 | auto | Retry (Tier 2) | Account class retry count |
| [`--account-delay`](../param/041_account_delay.md) | u32 | auto | Retry delay (Tier 2) | Account class delay |
| [`--retry-on-auth`](../param/042_retry_on_auth.md) | u8 | auto | Retry (Tier 2) | Auth class retry count |
| [`--auth-delay`](../param/043_auth_delay.md) | u32 | auto | Retry delay (Tier 2) | Auth class delay |
| [`--retry-on-service`](../param/044_retry_on_service.md) | u8 | auto | Retry (Tier 2) | Service class retry count |
| [`--service-delay`](../param/045_service_delay.md) | u32 | auto | Retry delay (Tier 2) | Service class delay |
| [`--retry-on-process`](../param/046_retry_on_process.md) | u8 | auto | Retry (Tier 2) | Process class retry count |
| [`--process-delay`](../param/047_process_delay.md) | u32 | auto | Retry delay (Tier 2) | Process class delay |
| [`--retry-on-validation`](../param/048_retry_on_validation.md) | u8 | auto | Retry (Tier 2) | Validation class retry count (only with `--expect-strategy retry`) |
| [`--validation-delay`](../param/049_validation_delay.md) | u32 | auto | Retry delay (Tier 2) | Validation class delay |
| [`--retry-on-runner`](../param/050_retry_on_runner.md) | u8 | auto | Retry (Tier 2) | Runner class retry count |
| [`--runner-delay`](../param/051_runner_delay.md) | u32 | auto | Retry delay (Tier 2) | Runner class delay |
| [`--retry-on-unknown`](../param/052_retry_on_unknown.md) | u8 | auto | Retry (Tier 2) | Unknown class retry count |
| [`--unknown-delay`](../param/053_unknown_delay.md) | u32 | auto | Retry delay (Tier 2) | Unknown class delay |
| [`--retry-override`](../param/054_retry_override.md) | u8 | auto | Retry (Tier 1) | Forces retry count for all error classes |
| [`--retry-override-delay`](../param/055_retry_override_delay.md) | u32 | auto | Retry delay (Tier 1) | Forces delay for all error classes |
| [`--retry-default`](../param/056_retry_default.md) | u8 | `2` | Retry (Tier 3) | Fallback retry count for all unset classes |
| [`--retry-default-delay`](../param/057_retry_default_delay.md) | u32 | `30` | Retry delay (Tier 3) | Fallback delay for all unset classes |
| [`--output-style`](../param/070_output_style.md) | enum | `summary` | Output renderer | Runner-level rendering: `summary` routes stdout through `render_summary()`; `raw` passthrough |
| [`--summary-fields`](../param/071_summary_fields.md) | string | `full` | Field selector | Select which CLR envelope fields render in summary header; presets: `minimal`/`standard`/`full`; or custom whitelist |
| [`--journal`](../param/072_journal.md) | enum | `full` | Journal level | Journal level for execution events: `full` (stdout+stderr ≤1MB each), `meta` (metadata only), `off` (disabled) |
| [`--journal-dir`](../param/073_journal_dir.md) | path | `~/.clr/journal/` | Journal directory | Directory for journal JSONL files; overrides `CLR_JOURNAL_DIR` |
| [`--args-file`](../param/075_args_file.md) | [`FilePath`](../type/12_file_path.md) | — | Config loader | Load all clr params from JSON file; also applies to `isolated` and `refresh` |
| [`--session-from`](../param/076_session_from.md) | [`DirectoryPath`](../type/02_directory_path.md) | — | Session source | Compute session storage for this dir and use it for `-c` injection (alias: `--from`) |

### Referenced Tests

| # | Test Spec | Scope |
|---|-----------|-------|
| 2 | [02_runner_control.md](../../../tests/docs/cli/param_group/02_runner_control.md) | Runner Control group behavior |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [001_interactive_repl.md](../user_story/001_interactive_repl.md) | Developer |
| 3 | [003_interactive_with_message.md](../user_story/003_interactive_with_message.md) | Developer |
| 4 | [004_dry_run_preview.md](../user_story/004_dry_run_preview.md) | Developer |
| 5 | [005_project_specific_execution.md](../user_story/005_project_specific_execution.md) | Developer |
| 6 | [006_verbose_debugging.md](../user_story/006_verbose_debugging.md) | Developer |
| 7 | [007_fresh_session.md](../user_story/007_fresh_session.md) | Developer |
| 8 | [008_trace_execution.md](../user_story/008_trace_execution.md) | Developer |
| 11 | [011_file_input.md](../user_story/011_file_input.md) | Developer |
| 12 | [012_code_block_extraction.md](../user_story/012_code_block_extraction.md) | Developer |
| 13 | [013_structured_json_pipeline.md](../user_story/013_structured_json_pipeline.md) | Developer |
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |
| 18 | [018_env_var_configuration.md](../user_story/018_env_var_configuration.md) | Developer |
| 20 | [020_suppress_effort_max.md](../user_story/020_suppress_effort_max.md) | Developer |
| 21 | [021_keep_claudecode_context.md](../user_story/021_keep_claudecode_context.md) | Developer |
| 22 | [022_session_isolation_subdir.md](../user_story/022_session_isolation_subdir.md) | Developer |
| 23 | [023_output_file_capture.md](../user_story/023_output_file_capture.md) | Developer |
| 24 | [024_enum_output_validation.md](../user_story/024_enum_output_validation.md) | Developer |
| 25 | [025_concurrency_gate.md](../user_story/025_concurrency_gate.md) | Developer |
| 28 | [028_session_transplant.md](../user_story/028_session_transplant.md) | Developer |

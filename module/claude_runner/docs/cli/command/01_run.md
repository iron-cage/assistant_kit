# CLI Command: run

Execute Claude Code as a subprocess with configurable flags. This is the
default command — invoked whenever no explicit subcommand is given.

**Syntax:**

```sh
clr [OPTIONS] [MESSAGE]
clr run [OPTIONS] [MESSAGE]
```

The `run` token is optional — both forms are equivalent. When `run` appears as the first positional token it is stripped before delegation to the default run mode.

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`[MESSAGE]`](../param/001_message.md) | [`MessageText`](../type/01_message_text.md) | — | Prompt text for Claude |
| [`-p`/`--print`](../param/002_print.md) | bool | auto | Print mode (default when message given; explicit alias) |
| [`--model`](../param/003_model.md) | [`ModelName`](../type/04_model_name.md) | — | Model to use |
| [`--verbose`](../param/004_verbose.md) | bool | false | Enable Claude verbose output |
| [`--no-skip-permissions`](../param/005_no_skip_permissions.md) | bool | false | Disable automatic permission bypass |
| [`--interactive`](../param/006_interactive.md) | bool | false | Interactive TTY passthrough when message given |
| [`--new-session`](../param/007_new_session.md) | bool | false | Start fresh session (disables default continuation) |
| [`--dir`](../param/008_dir.md) | [`DirectoryPath`](../type/02_directory_path.md) | cwd | Working directory |
| [`--subdir`](../param/028_subdir.md) | string | `.` | Named subdirectory appended to `--dir` (`/-NAME`); `.` = identity |
| [`--max-tokens`](../param/009_max_tokens.md) | [`TokenLimit`](../type/03_token_limit.md) | 200000 | Max output tokens |
| [`--session-dir`](../param/010_session_dir.md) | [`DirectoryPath`](../type/02_directory_path.md) | — | Session storage directory |
| [`--dry-run`](../param/011_dry_run.md) | bool | false | Print command without executing |
| [`--quiet`](../param/074_quiet.md) | bool | false | Suppress non-fatal runner diagnostics |
| [`--trace`](../param/013_trace.md) | bool | false | Print env+command to stderr then execute (like `set -x`) |
| [`--no-ultrathink`](../param/014_no_ultrathink.md) | bool | false | Disable default ultrathink message suffix |
| [`--system-prompt`](../param/015_system_prompt.md) | [`SystemPromptText`](../type/06_system_prompt_text.md) | — | Set system prompt (replaces the default) |
| [`--append-system-prompt`](../param/016_append_system_prompt.md) | [`SystemPromptText`](../type/06_system_prompt_text.md) | — | Append text to the default system prompt |
| [`--effort`](../param/017_effort.md) | [`EffortLevel`](../type/07_effort_level.md) | max | Override reasoning effort level (default: max) |
| [`--no-effort-max`](../param/018_no_effort_max.md) | bool | false | Suppress default `--effort max` injection |
| [`--no-chrome`](../param/021_no_chrome.md) | bool | false | Suppress default `--chrome` injection |
| [`--no-persist`](../param/022_no_persist.md) | bool | false | Disable session persistence (`--no-session-persistence`) |
| [`--json-schema`](../param/023_json_schema.md) | [`JsonSchemaText`](../type/10_json_schema_text.md) | — | JSON schema for structured output |
| [`--mcp-config`](../param/024_mcp_config.md) | [`McpConfigPath`](../type/11_mcp_config_path.md) | — | MCP server config file (repeatable, 0+) |
| [`--file`](../param/025_file.md) | [`FilePath`](../type/12_file_path.md) | — | File content piped as subprocess stdin |
| [`--strip-fences`](../param/026_strip_fences.md) | bool | false | Strip outermost markdown code fences from stdout |
| [`--keep-claudecode`](../param/027_keep_claudecode.md) | bool | false | Preserve `CLAUDECODE` env var in subprocess (default: removed) |
| [`--output-file`](../param/029_output_file.md) | string | — | Write captured stdout to file in addition to printing (tee behavior) |
| [`--expect`](../param/030_expect.md) | string | — | Pipe-separated enum values; stdout must match one after trim+lowercase |
| [`--expect-strategy`](../param/031_expect_strategy.md) | enum | `fail` | Mismatch handling: exit 3 (`fail`), retry (`retry`), or fallback (`default:<V>`) |
| [`--max-sessions`](../param/033_max_sessions.md) | u32 | `30` | Max concurrent claude sessions before blocking (0 = unlimited) |
| [`--retry-on-transient`](../param/034_retry_on_transient.md) | u8 | auto | Transient class retry count (Tier 2; effective default = 2 via fallback) |
| [`--transient-delay`](../param/035_transient_delay.md) | u32 | auto | Transient class delay (Tier 2; effective default = 30 via fallback) |
| [`--timeout`](../param/036_timeout.md) | u32 | `3600` (print-mode) / `0` (interactive) | Seconds before watchdog kills subprocess (absent → `DEFAULT_PRINT_TIMEOUT_SECS` for print-mode; 0 = unlimited) |
| [`--retry-on-account`](../param/040_retry_on_account.md) | u8 | auto | Account class retry count (Tier 2; effective default = 2 via fallback) |
| [`--account-delay`](../param/041_account_delay.md) | u32 | auto | Account class delay (Tier 2) |
| [`--retry-on-auth`](../param/042_retry_on_auth.md) | u8 | auto | Auth class retry count (Tier 2) |
| [`--auth-delay`](../param/043_auth_delay.md) | u32 | auto | Auth class delay (Tier 2) |
| [`--retry-on-service`](../param/044_retry_on_service.md) | u8 | auto | Service class retry count (Tier 2) |
| [`--service-delay`](../param/045_service_delay.md) | u32 | auto | Service class delay (Tier 2) |
| [`--retry-on-process`](../param/046_retry_on_process.md) | u8 | auto | Process class retry count (Tier 2) |
| [`--process-delay`](../param/047_process_delay.md) | u32 | auto | Process class delay (Tier 2) |
| [`--retry-on-validation`](../param/048_retry_on_validation.md) | u8 | auto | Validation class retry count (Tier 2; only with `--expect-strategy retry`) |
| [`--validation-delay`](../param/049_validation_delay.md) | u32 | auto | Validation class delay (Tier 2) |
| [`--retry-on-runner`](../param/050_retry_on_runner.md) | u8 | auto | Runner class retry count (Tier 2) |
| [`--runner-delay`](../param/051_runner_delay.md) | u32 | auto | Runner class delay (Tier 2) |
| [`--retry-on-unknown`](../param/052_retry_on_unknown.md) | u8 | auto | Unknown class retry count (Tier 2) |
| [`--unknown-delay`](../param/053_unknown_delay.md) | u32 | auto | Unknown class delay (Tier 2) |
| [`--retry-override`](../param/054_retry_override.md) | u8 | auto | Tier 1: forces retry count for all error classes |
| [`--retry-override-delay`](../param/055_retry_override_delay.md) | u32 | auto | Tier 1: forces delay for all error classes |
| [`--retry-default`](../param/056_retry_default.md) | u8 | `2` | Tier 3: fallback retry count for all unset classes |
| [`--retry-default-delay`](../param/057_retry_default_delay.md) | u32 | `30` | Tier 3: fallback delay for all unset classes |
| [`--output-style`](../param/070_output_style.md) | enum | `summary` | Output rendering style: `summary` (key:val header via `render_summary()`) or `raw` (bypass rendering) |
| [`--summary-fields`](../param/071_summary_fields.md) | string | `full` | Summary field selection: `minimal`/`standard`/`full`/custom whitelist |
| [`--journal`](../param/072_journal.md) | enum | `full` | Journal level: `full` (stdout+stderr ≤1MB each), `meta` (metadata only), `off` (disabled) |
| [`--journal-dir`](../param/073_journal_dir.md) | path | `~/.clr/journal/` | Directory for journal JSONL files; overrides `CLR_JOURNAL_DIR` |
| [`--output-format`](../param/061_output_format.md) | enum | — | Output format (`text`/`json`/`stream-json`) |
| [`--max-turns`](../param/062_max_turns.md) | u32 | — | Max agentic turns before stopping; unset = unlimited |
| [`--allowed-tools`](../param/063_allowed_tools.md) | string | — | Restrict Claude to specified tools only |
| [`--disallowed-tools`](../param/064_disallowed_tools.md) | string | — | Prevent Claude from using specified tools |
| [`--max-budget-usd`](../param/065_max_budget_usd.md) | f64 | — | Max dollar budget for session |
| [`--add-dir`](../param/066_add_dir.md) | path | — | Additional directory for Claude Code to access |
| [`--fallback-model`](../param/067_fallback_model.md) | string | — | Fallback model when primary unavailable |

**Execution Modes:**

All modes apply `-c` automatically (continuing the previous session).
Use `--new-session` to start fresh.

| Invocation | Mode | Path |
|------------|------|------|
| `clr` | Interactive REPL | `execute_interactive()` + `-c` |
| `clr run` | Interactive REPL (explicit) | strip `run`, `execute_interactive()` + `-c` |
| `clr "Fix bug"` | **Print (default)** | `execute()` + `--print` + `-c` |
| `clr run "Fix bug"` | Print (explicit alias) | strip `run`, `execute()` + `--print` + `-c` |
| `clr -p "Fix bug"` | Print (flag explicit) | `execute()` + `--print` + `-c` |
| `clr --interactive "Fix bug"` | Interactive | `execute_interactive()` + `-c` |
| `clr --dry-run "Fix bug"` | Preview only | `describe()` / `describe_env()` (shows `-c`) |
| `clr --trace "Fix bug"` | Trace (print then execute) | `describe_env()` + `describe()` to stderr, then `execute()` |
| `clr --new-session "Fix bug"` | Fresh session, print | `execute()` + `--print` (no `-c`) |

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (parse failure, print mode without message, execution error, binary not found) |
| 2 | Rate-limit passthrough from claude (subprocess exited 2); or runner-generated: Transient retries exhausted |
| 3 | Expect mismatch — output did not match `--expect` values after all retries |
| 4 | CLR-layer watchdog timeout: subprocess exceeded `--timeout`; stderr contains "Error: timeout after Ns" |
| N | Passthrough from claude subprocess (print mode propagates the subprocess exit code exactly) |
| 128+signal | Subprocess killed by signal; follows POSIX convention (e.g., SIGTERM → 143, SIGKILL → 137) |

**Examples:**

```sh
# Interactive REPL (no message)
clr

# Print mode — default when message given
clr "Explain this function" --model sonnet

# Explicit run subcommand — identical to implicit form
clr run "Explain this function" --model sonnet

# Explicit print mode (same as above)
clr -p "Explain this function" --model sonnet

# Interactive with message — opt in
clr --interactive "Fix bug" --dir /path/to/project

# Start a fresh session
clr --new-session "Analyse this new codebase"

# Dry-run preview with token limit (output shows -c)
clr --dry-run "Run tests" --max-tokens 50000
```

**Notes:**

`--dry-run` takes precedence over execution regardless of other flags. If present, no subprocess is launched.

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | — |
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | — |
| 3 | [System Prompt](../param_group/03_system_prompt.md) | Full | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [001_interactive_repl.md](../user_story/001_interactive_repl.md) | Developer |
| 2 | [002_print_mode_capture.md](../user_story/002_print_mode_capture.md) | Developer |
| 3 | [003_interactive_with_message.md](../user_story/003_interactive_with_message.md) | Developer |
| 4 | [004_dry_run_preview.md](../user_story/004_dry_run_preview.md) | Developer |
| 5 | [005_project_specific_execution.md](../user_story/005_project_specific_execution.md) | Developer |
| 6 | [006_verbose_debugging.md](../user_story/006_verbose_debugging.md) | Developer |
| 7 | [007_fresh_session.md](../user_story/007_fresh_session.md) | Developer |
| 8 | [008_trace_execution.md](../user_story/008_trace_execution.md) | Developer |
| 9 | [009_custom_system_prompt.md](../user_story/009_custom_system_prompt.md) | Developer |
| 11 | [011_file_input.md](../user_story/011_file_input.md) | Developer |
| 12 | [012_code_block_extraction.md](../user_story/012_code_block_extraction.md) | Developer |
| 13 | [013_structured_json_pipeline.md](../user_story/013_structured_json_pipeline.md) | Developer |
| 22 | [022_session_isolation_subdir.md](../user_story/022_session_isolation_subdir.md) | Developer |
| 23 | [023_output_file_capture.md](../user_story/023_output_file_capture.md) | Developer |
| 24 | [024_enum_output_validation.md](../user_story/024_enum_output_validation.md) | Developer |
| 25 | [025_concurrency_gate.md](../user_story/025_concurrency_gate.md) | Developer |

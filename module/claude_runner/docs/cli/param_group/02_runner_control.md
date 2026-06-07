# CLI Parameter Group: Runner Control

**Pattern:** Consumed by the runner before subprocess launch; may suppress, override, or replace automatic flag injections; never forwarded directly.

**Purpose:** Control runner execution behavior — before, during, or instead of subprocess invocation.

### Semantic Coherence Test

"Is this flag consumed by the runner, not Claude?" — YES for all 25.

### Why NOT X

- `--print`: forwarded to claude subprocess as `--print`
- `--model`: forwarded to claude subprocess as `--model`
- `--verbose`: forwarded to claude subprocess as `--verbose`
- `--effort`: forwarded to claude subprocess as `--effort <level>`
- `--dangerously-skip-permissions`: not a user flag — injected automatically by the runner (default-on)

### Invariants

No parameter is forwarded to the subprocess unchanged. Each is fully consumed by runner logic before subprocess construction.

### Notes

`[MESSAGE]` is not in any group — it is a positional argument serving as input content,
not a control flag. `--help` is handled separately as a universal override.

**Typical usage:**

```sh
clr --dir /project --max-tokens 50000 --verbosity 4 "test"
clr --interactive "Continue this work" --dir /project
clr --new-session --dry-run "check command"
clr --trace "Fix bug" --dir /project
```

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|------------|-----------------|-------|
| 1 | [`run`](../command/01_run.md) | Full | — | All 25 params apply; default command |
| 5 | [`ask`](../command/05_ask.md) | Full | — | All 25 params apply; identical behavior — pure alias for run |

### Referenced Parameters

| Parameter | Type | Default | Role in Group | Description |
|-----------|------|---------|---------------|-------------|
| [`--no-skip-permissions`](../param/005_no_skip_permissions.md) | bool | false | Injection suppressor | Disable automatic permission bypass |
| [`--interactive`](../param/006_interactive.md) | bool | false | Mode selector | Interactive TTY passthrough when message given |
| [`--new-session`](../param/007_new_session.md) | bool | false | Session mode | Start fresh session (disable default continuation) |
| [`--dir`](../param/008_dir.md) | [`DirectoryPath`](../type/02_directory_path.md) | cwd | Working directory | Working directory for subprocess |
| [`--subdir`](../param/028_subdir.md) | string | `.` | Named workspace | Named subdirectory appended to `--dir` (`/-NAME`); `.` = identity |
| [`--max-tokens`](../param/009_max_tokens.md) | [`TokenLimit`](../type/03_token_limit.md) | 200000 | Token cap | Max output tokens |
| [`--session-dir`](../param/010_session_dir.md) | [`DirectoryPath`](../type/02_directory_path.md) | — | Session storage | Session storage directory |
| [`--dry-run`](../param/011_dry_run.md) | bool | false | Execution gate | Preview without executing |
| [`--verbosity`](../param/012_verbosity.md) | [`VerbosityLevel`](../type/05_verbosity_level.md) | 3 | Diagnostic level | Runner output gate level |
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
| [`--expect-retries`](../param/032_expect_retries.md) | u8 | `0` | Retry cap | Re-invocation cap for `retry` strategy |
| [`--max-sessions`](../param/033_max_sessions.md) | u32 | 10 | Concurrency gate | Max concurrent Claude Code sessions before blocking; 0 = unlimited |
| [`--retry-on-rate-limit`](../param/034_retry_on_rate_limit.md) | u8 | `0` | Retry controller | Auto-retry count on transient rate-limit exit; 0 = no retry; `QuotaExhausted` never retried |
| [`--retry-delay`](../param/035_retry_delay.md) | u32 | `60` | Retry delay | Seconds between rate-limit retries; 0 = immediate; ignored when `--retry-on-rate-limit` is 0 |
| [`--timeout`](../param/036_timeout.md) | u32 | `0` | Execution watchdog | Seconds before watchdog kills subprocess; 0 = unlimited (run/ask only; contrast with param 20) |

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
| 20 | [020_suppress_effort_max.md](../user_story/020_suppress_effort_max.md) | Developer |
| 21 | [021_keep_claudecode_context.md](../user_story/021_keep_claudecode_context.md) | Developer |
| 22 | [022_session_isolation_subdir.md](../user_story/022_session_isolation_subdir.md) | Developer |
| 23 | [023_output_file_capture.md](../user_story/023_output_file_capture.md) | Developer |
| 24 | [024_enum_output_validation.md](../user_story/024_enum_output_validation.md) | Developer |
| 25 | [025_concurrency_gate.md](../user_story/025_concurrency_gate.md) | Developer |

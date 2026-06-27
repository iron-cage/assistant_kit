# Parameters

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 001_message.md | `[MESSAGE]` positional parameter spec |
| 002_print.md | `--print` / `-p` flag spec |
| 003_model.md | `--model` parameter spec |
| 004_verbose.md | `--verbose` flag spec |
| 005_no_skip_permissions.md | `--no-skip-permissions` flag spec |
| 006_interactive.md | `--interactive` flag spec |
| 007_new_session.md | `--new-session` flag spec |
| 008_dir.md | `--dir` parameter spec |
| 009_max_tokens.md | `--max-tokens` parameter spec |
| 010_session_dir.md | `--session-dir` parameter spec |
| 011_dry_run.md | `--dry-run` flag spec |
| 012_verbosity.md | `--verbosity` parameter spec |
| 013_trace.md | `--trace` flag spec |
| 014_no_ultrathink.md | `--no-ultrathink` flag spec |
| 015_system_prompt.md | `--system-prompt` parameter spec |
| 016_append_system_prompt.md | `--append-system-prompt` parameter spec |
| 017_effort.md | `--effort` parameter spec |
| 018_no_effort_max.md | `--no-effort-max` flag spec |
| 019_creds.md | `--creds` parameter spec |
| 020_timeout.md | `--timeout` parameter spec |
| 021_no_chrome.md | `--no-chrome` flag spec |
| 022_no_persist.md | `--no-persist` flag spec |
| 023_json_schema.md | `--json-schema` parameter spec |
| 024_mcp_config.md | `--mcp-config` parameter spec |
| 025_file.md | `--file` parameter spec |
| 026_strip_fences.md | `--strip-fences` flag spec |
| 027_keep_claudecode.md | `--keep-claudecode` flag spec |
| 028_subdir.md | `--subdir` parameter spec |
| 029_output_file.md | `--output-file` parameter spec |
| 030_expect.md | `--expect` parameter spec |
| 031_expect_strategy.md | `--expect-strategy` parameter spec |
| 033_max_sessions.md | `--max-sessions` parameter spec |
| 034_retry_on_transient.md | `--retry-on-transient` parameter spec (renamed from --retry-on-rate-limit) |
| 035_transient_delay.md | `--transient-delay` parameter spec (renamed from --retry-delay) |
| 036_timeout.md | `--timeout` parameter spec (run/ask) |
| 040_retry_on_account.md | `--retry-on-account` parameter spec |
| 041_account_delay.md | `--account-delay` parameter spec |
| 042_retry_on_auth.md | `--retry-on-auth` parameter spec |
| 043_auth_delay.md | `--auth-delay` parameter spec |
| 044_retry_on_service.md | `--retry-on-service` parameter spec (renamed from --retry-on-api-error) |
| 045_service_delay.md | `--service-delay` parameter spec (renamed from --api-error-delay) |
| 046_retry_on_process.md | `--retry-on-process` parameter spec |
| 047_process_delay.md | `--process-delay` parameter spec |
| 048_retry_on_validation.md | `--retry-on-validation` parameter spec (renamed from --expect-retries) |
| 049_validation_delay.md | `--validation-delay` parameter spec |
| 050_retry_on_runner.md | `--retry-on-runner` parameter spec |
| 051_runner_delay.md | `--runner-delay` parameter spec |
| 052_retry_on_unknown.md | `--retry-on-unknown` parameter spec (renamed from --retry-on-unknown-error) |
| 053_unknown_delay.md | `--unknown-delay` parameter spec |
| 054_retry_override.md | `--retry-override` parameter spec (Tier 1) |
| 055_retry_override_delay.md | `--retry-override-delay` parameter spec (Tier 1) |
| 056_retry_default.md | `--retry-default` parameter spec (Tier 3, default=2) |
| 057_retry_default_delay.md | `--retry-default-delay` parameter spec (Tier 3, default=30) |
| 058_mode.md | `--mode` parameter spec (ps session filter) |
| 059_columns.md | `--columns` parameter spec (ps column selector) |
| 060_wide.md | `--wide` flag spec (ps wide output) |
| 068_pid.md | `--pid` parameter spec (ps PID filter) |
| 069_inspect.md | `--inspect` flag spec (ps key:value output) |
| 070_output_style.md | `--output-style` parameter spec (runner-level rendering control) |
| 071_summary_fields.md | `--summary-fields` parameter spec (summary field selection) |
| 061_output_format.md | `--output-format` parameter spec |
| 062_max_turns.md | `--max-turns` parameter spec |
| 063_allowed_tools.md | `--allowed-tools` parameter spec |
| 064_disallowed_tools.md | `--disallowed-tools` parameter spec |
| 065_max_budget_usd.md | `--max-budget-usd` parameter spec |
| 066_add_dir.md | `--add-dir` parameter spec |
| 067_fallback_model.md | `--fallback-model` parameter spec |
| 072_journal.md | `--journal` parameter spec |
| 073_journal_dir.md | `--journal-dir` parameter spec |

### All Parameters (69 total)

| # | Parameter | Type | Default | Valid Values | Description | Used In |
|---|-----------|------|---------|--------------|-------------|---------|
| 1 | `[MESSAGE]` | [`MessageText`](../type/01_message_text.md) | — | Any text | Prompt text for Claude | 2 cmds |
| 2 | `-p`/`--print` | bool | auto | present/absent | Explicit print mode (default when message given) | 1 cmd |
| 3 | `--model` | [`ModelName`](../type/04_model_name.md) | — | Any model name | Claude model to use | 1 cmd |
| 4 | `--verbose` | bool | false | present/absent | Enable Claude verbose output | 1 cmd |
| 5 | `--no-skip-permissions` | bool | false | present/absent | Disable automatic permission bypass | 1 cmd |
| 6 | `--interactive` | bool | false | present/absent | Interactive TTY passthrough when message given | 1 cmd |
| 7 | `--new-session` | bool | false | present/absent | Start fresh session (disables default continuation) | 1 cmd |
| 8 | `--dir` | [`DirectoryPath`](../type/02_directory_path.md) | cwd | Any path | Working directory | 1 cmd |
| 9 | `--max-tokens` | [`TokenLimit`](../type/03_token_limit.md) | 200000 | 0 to 4294967295 | Max output tokens | 1 cmd |
| 10 | `--session-dir` | [`DirectoryPath`](../type/02_directory_path.md) | — | Any path | Session storage directory | 1 cmd |
| 11 | `--dry-run` | bool | false | present/absent | Print command without executing | 1 cmd |
| 12 | `--verbosity` | [`VerbosityLevel`](../type/05_verbosity_level.md) | 3 | 0 to 5 | Runner output gate level | 1 cmd |
| 13 | `--trace` | bool | false | present/absent | Print diagnostic details to stderr then execute | 3 cmds |
| 14 | `--no-ultrathink` | bool | false | present/absent | Disable default ultrathink message suffix | 1 cmd |
| 15 | `--system-prompt` | [`SystemPromptText`](../type/06_system_prompt_text.md) | — | Any text | Set system prompt (replaces the default) | 1 cmd |
| 16 | `--append-system-prompt` | [`SystemPromptText`](../type/06_system_prompt_text.md) | — | Any text | Append text to the default system prompt | 1 cmd |
| 17 | `--effort` | [`EffortLevel`](../type/07_effort_level.md) | max | low/medium/high/max | Reasoning effort level forwarded to claude | 1 cmd |
| 18 | `--no-effort-max` | bool | false | present/absent | Suppress default `--effort max` injection | 1 cmd |
| 19 | `--creds` | [`CredentialsFilePath`](../type/08_credentials_file_path.md) | `~/.claude/.credentials.json` | Any existing file path | Credentials JSON file (optional; defaults to current account) | 2 cmds |
| 20 | `--timeout` | [`TimeoutSecs`](../type/09_timeout_secs.md) | 30/45 | Non-negative integer | Max seconds to wait for subprocess (30 isolated, 45 refresh) | 2 cmds |
| 21 | `--no-chrome` | bool | false | present/absent | Suppress default `--chrome` injection | 1 cmd |
| 22 | `--no-persist` | bool | false | present/absent | Disable session persistence (`--no-session-persistence`) | 1 cmd |
| 23 | `--json-schema` | [`JsonSchemaText`](../type/10_json_schema_text.md) | — | Valid JSON object string | JSON Schema for structured output validation | 1 cmd |
| 24 | `--mcp-config` | [`McpConfigPath`](../type/11_mcp_config_path.md) | — | Any existing file path | MCP server config file (repeatable) | 1 cmd |
| 25 | `--file` | [`FilePath`](../type/12_file_path.md) | — | Any readable file path | File content piped as subprocess stdin | 1 cmd |
| 26 | `--strip-fences` | bool | false | present/absent | Strip outermost markdown code fences from stdout | 1 cmd |
| 27 | `--keep-claudecode` | bool | false | present/absent | Preserve `CLAUDECODE` env var in subprocess (default: removed) | 1 cmd |
| 28 | `--subdir` | string | `.` | `.` or any name | Named subdirectory appended to `--dir` (`/-NAME`); `.` = identity | 2 cmds |
| 29 | `--output-file` | string | — | Any writable path | Write stdout to file in addition to printing (tee behavior) | 2 cmds |
| 30 | `--expect` | string | — | `val1\|val2\|…` | Pipe-separated enum values; stdout must match one after trim+lowercase | 2 cmds |
| 31 | `--expect-strategy` | enum | `fail` | `fail`/`retry`/`default:<V>` | Mismatch handling: exit 3, retry N times, or output fallback value | 2 cmds |
| 33 | `--max-sessions` | u32 | 30 | 0 to 4294967295 | Max concurrent Claude Code sessions before blocking; 0 = unlimited | 2 cmds |
| 34 | `--retry-on-transient` | u8 | auto | 0–255 | Transient class retry count (Tier 2) | 2 cmds |
| 35 | `--transient-delay` | u32 | auto | 0 to 4294967295 | Transient class delay (Tier 2) | 2 cmds |
| 36 | `--timeout` | u32 | `3600` (print) / `0` (interactive) | 0 to 4294967295 | Seconds before watchdog kills subprocess; 0 = unlimited (run/ask only; contrast with param 20) | 2 cmds |
| 40 | `--retry-on-account` | u8 | auto | 0–255 | Account class retry count (Tier 2) | 2 cmds |
| 41 | `--account-delay` | u32 | auto | 0 to 4294967295 | Account class delay (Tier 2) | 2 cmds |
| 42 | `--retry-on-auth` | u8 | auto | 0–255 | Auth class retry count (Tier 2) | 2 cmds |
| 43 | `--auth-delay` | u32 | auto | 0 to 4294967295 | Auth class delay (Tier 2) | 2 cmds |
| 44 | `--retry-on-service` | u8 | auto | 0–255 | Service class retry count (Tier 2) | 2 cmds |
| 45 | `--service-delay` | u32 | auto | 0 to 4294967295 | Service class delay (Tier 2) | 2 cmds |
| 46 | `--retry-on-process` | u8 | auto | 0–255 | Process class retry count (Tier 2) | 2 cmds |
| 47 | `--process-delay` | u32 | auto | 0 to 4294967295 | Process class delay (Tier 2) | 2 cmds |
| 48 | `--retry-on-validation` | u8 | auto | 0–255 | Validation class retry count (Tier 2); only with `--expect-strategy retry` | 2 cmds |
| 49 | `--validation-delay` | u32 | auto | 0 to 4294967295 | Validation class delay (Tier 2) | 2 cmds |
| 50 | `--retry-on-runner` | u8 | auto | 0–255 | Runner class retry count (Tier 2) | 2 cmds |
| 51 | `--runner-delay` | u32 | auto | 0 to 4294967295 | Runner class delay (Tier 2) | 2 cmds |
| 52 | `--retry-on-unknown` | u8 | auto | 0–255 | Unknown class retry count (Tier 2) | 2 cmds |
| 53 | `--unknown-delay` | u32 | auto | 0 to 4294967295 | Unknown class delay (Tier 2) | 2 cmds |
| 54 | `--retry-override` | u8 | auto | 0–255 | Tier 1: forces retry count for all error classes | 2 cmds |
| 55 | `--retry-override-delay` | u32 | auto | 0 to 4294967295 | Tier 1: forces delay for all error classes | 2 cmds |
| 56 | `--retry-default` | u8 | `2` | 0–255 | Tier 3: fallback retry count for all unset classes | 2 cmds |
| 57 | `--retry-default-delay` | u32 | `30` | 0 to 4294967295 | Tier 3: fallback delay for all unset classes | 2 cmds |
| 58 | `--mode` | enum | `all` | `all`/`interactive`/`print` | Filter `clr ps` by session execution mode | 1 cmd |
| 59 | `--columns` | string | 9 default cols | Comma-separated column keys | Select which columns to display in `clr ps` | 1 cmd |
| 60 | `--wide` | bool | false | present/absent | Show all 11 columns in `clr ps` | 1 cmd |
| 61 | `--output-format` | enum | — | `text`/`json`/`stream-json` | Output format for Claude Code response | 2 cmds |
| 62 | `--max-turns` | u32 | — | 0 to 4294967295 | Max agentic turns before stopping; unset = unlimited | 2 cmds |
| 63 | `--allowed-tools` | string | — | Tool name list | Restrict Claude to specified tools only | 2 cmds |
| 64 | `--disallowed-tools` | string | — | Tool name list | Prevent Claude from using specified tools | 2 cmds |
| 65 | `--max-budget-usd` | f64 | — | Positive float | Max dollar budget for session | 2 cmds |
| 66 | `--add-dir` | path | — | Any path | Additional directory for Claude Code to access | 2 cmds |
| 67 | `--fallback-model` | string | — | Any model name | Fallback model when primary unavailable | 2 cmds |
| 68 | `--pid` | string | — | Comma-separated numeric PIDs | Filter `clr ps` active table to specified process IDs only | 1 cmd |
| 69 | `--inspect` | bool | false | present/absent | Switch `clr ps` to key:value record output showing all 12 session attributes | 1 cmd |
| 70 | `--output-style` | enum | `summary` | `summary`/`raw` | Runner-level rendering control: `summary` routes stdout through `render_summary()`; `raw` passes through unchanged | 2 cmds |
| 71 | `--summary-fields` | string | `full` | `minimal`/`standard`/`full`/custom | Select which CLR envelope fields appear in summary header; custom = comma-separated field names | 2 cmds |
| 72 | `--journal` | enum | `full` | `full`/`meta`/`off` | Journal level for clr execution events: `full` captures stdout+stderr (≤1MB each), `meta` omits output, `off` disables | 2 cmds |
| 73 | `--journal-dir` | path | `~/.clr/journal/` | Any writable path | Directory for journal JSONL files; overrides `CLR_JOURNAL_DIR` | 2 cmds |

**Total:** 69 parameters

**Groups:** Parameters 2–4, 17, 23, 24, and 61–67 form [Claude-Native Flags](../param_group/01_claude_native_flags.md). Parameters 5–14, 18, 21, 22, 25–36, 40–57, 70–73 form [Runner Control](../param_group/02_runner_control.md). Parameters 15–16 form [System Prompt](../param_group/03_system_prompt.md). Parameters 19–20 form [Credential Operations](../param_group/04_credential_operations.md). Parameters 58–60, 68–69 form [Session Listing](../param_group/05_session_listing.md).

### Navigation

- [`[MESSAGE]`](001_message.md)
- [`--print`](002_print.md)
- [`--model`](003_model.md)
- [`--verbose`](004_verbose.md)
- [`--no-skip-permissions`](005_no_skip_permissions.md)
- [`--interactive`](006_interactive.md)
- [`--new-session`](007_new_session.md)
- [`--dir`](008_dir.md)
- [`--max-tokens`](009_max_tokens.md)
- [`--session-dir`](010_session_dir.md)
- [`--dry-run`](011_dry_run.md)
- [`--verbosity`](012_verbosity.md)
- [`--trace`](013_trace.md)
- [`--no-ultrathink`](014_no_ultrathink.md)
- [`--system-prompt`](015_system_prompt.md)
- [`--append-system-prompt`](016_append_system_prompt.md)
- [`--effort`](017_effort.md)
- [`--no-effort-max`](018_no_effort_max.md)
- [`--creds`](019_creds.md)
- [`--timeout`](020_timeout.md)
- [`--no-chrome`](021_no_chrome.md)
- [`--no-persist`](022_no_persist.md)
- [`--json-schema`](023_json_schema.md)
- [`--mcp-config`](024_mcp_config.md)
- [`--file`](025_file.md)
- [`--strip-fences`](026_strip_fences.md)
- [`--keep-claudecode`](027_keep_claudecode.md)
- [`--subdir`](028_subdir.md)
- [`--output-file`](029_output_file.md)
- [`--expect`](030_expect.md)
- [`--expect-strategy`](031_expect_strategy.md)
- [`--max-sessions`](033_max_sessions.md)
- [`--retry-on-transient`](034_retry_on_transient.md)
- [`--transient-delay`](035_transient_delay.md)
- [`--timeout` (run/ask)](036_timeout.md)
- [`--retry-on-account`](040_retry_on_account.md)
- [`--account-delay`](041_account_delay.md)
- [`--retry-on-auth`](042_retry_on_auth.md)
- [`--auth-delay`](043_auth_delay.md)
- [`--retry-on-service`](044_retry_on_service.md)
- [`--service-delay`](045_service_delay.md)
- [`--retry-on-process`](046_retry_on_process.md)
- [`--process-delay`](047_process_delay.md)
- [`--retry-on-validation`](048_retry_on_validation.md)
- [`--validation-delay`](049_validation_delay.md)
- [`--retry-on-runner`](050_retry_on_runner.md)
- [`--runner-delay`](051_runner_delay.md)
- [`--retry-on-unknown`](052_retry_on_unknown.md)
- [`--unknown-delay`](053_unknown_delay.md)
- [`--retry-override`](054_retry_override.md)
- [`--retry-override-delay`](055_retry_override_delay.md)
- [`--retry-default`](056_retry_default.md)
- [`--retry-default-delay`](057_retry_default_delay.md)
- [`--mode`](058_mode.md)
- [`--columns`](059_columns.md)
- [`--wide`](060_wide.md)
- [`--pid`](068_pid.md)
- [`--inspect`](069_inspect.md)
- [`--output-style`](070_output_style.md)
- [`--summary-fields`](071_summary_fields.md)
- [`--journal`](072_journal.md)
- [`--journal-dir`](073_journal_dir.md)
- [`--output-format`](061_output_format.md)
- [`--max-turns`](062_max_turns.md)
- [`--allowed-tools`](063_allowed_tools.md)
- [`--disallowed-tools`](064_disallowed_tools.md)
- [`--max-budget-usd`](065_max_budget_usd.md)
- [`--add-dir`](066_add_dir.md)
- [`--fallback-model`](067_fallback_model.md)

### Quick Reference

**Required parameters:** `[MESSAGE]` is required for print mode (which is the default when a message is given).

**Most used parameters:** `--model` (model selection), `--dir` (project targeting), `--subdir` (session isolation by task name), `--dry-run` (debugging), `--new-session` (fresh start), `--interactive` (TTY passthrough with prompt), `--file` (stdin from file), `--strip-fences` (extract code block content).

**Commands by parameter count:** `run` = 62, `ask` = 62, `ps` = 5, `isolated` = 4, `refresh` = 3, `kill` = 0, `tools` = 0, `help` = 0.

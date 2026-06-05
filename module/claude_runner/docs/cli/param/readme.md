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

### All Parameters (28 total)

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

**Total:** 28 parameters

**Groups:** Parameters 2–4, 17, 23, and 24 form [Claude-Native Flags](../param_group/01_claude_native_flags.md). Parameters 5–14, 18, 21, 22, 25, 26, 27, and 28 form [Runner Control](../param_group/02_runner_control.md). Parameters 15–16 form [System Prompt](../param_group/03_system_prompt.md). Parameters 19–20 form [Credential Operations](../param_group/04_credential_operations.md).

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

### Quick Reference

**Required parameters:** `[MESSAGE]` is required for print mode (which is the default when a message is given).

**Most used parameters:** `--model` (model selection), `--dir` (project targeting), `--subdir` (session isolation by task name), `--dry-run` (debugging), `--new-session` (fresh start), `--interactive` (TTY passthrough with prompt), `--file` (stdin from file), `--strip-fences` (extract code block content).

**Commands by parameter count:** `run` = 26 parameters, `isolated` = 4 parameters, `refresh` = 3 parameters, `help` = 0 parameters.

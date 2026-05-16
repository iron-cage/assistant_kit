# Parameters

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_message.md | `[MESSAGE]` positional parameter spec |
| 02_print.md | `--print` / `-p` flag spec |
| 03_model.md | `--model` parameter spec |
| 04_verbose.md | `--verbose` flag spec |
| 05_no_skip_permissions.md | `--no-skip-permissions` flag spec |
| 06_interactive.md | `--interactive` flag spec |
| 07_new_session.md | `--new-session` flag spec |
| 08_dir.md | `--dir` parameter spec |
| 09_max_tokens.md | `--max-tokens` parameter spec |
| 10_session_dir.md | `--session-dir` parameter spec |
| 11_dry_run.md | `--dry-run` flag spec |
| 12_verbosity.md | `--verbosity` parameter spec |
| 13_trace.md | `--trace` flag spec |
| 14_no_ultrathink.md | `--no-ultrathink` flag spec |
| 15_system_prompt.md | `--system-prompt` parameter spec |
| 16_append_system_prompt.md | `--append-system-prompt` parameter spec |
| 17_effort.md | `--effort` parameter spec |
| 18_no_effort_max.md | `--no-effort-max` flag spec |

### All Parameters (18 total)

| # | Parameter | Type | Default | Valid Values | Description | Used In |
|---|-----------|------|---------|--------------|-------------|---------|
| 1 | `[MESSAGE]` | [`MessageText`](../type.md#type--1-messagetext) | — | Any text | Prompt text for Claude | 1 cmd |
| 2 | `-p`/`--print` | bool | auto | present/absent | Explicit print mode (default when message given) | 1 cmd |
| 3 | `--model` | [`ModelName`](../type.md#type--4-modelname) | — | Any model name | Claude model to use | 1 cmd |
| 4 | `--verbose` | bool | false | present/absent | Enable Claude verbose output | 1 cmd |
| 5 | `--no-skip-permissions` | bool | false | present/absent | Disable automatic permission bypass | 1 cmd |
| 6 | `--interactive` | bool | false | present/absent | Interactive TTY passthrough when message given | 1 cmd |
| 7 | `--new-session` | bool | false | present/absent | Start fresh session (disables default continuation) | 1 cmd |
| 8 | `--dir` | [`DirectoryPath`](../type.md#type--2-directorypath) | cwd | Any path | Working directory | 1 cmd |
| 9 | `--max-tokens` | [`TokenLimit`](../type.md#type--3-tokenlimit) | 200000 | 0 to 4294967295 | Max output tokens | 1 cmd |
| 10 | `--session-dir` | [`DirectoryPath`](../type.md#type--2-directorypath) | — | Any path | Session storage directory | 1 cmd |
| 11 | `--dry-run` | bool | false | present/absent | Print command without executing | 1 cmd |
| 12 | `--verbosity` | [`VerbosityLevel`](../type.md#type--5-verbositylevel) | 3 | 0 to 5 | Runner output gate level | 1 cmd |
| 13 | `--trace` | bool | false | present/absent | Print env+command to stderr then execute | 1 cmd |
| 14 | `--no-ultrathink` | bool | false | present/absent | Disable default ultrathink message suffix | 1 cmd |
| 15 | `--system-prompt` | [`SystemPromptText`](../type.md#type--6-systemprompttext) | — | Any text | Set system prompt (replaces the default) | 1 cmd |
| 16 | `--append-system-prompt` | [`SystemPromptText`](../type.md#type--6-systemprompttext) | — | Any text | Append text to the default system prompt | 1 cmd |
| 17 | `--effort` | [`EffortLevel`](../type.md#type--7-effortlevel) | max | low/medium/high/max | Reasoning effort level forwarded to claude | 1 cmd |
| 18 | `--no-effort-max` | bool | false | present/absent | Suppress default `--effort max` injection | 1 cmd |

**Total:** 18 parameters

**Groups:** Parameters 2–4 and 17 form [Claude-Native Flags](../param_group.md#group--1-claude-native-flags). Parameters 5–14 and 18 form [Runner Control](../param_group.md#group--2-runner-control). Parameters 15–16 form [System Prompt](../param_group.md#group--3-system-prompt).

### Navigation

- [`[MESSAGE]`](01_message.md)
- [`--print`](02_print.md)
- [`--model`](03_model.md)
- [`--verbose`](04_verbose.md)
- [`--no-skip-permissions`](05_no_skip_permissions.md)
- [`--interactive`](06_interactive.md)
- [`--new-session`](07_new_session.md)
- [`--dir`](08_dir.md)
- [`--max-tokens`](09_max_tokens.md)
- [`--session-dir`](10_session_dir.md)
- [`--dry-run`](11_dry_run.md)
- [`--verbosity`](12_verbosity.md)
- [`--trace`](13_trace.md)
- [`--no-ultrathink`](14_no_ultrathink.md)
- [`--system-prompt`](15_system_prompt.md)
- [`--append-system-prompt`](16_append_system_prompt.md)
- [`--effort`](17_effort.md)
- [`--no-effort-max`](18_no_effort_max.md)

### Quick Reference

**Required parameters:** `[MESSAGE]` is required for print mode (which is the default when a message is given).

**Most used parameters:** `--model` (model selection), `--dir` (project targeting), `--dry-run` (debugging), `--new-session` (fresh start), `--interactive` (TTY passthrough with prompt).

**Commands by parameter count:** `run` = 18 parameters, `help` = 0 parameters.

# Commands

### All Commands (4 total)

| # | Command | Description | Params | Example |
|---|---------|-------------|--------|---------|
| 1 | `run` (default) | Execute Claude Code with given parameters | 25 | `clr "Fix bug" --model sonnet` |
| 2 | `isolated` | Run Claude with credential-isolated temp HOME | 4 | `clr isolated --creds creds.json "Fix bug"` |
| 3 | `refresh` | Refresh OAuth credentials without running a task | 3 | `clr refresh --creds creds.json` |
| 4 | `help` | Print usage information and exit | 0 | `clr help` |

**Total:** 4 commands

---

### Command :: 1. `run`

Execute Claude Code as a subprocess with configurable flags. This is the
default command — invoked whenever no explicit subcommand is given.

**Syntax:**

```sh
clr [OPTIONS] [MESSAGE]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`[MESSAGE]`](param/01_message.md) | [`MessageText`](type.md#type--1-messagetext) | — | Prompt text for Claude |
| [`-p`/`--print`](param/02_print.md) | bool | auto | Print mode (default when message given; explicit alias) |
| [`--model`](param/03_model.md) | [`ModelName`](type.md#type--4-modelname) | — | Model to use |
| [`--verbose`](param/04_verbose.md) | bool | false | Enable Claude verbose output |
| [`--no-skip-permissions`](param/05_no_skip_permissions.md) | bool | false | Disable automatic permission bypass |
| [`--interactive`](param/06_interactive.md) | bool | false | Interactive TTY passthrough when message given |
| [`--new-session`](param/07_new_session.md) | bool | false | Start fresh session (disables default continuation) |
| [`--dir`](param/08_dir.md) | [`DirectoryPath`](type.md#type--2-directorypath) | cwd | Working directory |
| [`--max-tokens`](param/09_max_tokens.md) | [`TokenLimit`](type.md#type--3-tokenlimit) | 200000 | Max output tokens |
| [`--session-dir`](param/10_session_dir.md) | [`DirectoryPath`](type.md#type--2-directorypath) | — | Session storage directory |
| [`--dry-run`](param/11_dry_run.md) | bool | false | Print command without executing |
| [`--verbosity`](param/12_verbosity.md) | [`VerbosityLevel`](type.md#type--5-verbositylevel) | 3 | Runner output gate level |
| [`--trace`](param/13_trace.md) | bool | false | Print env+command to stderr then execute (like `set -x`) |
| [`--no-ultrathink`](param/14_no_ultrathink.md) | bool | false | Disable default ultrathink message suffix |
| [`--system-prompt`](param/15_system_prompt.md) | [`SystemPromptText`](type.md#type--6-systemprompttext) | — | Set system prompt (replaces the default) |
| [`--append-system-prompt`](param/16_append_system_prompt.md) | [`SystemPromptText`](type.md#type--6-systemprompttext) | — | Append text to the default system prompt |
| [`--effort`](param/17_effort.md) | [`EffortLevel`](type.md#type--7-effortlevel) | max | Override reasoning effort level (default: max) |
| [`--no-effort-max`](param/18_no_effort_max.md) | bool | false | Suppress default `--effort max` injection |
| [`--no-chrome`](param/21_no_chrome.md) | bool | false | Suppress default `--chrome` injection |
| [`--no-persist`](param/22_no_persist.md) | bool | false | Disable session persistence (`--no-session-persistence`) |
| [`--json-schema`](param/23_json_schema.md) | [`JsonSchemaText`](type.md#type--10-jsonschematext) | — | JSON schema for structured output |
| [`--mcp-config`](param/24_mcp_config.md) | [`McpConfigPath`](type.md#type--11-mcpconfigpath) | — | MCP server config file (repeatable, 0+) |
| [`--file`](param/25_file.md) | [`FilePath`](type.md#type--12-filepath) | — | File content piped as subprocess stdin |
| [`--strip-fences`](param/26_strip_fences.md) | bool | false | Strip outermost markdown code fences from stdout |
| [`--keep-claudecode`](param/27_keep_claudecode.md) | bool | false | Preserve `CLAUDECODE` env var in subprocess (default: removed) |

**Parameter Groups:** [Claude-Native Flags](param_group.md#group--1-claude-native-flags), [Runner Control](param_group.md#group--2-runner-control), [System Prompt](param_group.md#group--3-system-prompt)

**Execution Modes:**

All modes apply `-c` automatically (continuing the previous session).
Use `--new-session` to start fresh.

| Invocation | Mode | Path |
|------------|------|------|
| `clr` | Interactive REPL | `execute_interactive()` + `-c` |
| `clr "Fix bug"` | **Print (default)** | `execute()` + `--print` + `-c` |
| `clr -p "Fix bug"` | Print (explicit) | `execute()` + `--print` + `-c` |
| `clr --interactive "Fix bug"` | Interactive | `execute_interactive()` + `-c` |
| `clr --dry-run "Fix bug"` | Preview only | `describe()` / `describe_env()` (shows `-c`) |
| `clr --trace "Fix bug"` | Trace (print then execute) | `describe_env()` + `describe()` to stderr, then `execute()` |
| `clr --new-session "Fix bug"` | Fresh session, print | `execute()` + `--print` (no `-c`) |

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (parse failure, print mode without message, execution error) |
| N | Passthrough from claude subprocess |

**Examples:**

```sh
# Interactive REPL (no message)
clr

# Print mode — default when message given
clr "Explain this function" --model sonnet

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

### Referenced User Stories

| # | User Story | Notes |
|---|-----------|-------|
| 1 | [001 Interactive REPL](user_story/001_interactive_repl.md) | |
| 2 | [002 Print Mode Capture](user_story/002_print_mode_capture.md) | |
| 3 | [003 Interactive With Message](user_story/003_interactive_with_message.md) | |
| 4 | [004 Dry-run Preview](user_story/004_dry_run_preview.md) | |
| 5 | [005 Project-specific Execution](user_story/005_project_specific_execution.md) | |
| 6 | [006 Verbose Debugging](user_story/006_verbose_debugging.md) | |
| 7 | [007 Fresh Session](user_story/007_fresh_session.md) | |
| 8 | [008 Trace Execution](user_story/008_trace_execution.md) | |
| 9 | [009 Custom System Prompt](user_story/009_custom_system_prompt.md) | |
| 10 | [011 File Input](user_story/011_file_input.md) | |
| 11 | [012 Code Block Extraction](user_story/012_code_block_extraction.md) | |
| 12 | [013 Structured JSON Pipeline](user_story/013_structured_json_pipeline.md) | |

---

### Command :: 2. `isolated`

Run Claude in a credential-isolated subprocess. Creates a temporary `HOME`
directory containing only `.claude/.credentials.json` populated from
`--creds`, then spawns Claude with `HOME=<temp>`. Waits at most `--timeout`
seconds, then deletes the temp HOME unconditionally. If Claude refreshes its
OAuth token, the updated credentials are written back to `--creds` in-place.

**Syntax:**

```sh
clr isolated --creds <FILE> [--timeout <SECS>] [MESSAGE]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`[MESSAGE]`](param/01_message.md) | [`MessageText`](type.md#type--1-messagetext) | — | Prompt forwarded to Claude |
| [`--creds`](param/19_creds.md) | [`CredentialsFilePath`](type.md#type--8-credentialsfilepath) | — | Credentials JSON file path (required) |
| [`--timeout`](param/20_timeout.md) | [`TimeoutSecs`](type.md#type--9-timeoutsecs) | 30 | Max seconds to wait for subprocess |
| [`--trace`](param/13_trace.md) | bool | false | Print underlying call details to stderr then execute |
| `-h`/`--help` | — | — | Print isolated subcommand help and exit 0 |

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Claude exited successfully (may have refreshed creds in-place) |
| 1 | Error (creds file not found, claude not in PATH, I/O failure) |
| 2 | Timeout — subprocess did not finish within `--timeout` seconds |
| N | Passthrough from claude subprocess (non-zero) |

**Examples:**

```sh
# Quick prompt with isolated credentials
clr isolated --creds ~/.claude/.credentials.json "What is 2+2?"

# Custom timeout for long-running tasks
clr isolated --creds /path/to/creds.json --timeout 120 "Refactor this module"

# Verify credentials work (--version exits fast)
clr isolated --creds /path/to/creds.json -- --version

# Interactive isolated session (no message — REPL mode)
clr isolated --creds /path/to/creds.json
```

**Notes:**

The isolated subprocess has no access to the caller's real `$HOME` — no
`~/.claude/settings.json`, no previous conversation state, no CLAUDE.md
from the user's home. Only `.claude/.credentials.json` is present.

If the subprocess times out but already wrote refreshed credentials (OAuth
token refresh at startup before blocking on input), `clr isolated` exits 0
and writes updated credentials back to `--creds` instead of returning exit 2.
This matches the `IsolatedRunResult { exit_code: -1, credentials: Some(…) }`
path in `claude_runner_core::run_isolated()`.

### Referenced User Stories

| # | User Story | Notes |
|---|-----------|-------|
| 1 | [010 Credential-isolated Execution](user_story/010_credential_isolated_execution.md) | |

---

### Command :: 3. `refresh`

Refresh OAuth credentials without running an actual Claude task. Creates a
temporary `HOME` (like `isolated`), spawns `claude --print "."` to trigger the
startup token refresh, then writes the updated credentials back to `--creds`
in-place. No user task is executed — the subprocess returns immediately after
the token refresh completes.

**Syntax:**

```sh
clr refresh --creds <FILE> [--timeout <SECS>] [--trace]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`--creds`](param/19_creds.md) | [`CredentialsFilePath`](type.md#type--8-credentialsfilepath) | — | Credentials JSON file path (required) |
| [`--timeout`](param/20_timeout.md) | [`TimeoutSecs`](type.md#type--9-timeoutsecs) | 45 | Max seconds to wait for refresh |
| [`--trace`](param/13_trace.md) | bool | false | Print underlying call details to stderr then execute |
| `-h`/`--help` | — | — | Print refresh subcommand help and exit 0 |

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Credentials were refreshed and written back to `--creds` |
| 1 | Error (creds file not found, claude not in PATH, I/O failure, no refresh occurred) |
| 2 | Timeout — subprocess did not finish within `--timeout` seconds and no refresh occurred |

**Examples:**

```sh
# Refresh credentials with default 45s timeout
clr refresh --creds ~/.claude/.credentials.json

# Refresh with custom timeout for slow networks
clr refresh --creds /path/to/creds.json --timeout 90

# Trace the underlying call to see what happens
clr refresh --creds creds.json --trace
```

**Notes:**

Internally calls `run_isolated()` with fixed args `["--print", "."]`. The `claude`
binary refreshes its OAuth token at startup before processing the trivial `.` prompt,
then exits. If the token was refreshed, `clr refresh` writes the updated credentials
back to `--creds` and exits 0.

The default timeout of 45 seconds (vs 30 for `isolated`) allows headroom for slow
networks and API rate limiting during the OAuth token exchange.

### Referenced User Stories

| # | User Story | Notes |
|---|-----------|-------|
| 1 | [014 Credential Refresh](user_story/014_credential_refresh.md) | |

---

### Command :: 4. `help`

Print usage information listing all commands, flags, and their defaults,
then exit with code 0.

**Syntax:**

```sh
clr help
clr -h
clr --help
```

**Parameters:** none

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Always |

**Notes:** `clr help` is the canonical word-subcommand form. `--help` / `-h`
anywhere in argv are parameter aliases that trigger identical behavior. All
three forms override any other flags. Empty argv (no arguments) enters
interactive mode, not help.

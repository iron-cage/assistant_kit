# Commands

### All Commands (3 total)

| # | Command | Description | Params | Example |
|---|---------|-------------|--------|---------|
| 1 | `run` (default) | Execute Claude Code with given parameters | 18 | `clr "Fix bug" --model sonnet` |
| 2 | `isolated` | Run Claude with credential-isolated temp HOME | 3 | `clr isolated --creds creds.json "Fix bug"` |
| 3 | `help` | Print usage information and exit | 0 | `clr --help` |

**Total:** 3 commands

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

**Hypothesis (H1) — `--tools ""`:** Passing an empty tools list blocks tool *invocation*
but does NOT strip tool *definitions* from the assembled system prompt. The ~12k token cost
is paid regardless — Claude knows about tools but cannot call them. Status: ❓ unverified.
Validation: run `claude --tools "" --print "hi" --output-format json | jq '.usage'` and
compare input token count against a baseline without `--tools ""`; then observe whether
Claude attempts tool calls in a live conversation.

| Layer | `--system-prompt` | `--append-system-prompt` | `--tools ""` ❓ |
|-------|------------------|--------------------------|-----------------|
| Tool definitions (~12k tokens) | ✅ Preserved | ✅ Preserved | ❓ Likely preserved (unverified) |
| Tool invocation | ✅ Enabled | ✅ Enabled | ❓ Likely blocked (unverified) |
| Coding guidelines and style | ❌ Removed | ✅ Kept | ✅ Kept |
| Git safety rules | ❌ Removed | ✅ Kept | ✅ Kept |
| Security constraints | ❌ Removed | ✅ Kept | ✅ Kept |
| CLAUDE.md handling instructions | ❌ Removed | ✅ Kept | ✅ Kept |
| Output style (conciseness, no emojis) | ❌ Removed | ✅ Kept | ✅ Kept |
| Sub-agent coordination prompts | ❌ Removed | ✅ Kept | ✅ Kept |

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

---

### Command :: 3. `help`

Print usage information listing all commands, flags, and their defaults,
then exit with code 0.

**Syntax:**

```sh
clr -h
clr --help
```

**Parameters:** none

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Always |

**Notes:** `--help` / `-h` anywhere in argv overrides any other flags and
triggers help output. Empty argv (no arguments) enters interactive mode.

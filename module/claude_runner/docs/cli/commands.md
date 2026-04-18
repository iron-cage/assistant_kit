# Commands

### All Commands (2 total)

| # | Command | Description | Params | Example |
|---|---------|-------------|--------|---------|
| 1 | `run` (default) | Execute Claude Code with given parameters | 18 | `clr "Fix bug" --model sonnet` |
| 2 | `help` | Print usage information and exit | 0 | `clr --help` |

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
| [`[MESSAGE]`](params.md#parameter--1-message) | [`MessageText`](types.md#type--1-messagetext) | — | Prompt text for Claude |
| [`-p`/`--print`](params.md#parameter--2---print) | bool | auto | Print mode (default when message given; explicit alias) |
| [`--model`](params.md#parameter--3---model) | [`ModelName`](types.md#type--4-modelname) | — | Model to use |
| [`--verbose`](params.md#parameter--4---verbose) | bool | false | Enable Claude verbose output |
| [`--no-skip-permissions`](params.md#parameter--5---no-skip-permissions) | bool | false | Disable automatic permission bypass |
| [`--interactive`](params.md#parameter--6---interactive) | bool | false | Interactive TTY passthrough when message given |
| [`--new-session`](params.md#parameter--7---new-session) | bool | false | Start fresh session (disables default continuation) |
| [`--dir`](params.md#parameter--8---dir) | [`DirectoryPath`](types.md#type--2-directorypath) | cwd | Working directory |
| [`--max-tokens`](params.md#parameter--9---max-tokens) | [`TokenLimit`](types.md#type--3-tokenlimit) | 200000 | Max output tokens |
| [`--session-dir`](params.md#parameter--10---session-dir) | [`DirectoryPath`](types.md#type--2-directorypath) | — | Session storage directory |
| [`--dry-run`](params.md#parameter--11---dry-run) | bool | false | Print command without executing |
| [`--verbosity`](params.md#parameter--12---verbosity) | [`VerbosityLevel`](types.md#type--5-verbositylevel) | 3 | Runner output gate level |
| [`--trace`](params.md#parameter--13---trace) | bool | false | Print env+command to stderr then execute (like `set -x`) |
| [`--no-ultrathink`](params.md#parameter--14---no-ultrathink) | bool | false | Disable default ultrathink message suffix |
| [`--system-prompt`](params.md#parameter--15---system-prompt) | [`SystemPromptText`](types.md#type--6-systemprompttext) | — | Set system prompt (replaces the default) |
| [`--append-system-prompt`](params.md#parameter--16---append-system-prompt) | [`SystemPromptText`](types.md#type--6-systemprompttext) | — | Append text to the default system prompt |
| [`--effort`](params.md#parameter--17---effort) | [`EffortLevel`](types.md#type--7-effortlevel) | max | Override reasoning effort level (default: max) |
| [`--no-effort-max`](params.md#parameter--18---no-effort-max) | bool | false | Suppress default `--effort max` injection |

**Parameter Groups:** [Claude-Native Flags](parameter_groups.md#group--1-claude-native-flags), [Runner Control](parameter_groups.md#group--2-runner-control), [System Prompt](parameter_groups.md#group--3-system-prompt)

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

---

### Command :: 2. `help`

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

# Dictionary

### Scope

- **Purpose**: Provide domain vocabulary for clr CLI concepts, modes, and architecture.
- **Responsibility**: Define canonical terms used throughout the CLI documentation.
- **In Scope**: Command terms, mode terms, architecture terms.
- **Out of Scope**: Type definitions with full specs (→ `type/`), parameter usage (→ `param/`).

### Commands

| Term | Definition |
|------|------------|
| run | Default command that builds and executes a `claude` subprocess with the given flags |
| ask | Semantic alias for `run` with identical parameters and defaults; no behavioral differences |
| isolated | Subcommand that runs `claude` in a credential-isolated temporary HOME; requires `--creds` |
| refresh | Subcommand that refreshes OAuth credentials via `run_isolated()` with `["--print", "."]`; requires `--creds`; no task executed |
| ps | List running Claude Code sessions with process metrics; supports `--mode`, `--columns`, `--wide`, `--pid`, `--inspect` |
| kill | Terminate a running Claude Code session by PID via SIGTERM; canonical form `clr kill <PID>` |
| tools | List all Claude Code built-in tools available to the subprocess; canonical form `clr tools` |
| help | Display usage information and exit; canonical form `clr help`; `--help`/`-h` are parameter aliases |

### Modes

| Term | Definition |
|------|------------|
| interactive mode | Default TTY passthrough mode; stdin/stdout connected directly to the claude subprocess; continues previous session automatically |
| print mode | Non-interactive capture mode (`-p`/`--print`); stdout collected and printed for programmatic use; continues previous session automatically |
| dry-run | Preview mode (`--dry-run`); prints the assembled command without executing it; output shows `-c` when a prior session exists for the effective working directory |
| new session | Invocation with `--new-session`; starts a fresh Claude conversation with no prior context (omits the default `-c`) |
| ultrathink suffix | Text `"\n\nultrathink"` appended after every message before it is sent to the claude subprocess; activates Claude's extended thinking mode; default-on, suppressed with `--no-ultrathink` |
| credential-isolated mode | Invocation via `clr isolated`; subprocess runs with a temporary HOME containing only the provided credentials file; the caller's real HOME, settings, and conversation history are invisible to the subprocess |
| fence stripping | Post-processing of captured stdout by `--strip-fences`; removes the first and last `` ``` `` lines (with optional language tag); content between the fences is emitted unchanged; no-op if no fence pair is found |
| standalone mode | Default subprocess behavior: `CLAUDECODE` env var is removed before spawn so the subprocess behaves as a first-class Claude Code process, not a nested agent; opt out with `--keep-claudecode` |
| nested-agent mode | Subprocess behavior when `CLAUDECODE=1` is inherited from the parent; alters permission handling, output format, and tool availability; active when `--keep-claudecode` is set |
| credential refresh mode | Invocation via `clr refresh`; subprocess runs with `["--print", "."]` in a temporary HOME to trigger OAuth token refresh at startup; no user task is executed; default timeout 45 seconds |

### Architecture

| Term | Definition |
|------|------------|
| Claude-native flag | A flag forwarded to the claude subprocess (e.g., `--model`, `--verbose`) |
| runner-specific flag | A flag consumed by the runner itself, not forwarded to claude (e.g., `--dry-run`, `--quiet`, `--new-session`) |
| session continuation (automatic) | Default behavior: `-c` is passed to the claude subprocess when a prior session exists for the effective working directory and `--new-session` is not given; resumes the most recent conversation |
| ClaudeCommand | Builder pattern from `claude_runner_core` that assembles the subprocess invocation |
| session directory | Filesystem location where Claude Code persists conversation state; `clr` continues the session stored here by default |
| `--` separator | Double-dash token; everything after it becomes positional (part of the message) |
| last-wins | When a flag appears multiple times, the last occurrence takes effect |
| temp HOME | Temporary directory created by `clr isolated` containing only `.claude/.credentials.json`; set as `HOME` for the subprocess; deleted unconditionally on exit regardless of timeout or error |

### Provenance

| File | Notes |
|------|-------|
| [../dictionary.md](../dictionary.md) | Original un-migrated source; retained as reference |

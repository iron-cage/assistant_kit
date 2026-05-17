# Dictionary

### Core Terms

Domain vocabulary for the `clr` CLI. Terms are organized by category below.

### Commands

| Term | Definition |
|------|------------|
| run | Default command that builds and executes a `claude` subprocess with the given flags |
| isolated | Subcommand that runs `claude` in a credential-isolated temporary HOME; requires `--creds` |
| help | Display usage information and exit; triggered by `-h` / `--help` |

### Modes

| Term | Definition |
|------|------------|
| interactive mode | Default TTY passthrough mode; stdin/stdout connected directly to the claude subprocess; continues previous session automatically |
| print mode | Non-interactive capture mode (`-p`/`--print`); stdout collected and printed for programmatic use; continues previous session automatically |
| dry-run | Preview mode (`--dry-run`); prints the assembled command without executing it; output always shows `-c` (automatic continuation) |
| new session | Invocation with `--new-session`; starts a fresh Claude conversation with no prior context (omits the default `-c`) |
| ultrathink suffix | Text `"\n\nultrathink"` appended after every message before it is sent to the claude subprocess; activates Claude's extended thinking mode; default-on, suppressed with `--no-ultrathink` |
| credential-isolated mode | Invocation via `clr isolated`; subprocess runs with a temporary HOME containing only the provided credentials file; the caller's real HOME, settings, and conversation history are invisible to the subprocess |

### Types

| Term | Definition |
|------|------------|
| VerbosityLevel | Runner output gate: 0=silent, 1=errors, 2=warnings, 3=normal, 4=verbose, 5=debug |
| TokenLimit | Maximum output token count as u32 (0–4294967295); default 200000 |
| ModelName | Claude model identifier string (e.g., "sonnet", "opus") |
| DirectoryPath | Filesystem path to a directory |
| MessageText | Free-form prompt text; multiple positional words joined with space |
| CredentialsFilePath | Path to an existing credentials JSON file; resolved against caller's cwd; written back in-place if OAuth token refreshed |
| TimeoutSecs | Non-negative integer seconds to wait for an isolated subprocess; 0 = immediate expiry; default 30 |

### Architecture

| Term | Definition |
|------|------------|
| Claude-native flag | A flag forwarded to the claude subprocess (e.g., `--model`, `--verbose`) |
| runner-specific flag | A flag consumed by the runner itself, not forwarded to claude (e.g., `--dry-run`, `--verbosity`, `--new-session`) |
| session continuation (automatic) | Default behavior: `-c` is always passed to the claude subprocess unless `--new-session` is given; resumes the most recent conversation |
| ClaudeCommand | Builder pattern from `claude_runner_core` that assembles the subprocess invocation |
| session directory | Filesystem location where Claude Code persists conversation state; `clr` continues the session stored here by default |
| `--` separator | Double-dash token; everything after it becomes positional (part of the message) |
| last-wins | When a flag appears multiple times, the last occurrence takes effect |
| temp HOME | Temporary directory created by `clr isolated` containing only `.claude/.credentials.json`; set as `HOME` for the subprocess; deleted unconditionally on exit regardless of timeout or error |

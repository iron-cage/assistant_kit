# session_dir

Override the directory where Claude Code stores session state.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_SESSION_DIR=<path>
```

## Default

Auto-detect (standard: `~/.claude/projects/<encoded-cwd>/`)

## Description

Overrides the directory where Claude Code stores session files (conversation history, state). By default, sessions are stored under `~/.claude/projects/` using the current working directory as a key.

With this override, all session I/O goes to the specified directory instead.

Use cases:
- Storing sessions in a specific location (e.g., project-local `.sessions/` dir)
- Redirecting session state to a tmpfs or RAM disk for performance
- Isolating sessions by task in automation pipelines
- Custom session organization per project
- Testing with a clean, controlled session directory

Note: This is the `claude_runner_core` environment variable for overriding session storage. The `SessionManager` type (which manages the `-topic` directory structure for local invocations) uses `CLAUDE_CODE_SESSION_DIR` to specify its root.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

// Default: auto-detect
let cmd = ClaudeCommand::new();

// Override to custom directory
let cmd = ClaudeCommand::new()
  .with_session_dir( "/tmp/my-sessions" );
```

Builder method: `with_session_dir(dir: impl Into<PathBuf>)` — sets `CLAUDE_CODE_SESSION_DIR`.

## Examples

```bash
# Store sessions in project directory
CLAUDE_CODE_SESSION_DIR="./.sessions" claude --print "Work on this project"

# Isolated pipeline session
CLAUDE_CODE_SESSION_DIR="/tmp/pipeline-sessions/task-$(date +%s)" \
  claude --print "Process this task"

# RAM disk for performance
CLAUDE_CODE_SESSION_DIR="/dev/shm/claude-sessions" claude --print "High-perf session"
```

## Notes

- The directory is created if it doesn't exist
- Sessions stored here can still be resumed via `--resume` if the ID is known
- For per-invocation isolation, combine with `--no-session-persistence` instead

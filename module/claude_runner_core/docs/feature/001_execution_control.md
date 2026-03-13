# Feature: Execution Control

### Scope

- **Purpose**: Document the interactive vs. non-interactive execution mode design of claude_runner_core.
- **Responsibility**: Describe when each execution mode is used, what each method returns, and how the responsibility boundary with claude_profile is maintained.
- **In Scope**: execute() non-interactive mode, execute_interactive() TTY mode, ExecutionOutput, responsibility separation from claude_profile.
- **Out of Scope**: Dry-run mode (→ `feature/002_dry_run.md`), builder API details (→ `api/001_execution_api.md`), describe output (→ `feature/003_describe.md`).

### Design

claude_runner_core provides two execution modes for different use cases:

**Non-interactive mode (`execute()`):** Spawns Claude Code with stdout and stderr piped. The process runs silently and its output is returned as `ExecutionOutput { stdout, stderr, exit_code }`. Used for programmatic automation where output must be parsed or logged.

**Interactive mode (`execute_interactive()`):** Spawns Claude Code with the terminal attached (TTY). Claude Code takes over the user's terminal session. No output is captured — it flows directly to the user. Returns an `ExitStatus` when the session ends. Used when a human is present and needs to interact with Claude.

The choice of mode is caller-controlled: callers select `execute()` or `execute_interactive()` based on their context, not a flag on the builder.

**Responsibility boundary:** claude_runner_core owns process execution exclusively. Session storage path resolution (where sessions are stored on disk) belongs to claude_profile. The caller (dream_agent) composes both:

```
dream_agent → claude_profile.resolve_storage_path() → passes path to
dream_agent → ClaudeCommand::new().with_session_dir(path).execute()
```

claude_runner_core never calls into claude_profile. It receives resolved paths as plain `PathBuf` values via `with_*()` methods.

**Single execution surface:** All `Command::new("claude")` calls in the workspace are centralized in `command.rs`. Other crates use the builder and call `execute()` or `execute_interactive()` — they never spawn the `claude` binary directly. The `claude_version()` function is the second permitted location for `Command::new("claude")`.

**Tier-1 automation defaults:** `ClaudeCommand::new()` sets defaults appropriate for automation, not the Claude CLI's own defaults. This prevents common automation failures (token limit exceeded, telemetry noise, premature timeout).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| pattern | [pattern/001_command_builder.md](../pattern/001_command_builder.md) | Fluent builder pattern used to configure execution |
| api | [api/001_execution_api.md](../api/001_execution_api.md) | execute() and execute_interactive() method contracts |
| invariant | [invariant/001_single_execution_point.md](../invariant/001_single_execution_point.md) | Rule: all Command::new("claude") in one place |
| feature | [feature/002_dry_run.md](002_dry_run.md) | Dry-run mode that intercepts execute() |
| source | `../../src/command.rs` | execute() and execute_interactive() implementation |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | FR-6, FR-9, Design Principles (Single Responsibility), System Architecture (Execution Flow, Responsibility Boundary), Vocabulary (Interactive Mode, Non-Interactive Mode) |

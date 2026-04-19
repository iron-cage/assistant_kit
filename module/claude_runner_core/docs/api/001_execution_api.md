# API: Execution API

### Scope

- **Purpose**: Document the programmatic contracts for executing Claude Code processes via claude_runner_core.
- **Responsibility**: Specify execute(), execute_interactive(), ExecutionOutput, error handling, and method parameter contracts.
- **In Scope**: execute() non-interactive execution, execute_interactive() TTY mode, ExecutionOutput fields, error contracts, dry-run behavior.
- **Out of Scope**: Builder pattern design (→ `pattern/`), enum type definitions (→ `data_structure/`), describe/dry-run feature semantics (→ `feature/`).

### Abstract

`ClaudeCommand` exposes two execution methods: `execute()` for captured non-interactive output and `execute_interactive()` for TTY-attached interactive sessions. Both methods consume the builder (move semantics) and return structured results via `error_tools::Result`.

### Operations

#### `execute() -> error_tools::Result<ExecutionOutput>`

Spawns the Claude Code process with stdout and stderr piped. Waits for process completion and returns structured output.

When `with_dry_run(true)` is set, `execute()` returns an `ExecutionOutput` with:
- `stdout`: the output of `describe_compact()`
- `stderr`: empty string
- `exit_code`: 0

No process is spawned in dry-run mode.

**`ExecutionOutput` fields:**

| Field | Type | Description |
|-------|------|-------------|
| `stdout` | `String` | Full captured standard output from the process |
| `stderr` | `String` | Full captured standard error from the process |
| `exit_code` | `i32` | Process exit code (0 = success) |

#### `execute_interactive() -> error_tools::Result<std::process::ExitStatus>`

Spawns the Claude Code process with the terminal attached (TTY mode). Claude Code takes over the terminal for interactive use. Output is not captured — it flows directly to the user's terminal.

When `with_dry_run(true)` is set, `execute_interactive()` returns an exit status of 0 without spawning any process.

#### `claude_version() -> Option<String>`

Runs `claude --version` and returns the trimmed stdout string. Returns `None` if the binary is not found in PATH or produces no output. This is the canonical way to query the installed Claude Code version — other crates must not spawn `claude` directly for version checks.

#### `build_command_for_test() -> std::process::Command`

Exposes the constructed `std::process::Command` for assertion in tests. Not for production use. This method and `claude_version()` are the only two locations permitted to call `Command::new("claude")` in the workspace.

### Error Handling

All errors use `error_tools::Error` (not `anyhow` or `thiserror` directly). Common failure cases:

| Error Condition | Behavior |
|-----------------|----------|
| `claude` binary not in PATH | `execute()` returns `Err` with descriptive message |
| Permission denied on spawn | `execute()` returns `Err` with OS error details |
| Non-zero exit code | Returns `Ok(ExecutionOutput)` with `exit_code != 0` — caller checks the code |
| Working directory does not exist | `execute()` returns `Err` at spawn time |

Non-zero exit codes are not converted to errors — callers receive `ExecutionOutput` and decide how to interpret the exit code.

### Compatibility Guarantees

- `ExecutionOutput` fields (`stdout`, `stderr`, `exit_code`) are stable across patch versions.
- The `execute()` / `execute_interactive()` signatures are stable across minor versions.
- No compatibility is guaranteed for internal methods (`build_command()`, `build_command_for_test()`).
- The deprecated factory method (`ClaudeCommand::generate()`) does not exist in this crate and will not be re-introduced.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [pattern/001_command_builder.md](../pattern/001_command_builder.md) | Fluent builder pattern for constructing the command |
| doc | [feature/001_execution_control.md](../feature/001_execution_control.md) | Interactive vs non-interactive execution mode design |
| doc | [feature/002_dry_run.md](../feature/002_dry_run.md) | Dry-run mode semantics and describe_compact() output |
| doc | [invariant/001_single_execution_point.md](../invariant/001_single_execution_point.md) | All Command::new("claude") calls centralized here |
| source | `../../src/command.rs` | execute(), execute_interactive(), build_command() implementation |
| source | `../../src/types.rs` | ExecutionOutput struct definition |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | FR-7 through FR-9, FR-23 through FR-24, NFR-5, Component Roles (ClaudeCommand), System Architecture (Execution Flow) |

# API: Execution API

### Scope

- **Purpose**: Document the programmatic contracts for executing Claude Code processes via claude_runner_core.
- **Responsibility**: Specify execute(), execute_interactive(), ExecutionOutput, read_subprocess_model_pref(), error handling, and method parameter contracts.
- **In Scope**: execute() non-interactive execution, execute_interactive() TTY mode, ExecutionOutput fields, read_subprocess_model_pref() preference reader, error contracts, dry-run behavior.
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

No process is spawned in dry-run mode. No stdin file is opened in dry-run mode.

**Stdin file (`with_stdin_file`):**

When a `stdin_file` path is set, `execute()` opens the file for reading and attaches it as the subprocess's standard input before spawning. If the file cannot be opened, `execute()` returns `Err(...)` with a message including the path and OS error. See [feature/005_stdin_file.md](../feature/005_stdin_file.md).

**CLAUDECODE removal (`with_unset_claudecode`):**

Before spawning, `build_command()` calls `.env_remove("CLAUDECODE")` when `unset_claudecode` is `true` (the default). This prevents the subprocess from detecting a parent Claude Code session. See [feature/006_unset_claudecode.md](../feature/006_unset_claudecode.md).

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

#### `read_subprocess_model_pref() -> Option<String>`

Reads the `subprocess_model` field from `~/.clr/prefs.json`. Returns `Some(model_id)` when the
file exists, is valid JSON, and the key is a non-empty string. Returns `None` when the file is
absent, unreadable, malformed, or the key is missing or empty.

Re-exported from `claude_runner_core` (defined in `src/isolated.rs`). Gated by
`#[cfg(feature = "enabled")]`. Used by `dispatch_run()` in `claude_runner` to apply a pinned
model preference when no `--model` flag or `CLR_MODEL` env var is set (set via `clp .model.select`).

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
| doc | [feature/005_stdin_file.md](../feature/005_stdin_file.md) | stdin_file field — file opened and attached at execute() time |
| doc | [feature/006_unset_claudecode.md](../feature/006_unset_claudecode.md) | unset_claudecode field — CLAUDECODE removal before spawn |
| doc | [invariant/001_single_execution_point.md](../invariant/001_single_execution_point.md) | All Command::new("claude") calls centralized here |
| source | `../../src/command.rs` | execute(), execute_interactive(), build_command() implementation |
| source | `../../src/types.rs` | ExecutionOutput struct definition |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | FR-7 through FR-9, FR-23 through FR-24, NFR-5, Component Roles (ClaudeCommand), System Architecture (Execution Flow) |

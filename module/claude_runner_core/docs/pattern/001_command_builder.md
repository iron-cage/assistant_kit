# Pattern: Command Builder

### Scope

- **Purpose**: Document the fluent builder pattern used for constructing Claude Code commands.
- **Responsibility**: Describe the problem, solution, applicability, and consequences of the builder API design in claude_runner_core.
- **In Scope**: ClaudeCommand::new() entry point, with_*() method chaining, move semantics, execute() as terminal operation.
- **Out of Scope**: Execution API contracts (→ `api/`), type-safe enum definitions (→ `data_structure/`), dry-run and describe features (→ `feature/`).

### Problem

Claude Code requires up to 40+ configuration parameters for process execution. Without a structured API:

- Scattered `Command::new("claude")` calls spread across the codebase (measured: 2x duplication)
- Default 32K token limit causes "exceeded maximum" errors in automation
- No compile-time parameter validation — raw string arguments prone to typos
- Mixed responsibilities: the old claude_profile crate handled both storage paths and execution

Direct struct construction with 40+ fields is fragile, hard to read, and difficult to extend.

### Solution

`ClaudeCommand` implements the fluent builder pattern:

```
ClaudeCommand::new()           // entry point, sets tier-1 automation defaults
  .with_working_directory(dir) // configuration via with_*() methods
  .with_max_output_tokens(200_000)
  .with_continue_conversation(true)
  .execute()                   // terminal operation returning Result<ExecutionOutput>
```

Each `with_*()` method takes `self` by move and returns `Self`, enabling method chaining without `&mut self`. The `execute()` call consumes the builder and spawns the process.

`ClaudeCommand::new()` sets automation-safe tier-1 defaults:
- `max_output_tokens`: 200,000 (prevents "exceeded maximum" errors)
- `auto_continue`: true (enables programmatic automation)
- `telemetry`: false (disables telemetry in automation)
- `bash_timeout_ms`: 3,600,000ms (1 hour)

### Applicability

This pattern applies when:
- A process requires many optional configuration parameters
- Caller code must be readable and self-documenting
- Defaults must be safe for automation contexts (not the binary's own defaults)
- The construction surface must be extensible without breaking callers

This pattern does not apply when the command is simple (1–2 parameters) or when build-time validation requires type state machines.

### Consequences

**Benefits:**
- Single `Command::new("claude")` location — all execution centralized in `command.rs`
- Callers chain only the parameters they need; unused parameters take safe defaults
- New parameters added as `with_*()` methods without breaking existing callers
- Deprecated factory method (`generate()`) is removed entirely — old API cannot compile

**Costs:**
- Move semantics require callers to own the builder (not pass `&ClaudeCommand` between functions)
- No compile-time validation of required parameters; `execute()` can fail at runtime if configuration is invalid
- 40+ methods creates a large API surface to maintain

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [api/001_execution_api.md](../api/001_execution_api.md) | execute() and execute_interactive() contracts |
| doc | [data_structure/001_command_types.md](../data_structure/001_command_types.md) | Type-safe enum parameters accepted by with_*() methods |
| doc | [feature/001_execution_control.md](../feature/001_execution_control.md) | Interactive vs non-interactive execution modes |
| source | `../../src/command.rs` | ClaudeCommand builder implementation |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Design Principles, Builder Pattern section, FR-1 through FR-5, User Stories US-1 through US-3 |

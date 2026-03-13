# Invariant: Single Execution Point

### Scope

- **Purpose**: Document the constraint that all Claude Code process spawning must be centralized in claude_runner_core.
- **Responsibility**: State the invariant, explain its enforcement mechanism, and document the consequences of violation.
- **In Scope**: Command::new("claude") centralization rule, permitted locations, enforcement tests, violation consequences.
- **Out of Scope**: NFR constraints (→ `invariant/002_nfr_conformance.md`), execution API contracts (→ `api/001_execution_api.md`).

### Invariant Statement

All `Command::new("claude")` calls in the workspace must reside in `claude_runner_core`. No other crate may spawn the `claude` binary directly.

The two permitted locations are:
1. `src/command.rs` — `build_command()` method, used by `execute()` and `execute_interactive()`
2. `src/command.rs` — `claude_version()` function, for version queries only

No third location is permitted, regardless of crate, module, or justification.

### Enforcement Mechanism

**Compile-time:** The old factory method (`ClaudeCommand::generate()`) does not exist in this crate. Any attempt to call it produces a compile error. The deprecated `execute_non_interactive()` method is also absent.

**Test-time:** `tests/responsibility_single_execution_point_test.rs` verifies that:
- All `Command::new("claude")` occurrences in the workspace are in `claude_runner_core`
- dream_agent contains zero direct `Command::new("claude")` calls
- claude_profile contains zero process execution code

**Workspace enforcement:** Companion tests in other crates verify they do not violate this invariant:
- `dream_agent/tests/responsibility_builder_pattern_usage_test.rs`
- `claude_profile/tests/responsibility_no_process_execution_test.rs`

### Violation Consequences

If a crate outside claude_runner_core spawns `claude` directly:
- The single execution point is broken — token limit defaults revert to Claude CLI's 32K
- Automation defaults (bash timeout, telemetry, auto-continue) are lost
- Error handling bypasses `error_tools`, producing inconsistent errors
- The workspace-level tests detecting this violation will fail

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| feature | [feature/001_execution_control.md](../feature/001_execution_control.md) | Execution mode design that centralizes spawning here |
| api | [api/001_execution_api.md](../api/001_execution_api.md) | execute() and execute_interactive() as the only permitted callers |
| source | `../../src/command.rs` | The single permitted location for Command::new("claude") |
| test | `../../tests/responsibility_single_execution_point_test.rs` | Automated enforcement test |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | FR-6, Design Principles (Single Responsibility: Execution ONLY), System Architecture (Responsibility Boundary, Responsibility Rules), EXEC-1 through EXEC-4 conformance checklist |

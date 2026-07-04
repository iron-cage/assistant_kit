# Invariant: No Process Execution

### Scope

- **Purpose**: Keep `claude_profile` a pure credential-management library with no side-effects that could interfere with callers.
- **Responsibility**: Documents the zero-process-execution constraint (NFR-5).
- **In Scope**: `std::process::Command` and any process spawning — forbidden anywhere in the library.
- **Out of Scope**: The `claude_runner_core` crate which is explicitly responsible for all process execution.

### Invariant Statement

`claude_profile` MUST NOT execute any processes. `std::process::Command` import is a responsibility violation.

**Measurable threshold:** Zero occurrences of `std::process` in `src/` detected by automated test.

**Delegation principle:** All execution is delegated to `claude_runner_core`. Specifically forbidden in `claude_profile`:
- Browser launch (`xdg-open`, `claude auth login`)
- OAuth HTTP token refresh (requires network + process)
- Pulse-keeping invocations (periodic `claude` process)
- Any subprocess spawning for any reason

### Enforcement Mechanism

- Automated test: `tests/responsibility_no_process_execution_test.rs` greps `src/` for `std::process` and fails if found
- Code review: immediate rejection of any PR importing `std::process`
- Architecture: `claude_runner_core` owns the `ClaudeCommand` builder; `claude_profile` does not reference it directly — token refresh is coordinated through `claude_profile_core::account::refresh_account_token()`, which delegates subprocess spawning to `claude_runner_core::run_isolated()`

### Violation Consequences

- Process execution from within credential management creates unexpected side-effects for callers
- Breaks the single-responsibility boundary — callers can no longer reason about what `account::save()` does
- Introduces platform dependencies (processes behave differently across OS)
- Creates impossible-to-test scenarios in unit tests (process spawning requires real environment)

### Sources

| File | Relationship |
|------|-------------|
| `src/` | Entire source tree — must have zero `std::process` occurrences |

### Tests

| File | Relationship |
|------|-------------|
| `tests/responsibility_no_process_execution_test.rs` | Grep audit that fails CI if `std::process` appears in src/ |

### Invariants

| File | Relationship |
|------|-------------|
| [001_zero_third_party_deps.md](001_zero_third_party_deps.md) | Related boundary: zero crates.io deps in library path |
| [008_single_token_refresh_entry.md](008_single_token_refresh_entry.md) | Related boundary: single entry point for token refresh — `refresh_account_token()` delegates to `run_isolated()` in `claude_runner_core` |

# Pitfall: Vacuous Assertions Mask Stub Implementations

<!-- BUG-002 — new pitfall generalizing the hardcoded-stub-concealed-by-vacuous-tests defect pattern -->

### Scope

- **Purpose**: Document the vacuous-assertion pitfall that let a hardcoded stub ship as a completed command.
- **Responsibility**: Why a test suite must assert on real output content, not just process liveness.
- **In Scope**: Assertions that check only process exit/liveness instead of stdout/stderr content; commit messages presenting placeholder code as complete.
- **Out of Scope**: Parameter validation itself (→ `01_parameter_validation.md`), cross-command propagation (→ `02_cross_command_propagation.md`), test data format (→ `03_test_data_format.md`).

### Pitfall

A command routine can read its parameters into discarded bindings and return a hardcoded literal, while its companion tests assert only `Command::output().is_ok()` or discard the exit code — proving the process didn't crash, never that it produced correct output. Both the implementation and its tests compile, run, and pass CI indistinguishably from a correct implementation. A prior regression shipped exactly this pattern in `.tail`: three tests all passed against a routine that ignored every parameter and always printed the same three hardcoded lines, regardless of the real session content or requested entry count.

### Trigger

Writing (or reviewing) a test for a CLI command that:

- Asserts `result.is_ok()` on a `Command::output()` result and stops there
- Reads `out.status.code()` or `stdout` and discards it (`let _ = ...`)
- Contains a comment admitting the gap (e.g. "just confirms the process didn't hang")

### Required Pattern

Every integration test for a command routine MUST assert on the actual content of stdout (or stderr, for error paths) against a known expected value — never merely that the process exited or didn't hang:

1. Capture `Command::output()` and convert stdout to a `String`.
2. Assert the string equals (or contains, when appropriate) the exact expected content derived from a known fixture — not just `.is_ok()` on the `Result` or a discarded exit code.
3. For non-error paths, also assert the exit code equals the documented success code (usually `0`).
4. If a commit message or PR claims a command "works" or "is implemented", the described behavior must be traceable to a passing assertion on real content — not merely "the binary ran."

This is a mechanical review step, not optional: any test whose only assertion is `is_ok()`/`is_err()` on a process-spawn `Result`, with no check of stdout/stderr content, is presumptively vacuous and must be rewritten.

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| `.tail` | `.tail` | Stub returned hardcoded output; companion tests asserted only process liveness |

### Sources

- `src/cli/tail.rs:32` — real `tail_routine`, replacing the prior hardcoded stub
- `tests/cli_cmd_tail_test.rs` — real assertions replacing the vacuous liveness-only checks

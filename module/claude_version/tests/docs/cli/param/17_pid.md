# Test: `pid::`

Edge case coverage for the `pid::` parameter. See [param/readme.md](../../../../docs/cli/param/readme.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `pid::` parameter.
- **Responsibility**: Boundary values, invalid inputs, type violations, non-claude-process rejection, and default (absent) behavior for `pid::`.
- **Commands:** `.ps.kill`
- **In Scope**: Single-parameter edge cases, PID validation, targeted vs. bulk mode switching.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `pid::` absent → bulk kill mode (all processes) | Default Behavior |
| EC-2 | `pid::PID` with valid claude PID → targeted kill, exit 0 | Targeted Kill |
| EC-3 | `pid::PID` not a claude process → exit 1 | Validation |
| EC-4 | `pid::99999999` nonexistent PID → exit 1 | Validation |
| EC-5 | `pid::abc` → exit 1, non-integer | Format Violation |
| EC-6 | `pid::` (empty) → exit 1 | Empty Value |
| EC-7 | `pid::0` → exit 1, invalid PID range | Invalid Value |
| EC-8 | `pid::PID dry::1` → preview only, no kill executed | Interaction |

## Test Coverage Summary

- Default Behavior: 1 test
- Targeted Kill: 1 test
- Validation: 2 tests
- Format Violation: 1 test
- Empty Value: 1 test
- Invalid Value: 1 test
- Interaction: 1 test

**Total:** 8 edge cases

---

### EC-1: `pid::` absent → bulk mode

- **Given:** clean environment
- **When:** `clv .ps.kill dry::1`
- **Then:** exit 0; output references all detected processes (or "no active processes"); bulk mode is the default
- **Exit:** 0
- **Source:** [param/readme.md — pid:: default: absent](../../../../docs/cli/param/readme.md)

---

### EC-2: Valid claude PID → targeted kill

- **Given:** a running claude process with PID P
- **When:** `clv .ps.kill pid::P dry::1`
- **Then:** exit 0; output references only PID P; no other processes mentioned
- **Exit:** 0
- **Source:** [command/ps.md#command-8-pskill](../../../../docs/cli/command/ps.md)

---

### EC-3: Non-claude PID → exit 1

- **Given:** a PID that belongs to a non-claude process (e.g., PID 1)
- **When:** `clv .ps.kill pid::1`
- **Then:** exit 1; stderr references that PID 1 is not a claude process
- **Exit:** 1
- **Source:** [command/ps.md#command-8-pskill](../../../../docs/cli/command/ps.md)

---

### EC-4: Nonexistent PID → exit 1

- **Given:** clean environment
- **When:** `clv .ps.kill pid::99999999`
- **Then:** exit 1; stderr references that the PID was not found
- **Exit:** 1
- **Source:** [command/ps.md#command-8-pskill](../../../../docs/cli/command/ps.md)

---

### EC-5: `pid::abc` → exit 1

- **Given:** clean environment
- **When:** `clv .ps.kill pid::abc`
- **Then:** exit 1; error references non-integer PID value
- **Exit:** 1
- **Source:** [param/readme.md — pid:: type: u64](../../../../docs/cli/param/readme.md)

---

### EC-6: `pid::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .ps.kill pid::`
- **Then:** exit 1; error references empty pid:: value
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-7: `pid::0` → exit 1

- **Given:** clean environment
- **When:** `clv .ps.kill pid::0`
- **Then:** exit 1; error references out-of-range PID (0 is not a valid process PID)
- **Exit:** 1
- **Source:** [param/readme.md — pid:: type: u64](../../../../docs/cli/param/readme.md)

---

### EC-8: `pid::PID dry::1` → preview only

- **Given:** clean environment
- **When:** `clv .ps.kill pid::1 dry::1`
- **Then:** exit 0 (dry mode skips PID validation) or 1 (invalid PID); if valid, dry-run message shown with no actual kill
- **Exit:** 0 (dry mode) or 1 (invalid PID)
- **Source:** [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md)

---

### Source Functions

| Function | File |
|----------|------|
| `pid_ec1_absent_bulk_mode` | `tests/cli/pid_param_test.rs` |
| `pid_ec2_valid_pid_targeted` | `tests/cli/pid_param_test.rs` |
| `pid_ec3_non_claude_pid_exits_1` | `tests/cli/pid_param_test.rs` |
| `pid_ec4_nonexistent_pid_exits_1` | `tests/cli/pid_param_test.rs` |
| `pid_ec5_abc_exits_1` | `tests/cli/pid_param_test.rs` |
| `pid_ec6_empty_exits_1` | `tests/cli/pid_param_test.rs` |
| `pid_ec7_zero_exits_1` | `tests/cli/pid_param_test.rs` |
| `pid_ec8_dry_preview` | `tests/cli/pid_param_test.rs` |

# Test: `force::`

Edge case coverage for the `force::` parameter. See [005_params.md](../../../../docs/cli/005_params.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `force::` parameter.
- **Responsibility**: Boundary values, invalid inputs, type violations, and default behavior for `force::`.
- **In Scope**: Single-parameter edge cases, validation errors, type checking.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `dry::1 force::1` → dry wins, no install | Interaction (dry wins) |
| TC-312 | `dry::1 force::1` on `.processes.kill` → dry wins | Interaction (dry wins) |
| TC-406 | `.version.guard force::1 dry::1` → dry wins | Interaction (dry wins) |
| EC-2 | `force::1` on `.version.guard` → reinstalls despite match | Explicit True |
| EC-1 | Default (absent) → `force::0` (guard active) | Default Behavior |
| EC-2 | `force::0` explicit → same as absent | Explicit False |
| EC-3 | `force::2` → exit 1, out of range | Invalid Value |
| EC-4 | `force::-1` → exit 1, out of range | Invalid Value |
| EC-5 | `force::abc` → exit 1, non-integer | Format Violation |
| EC-6 | `force::` (empty) → exit 1 | Empty Value |
| EC-7 | `force::` only for `.version.install`, `.version.guard`, `.processes.kill` | Command Scope |

## Test Coverage Summary

- Interaction (dry wins): 3 tests
- Explicit True: 1 test
- Default Behavior: 1 test
- Explicit False: 1 test
- Invalid Value: 2 tests
- Format Violation: 1 test
- Empty Value: 1 test
- Command Scope: 1 test

**Total:** 12 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: `dry::1 force::1` → dry wins

- **Given:** clean environment
- **When:** `cm .version.install dry::1 force::1`
- **Then:** `[dry-run]` prefix; no install.; preview only
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — dry+force](../../../../docs/cli/004_parameter_interactions.md)

---

### EC-2: `force::1` bypasses match check

- **Given:** Installed version matches `preferredVersionResolved`.
- **When:** `cm .version.guard force::1`
- **Then:** Install proceeds; no "matches" skip message.; reinstall occurs
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### EC-3: `force::2` → exit 1

- **Given:** clean environment
- **When:** `cm .version.install force::2`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [005_params.md — force:: type: Boolean (0/1)](../../../../docs/cli/005_params.md)

---

### EC-4: `force::-1` → exit 1

- **Given:** clean environment
- **When:** `cm .version.install force::-1`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [005_params.md — force:: type: Boolean (0/1)](../../../../docs/cli/005_params.md)

---

### EC-5: `force::abc` → exit 1

- **Given:** clean environment
- **When:** `cm .version.install force::abc`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [005_params.md — force:: type: Boolean (0/1)](../../../../docs/cli/005_params.md)

---

### EC-6: `force::` (empty) → exit 1

- **Given:** clean environment
- **When:** `cm .version.install force::`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-7: `force::` only for its declared commands

- **Given:** clean environment
- **When:** `cm .settings.set key::k value::v force::1`
- **Then:** exit code 1; unknown parameter.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc021_force_param` | `cli_args_test.rs` |
| `tc035_force_true_rejected` | `cli_args_test.rs` |
| `tc037_force_0_accepted` | `cli_args_test.rs` |
| `tc250_version_install_dry_force_dry_wins` | `integration/cross_cutting_test.rs` |
| `tc251_processes_kill_dry_force_dry_wins` | `integration/cross_cutting_test.rs` |
| `tc303_version_install_dry_wins_over_force` | `integration/mutation_commands_test.rs` |
| `tc406_guard_dry_force_no_install` | `integration/mutation_commands_test.rs` |

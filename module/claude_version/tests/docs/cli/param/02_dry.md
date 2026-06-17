# Test: `dry::`

Edge case coverage for the `dry::` parameter. See [param/readme.md](../../../../docs/cli/param/readme.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `dry::` parameter.
- **Responsibility**: Boundary values, invalid inputs, type violations, and default behavior for `dry::`.
- **Commands:** `.version.install`, `.version.guard`, `.processes.kill`, `.settings.set`
- **In Scope**: Single-parameter edge cases, validation errors, type checking.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `dry::1` → `[dry-run]` prefix on `.version.install` | Explicit True |
| EC-11 | `dry::1` on `.processes.kill` → no kill | Explicit True |
| EC-12 | `dry::1` on `.settings.set` → no file change | Explicit True |
| EC-2 | `dry::1` wins over `force::1` | Interaction |
| EC-13 | `dry::1 force::1` on `.processes.kill` → dry wins | Interaction |
| EC-3 | `dry::1` does NOT write preference keys | Side-Effect Guard |
| EC-4 | Default (absent) resolves to `dry::0` (real action) | Default Behavior |
| EC-5 | `dry::0` explicit → same as absent | Explicit False |
| EC-6 | `dry::2` → exit 1, out of range | Invalid Value |
| EC-7 | `dry::-1` → exit 1, out of range | Invalid Value |
| EC-8 | `dry::abc` → exit 1, non-integer | Format Violation |
| EC-9 | `dry::` (empty) → exit 1 | Empty Value |
| EC-10 | `dry::` only accepted by mutation commands | Command Scope |

## Test Coverage Summary

- Explicit True: 3 tests
- Interaction (dry wins over force): 2 tests
- Side-Effect Guard: 1 test
- Default Behavior: 1 test
- Explicit False: 1 test
- Invalid Value: 2 tests
- Format Violation: 1 test
- Empty Value: 1 test
- Command Scope: 1 test

**Total:** 13 edge cases

**Behavioral Divergence Pair:** EC-1 (`dry::1` → `[dry-run]` prefix, exit 0) ↔ EC-4 (absent → `dry::0`, real action proceeds, exit 0)

---

### EC-1: `dry::1` → `[dry-run]` prefix

- **Given:** clean environment
- **When:** `clv .version.install dry::1`
- **Then:** output contains `[dry-run]`; exit code 0.; dry-run marker present
- **Exit:** 0
- **Source:** [feature/004_dry_run.md](../../../../docs/feature/004_dry_run.md)

---

### EC-2: `dry::1` wins over `force::1`

- **Given:** clean environment
- **When:** `clv .version.install dry::1 force::1`
- **Then:** output contains `[dry-run]`; no install.; preview mode only
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — dry+force precedence](../../../../docs/cli/004_parameter_interactions.md)

---

### EC-3: `dry::1` does NOT write preference keys

- **Given:** `HOME=<tmp>`; settings file empty.
- **When:** `clv .version.install dry::1 version::stable`
- **Then:** `settings.json` has no `preferredVersionSpec` key after command.; no settings written
- **Exit:** 0
- **Source:** [feature/004_dry_run.md](../../../../docs/feature/004_dry_run.md)

---

### EC-4: Default (absent) → `dry::0`

- **Given:** clean environment
- **When:** `clv .version.install` (no `dry::` parameter present)
- **Then:** install proceeds as real action; no `[dry-run]` prefix; behavior identical to explicit `dry::0`
- **Exit:** 0
- **Source:** [param/readme.md — dry:: default: 0](../../../../docs/cli/param/readme.md)

---

### EC-5: `dry::0` explicit → same as absent

- **Given:** Preferred version installed; version matches.
- **When:** `clv .version.guard dry::0`
- **Then:** Behavior identical to `clv .version.guard`; no `[dry-run]` prefix.; explicit zero equals absent
- **Exit:** 0
- **Source:** [param/readme.md — dry:: default: 0](../../../../docs/cli/param/readme.md)

---

### EC-6: `dry::2` → exit 1

- **Given:** clean environment
- **When:** `clv .version.install dry::2`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [param/readme.md — dry:: type: Boolean (0/1)](../../../../docs/cli/param/readme.md)

---

### EC-7: `dry::-1` → exit 1

- **Given:** clean environment
- **When:** `clv .version.install dry::-1`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [param/readme.md — dry:: type: Boolean (0/1)](../../../../docs/cli/param/readme.md)

---

### EC-8: `dry::abc` → exit 1

- **Given:** clean environment
- **When:** `clv .version.install dry::abc`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [param/readme.md — dry:: type: Boolean (0/1)](../../../../docs/cli/param/readme.md)

---

### EC-9: `dry::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .version.install dry::`
- **Then:** exit code 1; error about dry:: requiring a value.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-10: `dry::` only for mutation commands

- **Given:** clean environment
- **When:** `clv .version.list dry::1`
- **Then:** exit code 1; "unknown parameter" or similar.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-11: `dry::1` on `.processes.kill` → no kill

- **Given:** clean environment
- **When:** `clv .processes.kill dry::1`
- **Then:** exit 0; output contains `[dry-run]`; no process actually killed
- **Exit:** 0
- **Source:** [feature/004_dry_run.md](../../../../docs/feature/004_dry_run.md)

---

### EC-12: `dry::1` on `.settings.set` → no file change

- **Given:** `HOME=<tmp>`; settings absent
- **When:** `clv .settings.set key::theme value::dark dry::1`
- **Then:** exit 0; output contains `[dry-run]`; settings.json not created or modified
- **Exit:** 0
- **Source:** [feature/004_dry_run.md](../../../../docs/feature/004_dry_run.md)

---

### EC-13: `dry::1 force::1` on `.processes.kill` → dry wins

- **Given:** clean environment
- **When:** `clv .processes.kill dry::1 force::1`
- **Then:** exit 0; output contains `[dry-run]`; no process killed regardless of force flag
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — dry+force precedence](../../../../docs/cli/004_parameter_interactions.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc020_dry_run_param` | `cli_args_test.rs` |
| `tc033_dry_true_rejected` | `cli_args_test.rs` |
| `tc034_dry_yes_rejected` | `cli_args_test.rs` |
| `tc036_dry_0_accepted` | `cli_args_test.rs` |
| `tc493_dry_0_then_1_last_wins_dry_active` | `cli_args_test.rs` |
| `tc494_dry_1_then_0_last_wins_dry_inactive` | `cli_args_test.rs` |
| `tc300_version_install_dry_shows_prefix` | `integration/mutation_commands_test.rs` |
| `tc301_version_install_dry_stable` | `integration/mutation_commands_test.rs` |
| `tc252_settings_set_dry_no_write` | `integration/cross_cutting_test.rs` |
| `dry_ec6_2_exits_1` | `cli_args_test.rs` |
| `dry_ec7_negative_exits_1` | `cli_args_test.rs` |
| `dry_ec9_empty_exits_1` | `cli_args_test.rs` |
| `dry_ec10_command_scope_rejects_on_read` | `integration/dry_param_test.rs` |
| `dry_ec11_processes_kill_dry_run` | `integration/dry_param_test.rs` |
| `dry_ec12_settings_set_dry_no_file` | `integration/dry_param_test.rs` |
| `dry_ec13_processes_kill_dry_wins_over_force` | `integration/dry_param_test.rs` |

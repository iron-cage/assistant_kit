# Test: `version::`

Edge case coverage for the `version::` parameter. See [005_params.md](../../../../docs/cli/005_params.md) and [006_types.md](../../../../docs/cli/006_types.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `version::` parameter.
- **Responsibility**: Boundary values, invalid inputs, type violations, and default behavior for `version::`.
- **In Scope**: Single-parameter edge cases, validation errors, type checking.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-301 | `version::stable dry::1` → resolves to stable alias | Named Alias |
| TC-309 | `version::month dry::1` → resolves to pinned semver | Named Alias |
| TC-350 | `version::latest dry::1` → no-lock unlock mode | Named Alias (special) |
| TC-302 | `version::1.2.3 dry::1` → exact semver accepted | Valid Semver |
| TC-352 | `version::2.1.50 dry::1` → older semver accepted | Valid Semver |
| TC-355 | `version::0.0.0 dry::1` → zero-patch semver valid | Boundary |
| EC-1 | Absent `version::` → defaults to `stable` | Default Behavior |
| EC-2 | `version::STABLE` → wrong case, exit 1 | Invalid: case |
| EC-3 | `version::` (empty) → exit 1 | Invalid: empty |
| EC-4 | `version::1.2` → two-part semver, exit 1 | Invalid: format |
| TC-307 | `version::x` → unknown alias, exit 1 | Invalid: unknown |
| EC-5 | `version::01.02.03` → leading zeros, exit 1 | Invalid: format |
| EC-1 | `version::1.2.3.4` (four-part) → exit 1 | Invalid: format |
| EC-2 | `version::LATEST` → wrong case, exit 1 | Invalid: case |
| EC-3 | `version::MONTH` → wrong case, exit 1 | Invalid: case |
| EC-4 | `version::` only accepted by `.version.install` and `.version.guard` | Command Scope |

## Test Coverage Summary

- Named Alias: 3 tests (stable, month, latest)
- Valid Semver: 2 tests (1.2.3, 2.1.50)
- Boundary: 1 test (0.0.0)
- Default Behavior: 1 test
- Invalid (case): 3 tests
- Invalid (format): 3 tests
- Invalid (unknown): 1 test
- Command Scope: 1 test

**Total:** 16 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: Absent `version::` → defaults to `stable`

- **Given:** clean environment
- **When:** `cm .version.install dry::1`
- **Then:** output contains "stable".; Correct default applied
- **Exit:** 0
- **Source:** [005_params.md — version:: default: stable](../../../../docs/cli/005_params.md)

---

### EC-2: `version::STABLE` → wrong case

- **Given:** clean environment
- **When:** `cm .version.install version::STABLE`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [006_types.md — VersionSpec case-sensitive](../../../../docs/cli/006_types.md)

---

### EC-3: `version::` (empty) → exit 1

- **Given:** clean environment
- **When:** `cm .version.install version::`
- **Then:** exit code 1; error mentions version.
- **Exit:** 1
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-4: `version::1.2` → two-part semver

- **Given:** clean environment
- **When:** `cm .version.install version::1.2`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [006_types.md — VersionSpec semver format](../../../../docs/cli/006_types.md)

---

### EC-5: `version::01.02.03` → leading zeros

- **Given:** clean environment
- **When:** `cm .version.install version::01.02.03`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [006_types.md — VersionSpec no leading zeros](../../../../docs/cli/006_types.md)

---

### EC-1: `version::1.2.3.4` (four-part) → exit 1

- **Given:** clean environment
- **When:** `cm .version.install version::1.2.3.4`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [006_types.md — VersionSpec format](../../../../docs/cli/006_types.md)

---

### EC-2: `version::LATEST` → wrong case

- **Given:** clean environment
- **When:** `cm .version.install version::LATEST`
- **Then:** exit code 1.
- **Exit:** 1

---

### EC-3: `version::MONTH` → wrong case

- **Given:** clean environment
- **When:** `cm .version.install version::MONTH`
- **Then:** exit code 1.
- **Exit:** 1

---

### EC-4: `version::` only for `.version.install` and `.version.guard`

- **Given:** clean environment
- **When:** `cm .processes version::stable`
- **Then:** exit code 1; "unknown parameter 'version::'" or similar.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc304_version_install_wrong_case_exits_1` | `integration/mutation_commands_test.rs` |
| `tc305_version_install_empty_version_exits_1` | `integration/mutation_commands_test.rs` |
| `tc306_version_install_two_part_semver_exits_1` | `integration/mutation_commands_test.rs` |
| `tc307_version_install_unknown_alias_exits_1` | `integration/mutation_commands_test.rs` |
| `tc308_version_install_absent_version_defaults_to_stable` | `integration/mutation_commands_test.rs` |
| `tc354_version_install_leading_zeros_exits_1` | `integration/mutation_commands_test.rs` |
| `tc355_version_install_zero_parts_valid_dry` | `integration/mutation_commands_test.rs` |
| `tc016_version_param_empty_value` | `cli_args_test.rs` |
| `tc028_four_part_semver_rejected` | `cli_args_test.rs` |
| `tc029_leading_zero_semver_rejected` | `cli_args_test.rs` |

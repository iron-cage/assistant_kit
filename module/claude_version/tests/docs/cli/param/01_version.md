# Test: `version::`

Edge case coverage for the `version::` parameter. See [param/readme.md](../../../../docs/cli/param/readme.md) and [type/readme.md](../../../../docs/cli/type/readme.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `version::` parameter.
- **Responsibility**: Boundary values, invalid inputs, type violations, and default behavior for `version::`.
- **Commands:** `.version.install`, `.version.guard`
- **In Scope**: Single-parameter edge cases, validation errors, type checking.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-10 | `version::stable dry::1` → resolves to stable alias | Named Alias |
| EC-11 | `version::month dry::1` → resolves to pinned semver | Named Alias |
| EC-12 | `version::latest dry::1` → no-lock unlock mode | Named Alias (special) |
| EC-13 | `version::1.2.3 dry::1` → exact semver accepted | Valid Semver |
| EC-14 | `version::2.1.50 dry::1` → older semver accepted | Valid Semver |
| EC-15 | `version::0.0.0 dry::1` → zero-patch semver valid | Boundary |
| EC-1 | Absent `version::` → defaults to `stable` | Default Behavior |
| EC-2 | `version::STABLE` → wrong case, exit 1 | Invalid: case |
| EC-3 | `version::` (empty) → exit 1 | Invalid: empty |
| EC-4 | `version::1.2` → two-part semver, exit 1 | Invalid: format |
| EC-16 | `version::x` → unknown alias, exit 1 | Invalid: unknown |
| EC-5 | `version::01.02.03` → leading zeros, exit 1 | Invalid: format |
| EC-6 | `version::1.2.3.4` (four-part) → exit 1 | Invalid: format |
| EC-7 | `version::LATEST` → wrong case, exit 1 | Invalid: case |
| EC-8 | `version::MONTH` → wrong case, exit 1 | Invalid: case |
| EC-9 | `version::` only accepted by `.version.install` and `.version.guard` | Command Scope |

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

**Behavioral Divergence Pair:** EC-10 (`version::stable dry::1` → output contains "2.1.78", exit 0) ↔ EC-11 (`version::month dry::1` → output contains "2.1.74", exit 0)

---

### EC-1: Absent `version::` → defaults to `stable`

- **Given:** clean environment
- **When:** `clv .version.install dry::1`
- **Then:** output contains "stable".; Correct default applied
- **Exit:** 0
- **Source:** [param/readme.md — version:: default: stable](../../../../docs/cli/param/readme.md)

---

### EC-2: `version::STABLE` → wrong case

- **Given:** clean environment
- **When:** `clv .version.install version::STABLE`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [type/readme.md — VersionSpec case-sensitive](../../../../docs/cli/type/readme.md)

---

### EC-3: `version::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .version.install version::`
- **Then:** exit code 1; error mentions version.
- **Exit:** 1
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-4: `version::1.2` → two-part semver

- **Given:** clean environment
- **When:** `clv .version.install version::1.2`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [type/readme.md — VersionSpec semver format](../../../../docs/cli/type/readme.md)

---

### EC-5: `version::01.02.03` → leading zeros

- **Given:** clean environment
- **When:** `clv .version.install version::01.02.03`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [type/readme.md — VersionSpec no leading zeros](../../../../docs/cli/type/readme.md)

---

### EC-6: `version::1.2.3.4` (four-part) → exit 1

- **Given:** clean environment
- **When:** `clv .version.install version::1.2.3.4`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [type/readme.md — VersionSpec format](../../../../docs/cli/type/readme.md)

---

### EC-7: `version::LATEST` → wrong case

- **Given:** clean environment
- **When:** `clv .version.install version::LATEST`
- **Then:** exit code 1.
- **Exit:** 1

---

### EC-8: `version::MONTH` → wrong case

- **Given:** clean environment
- **When:** `clv .version.install version::MONTH`
- **Then:** exit code 1.
- **Exit:** 1

---

### EC-9: `version::` only for `.version.install` and `.version.guard`

- **Given:** clean environment
- **When:** `clv .processes version::stable`
- **Then:** exit code 1; "unknown parameter 'version::'" or similar.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-10: `version::stable dry::1` → resolves to stable alias

- **Given:** clean environment
- **When:** `clv .version.install version::stable dry::1`
- **Then:** exit 0; output references the `stable` alias or its pinned semver; no actual install
- **Exit:** 0
- **Source:** [param/01_version.md](../../../../docs/cli/param/01_version.md)

---

### EC-11: `version::month dry::1` → resolves to pinned semver

- **Given:** clean environment
- **When:** `clv .version.install version::month dry::1`
- **Then:** exit 0; output references the `month` alias or its pinned semver value; no actual install
- **Exit:** 0
- **Source:** [param/01_version.md](../../../../docs/cli/param/01_version.md)

---

### EC-12: `version::latest dry::1` → no-lock unlock mode

- **Given:** clean environment
- **When:** `clv .version.install version::latest dry::1`
- **Then:** exit 0; output references `latest`; preview shows unlock/no-pin behavior
- **Exit:** 0
- **Source:** [param/01_version.md](../../../../docs/cli/param/01_version.md)

---

### EC-13: `version::1.2.3 dry::1` → exact semver accepted

- **Given:** clean environment
- **When:** `clv .version.install version::1.2.3 dry::1`
- **Then:** exit 0; output contains `1.2.3`; dry-run marker present
- **Exit:** 0
- **Source:** [type/readme.md — VersionSpec semver format](../../../../docs/cli/type/readme.md)

---

### EC-14: `version::2.1.50 dry::1` → older semver accepted

- **Given:** clean environment
- **When:** `clv .version.install version::2.1.50 dry::1`
- **Then:** exit 0; output contains `2.1.50`; dry-run marker present
- **Exit:** 0
- **Source:** [type/readme.md — VersionSpec semver format](../../../../docs/cli/type/readme.md)

---

### EC-15: `version::0.0.0 dry::1` → zero-patch semver valid

- **Given:** clean environment
- **When:** `clv .version.install version::0.0.0 dry::1`
- **Then:** exit 0; `0.0.0` accepted as valid semver; dry-run marker present
- **Exit:** 0
- **Source:** [type/readme.md — VersionSpec boundary](../../../../docs/cli/type/readme.md)

---

### EC-16: `version::x` → unknown alias, exit 1

- **Given:** clean environment
- **When:** `clv .version.install version::x`
- **Then:** exit 1; error references unknown alias or invalid version spec
- **Exit:** 1
- **Source:** [type/readme.md — VersionSpec valid values](../../../../docs/cli/type/readme.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc304_version_install_wrong_case_exits_1` | `tests/cli/mutation_version_install_test.rs` |
| `tc305_version_install_empty_version_exits_1` | `tests/cli/mutation_version_install_test.rs` |
| `tc306_version_install_two_part_semver_exits_1` | `tests/cli/mutation_version_install_test.rs` |
| `tc307_version_install_unknown_alias_exits_1` | `tests/cli/mutation_version_install_test.rs` |
| `tc308_version_install_absent_version_defaults_to_stable` | `tests/cli/mutation_version_install_test.rs` |
| `tc354_version_install_leading_zeros_exits_1` | `tests/cli/mutation_version_install_test.rs` |
| `tc355_version_install_zero_parts_valid_dry` | `tests/cli/mutation_version_install_test.rs` |
| `tc016_version_param_empty_value` | `cli_args_test/param_numeric_test.rs` |
| `tc028_four_part_semver_rejected` | `cli_args_test/param_numeric_test.rs` |
| `tc029_leading_zero_semver_rejected` | `cli_args_test/param_numeric_test.rs` |
| `version_ec7_latest_wrong_case_exits_1` | `tests/cli/version_param_test.rs` |
| `version_ec8_month_wrong_case_exits_1` | `tests/cli/version_param_test.rs` |
| `version_ec9_command_scope_rejects_on_processes` | `tests/cli/version_param_test.rs` |
| `version_ec10_stable_alias_dry` | `tests/cli/version_param_test.rs` |
| `version_ec11_month_alias_dry` | `tests/cli/version_param_test.rs` |
| `version_ec12_latest_alias_dry` | `tests/cli/version_param_test.rs` |
| `version_ec13_exact_semver_dry` | `tests/cli/version_param_test.rs` |
| `version_ec14_older_semver_dry` | `tests/cli/version_param_test.rs` |
| `version_ec15_zero_patch_semver_dry` | `tests/cli/version_param_test.rs` |
| `version_ec16_unknown_alias_exits_1` | `tests/cli/version_param_test.rs` |

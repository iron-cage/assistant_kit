# Test: `VersionSpec`

Type compliance and validation tests for `VersionSpec`. See [type/03_version_spec.md](../../../../docs/cli/type/03_version_spec.md) for specification.

### Scope

- **Purpose**: Validate VersionSpec parsing for named aliases and semver strings.
- **Responsibility**: Named alias resolution, semver format enforcement, leading-zero rejection, and default behavior.
- **Commands:** `.version.install`, `.version.guard`
- **In Scope**: Alias recognition, semver dot-count and digit validation, case-sensitivity.
- **Out of Scope**: Install behavior after version resolution (→ `../command/`), guard behavior (→ `../command/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `version::stable dry::1` → resolves to pinned stable version | Named Alias |
| TC-2 | `version::month dry::1` → resolves to pinned month version | Named Alias |
| TC-3 | `version::latest dry::1` → accepted (dynamic resolution) | Named Alias |
| TC-4 | `version::1.2.3 dry::1` → exact semver accepted | Valid Semver |
| TC-5 | Absent `version::` → defaults to stable | Default |
| TC-6 | `version::STABLE` → exit 1 (case-sensitive) | Validation: case |
| TC-7 | `version::` (empty) → exit 1 | Validation: empty |
| TC-8 | `version::1.2` → exit 1 (two-part semver) | Validation: format |
| TC-9 | `version::1.2.3.4` → exit 1 (four-part semver) | Validation: format |
| TC-10 | `version::01.02.03` → exit 1 (leading zeros) | Validation: format |

## Test Coverage Summary

- Named Alias: 3 tests (TC-1, TC-2, TC-3)
- Valid Semver: 1 test (TC-4)
- Default Behavior: 1 test (TC-5)
- Case sensitivity: 1 test (TC-6)
- Empty value: 1 test (TC-7)
- Format violations: 3 tests (TC-8, TC-9, TC-10)

**Total:** 10 tests

**Behavioral Divergence Pair:** TC-1 (`version::stable dry::1` → output contains "2.1.78") ↔ TC-2 (`version::month dry::1` → output contains "2.1.74")

---

### TC-1: `version::stable dry::1` → stable alias

- **Given:** clean environment
- **When:** `clv .version.install version::stable dry::1`
- **Then:** exit 0; output contains "2.1.78" (stable pinned value); `[dry-run]` prefix present
- **Exit:** 0
- **Source:** [type/03_version_spec.md — Named Aliases: stable](../../../../docs/cli/type/03_version_spec.md)

---

### TC-2: `version::month dry::1` → month alias

- **Given:** clean environment
- **When:** `clv .version.install version::month dry::1`
- **Then:** exit 0; output contains "2.1.74" (month pinned value); `[dry-run]` prefix present
- **Exit:** 0
- **Source:** [type/03_version_spec.md — Named Aliases: month](../../../../docs/cli/type/03_version_spec.md)

---

### TC-3: `version::latest dry::1` → latest alias

- **Given:** clean environment
- **When:** `clv .version.install version::latest dry::1`
- **Then:** exit 0; `[dry-run]` prefix present; no error about unknown version
- **Exit:** 0
- **Source:** [type/03_version_spec.md — Named Aliases: latest](../../../../docs/cli/type/03_version_spec.md)

---

### TC-4: `version::1.2.3 dry::1` → exact semver

- **Given:** clean environment
- **When:** `clv .version.install version::1.2.3 dry::1`
- **Then:** exit 0; output contains "1.2.3"; `[dry-run]` prefix present
- **Exit:** 0
- **Source:** [type/03_version_spec.md — valid semver e.g. 1.2.3](../../../../docs/cli/type/03_version_spec.md)

---

### TC-5: Absent `version::` → defaults to stable

- **Given:** clean environment
- **When:** `clv .version.install dry::1` (no `version::` parameter)
- **Then:** output contains "stable" or "2.1.78" (stable default applied)
- **Exit:** 0
- **Source:** [type/03_version_spec.md — Default: stable](../../../../docs/cli/type/03_version_spec.md)

---

### TC-6: `version::STABLE` → exit 1

- **Given:** clean environment
- **When:** `clv .version.install version::STABLE`
- **Then:** exit code 1; error message references unknown version or expected values
- **Exit:** 1
- **Source:** [type/03_version_spec.md — case-sensitive alias matching](../../../../docs/cli/type/03_version_spec.md)

---

### TC-7: `version::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .version.install version::`
- **Then:** exit code 1; error message references version:: or empty value
- **Exit:** 1
- **Source:** [type/03_version_spec.md — Validation errors](../../../../docs/cli/type/03_version_spec.md)

---

### TC-8: `version::1.2` → exit 1

- **Given:** clean environment
- **When:** `clv .version.install version::1.2`
- **Then:** exit code 1; two-part semver rejected
- **Exit:** 1
- **Source:** [type/03_version_spec.md — semver: dot-count validation](../../../../docs/cli/type/03_version_spec.md)

---

### TC-9: `version::1.2.3.4` → exit 1

- **Given:** clean environment
- **When:** `clv .version.install version::1.2.3.4`
- **Then:** exit code 1; four-part semver rejected
- **Exit:** 1
- **Source:** [type/03_version_spec.md — semver: dot-count validation](../../../../docs/cli/type/03_version_spec.md)

---

### TC-10: `version::01.02.03` → exit 1

- **Given:** clean environment
- **When:** `clv .version.install version::01.02.03`
- **Then:** exit code 1; leading zeros rejected
- **Exit:** 1
- **Source:** [type/03_version_spec.md — semver: no leading zeros](../../../../docs/cli/type/03_version_spec.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc304_version_install_wrong_case_exits_1` | `tests/cli/mutation_version_install_test.rs` |
| `tc305_version_install_empty_version_exits_1` | `tests/cli/mutation_version_install_test.rs` |
| `tc_version_spec_month_alias_accepted` | `cli_args_test/type_surface_test.rs` |
| `tc_version_spec_latest_alias_accepted` | `cli_args_test/type_surface_test.rs` |

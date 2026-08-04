# Test: `record_only::`

Edge case coverage for the `record_only::` parameter. See [param/15_record_only.md](../../../../docs/cli/param/15_record_only.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `record_only::` parameter.
- **Responsibility**: Boundary values, invalid inputs, mutual exclusion with `dry::`, and default behavior for `record_only::`.
- **Commands:** `.version.install`
- **In Scope**: Bool validation, mutual exclusion with `dry::`, `force::` interaction, absence default.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `record_only::1` → settings written, no install | Valid: record-only active |
| EC-2 | `record_only::0` → normal install path | Valid: record-only inactive |
| EC-3 | Absent `record_only::` → defaults to 0 (normal install) | Default Behavior |
| EC-4 | `record_only::true` → exit 1 | Invalid: non-binary value |
| EC-5 | `record_only::` (empty) → exit 1 | Invalid: empty |
| EC-6 | `record_only::1 dry::1` → exit 1 (mutual exclusion) | Mutual Exclusion |
| EC-7 | `record_only::1 force::1` → `force::` silently ignored | Force Interaction |

## Test Coverage Summary

- Valid record-only active: 1 test (EC-1)
- Valid record-only inactive: 1 test (EC-2)
- Default Behavior: 1 test (EC-3)
- Invalid non-binary value: 1 test (EC-4)
- Invalid empty: 1 test (EC-5)
- Mutual Exclusion: 1 test (EC-6)
- Force Interaction: 1 test (EC-7)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (`record_only::1` → settings write, no install, exit 0) ↔ EC-6 (`record_only::1 dry::1` → exit 1 from mutual exclusion)

---

### EC-1: `record_only::1` → settings written, no install

- **Given:** clean environment with `dry::1` not set
- **When:** `clv .version.install record_only::1`
- **Then:** exit 0; settings.json updated with `preferredVersionSpec` / `preferredVersionResolved`; no download or binary swap occurs
- **Exit:** 0
- **Source:** [param/15_record_only.md](../../../../docs/cli/param/15_record_only.md)

---

### EC-2: `record_only::0` → normal install path

- **Given:** clean environment
- **When:** `clv .version.install record_only::0`
- **Then:** exit 0; behaves identically to the default (no `record_only::` specified); install proceeds normally
- **Exit:** 0
- **Source:** [param/15_record_only.md](../../../../docs/cli/param/15_record_only.md)

---

### EC-3: Absent `record_only::` → defaults to 0

- **Given:** clean environment
- **When:** `clv .version.install` (no `record_only::` parameter)
- **Then:** exit 0; behavior identical to `record_only::0`; normal install path taken
- **Exit:** 0
- **Source:** [param/15_record_only.md](../../../../docs/cli/param/15_record_only.md)

---

### EC-4: `record_only::true` → exit 1

- **Given:** clean environment
- **When:** `clv .version.install record_only::true`
- **Then:** exit 1; error message references `record_only::` or invalid boolean value; strictly `0` or `1` required
- **Exit:** 1
- **Source:** [param/15_record_only.md](../../../../docs/cli/param/15_record_only.md)

---

### EC-5: `record_only::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .version.install record_only::`
- **Then:** exit 1; error message references `record_only::` or empty value
- **Exit:** 1
- **Source:** [param/15_record_only.md](../../../../docs/cli/param/15_record_only.md)

---

### EC-6: `record_only::1 dry::1` → exit 1 (mutual exclusion)

- **Given:** clean environment
- **When:** `clv .version.install record_only::1 dry::1`
- **Then:** exit 1; error indicates `record_only::` and `dry::` are mutually exclusive; `ArgumentMissing` or equivalent conflict error
- **Exit:** 1
- **Source:** [param/15_record_only.md](../../../../docs/cli/param/15_record_only.md)

---

### EC-7: `record_only::1 force::1` → `force::` silently ignored

- **Given:** clean environment
- **When:** `clv .version.install record_only::1 force::1`
- **Then:** exit 0; `force::` is accepted (not rejected as unknown), but has no effect under `record_only::1`; behavior identical to `record_only::1` alone
- **Exit:** 0
- **Source:** [param/15_record_only.md](../../../../docs/cli/param/15_record_only.md)

---

### Source Functions

| Function | File |
|----------|------|
| `record_only_ec1_settings_written_no_install` | `tests/cli/record_only_param_test.rs` |
| `record_only_ec2_zero_normal_install` | `tests/cli/record_only_param_test.rs` |
| `record_only_ec3_absent_defaults_zero` | `tests/cli/record_only_param_test.rs` |
| `record_only_ec4_true_rejected` | `tests/cli/record_only_param_test.rs` |
| `record_only_ec5_empty_exits_1` | `tests/cli/record_only_param_test.rs` |
| `record_only_ec6_mutual_exclusion_with_dry` | `tests/cli/record_only_param_test.rs` |
| `record_only_ec7_force_silently_ignored` | `tests/cli/record_only_param_test.rs` |

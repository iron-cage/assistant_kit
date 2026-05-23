# Test: `VerbosityLevel`

Type validation cases for the `VerbosityLevel` type. See [01_verbosity_level.md](../../../../docs/cli/type/01_verbosity_level.md) for specification.

### Scope

- **Purpose**: Type validation tests for `VerbosityLevel` (u8, range 0–2).
- **Responsibility**: Boundary values, out-of-range inputs, wrong-type inputs, and default behavior.
- **Used by:** `v::` parameter
- **In Scope**: Accepted range, rejection behavior, validation error messages.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `v::0` → minimal output | Valid minimum |
| TC-2 | `v::2` → verbose output | Valid maximum |
| TC-3 | `v::1` → normal output (explicit default) | Valid nominal |
| TC-4 | `v::3` → exit 1, out of range | Invalid: above range |
| TC-5 | `v::-1` / negative value → exit 1 | Invalid: below range |
| TC-6 | `v::abc` → exit 1, non-integer | Invalid: wrong type |
| TC-7 | absent `v::` → defaults to 1, normal output | Default behavior |

## Test Coverage Summary

- Valid range (0, 1, 2): 3 tests (TC-1, TC-2, TC-3)
- Invalid above range: 1 test (TC-4)
- Invalid below range: 1 test (TC-5)
- Invalid wrong type: 1 test (TC-6)
- Default behavior: 1 test (TC-7)

**Total:** 7 type cases

**Behavioral Divergence Pair:** TC-1 (`v::0` → raw values, no labels) ↔ TC-2 (`v::2` → diagnostic details and extra context). Both valid; output verbosity differs.

---

### TC-1: `v::0` → minimal output

- **Given:** clean environment, command with `v::` support
- **When:** `cm .version.list v::0`
- **Then:** output contains raw values only, no descriptive labels; minimum output level
- **Exit:** 0
- **Source:** [01_verbosity_level.md — Level 0: minimal](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-2: `v::2` → verbose output

- **Given:** clean environment
- **When:** `cm .version.list v::2`
- **Then:** output contains diagnostic details or extra context beyond v::1; maximum output level
- **Exit:** 0
- **Source:** [01_verbosity_level.md — Level 2: verbose](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-3: `v::1` → normal output

- **Given:** clean environment
- **When:** `cm .version.list v::1`
- **Then:** output is identical to bare `cm .version.list`; labeled key-value pairs present
- **Exit:** 0
- **Source:** [01_verbosity_level.md — Level 1: normal (default)](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-4: `v::3` → exit 1, out of range

- **Given:** clean environment
- **When:** `cm .status v::3`
- **Then:** exit 1; error message contains "verbosity out of range" or "must be 0, 1, or 2"
- **Exit:** 1
- **Source:** [01_verbosity_level.md — validation errors](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-5: `v::-1` → exit 1, below range

- **Given:** clean environment
- **When:** `cm .status v::-1`
- **Then:** exit 1; negative integer rejected by range or type check
- **Exit:** 1
- **Source:** [01_verbosity_level.md — Constraints: 0 to 2](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-6: `v::abc` → exit 1, non-integer

- **Given:** clean environment
- **When:** `cm .status v::abc`
- **Then:** exit 1; error message contains "must be 0, 1, or 2" or "non-integer"
- **Exit:** 1
- **Source:** [01_verbosity_level.md — validation errors: non-integer](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-7: absent `v::` → defaults to 1

- **Given:** clean environment
- **When:** `cm .version.list`
- **Then:** output is labeled (v::1 behavior); default value of 1 applied
- **Exit:** 0
- **Source:** [01_verbosity_level.md — Default: 1](../../../../docs/cli/type/01_verbosity_level.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc245_last_occurrence_wins_for_verbosity` | `integration/read_commands_test.rs` |
| ⏳ `tc_verbosity_level_0_minimal` | `cli_args_test.rs` |
| ⏳ `tc_verbosity_level_2_verbose` | `cli_args_test.rs` |
| ⏳ `tc_verbosity_level_3_out_of_range` | `cli_args_test.rs` |
| ⏳ `tc_verbosity_level_abc_non_integer` | `cli_args_test.rs` |

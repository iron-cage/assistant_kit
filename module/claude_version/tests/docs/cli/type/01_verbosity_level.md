# Test: `VerbosityLevel`

Type compliance and validation tests for `VerbosityLevel`. See [type/01_verbosity_level.md](../../../../docs/cli/type/01_verbosity_level.md) for specification.

### Scope

- **Purpose**: Validate VerbosityLevel parsing, range enforcement, and level semantics.
- **Responsibility**: Boundary values, invalid inputs, default behavior, and per-level output semantics for `v::`.
- **Commands:** `.status`, `.version.show`, `.version.install`, `.version.list`, `.version.guard`, `.version.history`, `.processes`, `.processes.kill`, `.settings.show`, `.settings.get`
- **In Scope**: Type parsing, range validation, and observable output differences between levels.
- **Out of Scope**: Per-command output structure (→ `../command/`), parameter interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `v::0` → minimal output (raw values, no labels) | Level Semantics |
| TC-2 | `v::1` → normal output (labeled key-value pairs) | Level Semantics |
| TC-3 | `v::2` → verbose output (diagnostic context) | Level Semantics |
| TC-4 | Absent `v::` → defaults to level 1 | Default |
| TC-5 | `v::3` → exit 1 (out of range) | Validation: range |
| TC-6 | `v::` (empty) → exit 1 | Validation: empty |
| TC-7 | `v::abc` → exit 1 (non-integer) | Validation: type |
| TC-8 | `v::-1` → exit 1 (negative integer) | Validation: range |

## Test Coverage Summary

- Level Semantics: 3 tests (TC-1, TC-2, TC-3)
- Default Behavior: 1 test (TC-4)
- Range validation: 2 tests (TC-5, TC-8)
- Type validation: 2 tests (TC-6, TC-7)

**Total:** 8 tests

**Behavioral Divergence Pair:** TC-1 (`clv .status v::0` → raw values only, no "version:" label) ↔ TC-3 (`clv .status v::2` → labeled output plus diagnostic context lines absent from v::1)

---

### TC-1: `v::0` → minimal output

- **Given:** clean environment with Claude Code installed
- **When:** `clv .status v::0`
- **Then:** output contains version value directly without label prefixes; no "version:" label visible
- **Exit:** 0
- **Source:** [type/01_verbosity_level.md — Level 0: raw values only](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-2: `v::1` → normal labeled output

- **Given:** clean environment with Claude Code installed
- **When:** `clv .status v::1`
- **Then:** output contains labeled key-value pairs (e.g., "version: X.Y.Z"); same as default output
- **Exit:** 0
- **Source:** [type/01_verbosity_level.md — Level 1: labeled pairs](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-3: `v::2` → verbose output

- **Given:** clean environment with Claude Code installed
- **When:** `clv .status v::2`
- **Then:** output contains additional diagnostic context beyond TC-2 output; more lines or detail present
- **Exit:** 0
- **Source:** [type/01_verbosity_level.md — Level 2: diagnostic details](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-4: Absent `v::` → defaults to level 1

- **Given:** clean environment
- **When:** `clv .status` (no `v::` parameter)
- **Then:** output matches `v::1` labeled output; no raw-only or verbose format
- **Exit:** 0
- **Source:** [type/01_verbosity_level.md — Default: 1 (normal)](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-5: `v::3` → exit 1

- **Given:** clean environment
- **When:** `clv .status v::3`
- **Then:** exit code 1; stderr contains error message mentioning verbosity out of range
- **Exit:** 1
- **Source:** [type/01_verbosity_level.md — Validation: out of range](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-6: `v::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .status v::`
- **Then:** exit code 1; error message references `v::` or verbosity
- **Exit:** 1
- **Source:** [type/01_verbosity_level.md — Validation errors](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-7: `v::abc` → exit 1

- **Given:** clean environment
- **When:** `clv .status v::abc`
- **Then:** exit code 1; stderr contains error message referencing non-integer verbosity
- **Exit:** 1
- **Source:** [type/01_verbosity_level.md — Validation: non-integer](../../../../docs/cli/type/01_verbosity_level.md)

---

### TC-8: `v::-1` → exit 1

- **Given:** clean environment
- **When:** `clv .status v::-1`
- **Then:** exit code 1; negative integer treated as out-of-range (0–2 constraint)
- **Exit:** 1
- **Source:** [type/01_verbosity_level.md — Constraints: 0 to 2](../../../../docs/cli/type/01_verbosity_level.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc_verbosity_level_0_minimal` | `cli_args_test.rs` |
| `tc098_status_v1_has_labels` | `integration/read_commands_test.rs` |
| `tc_verbosity_level_2_verbose` | `cli_args_test.rs` |
| `verbosity_ec5_absent_defaults_to_1` | `cli_args_test.rs` |
| `tc_verbosity_level_3_out_of_range` | `cli_args_test.rs` |
| `tc005_verbosity_empty_value` | `cli_args_test.rs` |
| `tc_verbosity_level_abc_non_integer` | `cli_args_test.rs` |
| `verbosity_ec8_negative_exits_1` | `cli_args_test.rs` |
| `tc245_last_occurrence_wins_for_verbosity` | `integration/read_commands_test.rs` |

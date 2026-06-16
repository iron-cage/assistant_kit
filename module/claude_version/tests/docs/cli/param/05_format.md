# Test: `format::`

Edge case coverage for the `format::` parameter. See [005_params.md](../../../../docs/cli/param/readme.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `format::` parameter.
- **Responsibility**: Boundary values, invalid inputs, type violations, and default behavior for `format::`.
- **Commands:** `.status`, `.version.show`, `.version.install`, `.version.list`, `.version.guard`, `.version.history`, `.processes`, `.processes.kill`, `.settings.show`, `.settings.get`
- **In Scope**: Single-parameter edge cases, validation errors, type checking.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-11 | `.status format::json` → `{"version":...}` | Explicit json |
| EC-12 | `.version.show format::json` → `{"version":"..."}` | Explicit json |
| EC-13 | `.version.list format::json` → JSON array | Explicit json |
| EC-14 | `.processes format::json` → `{"processes":[...]}` | Explicit json |
| EC-15 | `.settings.show format::json` → JSON object | Explicit json |
| EC-16 | `.settings.get format::json` → `{"key":..,"value":..}` | Explicit json |
| EC-5 | `format::json` preserves bool/number types | Type Fidelity |
| EC-17 | `.version.history format::json` → version/date/summary fields | Explicit json |
| EC-1 | `.version.guard format::json dry::1` → JSON output, exit 0 | Explicit json |
| EC-2 | `format::xml` → exit 1, unknown format | Invalid |
| EC-3 | `format::JSON` (uppercase) → exit 1 | Invalid (case) |
| EC-4 | `format::` (empty) → exit 1 | Empty Value |
| EC-18 | `.version.history format::xml` → exit 1 | Invalid |
| EC-19 | `.version.history format::JSON` → exit 1 | Invalid (case) |
| EC-6 | Default (absent) → `format::text` | Default Behavior |
| EC-7 | `format::text` explicit → same as absent | Explicit text |
| EC-8 | `format::csv` → exit 1 | Invalid |
| EC-9 | `format::` only for output-returning commands | Command Scope |
| EC-10 | JSON output always starts with `{` or `[` depending on command | Structure |

## Test Coverage Summary

- Explicit json: 8 tests
- Type Fidelity: 1 test
- Invalid: 3 tests
- Invalid (case-sensitive): 2 tests
- Empty Value: 1 test
- Default Behavior: 1 test
- Explicit text: 1 test
- Command Scope: 1 test
- JSON Structure: 1 test

**Total:** 19 edge cases

**Behavioral Divergence Pair:** EC-1 (`format::json` → JSON output, exit 0) ↔ EC-6 (absent → `format::text` output, exit 0)

---

### EC-1: `.version.guard format::json dry::1` → JSON output

- **Given:** clean environment
- **When:** `clv .version.guard format::json dry::1`
- **Then:** exit code 0; stdout starts with `{`.; JSON output
- **Exit:** 0
- **Source:** [001_commands.md — .version.guard](../../../../docs/cli/command/readme.md#command--5-versionguard)

---

### EC-2: `format::xml` → exit 1

- **Given:** clean environment
- **When:** `clv .status format::xml` (cross-cutting — applies to all format-accepting commands).
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-3: `format::JSON` (uppercase) → exit 1

- **Given:** clean environment
- **When:** `clv .status format::JSON`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-4: `format::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .status format::`
- **Then:** exit code 1; error mentions format:: value.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-5: `format::json` preserves bool/number types

- **Given:** `HOME=<tmp>`; `settings.json` has `"flag": true` and `"count": 42`.
- **When:** `clv .settings.show format::json`
- **Then:** exit code 0; output contains unquoted `true` and `42`.; type-faithful JSON
- **Exit:** 0
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-6: Default (absent) → `format::text`

- **Given:** clean environment
- **When:** `clv .status`
- **Then:** Human-readable text (not JSON).; Output does not start with `{`
- **Exit:** 0
- **Source:** [005_params.md — format:: default: text](../../../../docs/cli/param/readme.md)

---

### EC-7: `format::text` explicit → same as absent

- **Given:** clean environment
- **When:** `clv .status format::text`
- **Then:** Behavior identical to `clv .status`; no JSON output.; explicit text equals absent
- **Exit:** 0
- **Source:** [005_params.md — format:: default: text](../../../../docs/cli/param/readme.md)

---

### EC-8: `format::csv` → exit 1

- **Given:** clean environment
- **When:** `clv .status format::csv`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-9: `format::` only for output-returning commands

- **Given:** clean environment
- **When:** `clv .settings.set format::json`
- **Then:** exit code 1; unknown parameter.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-10: JSON structure per command

- **Given:** clean environment
- **When:** `clv .status format::json` and `clv .version.list format::json`
- **Then:** `.status` output starts with `{`; `.version.list` output starts with `[`; both parse as valid JSON
- **Exit:** 0
- **Source:** [005_params.md](../../../../docs/cli/param/readme.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc015_format_empty_value` | `cli_args_test.rs` |
| `tc030_format_text_wrong_case_rejected` | `cli_args_test.rs` |
| `tc495_format_text_then_json_last_wins_json` | `cli_args_test.rs` |
| `tc242_unknown_format_exits_1` | `integration/read_commands_test.rs` |
| `tc243_uppercase_format_exits_1` | `integration/read_commands_test.rs` |
| `tc244_empty_format_exits_1` | `integration/read_commands_test.rs` |
| `tc258_status_format_json_is_valid_json` | `integration/cross_cutting_test.rs` |
| `tc260_format_uppercase_rejected` | `integration/cross_cutting_test.rs` |
| `tc261_version_install_format_json_accepted` | `integration/cross_cutting_test.rs` |
| `tc504_format_unknown_error_mentions_valid` | `integration/error_messages_test.rs` |

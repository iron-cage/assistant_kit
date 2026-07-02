# Test: `format::`

Edge case coverage for the `format::` parameter. See [param/05_format.md](../../../../docs/cli/param/05_format.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `format::` parameter.
- **Responsibility**: Boundary values, invalid inputs, type violations, and default behavior for `format::`.
- **Commands:** `.status`, `.version.show`, `.version.install`, `.version.list`, `.version.guard`, `.version.history`, `.processes`, `.processes.kill`, `.settings.show`, `.settings.get`, `.config`, `.params`
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
| EC-20 | `.params format::json` → JSON array; each entry has `name` field | Explicit json |

## Test Coverage Summary

- Explicit json: 9 tests
- Type Fidelity: 1 test
- Invalid: 3 tests
- Invalid (case-sensitive): 2 tests
- Empty Value: 1 test
- Default Behavior: 1 test
- Explicit text: 1 test
- Command Scope: 1 test
- JSON Structure: 1 test

**Total:** 20 edge cases

**Behavioral Divergence Pair:** EC-1 (`format::json` → JSON output, exit 0) ↔ EC-6 (absent → `format::text` output, exit 0)

---

### EC-1: `.version.guard format::json dry::1` → JSON output

- **Given:** clean environment
- **When:** `clv .version.guard format::json dry::1`
- **Then:** exit code 0; stdout starts with `{`.; JSON output
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--5-versionguard)

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
- **Source:** [param/readme.md — format:: default: text](../../../../docs/cli/param/readme.md)

---

### EC-7: `format::text` explicit → same as absent

- **Given:** clean environment
- **When:** `clv .status format::text`
- **Then:** Behavior identical to `clv .status`; no JSON output.; explicit text equals absent
- **Exit:** 0
- **Source:** [param/readme.md — format:: default: text](../../../../docs/cli/param/readme.md)

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
- **Source:** [param/readme.md](../../../../docs/cli/param/readme.md)

---

### EC-11: `.status format::json` → JSON object

- **Given:** clean environment with valid Claude installation
- **When:** `clv .status format::json`
- **Then:** exit 0; stdout is valid JSON starting with `{`; contains version-related fields
- **Exit:** 0
- **Source:** [command/readme.md — .status](../../../../docs/cli/command/readme.md)

---

### EC-12: `.version.show format::json` → `{"version":"..."}`

- **Given:** clean environment
- **When:** `clv .version.show format::json`
- **Then:** exit 0; stdout is valid JSON object containing a `"version"` key with semver string value
- **Exit:** 0
- **Source:** [command/version.md — .version.show](../../../../docs/cli/command/version.md)

---

### EC-13: `.version.list format::json` → JSON array

- **Given:** clean environment
- **When:** `clv .version.list format::json`
- **Then:** exit 0; stdout is valid JSON starting with `[`; contains alias entries
- **Exit:** 0
- **Source:** [command/version.md — .version.list](../../../../docs/cli/command/version.md)

---

### EC-14: `.processes format::json` → `{"processes":[...]}`

- **Given:** clean environment
- **When:** `clv .processes format::json`
- **Then:** exit 0; stdout is valid JSON object; contains `"processes"` array key
- **Exit:** 0
- **Source:** [command/readme.md — .processes](../../../../docs/cli/command/readme.md)

---

### EC-15: `.settings.show format::json` → JSON object

- **Given:** `HOME=<tmp>`; settings.json has at least one key
- **When:** `clv .settings.show format::json`
- **Then:** exit 0; stdout is valid JSON object mirroring the settings file; top-level is `{}`
- **Exit:** 0
- **Source:** [command/settings.md — .settings.show](../../../../docs/cli/command/settings.md)

---

### EC-16: `.settings.get format::json` → `{"key":"..","value":..}`

- **Given:** `HOME=<tmp>`; settings.json contains `myKey = "myValue"`
- **When:** `clv .settings.get key::myKey format::json`
- **Then:** exit 0; stdout is valid JSON object with `"key"` and `"value"` fields
- **Exit:** 0
- **Source:** [command/settings.md — .settings.get](../../../../docs/cli/command/settings.md)

---

### EC-17: `.version.history format::json` → version/date/summary fields

- **Given:** network available
- **When:** `clv .version.history format::json count::3`
- **Then:** exit 0; stdout is a valid JSON array; each element has at minimum `version`, `date`, and `summary` fields
- **Exit:** 0
- **Source:** [command/version.md — .version.history](../../../../docs/cli/command/version.md)

---

### EC-18: `.version.history format::xml` → exit 1

- **Given:** clean environment
- **When:** `clv .version.history format::xml`
- **Then:** exit 1; error message references unknown format value
- **Exit:** 1
- **Source:** [command/version.md — .version.history](../../../../docs/cli/command/version.md)

---

### EC-19: `.version.history format::JSON` (uppercase) → exit 1

- **Given:** clean environment
- **When:** `clv .version.history format::JSON`
- **Then:** exit 1; same error as unknown format; `format::` is case-sensitive
- **Exit:** 1
- **Source:** [command/version.md — .version.history](../../../../docs/cli/command/version.md)

---

### EC-20: `.params format::json` → JSON array with param field structure

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv .params format::json`
- **Then:** exit 0; stdout is valid JSON starting with `[`; each element has at minimum a `name` field; array structure (not object) distinguishes it from single-result commands
- **Exit:** 0
- **Source:** [command/params.md](../../../../docs/cli/command/params.md)

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
| `format_ec11_status_format_json_object` | `integration/format_param_test.rs` |
| `format_ec12_version_show_format_json_has_version_key` | `integration/format_param_test.rs` |
| `format_ec13_version_list_format_json_array` | `integration/format_param_test.rs` |
| `format_ec14_processes_format_json_has_processes_key` | `integration/format_param_test.rs` |
| `format_ec15_settings_show_format_json_object` | `integration/format_param_test.rs` |
| `format_ec16_settings_get_format_json_has_key_value` | `integration/format_param_test.rs` |
| `format_ec17_history_format_json_fields` | `integration/format_param_test.rs` |
| `format_ec18_history_format_xml_exits_1` | `integration/format_param_test.rs` |
| `format_ec19_history_format_json_uppercase_exits_1` | `integration/format_param_test.rs` |
| `format_ec6_absent_defaults_to_text` | `cli_args_test.rs` |
| `format_ec7_text_explicit_same_as_absent` | `cli_args_test.rs` |
| `format_ec8_csv_exits_1` | `cli_args_test.rs` |
| `format_ec20_params_format_json_array` | `integration/format_param_test.rs` |
| `tc241_settings_show_json_preserves_types` | `integration/read_commands_test.rs` |
| `ft005_9_per_cmd_validation` | `integration/catalog_surface_test.rs` |

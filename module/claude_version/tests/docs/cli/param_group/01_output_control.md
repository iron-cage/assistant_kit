# Test: Output Control Group

Interaction tests for the `v::` (verbosity) and `format::` parameter group.
See [param_group/readme.md](../../../../docs/cli/param_group/readme.md) and [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md).

### Scope

- **Purpose**: Interaction tests for the Output Control parameter group.
- **Responsibility**: Cross-parameter semantics between `v::` and `format::`, precedence rules, and combined behavior.
- **Commands:** `.status`, `.version.show`, `.version.install`, `.version.list`, `.version.guard`, `.version.history`, `.processes`, `.processes.kill`, `.settings.show`, `.settings.get`, `.config`, `.params`, `.paths`
- **In Scope**: Multi-parameter interactions within the group, override semantics, format-overrides-verbosity rule.
- **Out of Scope**: Individual parameter edge cases (→ `../param/`), command behavior (→ `../command/`).

## Group Summary

| Parameter | Type | Default | Commands |
|-----------|------|---------|---------|
| `v::` | u8 (0-2) | 1 | 13 commands (all except `.settings.set`) |
| `format::` | text\|json | text | 13 commands (all except `.settings.set`) |

## Behavioral Divergence Pair

Two valid invocations produce distinct output formats:

- **Input A:** `clv .version.list v::0 format::json` → JSON array output (CC-2)
- **Input B:** `clv .version.list v::1 format::text` → labeled text output (CC-6)

Both are valid invocations; the format of the output differs.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | Last `v::` wins when duplicated | Duplicate param |
| CC-2 | `v::0 format::json` → JSON ignores verbosity | format overrides v:: |
| CC-3 | `v::2 format::json` identical to `v::0 format::json` | format overrides v:: |
| CC-4 | `v::0` consistently minimal across commands | Cross-command v::0 |
| CC-5 | `format::json` always produces valid JSON regardless of `v::` | JSON validity |
| CC-6 | `v::1 format::text` → same as default | Explicit defaults |
| CC-7 | `v::3 format::json` → exit 1 (v:: range check before format) | Validation order |
| CC-8 | `v::0 format::xml` → exit 1 | Invalid format |
| CC-9 | `v::abc format::json` → exit 1 | v:: type check |
| CC-10 | `v::` and `format::` absent → text v::1 default | Both absent |

## Test Coverage Summary

- Duplicate param (last-wins): 1 test (CC-1)
- format overrides v:: for JSON: 2 tests (CC-2, CC-3)
- Cross-command consistency: 1 test (CC-4)
- JSON validity: 1 test (CC-5)
- Explicit defaults: 1 test (CC-6)
- Validation (invalid v:: + valid format): 1 test (CC-7)
- Validation (valid v:: + invalid format): 1 test (CC-8)
- Validation (invalid v:: type): 1 test (CC-9)
- Both absent (defaults): 1 test (CC-10)

**Total:** 10 interaction tests

---

### CC-1: Last `v::` wins on duplicate

- **Given:** clean environment
- **When:** `clv .version.list v::0 v::1`
- **Then:** output shows descriptions (v::1 behavior); v::1 applied because it is last
- **Exit:** 0
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### CC-2: `v::0 format::json` → JSON ignores verbosity

- **Given:** clean environment
- **When:** `clv .version.list v::0 format::json`
- **Then:** stdout is valid JSON; output identical to `v::2 format::json`
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — format::json overrides v::](../../../../docs/cli/004_parameter_interactions.md)

---

### CC-3: `v::2 format::json` identical to `v::0 format::json`

- **Given:** clean environment
- **When:** `clv .version.list v::2 format::json` and `clv .version.list v::0 format::json`
- **Then:** both outputs parse as identical JSON; v:: has no effect on JSON structure
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — format::json overrides v::](../../../../docs/cli/004_parameter_interactions.md)

---

### CC-4: `v::0` consistently minimal across commands

- **Given:** appropriate state for each command
- **When:** `clv .status v::0`, `clv .version.list v::0`, `clv .settings.get key::k v::0`
- **Then:** each produces compact, label-free output; consistent minimum-output behavior across commands
- **Exit:** 0
- **Source:** [type/readme.md — verbosity levels](../../../../docs/cli/type/readme.md)

---

### CC-5: `format::json` always produces valid JSON regardless of `v::`

- **Given:** clean environment
- **When:** `clv .version.list format::json v::0` and `clv .version.list format::json v::2`
- **Then:** both outputs parse as valid JSON; neither produces text-format output; v:: has no effect on JSON structure
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — format::json overrides v::](../../../../docs/cli/004_parameter_interactions.md)

---

### CC-6: `v::1 format::text` → same as default

- **Given:** clean environment
- **When:** `clv .version.list v::1 format::text`
- **Then:** output is identical to bare `clv .version.list`; explicitly setting both parameters to their defaults produces no behavioral change
- **Exit:** 0
- **Source:** [param_group/readme.md — Output Control](../../../../docs/cli/param_group/readme.md)

---

### CC-7: `v::3 format::json` → exit 1

- **Given:** clean environment
- **When:** `clv .status v::3 format::json`
- **Then:** exit 1; v:: range check fails before format:: is evaluated
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### CC-8: `v::0 format::xml` → exit 1

- **Given:** clean environment
- **When:** `clv .status v::0 format::xml`
- **Then:** exit 1; format:: value rejected (xml is not a valid format)
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### CC-9: `v::abc format::json` → exit 1

- **Given:** clean environment
- **When:** `clv .status v::abc format::json`
- **Then:** exit 1; v:: type check fails (non-integer value)
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### CC-10: Both absent → text v::1 defaults

- **Given:** clean environment
- **When:** `clv .version.list`
- **Then:** output is labeled text (not JSON, not bare names); default behavior: labeled text output
- **Exit:** 0
- **Source:** [param_group/readme.md — Output Control](../../../../docs/cli/param_group/readme.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc255_status_v0_fewer_lines_than_v1` | `tests/cli/cross_cutting_test.rs` |
| `tc257_v_param_identical` | `tests/cli/cross_cutting_test.rs` |
| `tc258_status_format_json_is_valid_json` | `tests/cli/cross_cutting_test.rs` |
| `tc259_status_format_json_v0_still_complete` | `tests/cli/cross_cutting_test.rs` |
| `tc260_format_uppercase_rejected` | `tests/cli/cross_cutting_test.rs` |
| `tc261_version_install_format_json_accepted` | `tests/cli/cross_cutting_test.rs` |
| `tc262_version_guard_v0_accepted` | `tests/cli/cross_cutting_test.rs` |
| `tc245_last_occurrence_wins_for_verbosity` | `tests/cli/read_status_test.rs` |

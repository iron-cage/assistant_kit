# Test: Output Control Group

Interaction tests for the `v::` (verbosity) and `format::` parameter group.
See [003_parameter_groups.md](../../../../docs/cli/003_parameter_groups.md) and [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md).

### Scope

- **Purpose**: Interaction tests for the Output Control parameter group.
- **Responsibility**: Cross-parameter semantics between `v::` and `format::`, precedence rules, and combined behavior.
- **In Scope**: Multi-parameter interactions within the group, override semantics, format-overrides-verbosity rule.
- **Out of Scope**: Individual parameter edge cases (→ `../param/`), command behavior (→ `../command/`).

## Group Summary

| Parameter | Type | Default | Commands |
|-----------|------|---------|---------|
| `v::` | u8 (0-2) | 1 | 10 commands (all except `.settings.set`) |
| `format::` | text\|json | text | 10 commands (all except `.settings.set`) |

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-104 | `.status v::0` has ≤ lines than `v::1` | Verbosity ordering |
| EC-4 | Last `v::` wins when duplicated | Duplicate param |
| EC-1 | `v::0 format::json` → json format ignores verbosity | format overrides v:: |
| EC-2 | `v::2 format::json` → same as v::0 format::json | format overrides v:: |
| EC-3 | `v::0` is always machine-parseable across commands | Cross-command v::0 |
| IT-4 | `format::json` always produces valid JSON regardless of v:: | JSON validity |
| IT-5 | `v::1 format::text` → same as default (both explicitly default) | Explicit defaults |
| EC-1 | `v::3 format::json` → exit 1 (v:: range check before format) | Validation order |
| EC-2 | `v::0 format::xml` → exit 1 (format:: check) | Both invalid |
| EC-3 | `v::abc format::json` → exit 1 | v:: type check |
| EC-4 | `v::` and `format::` absent → always text v::1 default | Both absent |

## Test Coverage Summary

- Verbosity ordering: 1 test
- Duplicate param (last-wins): 1 test
- format overrides v:: for JSON: 2 tests
- Cross-command consistency: 2 tests
- Explicit defaults: 1 test
- Validation (invalid v:: + valid format): 1 test
- Validation (valid v:: + invalid format): 1 test
- Validation (invalid v:: type): 1 test
- Both absent (defaults): 1 test

**Total:** 11 interaction tests

---

### EC-1: `v::0 format::json` → JSON ignores verbosity

- **Given:** clean environment
- **When:**
  `cm .version.list v::0 format::json` vs `cm .version.list v::2 format::json`
  **Expected:** Both produce identical JSON output.
- **Then:** JSON output independent of v::
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — format::json overrides v::](../../../../docs/cli/004_parameter_interactions.md)

---

### EC-2: `v::2 format::json` identical to `v::0 format::json`

- **Given:** clean environment
- **When:**
  Run `.version.list v::2 format::json` and `.version.list v::0 format::json`.
  **Expected:** Identical JSON outputs.
- **Then:** v:: has no effect on JSON format
- **Exit:** 0

---

### EC-3: `v::0` is machine-parseable across commands

- **Given:** Appropriate state for each command.
**Commands:** `v::0` on `.status`, `.version.list`, `.settings.get`, `.processes`.
**Expected:** Each produces compact, label-free output.
- **When:** 
- **Then:** Consistent minimum-output behavior
- **Exit:** 0

---

### EC-4: Last `v::` wins on duplicate

- **Given:** clean environment
- **When:**
  `cm .version.list v::0 v::1`
  **Expected:** v::1 behavior (descriptions present) because v::1 is last.
- **Then:** v::1 applied
- **Exit:** 0
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-1: `v::3 format::json` → exit 1

- **Given:** clean environment
- **When:**
  `cm .status v::3 format::json`
  **Expected:** Exit 1 (v:: range check fails).
- **Then:** see spec
- **Exit:** 1

---

### EC-2: `v::0 format::xml` → exit 1

- **Given:** clean environment
- **When:**
  `cm .status v::0 format::xml`
  **Expected:** Exit 1 (format:: value rejected).
- **Then:** see spec
- **Exit:** 1

---

### EC-4: Both absent → text v::1 defaults

- **Given:** clean environment
- **When:**
  `cm .version.list`
  **Expected:** Output is labeled text (not JSON, not bare names).
- **Then:** Default behavior: labeled text output
- **Exit:** 0

---

### IT-4: `format::json` always produces valid JSON regardless of v::

- **Given:** clean environment
- **When:** `cm .version.list format::json v::0` and `cm .version.list format::json v::2`
- **Then:** Both outputs parse as valid JSON; neither produces text-format output; v:: has no effect on JSON structure
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — format::json overrides v::](../../../../docs/cli/004_parameter_interactions.md)

---

### IT-5: `v::1 format::text` → same as default

- **Given:** clean environment
- **When:** `cm .version.list v::1 format::text`
- **Then:** Output is identical to bare `cm .version.list`; explicitly setting both parameters to their defaults produces no behavioral change
- **Exit:** 0
- **Source:** [003_parameter_groups.md — Output Control](../../../../docs/cli/003_parameter_groups.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc255_status_v0_fewer_lines_than_v1` | `integration/cross_cutting_test.rs` |
| `tc257_v_param_identical` | `integration/cross_cutting_test.rs` |
| `tc258_status_format_json_is_valid_json` | `integration/cross_cutting_test.rs` |
| `tc259_status_format_json_v0_still_complete` | `integration/cross_cutting_test.rs` |
| `tc260_format_uppercase_rejected` | `integration/cross_cutting_test.rs` |
| `tc261_version_install_format_json_accepted` | `integration/cross_cutting_test.rs` |
| `tc262_version_guard_v0_accepted` | `integration/cross_cutting_test.rs` |
| `tc245_last_occurrence_wins_for_verbosity` | `integration/read_commands_test.rs` |

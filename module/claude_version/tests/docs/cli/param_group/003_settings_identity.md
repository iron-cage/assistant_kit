# Parameter Group :: Settings Identity

Interaction tests for Group 3 (Settings Identity): `key::` and `value::`. Tests validate required pairing, read vs. write semantics, and error behavior when one is missing.

### Scope

- **Purpose**: Interaction tests for the Settings Identity parameter group.
- **Responsibility**: Cross-parameter semantics between `key::` and `value::`, required pairing rules, and read-vs-write behavior.
- **In Scope**: Multi-parameter interactions within the group, missing-value errors, read-only override.
- **Out of Scope**: Individual parameter edge cases (→ `../param/`), command behavior (→ `../command/`).

**Source:** [003_parameter_groups.md#group--3-settings-identity](../../../../docs/cli/003_parameter_groups.md#group--3-settings-identity)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `key::` + `value::` together → settings written | Happy Path |
| EC-2 | `key::` without `value::` → exit 1 on `.settings.set` | Missing Value |
| EC-3 | `key::` alone → value read on `.settings.get` | Read Mode |
| EC-4 | Both present on `.settings.get` → `value::` ignored (read-only) | Interaction |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Missing Value: 1 test (EC-2)
- Read Mode: 1 test (EC-3)
- Interaction: 1 test (EC-4)

**Total:** 4 edge cases

## Test Cases
---

### EC-1: `key::` + `value::` → settings written:

- **Given:** clean environment
- **When:** `cm .settings.set key::theme value::dark`
- **Then:** Setting `theme=dark` written; exit 0
- **Exit:** 0
- **Source:** [003_parameter_groups.md#group--3-settings-identity](../../../../docs/cli/003_parameter_groups.md#group--3-settings-identity)
---

### EC-2: `key::` without `value::` on `.settings.set` → exit 1:

- **Given:** clean environment
- **When:** `cm .settings.set key::theme`
- **Then:** Exit 1; error indicating `value::` is required for `.settings.set`
- **Exit:** 1
- **Source:** [003_parameter_groups.md#group--3-settings-identity](../../../../docs/cli/003_parameter_groups.md#group--3-settings-identity)
---

### EC-3: `key::` alone on `.settings.get` → value read:

- **Given:** clean environment with `theme` key set
- **When:** `cm .settings.get key::theme`
- **Then:** Current value of `theme` printed; exit 0
- **Exit:** 0
- **Source:** [003_parameter_groups.md#group--3-settings-identity](../../../../docs/cli/003_parameter_groups.md#group--3-settings-identity)
---

### EC-4: Both on `.settings.get` → `value::` ignored:

- **Given:** clean environment
- **When:** `cm .settings.get key::theme value::dark`
- **Then:** Current setting value read and printed; `value::dark` has no effect on get
- **Exit:** 0
- **Source:** [003_parameter_groups.md#group--3-settings-identity](../../../../docs/cli/003_parameter_groups.md#group--3-settings-identity)

---

### Source Functions

| Function | File |
|----------|------|
| `tc320_settings_set_missing_key_exits_1` | `integration/mutation_commands_test.rs` |
| `tc321_settings_set_missing_value_exits_1` | `integration/mutation_commands_test.rs` |
| `tc322_settings_set_stores_boolean_true` | `integration/mutation_commands_test.rs` |
| `tc323_settings_set_stores_boolean_false` | `integration/mutation_commands_test.rs` |
| `tc324_settings_set_zero_stored_as_number` | `integration/mutation_commands_test.rs` |
| `tc325_settings_set_stores_number` | `integration/mutation_commands_test.rs` |
| `tc326_settings_set_stores_string` | `integration/mutation_commands_test.rs` |
| `tc176_settings_get_existing_key` | `integration/read_commands_test.rs` |
| `tc179_settings_get_v0_bare_value` | `integration/read_commands_test.rs` |
| `tc182_settings_get_format_json` | `integration/read_commands_test.rs` |

# Test: Settings Identity Group

Interaction tests for Group 3 (Settings Identity): `key::` and `value::`. Tests validate required pairing, read vs. write semantics, and behavior when one is missing.

### Scope

- **Purpose**: Interaction tests for the Settings Identity parameter group.
- **Responsibility**: Cross-parameter semantics between `key::` and `value::`, required pairing rules, and read-vs-write behavior.
- **Commands:** `.settings.get`, `.settings.set`
- **In Scope**: Multi-parameter interactions within the group, missing-value errors, read-only override.
- **Out of Scope**: Individual parameter edge cases (→ `../param/`), command behavior (→ `../command/`).

**Source:** [param_group/readme.md#group--3-settings-identity](../../../../docs/cli/param_group/03_settings_identity.md)

## Behavioral Divergence Pair

Two valid invocations produce distinct operations on the same key:

- **Input A:** `clv .settings.set key::theme value::dark` → setting written (write operation)
- **Input B:** `clv .settings.get key::theme` → current value printed (read operation)

Both are valid invocations; the direction of data flow differs (write vs read).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `key::` + `value::` together → settings written | Happy Path |
| CC-2 | `key::` without `value::` → exit 1 on `.settings.set` | Missing Value |
| CC-3 | `key::` alone → value read on `.settings.get` | Read Mode |
| CC-4 | Both present on `.settings.get` → `value::` ignored (read-only) | Interaction |

## Test Coverage Summary

- Happy Path: 1 test (CC-1)
- Missing Value: 1 test (CC-2)
- Read Mode: 1 test (CC-3)
- Interaction: 1 test (CC-4)

**Total:** 4 interaction tests

---

### CC-1: `key::` + `value::` → settings written

- **Given:** clean environment
- **When:** `clv .settings.set key::theme value::dark`
- **Then:** setting `theme=dark` written to `settings.json`; exit 0
- **Exit:** 0
- **Source:** [param_group/readme.md#group--3-settings-identity](../../../../docs/cli/param_group/03_settings_identity.md)

---

### CC-2: `key::` without `value::` on `.settings.set` → exit 1

- **Given:** clean environment
- **When:** `clv .settings.set key::theme`
- **Then:** exit 1; error indicating `value::` is required for `.settings.set`
- **Exit:** 1
- **Source:** [param_group/readme.md#group--3-settings-identity](../../../../docs/cli/param_group/03_settings_identity.md)

---

### CC-3: `key::` alone on `.settings.get` → value read

- **Given:** clean environment with `theme` key set
- **When:** `clv .settings.get key::theme`
- **Then:** current value of `theme` printed; exit 0
- **Exit:** 0
- **Source:** [param_group/readme.md#group--3-settings-identity](../../../../docs/cli/param_group/03_settings_identity.md)

---

### CC-4: Both on `.settings.get` → `value::` ignored

- **Given:** clean environment
- **When:** `clv .settings.get key::theme value::dark`
- **Then:** current setting value read and printed; `value::dark` has no effect on get operation
- **Exit:** 0
- **Source:** [param_group/readme.md#group--3-settings-identity](../../../../docs/cli/param_group/03_settings_identity.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc320_settings_set_missing_key_exits_1` | `tests/cli/mutation_settings_set_test.rs` |
| `tc321_settings_set_missing_value_exits_1` | `tests/cli/mutation_settings_set_test.rs` |
| `tc322_settings_set_stores_boolean_true` | `tests/cli/mutation_settings_set_test.rs` |
| `tc323_settings_set_stores_boolean_false` | `tests/cli/mutation_settings_set_test.rs` |
| `tc324_settings_set_zero_stored_as_number` | `tests/cli/mutation_settings_set_test.rs` |
| `tc325_settings_set_stores_number` | `tests/cli/mutation_settings_set_test.rs` |
| `tc326_settings_set_stores_string` | `tests/cli/mutation_settings_set_test.rs` |
| `tc176_settings_get_existing_key` | `tests/cli/read_settings_test.rs` |
| `tc179_settings_get_v0_bare_value` | `tests/cli/read_settings_test.rs` |
| `tc182_settings_get_format_json` | `tests/cli/read_settings_test.rs` |

# Test: `ConfigScope`

Type compliance and validation tests for `ConfigScope`. See [type/06_config_scope.md](../../../../docs/cli/type/06_config_scope.md) for specification.

### Scope

- **Purpose**: Validate ConfigScope enum parsing and constraint enforcement.
- **Responsibility**: Two-value enum acceptance, invalid variant rejection, default behavior.
- **Commands:** `.config`
- **In Scope**: Enum variant parsing (`user`, `project`), invalid value rejection, default fallback.
- **Out of Scope**: Config file I/O behavior (-> `../command/13_config.md`), key/value semantics (-> `07_config_key.md`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `scope::user` -> accepted, targets user config | Valid: user variant |
| TC-2 | `scope::project` -> accepted, targets project config | Valid: project variant |
| TC-3 | absent `scope::` -> defaults to `user` | Valid: default |
| TC-4 | `scope::global` -> exit 1, unknown variant | Invalid: unknown variant |
| TC-5 | `scope::USER` -> exit 1, case-sensitive | Invalid: wrong case |
| TC-6 | `scope::` (empty) -> exit 1, empty value | Invalid: empty |

## Test Coverage Summary

- Valid enum variants: 2 tests (TC-1, TC-2)
- Default behavior: 1 test (TC-3)
- Invalid variants: 2 tests (TC-4, TC-5)
- Empty value: 1 test (TC-6)

**Total:** 6 tests

**Behavioral Divergence Pair:** TC-1 (`scope::user` -> writes to `~/.claude/settings.json`) <-> TC-2 (`scope::project` -> writes to `{cwd}/.claude/settings.json`)

---

### TC-1: `scope::user` -> accepted, targets user config

- **Given:** clean HOME with `~/.claude/settings.json` containing `{}`
- **When:** `clv .config key::theme value::dark scope::user`
- **Then:** exit 0; `~/.claude/settings.json` contains `"theme": "dark"`
- **Exit:** 0
- **Source:** [type/06_config_scope.md -- user variant](../../../../docs/cli/type/06_config_scope.md)

---

### TC-2: `scope::project` -> accepted, targets project config

- **Given:** clean HOME; cwd has no `.claude/` directory
- **When:** `clv .config key::theme value::dark scope::project`
- **Then:** exit 0; `{cwd}/.claude/settings.json` contains `"theme": "dark"`; user config unchanged
- **Exit:** 0
- **Source:** [type/06_config_scope.md -- project variant](../../../../docs/cli/type/06_config_scope.md)

---

### TC-3: absent `scope::` -> defaults to `user`

- **Given:** clean HOME with `~/.claude/settings.json` containing `{}`
- **When:** `clv .config key::theme value::dark` (no `scope::` parameter)
- **Then:** exit 0; `~/.claude/settings.json` contains `"theme": "dark"` (same as `scope::user`)
- **Exit:** 0
- **Source:** [type/06_config_scope.md -- Default: user](../../../../docs/cli/type/06_config_scope.md)

---

### TC-4: `scope::global` -> exit 1, unknown variant

- **Given:** clean environment
- **When:** `clv .config key::theme value::dark scope::global`
- **Then:** exit 1; error message contains "scope:: must be 'user' or 'project'" or equivalent
- **Exit:** 1
- **Source:** [type/06_config_scope.md -- Validation: any other value -> exit 1](../../../../docs/cli/type/06_config_scope.md)

---

### TC-5: `scope::USER` -> exit 1, case-sensitive

- **Given:** clean environment
- **When:** `clv .config key::theme value::dark scope::USER`
- **Then:** exit 1; error message indicates invalid scope value
- **Exit:** 1
- **Source:** [type/06_config_scope.md -- Constraints: exactly user or project](../../../../docs/cli/type/06_config_scope.md)

---

### TC-6: `scope::` (empty) -> exit 1, empty value

- **Given:** clean environment
- **When:** `clv .config key::theme value::dark scope::`
- **Then:** exit 1; error message indicates empty scope value
- **Exit:** 1
- **Source:** [type/06_config_scope.md -- Constraints: exactly user or project](../../../../docs/cli/type/06_config_scope.md)

---

### Source Functions

| Function | File | Status |
|----------|------|--------|
| `tc01_006_scope_user_accepted` | `tests/cli/config_commands_test.rs` | ✅ |
| `tc02_006_scope_project_accepted` | `tests/cli/config_commands_test.rs` | ✅ |
| `tc03_006_scope_absent_defaults_to_user` | `tests/cli/config_commands_test.rs` | ✅ |
| `tc04_006_scope_global_exits_1` | `tests/cli/config_commands_test.rs` | ✅ |
| `tc05_006_scope_wrong_case_exits_1` | `tests/cli/config_commands_test.rs` | ✅ |
| `tc06_006_scope_empty_exits_1` | `tests/cli/config_commands_test.rs` | ✅ |

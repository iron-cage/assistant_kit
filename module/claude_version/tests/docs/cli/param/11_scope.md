# Test: `scope::`

Edge case coverage for the `scope::` parameter. See [param/11_scope.md](../../../../docs/cli/param/11_scope.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `scope::` parameter.
- **Responsibility**: Boundary values, invalid inputs, and default behavior for `scope::`.
- **Commands:** `.config`
- **In Scope**: Single-parameter edge cases, validation errors, default behavior.
- **Out of Scope**: Command integration (-> `../command/`), group interactions (-> `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `scope::user` (default) writes to `~/.claude/settings.json` | Valid: default |
| EC-2 | `scope::project` writes to `{cwd}/.claude/settings.json` | Valid: project |
| EC-3 | `scope::invalid` -> exit 1, unknown scope | Invalid Value |
| EC-4 | `scope::` (empty value) -> exit 1 | Empty Value |
| EC-5 | `scope::user` without `key::` and `value::` -> exit 1, scope only applies to writes | Invalid Combination |
| EC-6 | `scope::project` with `key::K value::V` creates `.claude/` directory if absent | Project Creation |
| EC-7 | `scope::project` in show-all mode (no key::) -> exit 1 | Invalid Combination |

## Test Coverage Summary

- Valid (default scope): 1 test
- Valid (project scope): 1 test
- Invalid Value: 1 test
- Empty Value: 1 test
- Invalid Combination: 2 tests
- Project Creation: 1 test

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (`scope::user` -> writes to user config) <-> EC-2 (`scope::project` -> writes to project config)

---

### EC-1: `scope::user` (default) writes to `~/.claude/settings.json`

- **Given:** HOME is set; `~/.claude/settings.json` accessible
- **When:** `clv .config key::theme value::dark scope::user`
- **Then:** exit 0; `~/.claude/settings.json` updated with `"theme": "dark"`; project config unchanged
- **Exit:** 0
- **Source:** [param/11_scope.md](../../../../docs/cli/param/11_scope.md)

---

### EC-2: `scope::project` writes to `{cwd}/.claude/settings.json`

- **Given:** cwd has `.claude/` directory; HOME is set
- **When:** `clv .config key::theme value::dark scope::project`
- **Then:** exit 0; `.claude/settings.json` in cwd updated with `"theme": "dark"`; `~/.claude/settings.json` unchanged
- **Exit:** 0
- **Source:** [param/11_scope.md](../../../../docs/cli/param/11_scope.md)

---

### EC-3: `scope::invalid` -> exit 1, unknown scope

- **Given:** valid `key::` and `value::` supplied
- **When:** `clv .config key::theme value::dark scope::invalid`
- **Then:** exit 1; error message references unknown scope value; no file modified
- **Exit:** 1
- **Source:** [param/11_scope.md](../../../../docs/cli/param/11_scope.md)

---

### EC-4: `scope::` (empty value) -> exit 1

- **Given:** valid `key::` and `value::` supplied
- **When:** `clv .config key::theme value::dark scope::`
- **Then:** exit 1; error: empty value not accepted for `scope::`; no file modified
- **Exit:** 1
- **Source:** [param/11_scope.md](../../../../docs/cli/param/11_scope.md)

---

### EC-5: `scope::user` without `key::` and `value::` -> exit 1

- **Given:** no `key::` or `value::` supplied (show-all invocation)
- **When:** `clv .config scope::user`
- **Then:** exit 1; error: `scope::` only applies to write operations; no output
- **Exit:** 1
- **Source:** [param/11_scope.md](../../../../docs/cli/param/11_scope.md)

---

### EC-6: `scope::project` with `key::K value::V` creates `.claude/` directory if absent

- **Given:** cwd has no `.claude/` subdirectory
- **When:** `clv .config key::theme value::dark scope::project`
- **Then:** exit 0; `.claude/` directory created in cwd; `.claude/settings.json` created with `"theme": "dark"`
- **Exit:** 0
- **Source:** [param/11_scope.md](../../../../docs/cli/param/11_scope.md)

---

### EC-7: `scope::project` in show-all mode (no key::) -> exit 1

- **Given:** no `key::` supplied; `scope::project` present
- **When:** `clv .config scope::project`
- **Then:** exit 1; error: `scope::` requires a write operation (`key::` + `value::` or `unset::1`)
- **Exit:** 1
- **Source:** [param/11_scope.md](../../../../docs/cli/param/11_scope.md)

---

### Source Functions

| Function | File |
|----------|------|
| `scope_ec1_user_writes_to_user_settings` | `tests/cli/scope_param_test.rs` |
| `scope_ec2_project_writes_to_project_settings` | `tests/cli/scope_param_test.rs` |
| `scope_ec3_invalid_scope_value_exits_1` | `tests/cli/scope_param_test.rs` |
| `scope_ec4_empty_scope_value_exits_1` | `tests/cli/scope_param_test.rs` |
| `scope_ec5_scope_without_write_op_exits_1` | `tests/cli/scope_param_test.rs` |
| `scope_ec6_project_creates_directory_when_absent` | `tests/cli/scope_param_test.rs` |
| `scope_ec7_project_in_show_all_mode_exits_1` | `tests/cli/scope_param_test.rs` |

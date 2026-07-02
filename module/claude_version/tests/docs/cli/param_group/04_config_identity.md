# Test: Config Identity Group

Interaction tests for Parameter Group 4 (Config Identity). See [param_group/04_config_identity.md](../../../../docs/cli/param_group/04_config_identity.md) for specification.

### Scope

- **Purpose**: Interaction tests for the Config Identity parameter group.
- **Responsibility**: Cross-parameter interaction coverage for `key::`, `value::`, `scope::`, and `unset::` within `.config`.
- **In Scope**: Group-level interactions, mutual exclusion, required parameter combinations.
- **Out of Scope**: Individual parameter edge cases (-> `../param/`), command-level tests (-> `../command/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `key::K value::V` -> set mode, writes to user config | Valid: set |
| CC-2 | `key::K value::V scope::project` -> set mode, writes to project config | Valid: set + scope |
| CC-3 | `key::K unset::1` -> unset mode, removes key from user config | Valid: unset |
| CC-4 | `key::K unset::1 scope::project` -> unset mode, removes from project config | Valid: unset + scope |
| CC-5 | `key::K value::V unset::1` -> exit 1, mutual exclusion | Invalid: mutual exclusion |
| CC-6 | `value::V` without `key::K` -> exit 1, key required | Invalid: missing key |
| CC-7 | `unset::1` without `key::K` -> exit 1, key required | Invalid: missing key |
| CC-8 | `scope::project` without write operation -> exit 1, scope only applies to writes | Invalid: scope without write |
| CC-9 | `key::K` alone -> get mode (no write), scope:: ignored | Valid: get mode |
| CC-10 | `key::K value::V dry::1` -> preview, no file modification | Cross-group: dry run |

## Test Coverage Summary

- Valid (set combinations): 2 tests
- Valid (unset combinations): 2 tests
- Valid (get mode): 1 test
- Invalid (mutual exclusion): 1 test
- Invalid (missing required): 2 tests
- Invalid (scope misuse): 1 test
- Cross-group (dry run): 1 test

**Total:** 10 interaction tests

---

### CC-1: `key::K value::V` -> set mode, writes to user config

- **Given:** HOME set; `~/.claude/settings.json` accessible
- **When:** `clv .config key::theme value::dark`
- **Then:** exit 0; `~/.claude/settings.json` updated with `"theme": "dark"` at user scope
- **Exit:** 0
- **Commands:** `.config`

---

### CC-2: `key::K value::V scope::project` -> set mode, writes to project config

- **Given:** cwd accessible; HOME set; user settings unchanged
- **When:** `clv .config key::theme value::dark scope::project`
- **Then:** exit 0; `.claude/settings.json` in cwd updated with `"theme": "dark"`; `~/.claude/settings.json` unchanged
- **Exit:** 0
- **Commands:** `.config`

---

### CC-3: `key::K unset::1` -> unset mode, removes key from user config

- **Given:** HOME set; `~/.claude/settings.json` contains key `K`
- **When:** `clv .config key::K unset::1`
- **Then:** exit 0; key `K` removed from `~/.claude/settings.json`; other keys unchanged
- **Exit:** 0
- **Commands:** `.config`

---

### CC-4: `key::K unset::1 scope::project` -> unset mode, removes from project config

- **Given:** cwd has `.claude/settings.json` with key `K`; user settings unchanged
- **When:** `clv .config key::K unset::1 scope::project`
- **Then:** exit 0; key `K` removed from `.claude/settings.json` in cwd; `~/.claude/settings.json` unchanged
- **Exit:** 0
- **Commands:** `.config`

---

### CC-5: `key::K value::V unset::1` -> exit 1, mutual exclusion

- **Given:** any invocation with `value::V` and `unset::1` both present
- **When:** `clv .config key::theme value::dark unset::1`
- **Then:** exit 1; error states `value::` and `unset::` are mutually exclusive; no file modified
- **Exit:** 1
- **Commands:** `.config`

---

### CC-6: `value::V` without `key::K` -> exit 1, key required

- **Given:** `value::V` supplied without `key::`
- **When:** `clv .config value::dark`
- **Then:** exit 1; error: `key::` is required; no file modified
- **Exit:** 1
- **Commands:** `.config`

---

### CC-7: `unset::1` without `key::K` -> exit 1, key required

- **Given:** `unset::1` supplied without `key::`
- **When:** `clv .config unset::1`
- **Then:** exit 1; error: `key::` is required when `unset::1`; no file modified
- **Exit:** 1
- **Commands:** `.config`

---

### CC-8: `scope::project` without write operation -> exit 1

- **Given:** `scope::project` supplied with no `key::`, `value::`, or `unset::`
- **When:** `clv .config scope::project`
- **Then:** exit 1; error: `scope::` only applies to write operations
- **Exit:** 1
- **Commands:** `.config`

---

### CC-9: `key::K` alone -> get mode (no write), scope:: ignored

- **Given:** HOME set; `~/.claude/settings.json` contains key `K`
- **When:** `clv .config key::K`
- **Then:** exit 0; effective value for `K` shown with source annotation; no write occurs
- **Exit:** 0
- **Commands:** `.config`

---

### CC-10: `key::K value::V dry::1` -> preview, no file modification

- **Given:** HOME set; `~/.claude/settings.json` accessible
- **When:** `clv .config key::theme value::dark dry::1`
- **Then:** exit 0; output shows `[dry-run]` preview of the write; settings file not modified
- **Exit:** 0
- **Commands:** `.config`

---

### Source Functions

| Function | File |
|----------|------|
| `config_identity_gi1_set_mode_writes_user_config` | `tests/cli/config_identity_test.rs` |
| `config_identity_gi2_set_mode_project_scope` | `tests/cli/config_identity_test.rs` |
| `config_identity_gi3_unset_mode_removes_from_user_config` | `tests/cli/config_identity_test.rs` |
| `config_identity_gi4_unset_mode_project_scope` | `tests/cli/config_identity_test.rs` |
| `config_identity_gi5_value_unset_mutual_exclusion_exits_1` | `tests/cli/config_identity_test.rs` |
| `config_identity_gi6_value_without_key_exits_1` | `tests/cli/config_identity_test.rs` |
| `config_identity_gi7_unset_without_key_exits_1` | `tests/cli/config_identity_test.rs` |
| `config_identity_gi8_scope_without_write_exits_1` | `tests/cli/config_identity_test.rs` |
| `config_identity_gi9_key_alone_get_mode` | `tests/cli/config_identity_test.rs` |
| `config_identity_gi10_dry_run_no_file_modification` | `tests/cli/config_identity_test.rs` |

# Test: `ConfigKey`

Type compliance and validation tests for `ConfigKey`. See [type/07_config_key.md](../../../../docs/cli/type/07_config_key.md) for specification.

### Scope

- **Purpose**: Validate ConfigKey parsing, catalog awareness, and arbitrary key acceptance.
- **Responsibility**: Non-empty string acceptance, catalog key resolution, arbitrary key passthrough, empty/missing rejection.
- **Commands:** `.config`
- **In Scope**: Key string parsing, catalog key vs arbitrary key distinction, missing and empty key rejection.
- **Out of Scope**: Resolution algorithm (-> `../../algorithm/02_config_resolution.md`), value type inference (-> `05_settings_value.md`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `key::model` -> catalog key, resolves env + default | Valid: catalog key |
| TC-2 | `key::myCustomSetting` -> arbitrary key, no catalog default | Valid: arbitrary key |
| TC-3 | `key::theme` -> catalog key, resolves user config + default | Valid: catalog key |
| TC-4 | `key::a.b.c` -> dot is literal, not path separator | Valid: dot-literal |
| TC-5 | Missing `key::` on `.config` get -> exit 0, show-all mode | Valid: absent triggers show-all |
| TC-6 | `key::` (empty) -> exit 1 | Invalid: empty |

## Test Coverage Summary

- Catalog key resolution: 2 tests (TC-1, TC-3)
- Arbitrary key passthrough: 1 test (TC-2)
- Dot-literal behavior: 1 test (TC-4)
- Absent key (show-all mode): 1 test (TC-5)
- Empty value: 1 test (TC-6)

**Total:** 6 tests

**Behavioral Divergence Pair:** TC-1 (`key::model` -> catalog key with env mapping and default `claude-sonnet-5`) <-> TC-2 (`key::myCustomSetting` -> arbitrary key with no catalog entry, no default, no env mapping)

---

### TC-1: `key::model` -> catalog key, resolves env + default

- **Given:** clean HOME; no `CLAUDE_MODEL` env var; no user config
- **When:** `clv .config key::model`
- **Then:** exit 0; output contains `claude-sonnet-5` and source annotation `(default)`
- **Exit:** 0
- **Source:** [type/07_config_key.md -- Known catalog keys](../../../../docs/cli/type/07_config_key.md)

---

### TC-2: `key::myCustomSetting` -> arbitrary key, no catalog default

- **Given:** clean HOME; no user config
- **When:** `clv .config key::myCustomSetting`
- **Then:** exit 0; output indicates absent value (no default, no env mapping)
- **Exit:** 0
- **Source:** [type/07_config_key.md -- Unknown keys accepted without error](../../../../docs/cli/type/07_config_key.md)

---

### TC-3: `key::theme` -> catalog key, resolves user config + default

- **Given:** user config contains `{"theme": "dark"}`
- **When:** `clv .config key::theme`
- **Then:** exit 0; output contains `dark` and source annotation `(user)`
- **Exit:** 0
- **Source:** [type/07_config_key.md -- Known catalog keys](../../../../docs/cli/type/07_config_key.md)

---

### TC-4: `key::a.b.c` -> dot is literal

- **Given:** user config contains `{"a.b.c": "test"}`
- **When:** `clv .config key::a.b.c`
- **Then:** exit 0; output contains `test`; dot treated as part of key name, not path separator
- **Exit:** 0
- **Source:** [type/07_config_key.md -- dot characters are literal, not path separators](../../../../docs/cli/type/07_config_key.md)

---

### TC-5: Missing `key::` on `.config` -> show-all mode

- **Given:** user config contains at least one setting
- **When:** `clv .config` (no `key::` parameter)
- **Then:** exit 0; output lists all resolved settings (show-all mode, not an error)
- **Exit:** 0
- **Source:** [type/07_config_key.md -- key:: is optional on .config (triggers show-all)](../../../../docs/cli/type/07_config_key.md)

---

### TC-6: `key::` (empty) -> exit 1

- **Given:** clean environment
- **When:** `clv .config key::`
- **Then:** exit 1; error message says "key:: value cannot be empty" or equivalent
- **Exit:** 1
- **Source:** [type/07_config_key.md -- Validation: missing or empty -> exit 1](../../../../docs/cli/type/07_config_key.md)

---

### Source Functions

| Function | File | Status |
|----------|------|--------|
| `tc01_007_config_key_catalog_default` | `integration/config_commands_test.rs` | ✅ |
| `tc02_007_config_key_arbitrary_absent` | `integration/config_commands_test.rs` | ✅ |
| `tc03_007_config_key_catalog_user_config` | `integration/config_commands_test.rs` | ✅ |
| `tc04_007_config_key_dot_literal` | `integration/config_commands_test.rs` | ✅ |
| `tc05_007_config_key_absent_show_all` | `integration/config_commands_test.rs` | ✅ |
| `tc06_007_config_key_empty_exits_1` | `integration/config_commands_test.rs` | ✅ |

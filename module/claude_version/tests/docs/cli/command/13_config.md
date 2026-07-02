# Test: `.config`

### Scope

- **Purpose**: Integration test cases for the `.config` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for all four modes (show-all, get, set, unset).
- **In Scope**: Mode dispatch, parameter combinations, exit codes, output format, scope targeting, dry-run, invalid combinations.
- **Out of Scope**: Resolution algorithm unit tests (→ `../../algorithm/02_config_resolution.md`), parameter edge cases (→ `../param/`).

Integration test planning for `.config`. See [command/config.md](../../../../docs/cli/command/config.md) for specification.

## Test Factor Analysis

### Factor 1: Mode (derived from params)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| no params | show-all mode | Default behavior |
| key only | get mode | Read-only |
| key+value | set mode | Write user scope |
| key+value+scope::project | set project scope | Write project scope |
| key+unset::1 | unset mode | Delete key |

### Factor 2: `format::` (String, optional, default text)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default text output | Default behavior |
| `text` | Explicit text | Explicit valid |
| `json` | JSON output with source fields | Alternate valid |
| `xml` | Unrecognized value | Invalid: exit 1 |

### Factor 3: `dry::` (Boolean, optional, default 0)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Real write | Default behavior |
| 1 | Preview only | Explicit true |
| 2 | Out-of-range boolean | Invalid: exit 1 |

### Factor 4: `scope::` (String, optional, default user)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent / `user` | User settings.json | Default behavior |
| `project` | Project settings.json | Alternate valid |
| `global` | Unrecognized value | Invalid: exit 1 |

### Factor 5: Invalid combinations

| Combination | Equivalence Class |
|-------------|-------------------|
| value:: without key:: | Invalid: exit 1 |
| unset::1 without key:: | Invalid: exit 1 |
| value:: and unset::1 together | Invalid: exit 1 |
| scope:: (any value) without write operation | Invalid: exit 1 — covered by scope param tests EC-5, EC-7 (see `../param/011_scope.md`) and config identity GI-8 |

### Factor 6: HOME environment

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| set | Normal | Happy path |
| empty | Cannot resolve path | Failure: exit 2 |

---

## Test Matrix

### Positive Tests

| TC | Description | Mode | Exit | Factors |
|----|-------------|------|------|---------|
| IT-1 | No params → show-all with source labels | show-all | 0 | F1=no-params |
| IT-2 | `key::theme` → get with source annotation | get | 0 | F1=key-only |
| IT-3 | `key::theme value::dark` → set user, bool inferred | set | 0 | F1=key+value |
| IT-4 | `key::model value::claude-opus-4-8 scope::project` → project write | set | 0 | F1=scope-project, F4=project |
| IT-5 | `key::theme unset::1` → key removed from user settings | unset | 0 | F1=unset |
| IT-6 | `format::json` → JSON with source fields | show-all | 0 | F2=json |
| IT-7 | `key::model` with `CLAUDE_MODEL` set → shows env value | get | 0 | F1=key-only |
| IT-8 | `key::unknownArbitraryKey value::v` → accepted, written | set | 0 | F1=key+value |
| IT-9 | `key::model` no env/config → shows catalog default | get | 0 | F1=key-only |
| IT-10 | `key::theme value::dark dry::1` → preview, no write | set | 0 | F3=1 |

### Negative Tests

| TC | Description | Mode | Exit | Factors |
|----|-------------|------|------|---------|
| IT-11 | `value::v` without `key::` → exit 1 | — | 1 | F5=value-without-key |
| IT-12 | `unset::1` without `key::` → exit 1 | — | 1 | F5=unset-without-key |
| IT-13 | `value::v unset::1 key::k` → exit 1 (mutually exclusive) | — | 1 | F5=value+unset |
| IT-14 | `scope::global` → exit 1 (invalid value) | — | 1 | F4=global |
| IT-15 | `format::xml` → exit 1 | — | 1 | F2=xml |
| IT-16 | `HOME` unset → exit 2 | — | 2 | F6=empty |
| IT-17 | `dry::2` → exit 1, out-of-range | — | 1 | F3=2 |

### Summary

- **Total:** 17 tests (10 positive, 7 negative)
- **Negative ratio:** 41.2% ✅ (≥40%)
- **TC range:** IT-1 to IT-17

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success | IT-1 through IT-10 |
| 1 | Invalid arguments | IT-11 through IT-15, IT-17 |
| 2 | Runtime error | IT-16 |

### Mode Coverage

| Mode | Tests |
|------|-------|
| show-all | IT-1, IT-6 |
| get | IT-2, IT-7, IT-9 |
| set (user) | IT-3, IT-8, IT-10 |
| set (project) | IT-4 |
| unset | IT-5 |

---

## Test Case Details

---

### IT-1: No params → show-all with source labels

- **Given:** `HOME=<tmp>` with `~/.claude/settings.json` containing at least one setting
- **When:** `clv .config`
- **Then:** exit 0; output lists all settings with source labels (e.g., `theme: dark (user)`)
- **Exit:** 0
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-2: `key::theme` → get with source annotation

- **Given:** `HOME=<tmp>` with `~/.claude/settings.json` containing `{"theme": "dark"}`
- **When:** `clv .config key::theme`
- **Then:** exit 0; output contains `dark` and a source annotation (e.g., `(user)`)
- **Exit:** 0
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-3: `key::theme value::dark` → set user config

- **Given:** `HOME=<tmp>` with `~/.claude/settings.json` existing (may be empty)
- **When:** `clv .config key::theme value::dark`
- **Then:** exit 0; `~/.claude/settings.json` contains `"theme": "dark"`
- **Exit:** 0
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-4: `key::model value::claude-opus-4-8 scope::project` → project write

- **Given:** `HOME=<tmp>`; cwd accessible for `.claude/settings.json` write
- **When:** `clv .config key::model value::claude-opus-4-8 scope::project`
- **Then:** exit 0; `{cwd}/.claude/settings.json` contains `"model": "claude-opus-4-8"`; user config unchanged
- **Exit:** 0
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-5: `key::theme unset::1` → key removed from user settings

- **Given:** `HOME=<tmp>` with `~/.claude/settings.json` containing `{"theme": "dark"}`
- **When:** `clv .config key::theme unset::1`
- **Then:** exit 0; `~/.claude/settings.json` no longer contains `"theme"` key
- **Exit:** 0
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-6: `format::json` → JSON with source fields

- **Given:** `HOME=<tmp>` with `~/.claude/settings.json` containing at least one setting
- **When:** `clv .config format::json`
- **Then:** exit 0; stdout is valid JSON; each entry includes key, value, and source fields
- **Exit:** 0
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-7: `key::model` with `CLAUDE_MODEL` set → shows env value

- **Given:** `HOME=<tmp>`; `CLAUDE_MODEL=claude-opus-4-8` in environment; user config omits `model`
- **When:** `clv .config key::model`
- **Then:** exit 0; output shows `claude-opus-4-8` with source annotation `(env)` or `(environment)`
- **Exit:** 0
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-8: `key::unknownArbitraryKey value::v` → accepted, written

- **Given:** `HOME=<tmp>` with `~/.claude/settings.json` containing `{}`
- **When:** `clv .config key::unknownArbitraryKey value::v`
- **Then:** exit 0; `~/.claude/settings.json` contains `"unknownArbitraryKey": "v"`; unknown keys accepted without error
- **Exit:** 0
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-9: `key::model` no env/config → shows catalog default

- **Given:** `HOME=<tmp>` with empty `~/.claude/settings.json`; `CLAUDE_MODEL` unset
- **When:** `clv .config key::model`
- **Then:** exit 0; output shows `claude-sonnet-5` (catalog default) with source annotation `(default)`
- **Exit:** 0
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-10: `key::theme value::dark dry::1` → preview, no write

- **Given:** `HOME=<tmp>` with `~/.claude/settings.json` containing `{}`
- **When:** `clv .config key::theme value::dark dry::1`
- **Then:** exit 0; stdout shows preview with `[dry-run]` marker; `settings.json` unchanged
- **Exit:** 0
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-11: `value::v` without `key::` → exit 1

- **Given:** clean environment
- **When:** `clv .config value::v`
- **Then:** exit 1; error indicates `key::` is required when `value::` is provided
- **Exit:** 1
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-12: `unset::1` without `key::` → exit 1

- **Given:** clean environment
- **When:** `clv .config unset::1`
- **Then:** exit 1; error indicates `key::` is required when `unset::` is provided
- **Exit:** 1
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-13: `value::v unset::1 key::k` → exit 1 (mutually exclusive)

- **Given:** clean environment
- **When:** `clv .config key::k value::v unset::1`
- **Then:** exit 1; error indicates `value::` and `unset::` are mutually exclusive
- **Exit:** 1
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-14: `scope::global` → exit 1 (invalid value)

- **Given:** clean environment
- **When:** `clv .config scope::global`
- **Then:** exit 1; error indicates `scope::` must be `user` or `project`
- **Exit:** 1
- **Source:** [type/06_config_scope.md](../../../../docs/cli/type/06_config_scope.md)

---

### IT-15: `format::xml` → exit 1

- **Given:** clean environment
- **When:** `clv .config format::xml`
- **Then:** exit 1; error references invalid format value
- **Exit:** 1
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-16: `HOME` unset → exit 2

- **Given:** `HOME` environment variable unset or empty
- **When:** `clv .config`
- **Then:** exit 2; error indicates HOME is required to resolve settings path
- **Exit:** 2
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### IT-17: `dry::2` → exit 1, out-of-range

- **Given:** clean environment
- **When:** `clv .config dry::2`
- **Then:** exit 1; error indicates `dry::` value is out of range (must be 0 or 1)
- **Exit:** 1
- **Source:** [command/config.md](../../../../docs/cli/command/config.md)

---

### Source Functions

| Function | File |
|----------|------|
| `it01_config_show_all_source_labels` | `integration/config_commands_test.rs` |
| `it02_config_get_shows_source_annotation` | `integration/config_commands_test.rs` |
| `it03_config_set_user_scope` | `integration/config_commands_test.rs` |
| `it04_config_set_project_scope` | `integration/config_commands_test.rs` |
| `it05_config_unset_removes_key` | `integration/config_commands_test.rs` |
| `it06_config_show_all_json_format` | `integration/config_commands_test.rs` |
| `it07_config_get_env_override` | `integration/config_commands_test.rs` |
| `it08_config_arbitrary_key_accepted` | `integration/config_commands_test.rs` |
| `it09_config_catalog_default_model` | `integration/config_commands_test.rs` |
| `it10_config_set_dry_run_no_write` | `integration/config_commands_test.rs` |
| `it11_config_value_without_key_exits_1` | `integration/config_commands_test.rs` |
| `it12_config_unset_without_key_exits_1` | `integration/config_commands_test.rs` |
| `it13_config_value_and_unset_together_exits_1` | `integration/config_commands_test.rs` |
| `it14_config_invalid_scope_exits_1` | `integration/config_commands_test.rs` |
| `it15_config_invalid_format_exits_1` | `integration/config_commands_test.rs` |
| `it16_config_home_unset_exits_2` | `integration/config_commands_test.rs` |
| `it17_config_dry_out_of_range_exits_1` | `integration/config_commands_test.rs` |

# Feature Test: Config Command

### Scope

- **Purpose**: FT- test cases for the `.config` command — show-all, get, set, unset modes, 4-layer resolution, scope, dry-run, and error paths.
- **Responsibility**: Acceptance criteria verifying effective-value resolution, catalog defaults, env var overrides, project/user scope, unset, and HOME dependency.
- **In Scope**: All four `.config` modes, resolution chain, catalog keys, scope:: parameter, unset:: parameter, dry::1, format::json, exit codes.
- **Out of Scope**: Type inference algorithm (→ `../../algorithm/01_settings_type_inference.md`), resolution algorithm step-by-step (→ `../../algorithm/02_config_resolution.md`), deprecated `.settings.*` commands (→ `03_settings_management.md`).

Feature test surface for `.config`. See [feature/006_config_command.md](../../../docs/feature/006_config_command.md) for specification.

## Behavioral Divergence Pair

Two valid `.config` invocations that produce structurally different output:

- **Input A:** `clv .config` (no params) → shows all resolved keys with source annotations
- **Input B:** `clv .config key::model` → shows single key effective value with source layer

Both are valid; the scope of resolution differs.

## Test Case Index

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-1 | AC-1 | `.config` (no params) prints resolved settings in text format | ✅ `ft1_006_config_show_all_text` |
| FT-2 | AC-2 | `.config key::K` prints value with source layer annotation | ✅ `ft2_006_config_get_shows_source` |
| FT-3 | AC-3 | `.config key::K value::V` writes to user settings.json with type inference | ✅ `ft3_006_config_set_user_scope` |
| FT-4 | AC-4 | `.config key::K value::V scope::project` writes to project settings.json | ✅ `ft4_006_config_set_project_scope` |
| FT-5 | AC-5 | `.config key::K unset::1` removes key from user settings | ✅ `ft5_006_config_unset_removes_key` |
| FT-6 | AC-6 | `.config format::json` returns resolved settings as JSON with source fields | ✅ `ft6_006_config_show_all_json` |
| FT-7 | AC-7 | Env var (CLAUDE_MODEL) overrides project and user config for `model` key | ✅ `ft7_006_config_env_overrides_user` |
| FT-8 | AC-8 | `.config key::K` absent everywhere → exit 0 with absent indicator | ✅ `ft8_006_config_get_absent_key` |
| FT-9 | AC-9 | `.config key::K value::V dry::1` → preview, no file change | ✅ `ft9_006_config_set_dry_run` |
| FT-10 | AC-10 | HOME unset → exit 2 for any filesystem operation | ✅ `ft10_006_config_home_unset_exits_2` |
| FT-11 | AC-11 | Non-catalog key is accepted and written without error | ✅ `ft11_006_config_arbitrary_key_accepted` |
| FT-12 | AC-12 | Catalog default for `model` is `claude-sonnet-5` when no env or config | ✅ `ft12_006_config_catalog_default_model` |

## Test Coverage Summary

- Show-all mode: 2 tests (FT-1, FT-6)
- Get mode: 3 tests (FT-2, FT-8, FT-12)
- Set mode: 3 tests (FT-3, FT-4, FT-11)
- Unset mode: 1 test (FT-5)
- Dry-run: 1 test (FT-9)
- Error path: 1 test (FT-10)
- Resolution priority: 1 test (FT-7)

**Total:** 12 tests

---

### FT-1: show-all prints resolved settings in text format

- **Given:** isolated HOME with `settings.json` containing `{"theme": "dark"}`; no project config; `CLAUDE_MODEL` unset
- **When:** `clv .config`
- **Then:** stdout contains `model` with default value `claude-sonnet-5` (source: default) and `theme` with value `dark` (source: user); exit 0
- **Exit:** 0
- **Source:** [feature/006_config_command.md — AC-1](../../../docs/feature/006_config_command.md)

---

### FT-2: get shows effective value with source layer

- **Given:** isolated HOME; `settings.json` contains `{"theme": "light"}`; no project config
- **When:** `clv .config key::theme`
- **Then:** stdout contains `light` and `(user)` source annotation; exit 0
- **Exit:** 0
- **Source:** [feature/006_config_command.md — AC-2](../../../docs/feature/006_config_command.md)

---

### FT-3: set writes to user settings with type inference

- **Given:** isolated HOME; empty `settings.json`
- **When:** `clv .config key::autoUpdates value::false`
- **Then:** `settings.json` contains `"autoUpdates": false` (JSON bool); exit 0
- **Exit:** 0
- **Source:** [feature/006_config_command.md — AC-3](../../../docs/feature/006_config_command.md)

---

### FT-4: set with scope::project writes to project settings

- **Given:** isolated HOME and working directory with no `.claude/settings.json`
- **When:** `clv .config key::model value::claude-haiku-4-5-20251001 scope::project`
- **Then:** `{cwd}/.claude/settings.json` contains `"model": "claude-haiku-4-5-20251001"`; user `~/.claude/settings.json` unchanged; exit 0
- **Exit:** 0
- **Source:** [feature/006_config_command.md — AC-4](../../../docs/feature/006_config_command.md)

---

### FT-5: unset removes key from user settings

- **Given:** isolated HOME; `settings.json` contains `{"theme": "dark"}`
- **When:** `clv .config key::theme unset::1`
- **Then:** `settings.json` no longer contains `theme` key; other keys preserved; exit 0
- **Exit:** 0
- **Source:** [feature/006_config_command.md — AC-5](../../../docs/feature/006_config_command.md)

---

### FT-6: format::json returns resolved settings with source fields

- **Given:** isolated HOME; `settings.json` contains `{"theme": "dark"}`; `CLAUDE_MODEL` unset
- **When:** `clv .config format::json`
- **Then:** stdout is valid JSON object containing `"model"` with `"source": "default"` and `"theme"` with `"source": "user"`; exit 0
- **Exit:** 0
- **Source:** [feature/006_config_command.md — AC-6](../../../docs/feature/006_config_command.md)

---

### FT-7: env var overrides user config for model key

- **Given:** isolated HOME; `settings.json` contains `{"model": "claude-sonnet-5"}`; `CLAUDE_MODEL=claude-opus-4-8` set in env
- **When:** `clv .config key::model`
- **Then:** stdout shows `claude-opus-4-8` with `(env)` source annotation; exit 0
- **Exit:** 0
- **Source:** [feature/006_config_command.md — AC-7](../../../docs/feature/006_config_command.md)

---

### FT-8: absent key shows absent indicator, exit 0

- **Given:** isolated HOME; empty `settings.json`; `CLAUDE_MODEL` unset
- **When:** `clv .config key::hasCompletedOnboarding`
- **Then:** stdout shows `false` with `(default)` source annotation (catalog default); exit 0
- **Exit:** 0
- **Source:** [feature/006_config_command.md — AC-8](../../../docs/feature/006_config_command.md)

---

### FT-9: dry::1 previews set, no file change

- **Given:** isolated HOME; `settings.json` contains `{"theme": "light"}`
- **When:** `clv .config key::theme value::dark dry::1`
- **Then:** stdout contains preview indicator; `settings.json` still contains `"theme": "light"`; exit 0
- **Exit:** 0
- **Source:** [feature/006_config_command.md — AC-9](../../../docs/feature/006_config_command.md)

---

### FT-10: HOME unset → exit 2

- **Given:** environment with `HOME` unset
- **When:** `clv .config key::theme`
- **Then:** exit 2 (runtime error, HOME missing)
- **Exit:** 2
- **Source:** [feature/006_config_command.md — AC-10](../../../docs/feature/006_config_command.md)

---

### FT-11: non-catalog key accepted and written

- **Given:** isolated HOME; empty `settings.json`
- **When:** `clv .config key::myCustomKey value::customValue`
- **Then:** `settings.json` contains `"myCustomKey": "customValue"`; exit 0
- **Exit:** 0
- **Source:** [feature/006_config_command.md — AC-11](../../../docs/feature/006_config_command.md)

---

### FT-12: catalog default for model is claude-sonnet-5

- **Given:** isolated HOME; empty `settings.json`; `CLAUDE_MODEL` unset; no project config
- **When:** `clv .config key::model`
- **Then:** stdout shows `claude-sonnet-5` with `(default)` annotation; exit 0
- **Exit:** 0
- **Source:** [feature/006_config_command.md — AC-12](../../../docs/feature/006_config_command.md)

---

### Source Functions

| Function | File |
|----------|------|
| `ft1_006_config_show_all_text` | ✅ `config_commands_test.rs` |
| `ft2_006_config_get_shows_source` | ✅ `config_commands_test.rs` |
| `ft3_006_config_set_user_scope` | ✅ `config_commands_test.rs` |
| `ft4_006_config_set_project_scope` | ✅ `config_commands_test.rs` |
| `ft5_006_config_unset_removes_key` | ✅ `config_commands_test.rs` |
| `ft6_006_config_show_all_json` | ✅ `config_commands_test.rs` |
| `ft7_006_config_env_overrides_user` | ✅ `config_commands_test.rs` |
| `ft8_006_config_get_absent_key` | ✅ `config_commands_test.rs` |
| `ft9_006_config_set_dry_run` | ✅ `config_commands_test.rs` |
| `ft10_006_config_home_unset_exits_2` | ✅ `config_commands_test.rs` |
| `ft11_006_config_arbitrary_key_accepted` | ✅ `config_commands_test.rs` |
| `ft12_006_config_catalog_default_model` | ✅ `config_commands_test.rs` |

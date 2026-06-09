# Algorithm Test: Config Resolution

### Scope

- **Purpose**: Test cases for the 4-layer config resolution algorithm.
- **Responsibility**: Verify layer priority, env var mapping, project config ancestor search, catalog defaults, and source annotation.
- **In Scope**: All 4 resolution layers, absent-key semantics, show-all union, source annotation values.
- **Out of Scope**: `.config` command handler integration (→ `../../feature/006_config_command.md`), type inference (→ `001_settings_type_inference.md`).

Test surface for `claude_version_core::config_resolve`. See [algorithm/002_config_resolution.md](../../../../docs/algorithm/002_config_resolution.md) for specification.

## Test Case Index

| AT | Scenario | Source fn |
|----|----------|-----------|
| AT-01 | Env var present → source=Env, overrides user config | ⏳ `at01_002_env_overrides_user` |
| AT-02 | Env var absent, key in user config → source=User | ⏳ `at02_002_user_config_wins_without_env` |
| AT-03 | Key in project config, not in user config → source=Project | ⏳ `at03_002_project_config_key` |
| AT-04 | Key only in catalog defaults → source=Default | ⏳ `at04_002_catalog_default_returned` |
| AT-05 | Key absent everywhere → source=Absent, value=None | ⏳ `at05_002_all_layers_absent` |
| AT-06 | Project config overrides user config when both have key | ⏳ `at06_002_project_overrides_user` |

**Total:** 6 tests

---

### AT-01: env var overrides user config

- **Given:** `CLAUDE_MODEL=claude-opus-4-6`; user settings has `{"model": "claude-sonnet-4-6"}`
- **When:** resolve(`model`)
- **Then:** `ResolvedValue { value: Some("claude-opus-4-6"), source: Env }`
- **Source:** [algorithm/002_config_resolution.md — Step 1](../../../../docs/algorithm/002_config_resolution.md)

---

### AT-02: user config wins when env absent

- **Given:** `CLAUDE_MODEL` unset; user settings has `{"model": "claude-haiku-4-5-20251001"}`
- **When:** resolve(`model`)
- **Then:** `ResolvedValue { value: Some("claude-haiku-4-5-20251001"), source: User }`
- **Source:** [algorithm/002_config_resolution.md — Step 3](../../../../docs/algorithm/002_config_resolution.md)

---

### AT-03: project config key returned

- **Given:** `CLAUDE_MODEL` unset; project settings has `{"model": "claude-opus-4-6"}`; user settings empty
- **When:** resolve(`model`)
- **Then:** `ResolvedValue { value: Some("claude-opus-4-6"), source: Project }`
- **Source:** [algorithm/002_config_resolution.md — Step 2](../../../../docs/algorithm/002_config_resolution.md)

---

### AT-04: catalog default returned when all layers absent

- **Given:** `CLAUDE_MODEL` unset; no project config; user settings empty
- **When:** resolve(`model`)
- **Then:** `ResolvedValue { value: Some("claude-sonnet-4-6"), source: Default }`
- **Source:** [algorithm/002_config_resolution.md — Step 4](../../../../docs/algorithm/002_config_resolution.md)

---

### AT-05: absent everywhere → None

- **Given:** no env mapping; no project config; no user config; no catalog default for key
- **When:** resolve(`myArbitraryKey`)
- **Then:** `ResolvedValue { value: None, source: Absent }`
- **Source:** [algorithm/002_config_resolution.md — Step 4](../../../../docs/algorithm/002_config_resolution.md)

---

### AT-06: project config overrides user config

- **Given:** `CLAUDE_MODEL` unset; project settings has `{"theme": "dark"}`; user settings has `{"theme": "light"}`
- **When:** resolve(`theme`)
- **Then:** `ResolvedValue { value: Some("dark"), source: Project }`
- **Source:** [algorithm/002_config_resolution.md — Step 2](../../../../docs/algorithm/002_config_resolution.md)

---

### Source Functions

| Function | File |
|----------|------|
| `at01_002_env_overrides_user` | ⏳ TBD |
| `at02_002_user_config_wins_without_env` | ⏳ TBD |
| `at03_002_project_config_key` | ⏳ TBD |
| `at04_002_catalog_default_returned` | ⏳ TBD |
| `at05_002_all_layers_absent` | ⏳ TBD |
| `at06_002_project_overrides_user` | ⏳ TBD |

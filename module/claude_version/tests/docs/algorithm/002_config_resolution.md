# Algorithm Test: Config Resolution

### Scope

- **Purpose**: Test cases for the 4-layer config resolution algorithm.
- **Responsibility**: Verify layer priority, env var mapping, project config ancestor search, catalog defaults, and source annotation.
- **In Scope**: All 4 resolution layers, absent-key semantics, show-all union, source annotation values.
- **Out of Scope**: `.config` command handler integration (→ `../../feature/006_config_command.md`), type inference (→ `001_settings_type_inference.md`).

Test surface for `claude_version_core::config_resolve`. See [algorithm/002_config_resolution.md](../../../../docs/algorithm/002_config_resolution.md) for specification.

## Test Case Index

| AC | Scenario | Source fn |
|----|----------|-----------|
| AC-1 | Env var present → source=Env, overrides user config | ⏳ `ac01_002_env_overrides_user` |
| AC-2 | Env var absent, key in user config → source=User | ⏳ `ac02_002_user_config_wins_without_env` |
| AC-3 | Key in project config, not in user config → source=Project | ⏳ `ac03_002_project_config_key` |
| AC-4 | Key only in catalog defaults → source=Default | ⏳ `ac04_002_catalog_default_returned` |
| AC-5 | Key absent everywhere → source=Absent, value=None | ⏳ `ac05_002_all_layers_absent` |
| AC-6 | Project config overrides user config when both have key | ⏳ `ac06_002_project_overrides_user` |

**Total:** 6 tests

---

### AC-1: env var overrides user config

- **Given:** `CLAUDE_MODEL=claude-opus-4-6`; user settings has `{"model": "claude-sonnet-4-6"}`
- **When:** resolve(`model`)
- **Then:** `ResolvedValue { value: Some("claude-opus-4-6"), source: Env }`
- **Source:** [algorithm/002_config_resolution.md — Step 1](../../../../docs/algorithm/002_config_resolution.md)

---

### AC-2: user config wins when env absent

- **Given:** `CLAUDE_MODEL` unset; user settings has `{"model": "claude-haiku-4-5-20251001"}`
- **When:** resolve(`model`)
- **Then:** `ResolvedValue { value: Some("claude-haiku-4-5-20251001"), source: User }`
- **Source:** [algorithm/002_config_resolution.md — Step 3](../../../../docs/algorithm/002_config_resolution.md)

---

### AC-3: project config key returned

- **Given:** `CLAUDE_MODEL` unset; project settings has `{"model": "claude-opus-4-6"}`; user settings empty
- **When:** resolve(`model`)
- **Then:** `ResolvedValue { value: Some("claude-opus-4-6"), source: Project }`
- **Source:** [algorithm/002_config_resolution.md — Step 2](../../../../docs/algorithm/002_config_resolution.md)

---

### AC-4: catalog default returned when all layers absent

- **Given:** `CLAUDE_MODEL` unset; no project config; user settings empty
- **When:** resolve(`model`)
- **Then:** `ResolvedValue { value: Some("claude-sonnet-4-6"), source: Default }`
- **Source:** [algorithm/002_config_resolution.md — Step 4](../../../../docs/algorithm/002_config_resolution.md)

---

### AC-5: absent everywhere → None

- **Given:** no env mapping; no project config; no user config; no catalog default for key
- **When:** resolve(`myArbitraryKey`)
- **Then:** `ResolvedValue { value: None, source: Absent }`
- **Source:** [algorithm/002_config_resolution.md — Step 4](../../../../docs/algorithm/002_config_resolution.md)

---

### AC-6: project config overrides user config

- **Given:** `CLAUDE_MODEL` unset; project settings has `{"theme": "dark"}`; user settings has `{"theme": "light"}`
- **When:** resolve(`theme`)
- **Then:** `ResolvedValue { value: Some("dark"), source: Project }`
- **Source:** [algorithm/002_config_resolution.md — Step 2](../../../../docs/algorithm/002_config_resolution.md)

---

### Source Functions

| Function | File |
|----------|------|
| `ac01_002_env_overrides_user` | ⏳ TBD |
| `ac02_002_user_config_wins_without_env` | ⏳ TBD |
| `ac03_002_project_config_key` | ⏳ TBD |
| `ac04_002_catalog_default_returned` | ⏳ TBD |
| `ac05_002_all_layers_absent` | ⏳ TBD |
| `ac06_002_project_overrides_user` | ⏳ TBD |

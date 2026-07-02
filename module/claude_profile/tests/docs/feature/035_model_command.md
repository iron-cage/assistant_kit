# FT — Feature 035: Dedicated Model Get/Set Command

### Scope

- **Purpose**: Test cases for the `.model` get/set command.
- **Source**: `docs/feature/035_model_command.md`
- **Covers**: AC-01 through AC-12

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | Get: `{"model":"opus"}` in settings.json → prints `model: opus` | ✅ `ft01_get_model_opus` |
| FT-02 | AC-02 | Get: `{"model":"sonnet"}` in settings.json → prints `model: sonnet` | ✅ `ft02_get_model_sonnet` |
| FT-03 | AC-03 | Get: model key absent → prints `model: (unset)` | ✅ `ft03_get_model_unset_key_absent` |
| FT-04 | AC-04 | Get: settings.json absent → prints `model: (unset)` | ✅ `ft04_get_model_unset_file_absent` |
| FT-05 | AC-05 | `set::opus` → writes `claude-opus-4-8` | ✅ `ft05_set_opus_writes_full_id` |
| FT-06 | AC-06 | `set::sonnet` → writes `claude-sonnet-5` | ✅ `ft06_set_sonnet_writes_full_id` |
| FT-07 | AC-07 | `set::haiku` → writes `claude-haiku-4-5-20251001` | ✅ `ft07_set_haiku_writes_full_id` |
| FT-08 | AC-08 | `set::default` → removes model key; other keys preserved | ✅ `ft08_set_default_removes_key_preserves_others` |
| FT-09 | AC-09 | `set::bad` → exit 1, stderr names all four valid values | ✅ `ft09_set_bad_value_exits_1` |
| FT-10 | AC-10 | Set on absent settings.json → file created with model key | ✅ `ft10_set_creates_file_when_absent` |
| FT-11 | AC-11 | Set on settings.json with other keys → all keys preserved | ✅ `ft11_set_preserves_existing_keys` |
| FT-12 | AC-12 | Get `format::json` → `{"model":"opus"}` or `{"model":null}` | ✅ `ft12_get_json_format` |

### Notes

- All FT cases are integration tests in `tests/cli/model_test.rs`.
- AC-13 (`.model` listed in `clp .help`) is verified at the command level (IT-13 in `tests/docs/cli/command/17_model.md`) — no dedicated feature-level test.
- AC-14 (no-duplication) is an architectural constraint verified by code review — no dedicated runtime test.
- FT-03 and FT-04 require an isolated `~/.claude/` home fixture without a model key.
- FT-08 and FT-11 require seeded `settings.json` with pre-existing keys (e.g., `{"theme":"dark"}`).
- FT-09 does not require a credential store (parameter validation fires before any store access).

---

### FT-01: Get returns `model: opus` when settings.json contains opus

- **Given:** `~/.claude/settings.json` exists and contains `{"model": "opus"}`.
- **When:** `clp .model`
- **Then:** Stdout is `model: opus\n`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft01_get_model_opus`
- **Source:** [035_model_command.md AC-01](../../../docs/feature/035_model_command.md)

---

### FT-02: Get returns `model: sonnet`

- **Given:** `~/.claude/settings.json` exists and contains `{"model": "sonnet"}`.
- **When:** `clp .model`
- **Then:** Stdout is `model: sonnet\n`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft02_get_model_sonnet`
- **Source:** [035_model_command.md AC-02](../../../docs/feature/035_model_command.md)

---

### FT-03: Get returns `model: (unset)` when key absent

- **Given:** `~/.claude/settings.json` exists but contains `{}` (no model key).
- **When:** `clp .model`
- **Then:** Stdout is `model: (unset)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft03_get_model_unset_key_absent`
- **Source:** [035_model_command.md AC-03](../../../docs/feature/035_model_command.md)

---

### FT-04: Get returns `model: (unset)` when file absent

- **Given:** `~/.claude/settings.json` does not exist.
- **When:** `clp .model`
- **Then:** Stdout is `model: (unset)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft04_get_model_unset_file_absent`
- **Source:** [035_model_command.md AC-04](../../../docs/feature/035_model_command.md)

---

### FT-05: `set::opus` writes `claude-opus-4-8`

- **Given:** Any `settings.json` state (may or may not exist).
- **When:** `clp .model set::opus`
- **Then:** `~/.claude/settings.json` contains `"model": "claude-opus-4-8"`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft05_set_opus_writes_full_id`
- **Source:** [035_model_command.md AC-05](../../../docs/feature/035_model_command.md)

---

### FT-06: `set::sonnet` writes `claude-sonnet-5`

- **Given:** Any `settings.json` state.
- **When:** `clp .model set::sonnet`
- **Then:** `~/.claude/settings.json` contains `"model": "claude-sonnet-5"`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft06_set_sonnet_writes_full_id`
- **Source:** [035_model_command.md AC-06](../../../docs/feature/035_model_command.md)

---

### FT-07: `set::haiku` writes `claude-haiku-4-5-20251001`

- **Given:** Any `settings.json` state.
- **When:** `clp .model set::haiku`
- **Then:** `~/.claude/settings.json` contains `"model": "claude-haiku-4-5-20251001"`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft07_set_haiku_writes_full_id`
- **Source:** [035_model_command.md AC-07](../../../docs/feature/035_model_command.md)

---

### FT-08: `set::default` removes model key; other keys preserved

- **Given:** `~/.claude/settings.json` contains `{"model": "claude-opus-4-8", "theme": "dark"}`.
- **When:** `clp .model set::default`
- **Then:** `settings.json` no longer contains `"model"` key. `"theme": "dark"` is preserved. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft08_set_default_removes_key_preserves_others`
- **Source:** [035_model_command.md AC-08](../../../docs/feature/035_model_command.md)

---

### FT-09: `set::bad` exits 1 with all valid values named in stderr

- **Given:** Any environment.
- **When:** `clp .model set::bad`
- **Then:** Exits 1. Stderr contains each of: `opus`, `sonnet`, `haiku`, `default`.
- **Exit:** 1
- **Source fn:** ✅ `ft09_set_bad_value_exits_1`
- **Source:** [035_model_command.md AC-09](../../../docs/feature/035_model_command.md)

---

### FT-10: Set creates `settings.json` when absent

- **Given:** `~/.claude/settings.json` does not exist.
- **When:** `clp .model set::opus`
- **Then:** `~/.claude/settings.json` is created; contains `"model": "claude-opus-4-8"`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft10_set_creates_file_when_absent`
- **Source:** [035_model_command.md AC-10](../../../docs/feature/035_model_command.md)

---

### FT-11: Set preserves existing settings.json keys

- **Given:** `~/.claude/settings.json` contains `{"theme": "dark", "autoUpdaterStatus": "disabled"}`.
- **When:** `clp .model set::opus`
- **Then:** All pre-existing keys preserved; `"model": "claude-opus-4-8"` added. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft11_set_preserves_existing_keys`
- **Source:** [035_model_command.md AC-11](../../../docs/feature/035_model_command.md)

---

### FT-12: Get `format::json` returns structured JSON

- **Given:** `~/.claude/settings.json` contains `{"model": "opus"}`.
- **When:** `clp .model format::json`
- **Then:** Stdout is `{"model":"opus"}` (or valid JSON equivalent). Exits 0.
- **Variant:** When model absent → `{"model":null}`.
- **Exit:** 0
- **Source fn:** ✅ `ft12_get_json_format`
- **Source:** [035_model_command.md AC-12](../../../docs/feature/035_model_command.md)

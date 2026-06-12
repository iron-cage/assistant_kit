# Command Tests :: `.model`

### Scope

- **Purpose**: Integration test cases for the `.model` get/set command.
- **Source**: `docs/cli/command/007_model.md`, `docs/feature/035_model_command.md`
- **Covers**: AC-01 through AC-13

### Test Cases

| IT | AC | Scenario | Source fn |
|----|----|-----------|-|
| IT-01 | AC-01 | Get: settings.json has `{"model":"opus"}` → prints `model: opus` | `ft01_get_model_opus` |
| IT-02 | AC-02 | Get: settings.json has `{"model":"sonnet"}` → prints `model: sonnet` | `ft02_get_model_sonnet` |
| IT-03 | AC-03 | Get: model key absent → prints `model: (unset)` | `ft03_get_model_unset_key_absent` |
| IT-04 | AC-04 | Get: settings.json does not exist → prints `model: (unset)` | `ft04_get_model_unset_file_absent` |
| IT-05 | AC-05 | `set::opus` → writes `claude-opus-4-6` to settings.json | `ft05_set_opus_writes_full_id` |
| IT-06 | AC-06 | `set::sonnet` → writes `claude-sonnet-4-6` | `ft06_set_sonnet_writes_full_id` |
| IT-07 | AC-07 | `set::haiku` → writes `claude-haiku-4-5-20251001` | `ft07_set_haiku_writes_full_id` |
| IT-08 | AC-08 | `set::default` → removes model key; other keys preserved | `ft08_set_default_removes_key_preserves_others` |
| IT-09 | AC-09 | `set::bad` → exit 1, stderr names all 4 valid values | `ft09_set_bad_value_exits_1` |
| IT-10 | AC-10 | Set on absent settings.json → file created with model key | `ft10_set_creates_file_when_absent` |
| IT-11 | AC-11 | Set on settings.json with other keys → all keys preserved | `ft11_set_preserves_existing_keys` |
| IT-12 | AC-12 | Get `format::json` → `{"model":"opus"}` or `{"model":null}` | `ft12_get_json_format` |
| IT-13 | AC-13 | `.model` appears in `clp .help` output | `it13_model_listed_in_help_output` |

### Notes

- All IT cases are integration tests in `tests/cli/model_test.rs`.
- IT-03 and IT-04 require an isolated `~/.claude/` home fixture without a model key.
- IT-08 and IT-11 require seeded `settings.json` with pre-existing keys.
- IT-09 does not require a credential store — parameter validation fires first.

---

### IT-01: Get returns `model: opus`

- **Given:** `~/.claude/settings.json` exists and contains `{"model": "opus"}`.
- **When:** `clp .model`
- **Then:** Stdout is `model: opus\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft01_get_model_opus`
- **Source:** [035_model_command.md AC-01](../../../../docs/feature/035_model_command.md)

---

### IT-02: Get returns `model: sonnet`

- **Given:** `~/.claude/settings.json` exists and contains `{"model": "sonnet"}`.
- **When:** `clp .model`
- **Then:** Stdout is `model: sonnet\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft02_get_model_sonnet`
- **Source:** [035_model_command.md AC-02](../../../../docs/feature/035_model_command.md)

---

### IT-03: Get returns `model: (unset)` when key absent

- **Given:** `~/.claude/settings.json` exists but has no `"model"` key.
- **When:** `clp .model`
- **Then:** Stdout is `model: (unset)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft03_get_model_unset_key_absent`
- **Source:** [035_model_command.md AC-03](../../../../docs/feature/035_model_command.md)

---

### IT-04: Get returns `model: (unset)` when settings.json absent

- **Given:** `~/.claude/settings.json` does not exist.
- **When:** `clp .model`
- **Then:** Stdout is `model: (unset)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft04_get_model_unset_file_absent`
- **Source:** [035_model_command.md AC-04](../../../../docs/feature/035_model_command.md)

---

### IT-05: `set::opus` writes `claude-opus-4-6`

- **Given:** Any `settings.json` state (may or may not exist).
- **When:** `clp .model set::opus`
- **Then:** `~/.claude/settings.json` contains `"model": "claude-opus-4-6"`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft05_set_opus_writes_full_id`
- **Source:** [035_model_command.md AC-05](../../../../docs/feature/035_model_command.md)

---

### IT-06: `set::sonnet` writes `claude-sonnet-4-6`

- **Given:** Any `settings.json` state.
- **When:** `clp .model set::sonnet`
- **Then:** `~/.claude/settings.json` contains `"model": "claude-sonnet-4-6"`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft06_set_sonnet_writes_full_id`
- **Source:** [035_model_command.md AC-06](../../../../docs/feature/035_model_command.md)

---

### IT-07: `set::haiku` writes `claude-haiku-4-5-20251001`

- **Given:** Any `settings.json` state.
- **When:** `clp .model set::haiku`
- **Then:** `~/.claude/settings.json` contains `"model": "claude-haiku-4-5-20251001"`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft07_set_haiku_writes_full_id`
- **Source:** [035_model_command.md AC-07](../../../../docs/feature/035_model_command.md)

---

### IT-08: `set::default` removes model key; other keys preserved

- **Given:** `~/.claude/settings.json` contains `{"model": "claude-opus-4-6", "theme": "dark"}`.
- **When:** `clp .model set::default`
- **Then:** `settings.json` no longer contains `"model"` key. `"theme": "dark"` is preserved. Exits 0.
- **Exit:** 0
- **Source fn:** `ft08_set_default_removes_key_preserves_others`
- **Source:** [035_model_command.md AC-08](../../../../docs/feature/035_model_command.md)

---

### IT-09: `set::bad` exits 1 with valid values named

- **Given:** Any environment.
- **When:** `clp .model set::bad`
- **Then:** Exits 1. Stderr contains each of: `opus`, `sonnet`, `haiku`, `default`.
- **Exit:** 1
- **Source fn:** `ft09_set_bad_value_exits_1`
- **Source:** [035_model_command.md AC-09](../../../../docs/feature/035_model_command.md)

---

### IT-10: Set creates `settings.json` when absent

- **Given:** `~/.claude/settings.json` does not exist.
- **When:** `clp .model set::opus`
- **Then:** `~/.claude/settings.json` is created; contains `"model": "claude-opus-4-6"`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft10_set_creates_file_when_absent`
- **Source:** [035_model_command.md AC-10](../../../../docs/feature/035_model_command.md)

---

### IT-11: Set preserves pre-existing `settings.json` keys

- **Given:** `~/.claude/settings.json` contains `{"theme": "dark", "autoUpdaterStatus": "disabled"}`.
- **When:** `clp .model set::opus`
- **Then:** All pre-existing keys preserved; `"model": "claude-opus-4-6"` added. Exits 0.
- **Exit:** 0
- **Source fn:** `ft11_set_preserves_existing_keys`
- **Source:** [035_model_command.md AC-11](../../../../docs/feature/035_model_command.md)

---

### IT-12: Get `format::json` returns structured JSON

- **Given:** `~/.claude/settings.json` contains `{"model": "opus"}`.
- **When:** `clp .model format::json`
- **Then:** Stdout is `{"model":"opus"}` (or valid JSON equivalent). Exits 0.
- **Variant:** When model absent → `{"model":null}`.
- **Exit:** 0
- **Source fn:** `ft12_get_json_format`
- **Source:** [035_model_command.md AC-12](../../../../docs/feature/035_model_command.md)

---

### IT-13: `.model` listed in `clp .help` output

- **Given:** Any environment.
- **When:** `clp .help` (or `clp .`)
- **Then:** Output contains `.model`. Exits 0.
- **Exit:** 0
- **Source fn:** `it13_model_listed_in_help_output`
- **Source:** [035_model_command.md AC-13](../../../../docs/feature/035_model_command.md)

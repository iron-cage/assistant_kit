# FT — Feature 034: Explicit Session Model Override

### Scope

- **Purpose**: Test cases for explicit session model override via `set_model::` on `.account.use` and `.usage`.
- **Source**: `docs/feature/034_explicit_session_model_override.md`
- **Covers**: AC-01 through AC-11

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | `set_model::opus` writes `claude-opus-4-6` to `settings.json` | ✅ `ft01_set_model_opus_writes_full_id` |
| FT-02 | AC-02 | `set_model::sonnet` writes `claude-sonnet-4-6` to `settings.json` | ✅ `ft02_set_model_sonnet_writes_full_id` |
| FT-03 | AC-03 | `set_model::haiku` writes `claude-haiku-4-5-20251001` to `settings.json` | ✅ `ft03_set_model_haiku_writes_full_id` |
| FT-04 | AC-04 | `set_model::default` removes `model` key; other keys preserved | ✅ `ft04_set_model_default_removes_key_preserves_others` |
| FT-05 | AC-05 | Explicit `set_model::` wins over `switch_account` per-account model restore (`.account.use` path) | ✅ `ft05_explicit_set_model_wins_over_switch_restore` |
| FT-06 | AC-06 | `trace::1` + `set_model::X` emits `[trace] account.use  {name}  set_model: X` | ✅ `ft06_trace_line_emitted_with_set_model` |
| FT-07 | AC-07 | `set_model::bad` exits 1 with all four valid values in stderr | ✅ `ft07_set_model_bad_value_exits_1` |
| FT-08 | AC-08 | `set_model::` appears in `--help` on `.account.use` and `.usage` | ✅ `ft08_set_model_appears_in_help_output` |
| FT-09 | AC-09 | `set_model::` has no effect on `format::json` output or subprocess args | ✅ `ft09_set_model_no_set_model_key_in_json` |
| FT-10 | AC-10 | `set_session_model()` preserves all pre-existing `settings.json` keys | ✅ `ft10_set_session_model_preserves_existing_keys` |
| FT-11 | AC-11 | `settings.json` is created when absent; contains only the requested `model` key | ✅ `ft11_set_session_model_creates_file_when_absent` |

### Notes

- All FT cases are integration tests in `tests/cli/` (TSK-262).
- FT-05 and FT-06 require `.account.use` with a real account fixture and a seeded `settings.json`.
- FT-07 through FT-09 can be exercised with an empty credential store.
- FT-10 and FT-11 are unit-level tests for `set_session_model()` in `claude_profile_core/tests/account_test.rs`.

---

### FT-01: `set_model::opus` writes `claude-opus-4-6` to `settings.json`

- **Given:** An account `alice` in the credential store. `~/.claude/settings.json` exists (may have existing keys).
- **When:** `clp .account.use name::alice set_model::opus` (or `clp .usage set_model::opus`)
- **Then:** `~/.claude/settings.json` contains `"model": "claude-opus-4-6"`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft01_set_model_opus_writes_full_id`
- **Source:** [034_explicit_session_model_override.md AC-01](../../../docs/feature/034_explicit_session_model_override.md)

---

### FT-02: `set_model::sonnet` writes `claude-sonnet-4-6` to `settings.json`

- **Given:** An account `alice` in the credential store.
- **When:** `clp .account.use name::alice set_model::sonnet`
- **Then:** `~/.claude/settings.json` contains `"model": "claude-sonnet-4-6"`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft02_set_model_sonnet_writes_full_id`
- **Source:** [034_explicit_session_model_override.md AC-02](../../../docs/feature/034_explicit_session_model_override.md)

---

### FT-03: `set_model::haiku` writes `claude-haiku-4-5-20251001` to `settings.json`

- **Given:** An account `alice` in the credential store.
- **When:** `clp .account.use name::alice set_model::haiku`
- **Then:** `~/.claude/settings.json` contains `"model": "claude-haiku-4-5-20251001"`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft03_set_model_haiku_writes_full_id`
- **Source:** [034_explicit_session_model_override.md AC-03](../../../docs/feature/034_explicit_session_model_override.md)

---

### FT-04: `set_model::default` removes `model` key; other keys preserved

- **Given:** `~/.claude/settings.json` contains `{"model": "claude-opus-4-6", "theme": "dark"}`.
- **When:** `clp .account.use name::alice set_model::default`
- **Then:** `~/.claude/settings.json` no longer contains the `"model"` key. `"theme": "dark"` is preserved. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft04_set_model_default_removes_key_preserves_others`
- **Source:** [034_explicit_session_model_override.md AC-04](../../../docs/feature/034_explicit_session_model_override.md)

---

### FT-05: Explicit `set_model::` wins over `switch_account` model restore

- **Given:** An account `alice` whose `{name}.json` stores `"model": "claude-opus-4-6"` (what `switch_account` would restore to `settings.json`).
- **When:** `clp .account.use name::alice set_model::sonnet`
- **Then:** `settings.json` contains `"model": "claude-sonnet-4-6"` — the explicit post-match write overwrites the per-account restore. Exits 0.
- **Exit:** 0
- **Note:** Tests the `.account.use` post-match ordering (explicit `set_session_model` runs AFTER `switch_account`). The `.usage` path's `apply_model_override` mutual exclusion (`if set_model.is_some() { write } else { apply_model_override }`) is covered by EC-7.
- **Source fn:** ✅ `ft05_explicit_set_model_wins_over_switch_restore`
- **Source:** [034_explicit_session_model_override.md AC-05](../../../docs/feature/034_explicit_session_model_override.md)

---

### FT-06: Trace line emitted on `.account.use` with `trace::1`

- **Given:** An account `alice` in the credential store.
- **When:** `clp .account.use name::alice set_model::opus trace::1`
- **Then:** Stderr contains `[trace] account.use  alice  set_model: opus`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft06_trace_line_emitted_with_set_model`
- **Source:** [034_explicit_session_model_override.md AC-06](../../../docs/feature/034_explicit_session_model_override.md)

---

### FT-07: `set_model::bad` exits 1 with valid values in stderr

- **Given:** Empty credential store (parameter validation happens before account lookup).
- **When:** `clp .usage set_model::bad`
- **Then:** Exits 1. Stderr contains each of the four valid values: `opus`, `sonnet`, `haiku`, `default`.
- **Exit:** 1
- **Source fn:** ✅ `ft07_set_model_bad_value_exits_1`
- **Source:** [034_explicit_session_model_override.md AC-07](../../../docs/feature/034_explicit_session_model_override.md)

---

### FT-08: `set_model::` appears in `--help` output

- **Given:** Any environment.
- **When:** `clp .account.use --help` and `clp .usage --help`
- **Then:** Both help outputs contain `set_model::`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft08_set_model_appears_in_help_output`
- **Source:** [034_explicit_session_model_override.md AC-08](../../../docs/feature/034_explicit_session_model_override.md)

---

### FT-09: `set_model::` has no effect on `format::json` output or subprocess args

- **Given:** An account `alice` with idle 5h window.
- **When:** `clp .usage set_model::opus format::json`
- **Then:** JSON output structure is unchanged — no `set_model` field in the JSON; subprocess args (if any) do not contain the model string from `set_model`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft09_set_model_no_set_model_key_in_json`
- **Source:** [034_explicit_session_model_override.md AC-09](../../../docs/feature/034_explicit_session_model_override.md)

---

### FT-10: `set_model::opus` preserves pre-existing `settings.json` keys

- **Given:** Account `alice` in credential store. `~/.claude/settings.json` contains `{"theme": "dark", "autoUpdaterStatus": "disabled"}`.
- **When:** `clp .account.use name::alice set_model::opus`
- **Then:** `~/.claude/settings.json` contains `"theme": "dark"`, `"autoUpdaterStatus": "disabled"`, and `"model": "claude-opus-4-6"`. No pre-existing keys removed. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft10_set_session_model_preserves_existing_keys`
- **Source:** [034_explicit_session_model_override.md AC-10](../../../docs/feature/034_explicit_session_model_override.md)

---

### FT-11: `set_model::opus` creates `settings.json` when absent

- **Given:** Account `alice` in credential store. `~/.claude/settings.json` does not exist.
- **When:** `clp .account.use name::alice set_model::opus`
- **Then:** `~/.claude/settings.json` is created and contains `"model": "claude-opus-4-6"`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `ft11_set_session_model_creates_file_when_absent`
- **Source:** [034_explicit_session_model_override.md AC-11](../../../docs/feature/034_explicit_session_model_override.md)

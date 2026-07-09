# Schema 008: CLR Preferences — `~/.clr/prefs.json`

SC test cases for `docs/schema/008_clr_prefs_json.md`. Verifies the `prefs.json`
write contract: `subprocess_model` field write/read semantics, auto-creation on
first write, graceful absent-file/absent-field fallback, and preservation of
unrelated keys across write and reset operations.

**Source:** [docs/schema/008_clr_prefs_json.md](../../../docs/schema/008_clr_prefs_json.md)

### SC Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| SC-1 | `id::VALUE` writes `subprocess_model` and pins the model | Field Semantics | ✅ |
| SC-2 | Absent file / absent field is graceful — treated as unset | Error Path | ✅ |
| SC-3 | `prefs.json` created automatically on first write when `~/.clr/` exists but file is absent | Write Semantics | ✅ |
| SC-4 | `reset::1` removes only `subprocess_model`, preserves other keys and the file itself | Write Isolation | ✅ |

---

### SC-1: `id::VALUE` writes `subprocess_model` and pins the model

- **Given:** No preference pinned, or an existing `subprocess_model` value in `~/.clr/prefs.json`
- **When:** `clp .model.select id::claude-opus-4-8` is invoked
- **Then:** `~/.clr/prefs.json` contains `"subprocess_model": "claude-opus-4-8"`, and a subsequent `clp .model.select` (get form) prints the pinned value — proving the field is both written and read back correctly
- **Source fn:** `it03_set_opus_pins_model`, `it02_get_shows_pinned_value` (model_select_test.rs)
- **Source:** [docs/schema/008_clr_prefs_json.md §Fields](../../../docs/schema/008_clr_prefs_json.md)

---

### SC-2: Absent file / absent field is graceful — treated as unset

- **Given:** No `~/.clr/prefs.json` exists, or the file exists without a `subprocess_model` key
- **When:** `clp .model.select` (get form) is invoked, or `claude_runner_core/src/isolated.rs` resolves the subprocess model
- **Then:** The preference is treated as no preference — `clp .model.select` reports `(unset)`; readers fall back to `ISOLATED_DEFAULT_MODEL`. No error, no panic.
- **Source fn:** `it01_get_unset_no_file` (model_select_test.rs)
- **Source:** [docs/schema/008_clr_prefs_json.md §Empty file / absent file / null field](../../../docs/schema/008_clr_prefs_json.md)

---

### SC-3: `prefs.json` created automatically on first write when `~/.clr/` exists but file is absent

- **Given:** `~/.clr/` directory exists (already created by clr for the journal) but `prefs.json` does not
- **When:** `clp .model.select id::VALUE` is invoked
- **Then:** `~/.clr/prefs.json` is created containing the written `subprocess_model` value — no error occurs due to the missing file, and `~/.clr/` itself is not re-created (clr already owns it)
- **Source fn:** `it07_set_creates_file_when_absent` (model_select_test.rs)
- **Source:** [docs/schema/008_clr_prefs_json.md §File location](../../../docs/schema/008_clr_prefs_json.md)

---

### SC-4: `reset::1` removes only `subprocess_model`, preserves other keys and the file itself

- **Given:** `~/.clr/prefs.json` contains `subprocess_model` alongside at least one unrelated key
- **When:** `clp .model.select reset::1` is invoked
- **Then:** `subprocess_model` is removed from the file, but the file continues to exist and all other keys remain unchanged — writers preserve unknown fields per the forward-compatibility contract
- **Source fn:** `it05_reset_removes_key_preserves_others`, `it08_set_preserves_other_keys` (model_select_test.rs)
- **Source:** [docs/schema/008_clr_prefs_json.md §Extra fields](../../../docs/schema/008_clr_prefs_json.md)

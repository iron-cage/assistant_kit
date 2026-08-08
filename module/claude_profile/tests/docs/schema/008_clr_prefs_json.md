# Schema 008: CLR Preferences — `~/.clr/prefs.json`

> Deprecated: task 410 retired `~/.clr/prefs.json` — the subprocess model preference is now stored as the `model` key in `~/.clr/config.toml`, written/read via `clp .model scope::subprocess` (Feature 035; formerly `.model.select`, retired). Current CLI-level coverage lives in [tests/docs/cli/command/20_model_select.md](../cli/command/20_model_select.md); schema-level coverage for the new format lives in `claude_core/docs/api/002_toml_io.md`. Cases below are suspended per `l1_imp_surface.rulebook.md § Spec : NA Case Protocol` — preserved for historical reference only.
>
> **Source fn staleness note:** each case's `Source fn:` cites `model_select_test.rs` function names (`it01`–`it03`, `it05`, `it07`, `it08`) that were accurate when this spec was suspended (confirmed via `git show` against the pre-refactor revision). Task 465's later consolidation refactor rewrote that file down to 3 migration-stub tests (`t23_get_form_exits_1_with_migration_message`, `t23_id_form_exits_1_with_migration_message`, `t23_reset_form_exits_1_with_migration_message`), so none of the cited names resolve in the current file. The citations remain as an accurate historical record of pre-retirement coverage, not a live reference — do not follow them expecting to find the function today.

SC test cases for `docs/schema/008_clr_prefs_json.md`. Verifies the `prefs.json`
write contract: `subprocess_model` field write/read semantics, auto-creation on
first write, graceful absent-file/absent-field fallback, and preservation of
unrelated keys across write and reset operations.

**Source:** [docs/schema/008_clr_prefs_json.md](../../../docs/schema/008_clr_prefs_json.md)

### SC Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| SC-1 | `id::VALUE` writes `subprocess_model` and pins the model | Field Semantics | N/A |
| SC-2 | Absent file / absent field is graceful — treated as unset | Error Path | N/A |
| SC-3 | `prefs.json` created automatically on first write when `~/.clr/` exists but file is absent | Write Semantics | N/A |
| SC-4 | `reset::1` removes only `subprocess_model`, preserves other keys and the file itself | Write Isolation | N/A |

---

### SC-1: `id::VALUE` writes `subprocess_model` and pins the model

> **N/A** — `~/.clr/prefs.json` is no longer written or read by any code path; task 410 migrated `.model.select` onto `~/.clr/config.toml`.
> Becomes testable when: no committed task.

- **Given:** No preference pinned, or an existing `subprocess_model` value in `~/.clr/prefs.json`
- **When:** `clp .model.select id::claude-opus-4-8` is invoked
- **Then:** `~/.clr/prefs.json` contains `"subprocess_model": "claude-opus-4-8"`, and a subsequent `clp .model.select` (get form) prints the pinned value — proving the field is both written and read back correctly
- **Source fn:** `it03_set_opus_pins_model`, `it02_get_shows_pinned_value` (model_select_test.rs)
- **Source:** [docs/schema/008_clr_prefs_json.md §Fields](../../../docs/schema/008_clr_prefs_json.md)

---

### SC-2: Absent file / absent field is graceful — treated as unset

> **N/A** — `~/.clr/prefs.json` is no longer written or read by any code path; task 410 migrated `.model.select` onto `~/.clr/config.toml`.
> Becomes testable when: no committed task.

- **Given:** No `~/.clr/prefs.json` exists, or the file exists without a `subprocess_model` key
- **When:** `clp .model.select` (get form) is invoked, or `claude_runner_core/src/isolated.rs` resolves the subprocess model
- **Then:** The preference is treated as no preference — `clp .model.select` reports `(unset)`; readers fall back to `ISOLATED_DEFAULT_MODEL`. No error, no panic.
- **Source fn:** `it01_get_unset_no_file` (model_select_test.rs)
- **Source:** [docs/schema/008_clr_prefs_json.md §Empty file / absent file / null field](../../../docs/schema/008_clr_prefs_json.md)

---

### SC-3: `prefs.json` created automatically on first write when `~/.clr/` exists but file is absent

> **N/A** — `~/.clr/prefs.json` is no longer written or read by any code path; task 410 migrated `.model.select` onto `~/.clr/config.toml`.
> Becomes testable when: no committed task.

- **Given:** `~/.clr/` directory exists (already created by clr for the journal) but `prefs.json` does not
- **When:** `clp .model.select id::VALUE` is invoked
- **Then:** `~/.clr/prefs.json` is created containing the written `subprocess_model` value — no error occurs due to the missing file, and `~/.clr/` itself is not re-created (clr already owns it)
- **Source fn:** `it07_set_creates_file_when_absent` (model_select_test.rs)
- **Source:** [docs/schema/008_clr_prefs_json.md §File location](../../../docs/schema/008_clr_prefs_json.md)

---

### SC-4: `reset::1` removes only `subprocess_model`, preserves other keys and the file itself

> **N/A** — `~/.clr/prefs.json` is no longer written or read by any code path; task 410 migrated `.model.select` onto `~/.clr/config.toml`.
> Becomes testable when: no committed task.

- **Given:** `~/.clr/prefs.json` contains `subprocess_model` alongside at least one unrelated key
- **When:** `clp .model.select reset::1` is invoked
- **Then:** `subprocess_model` is removed from the file, but the file continues to exist and all other keys remain unchanged — writers preserve unknown fields per the forward-compatibility contract
- **Source fn:** `it05_reset_removes_key_preserves_others`, `it08_set_preserves_other_keys` (model_select_test.rs)
- **Source:** [docs/schema/008_clr_prefs_json.md §Extra fields](../../../docs/schema/008_clr_prefs_json.md)

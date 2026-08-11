# Schema 006: Session Settings — `~/.claude/settings.json`

SC test cases for `docs/schema/006_settings_json.md`. Verifies the `settings.json`
write contract: model field semantics, effortLevel unconditional write, preservation of
non-managed fields via read-modify-write, and the Opus/Sonnet effort level mapping.

**Source:** [docs/schema/006_settings_json.md](../../../docs/schema/006_settings_json.md)

### SC Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| SC-1 | `model` field controls session model shorthand | Field Semantics | ✅ |
| SC-2 | Opus model → `effortLevel` written as `"max"` unconditionally | Effort Write Rule | ✅ |
| SC-3 | Sonnet model (sufficient quota) → `effortLevel` written as `"high"` unconditionally | Effort Write Rule | ✅ |
| SC-4 | Non-managed fields preserved on write (read-modify-write) | Write Semantics | ✅ |
| SC-5 | Malformed `settings.json` — `get` returns absent/unset, does not crash | Error Path | ✅ |

---

### SC-1: `model` field controls which session model is used

- **Given:** No pre-existing `settings.json`; account `alice@example.com` is switched to with `set_model::sonnet`.
- **When:** `clp .account.use name::alice@example.com set_model::sonnet` runs — the `sonnet` shorthand is resolved to a full model ID before `set_session_model()` writes it.
- **Then:** `settings.json` contains `"model": "claude-sonnet-5"` — shorthand-to-full-ID resolution happens at write time, not read time; `get_session_model()` itself is a passive reader with no mapping logic (`claude_profile_core/src/account.rs::get_session_model`, confirmed by direct inspection — it returns whatever raw string is stored under the `"model"` key).
- **Source fn:** `ft02_set_model_sonnet_writes_full_id` (cli/set_model_test.rs) — corrected; previous citation (`mre_bug322_opus_override_sets_effort_max`) tested an unrelated opus/effort scenario, and the original Given/When/Then (claiming `get_session_model()` itself performs shorthand→full-ID mapping) did not match `get_session_model()`'s actual passive-read implementation
- **Source:** [docs/schema/006_settings_json.md §Fields Managed by clp](../../../docs/schema/006_settings_json.md)

---

### SC-2: Opus branch sets `effortLevel` to `"max"` unconditionally (Fix BUG-322, TSK-335)

- **Given:** Account quota causes `apply_model_override()` to select Opus (near-exhausted Sonnet)
- **When:** `apply_model_override()` completes
- **Then:** `settings.json` contains `"effortLevel": "max"` — effort is written unconditionally in the Opus branch regardless of whether the model changed
- **Source fn:** `mre_bug322_opus_override_sets_effort_max` (usage/api_tests_a.rs)
- **Source:** [docs/schema/006_settings_json.md §Effort Tracking Behavior](../../../docs/schema/006_settings_json.md)

---

### SC-3: Sonnet branch sets `effortLevel` to `"high"` unconditionally (Fix BUG-322, TSK-335)

- **Given:** Account quota causes `apply_model_override()` to select Sonnet (sufficient quota or absent tier)
- **When:** `apply_model_override()` completes
- **Then:** `settings.json` contains `"effortLevel": "high"` — effort is written unconditionally in the Sonnet branch
- **Source fn:** `t11_opus_to_sonnet_sets_effort_high` (api_tests_a.rs)
- **Source:** [docs/schema/006_settings_json.md §Effort Tracking Behavior](../../../docs/schema/006_settings_json.md)

---

### SC-4: Non-managed fields are preserved on write

- **Given:** `settings.json` pre-seeded with `{"model":"claude-opus-4-8","theme":"dark"}` — `theme` is a field beyond `model`/`effortLevel`, owned by the Claude binary.
- **When:** `clp .account.use name::alice@example.com set_model::default touch::0` runs — `set_model::default` removes the `model` key via `set_session_model()`.
- **Then:** `settings.json` no longer contains `"model"`, but still contains `"theme"` — the unrelated field survives the read-modify-write.
- **Source fn:** `ft04_set_model_default_removes_key_preserves_others` (cli/set_model_test.rs) — corrected; previous citation (`acc28_save_succeeds_without_settings_json`) tested a different command (`.account.save`) under a settings.json-absent scenario, not the pre-existing-extra-field preservation this case claims
- **Source:** [docs/schema/006_settings_json.md §Write Rules](../../../docs/schema/006_settings_json.md)

---

### SC-5: Malformed `settings.json` — read returns absent/unset without crashing

- **Given:** `~/.claude/settings.json` contains malformed or truncated JSON
- **When:** `get_session_model()` or `get_session_effort()` is called
- **Then:** Returns `None` (field treated as absent) — no panic, no error propagation that blocks the calling command
- **Source fn:** *(coverage gap — `cc_c_malformed_settings_json_get_returns_unset` was real at this exact path (`cli/model_test.rs`) but was deleted, with no replacement, during the `.model` command consolidation refactor (`git log -S`: added in `b5a100b2`, removed in `f3fc731e`); no test in the current suite exercises `get_session_model()`/`get_session_effort()` against malformed `settings.json`)*
- **Source:** [docs/schema/006_settings_json.md §Fields Managed by clp](../../../docs/schema/006_settings_json.md)

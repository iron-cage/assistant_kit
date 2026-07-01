# Schema 006: Session Settings — `~/.claude/settings.json`

SC test cases for `docs/schema/006_settings_json.md`. Verifies the `settings.json`
write contract: model field semantics, effortLevel unconditional write, preservation of
non-managed fields via read-modify-write, and the Opus/Sonnet effort level mapping.

**Source:** [docs/schema/006_settings_json.md](../../../../docs/schema/006_settings_json.md)

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

- **Given:** `settings.json` contains `"model": "sonnet"`
- **When:** `get_session_model()` is called
- **Then:** Returns `Some("claude-sonnet-4-6")` — the model shorthand maps to the full model ID used for interactive sessions
- **Source fn:** `mre_bug322_opus_override_sets_effort_max` (usage/api_tests_a.rs; verifies model written alongside effort)
- **Source:** [docs/schema/006_settings_json.md §Fields Managed by clp](../../../../docs/schema/006_settings_json.md)

---

### SC-2: Opus branch sets `effortLevel` to `"max"` unconditionally (Fix BUG-322, TSK-335)

- **Given:** Account quota causes `apply_model_override()` to select Opus (near-exhausted Sonnet)
- **When:** `apply_model_override()` completes
- **Then:** `settings.json` contains `"effortLevel": "max"` — effort is written unconditionally in the Opus branch regardless of whether the model changed
- **Source fn:** `mre_bug322_opus_override_sets_effort_max` (usage/api_tests_a.rs)
- **Source:** [docs/schema/006_settings_json.md §Effort Tracking Behavior](../../../../docs/schema/006_settings_json.md)

---

### SC-3: Sonnet branch sets `effortLevel` to `"high"` unconditionally (Fix BUG-322, TSK-335)

- **Given:** Account quota causes `apply_model_override()` to select Sonnet (sufficient quota or absent tier)
- **When:** `apply_model_override()` completes
- **Then:** `settings.json` contains `"effortLevel": "high"` — effort is written unconditionally in the Sonnet branch
- **Source fn:** `t11_opus_to_sonnet_sets_effort_high` (usage/api_tests_b.rs)
- **Source:** [docs/schema/006_settings_json.md §Effort Tracking Behavior](../../../../docs/schema/006_settings_json.md)

---

### SC-4: Non-managed fields are preserved on write

- **Given:** `settings.json` contains additional fields beyond `model` and `effortLevel` (e.g., user configuration keys owned by the Claude binary)
- **When:** `clp` writes `model` or `effortLevel` via `apply_model_override()` or `set_session_model()`
- **Then:** All other fields in `settings.json` remain unchanged — `clp` reads the entire file, modifies only its owned fields, and writes the complete object back
- **Source fn:** `acc28_save_succeeds_without_settings_json` (cli/accounts_list_test_b.rs; verifies settings.json absent is graceful)
- **Source:** [docs/schema/006_settings_json.md §Write Rules](../../../../docs/schema/006_settings_json.md)

---

### SC-5: Malformed `settings.json` — read returns absent/unset without crashing

- **Given:** `~/.claude/settings.json` contains malformed or truncated JSON
- **When:** `get_session_model()` or `get_session_effort()` is called
- **Then:** Returns `None` (field treated as absent) — no panic, no error propagation that blocks the calling command
- **Source fn:** `cc_c_malformed_settings_json_get_returns_unset` (cli/model_test.rs)
- **Source:** [docs/schema/006_settings_json.md §Fields Managed by clp](../../../../docs/schema/006_settings_json.md)

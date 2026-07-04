# Test: `set_model::` Parameter

Edge case coverage for the `set_model::` parameter on `.account.use` and `.usage`. See [param/054_set_model.md](../../../../docs/cli/param/054_set_model.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `set_model::opus` accepted; writes `claude-opus-4-8` to `settings.json` | Behavioral Divergence |
| EC-2 | `set_model::sonnet` accepted; writes `claude-sonnet-5` to `settings.json` | ✅ Valid Value |
| EC-3 | `set_model::haiku` accepted; writes `claude-haiku-4-5-20251001` to `settings.json` | ✅ Valid Value |
| EC-4 | `set_model::default` accepted; removes `model` key from `settings.json` | Behavioral Divergence |
| EC-5 | `set_model::bad` exits 1; stderr names all four valid values | ✅ Invalid Value |
| EC-6 | `.account.use` with `set_model::opus` — override wins over `apply_post_switch_touch` | ✅ Precedence |
| EC-7 | `.usage` with `set_model::opus` — `apply_model_override()` not called | ✅ Precedence |

---

### EC-1: `set_model::opus` accepted; writes `claude-opus-4-8`

- **Given:** Account `alice` in the credential store.
- **When:** `clp .account.use name::alice set_model::opus`
- **Then:** Exits 0. `~/.claude/settings.json` contains `"model": "claude-opus-4-8"`. No error about unrecognized parameter.
- **Exit:** 0
- **Source fn:** ✅ `ec1_set_model_opus_accepted_no_unrecognized_error`
- **Source:** [param/054_set_model.md](../../../../docs/cli/param/054_set_model.md)

---

### EC-2: `set_model::sonnet` accepted; writes `claude-sonnet-5`

- **Given:** Account `alice` in the credential store.
- **When:** `clp .account.use name::alice set_model::sonnet`
- **Then:** Exits 0. `~/.claude/settings.json` contains `"model": "claude-sonnet-5"`.
- **Exit:** 0
- **Source fn:** ✅ `ec2_set_model_sonnet_accepted_writes_full_id`
- **Source:** [param/054_set_model.md](../../../../docs/cli/param/054_set_model.md)

---

### EC-3: `set_model::haiku` accepted; writes `claude-haiku-4-5-20251001`

- **Given:** Account `alice` in the credential store.
- **When:** `clp .account.use name::alice set_model::haiku`
- **Then:** Exits 0. `~/.claude/settings.json` contains `"model": "claude-haiku-4-5-20251001"`.
- **Exit:** 0
- **Source fn:** ✅ `ec3_set_model_haiku_accepted_writes_full_id`
- **Source:** [param/054_set_model.md](../../../../docs/cli/param/054_set_model.md)

---

### EC-4: `set_model::default` accepted; removes `model` key

- **Given:** `~/.claude/settings.json` contains `"model": "claude-opus-4-8"`.
- **When:** `clp .account.use name::alice set_model::default`
- **Then:** Exits 0. `~/.claude/settings.json` no longer has a `"model"` key.
- **Exit:** 0
- **Source fn:** ✅ `ec4_set_model_default_accepted_removes_key`
- **Source:** [param/054_set_model.md](../../../../docs/cli/param/054_set_model.md)

---

### EC-5: `set_model::bad` exits 1 (invalid value)

- **Given:** Any environment (empty credential store sufficient).
- **When:** `clp .usage set_model::bad`
- **Then:** Exits 1. Stderr contains all four valid values: `opus`, `sonnet`, `haiku`, `default`.
- **Exit:** 1
- **Source fn:** ✅ `ec5_set_model_bad_exits_1_all_valid_values_named`
- **Source:** [feature/034_explicit_session_model_override.md AC-07](../../../../docs/feature/034_explicit_session_model_override.md)

---

### EC-6: `.account.use` explicit override wins over `switch_account` model restore

- **Given:** Account `alice` whose `{name}.json` stores `"model": "claude-opus-4-8"` (what `switch_account` would restore to `settings.json`).
- **When:** `clp .account.use name::alice set_model::sonnet`
- **Then:** Exits 0. `settings.json` contains `"model": "claude-sonnet-5"` — the explicit post-match write overwrites the per-account restore.
- **Exit:** 0
- **Source fn:** ✅ `ec6_account_use_set_model_wins_over_switch_restore`
- **Source:** [feature/034_explicit_session_model_override.md AC-05](../../../../docs/feature/034_explicit_session_model_override.md)

---

### EC-7: `.usage` explicit `set_model::` writes to `settings.json`

- **Given:** Account `alice`. `settings.json` pre-seeded with `"model": "claude-opus-4-8"` (simulating what `apply_model_override()` would write).
- **When:** `clp .usage set_model::sonnet`
- **Then:** Exits 0. `settings.json` contains `"model": "claude-sonnet-5"` — explicit `set_model::sonnet` overwrote the pre-seeded opus value, proving `apply_model_override()` was bypassed.
- **Exit:** 0
- **Source fn:** ✅ `ec7_usage_set_model_writes_to_settings`
- **Source:** [feature/034_explicit_session_model_override.md AC-05](../../../../docs/feature/034_explicit_session_model_override.md)

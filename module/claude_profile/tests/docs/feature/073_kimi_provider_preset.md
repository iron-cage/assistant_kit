# FT — Feature 073: Kimi Provider Preset

### Scope

- **Purpose**: Test cases for the `preset::kimi` convenience param on `.account.save` (default-filling `backend::`/`base_url::`/`inference_provider::`) and the 7 Kimi-tier `settings.json` `env.*` variables `switch_account()` writes/clears for redirect accounts tagged `inference_provider: "kimi"`.
- **Source**: `docs/feature/073_kimi_provider_preset.md`
- **Covers**: AC-01 through AC-10

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | Switch to kimi redirect writes all 10 env vars; 5 mirror `redirect_model`; effort `"max"` | `ft01_073_switch_to_kimi_redirect_writes_all_tier_env_vars` |
| FT-02 | AC-02 | Compact window `"1048576"` for `kimi-k3*`, `"262144"` otherwise | `ft01_073_switch_to_kimi_redirect_writes_all_tier_env_vars` (k3 wide), `ft02_073_switch_to_kimi_redirect_uses_narrow_compact_window_for_non_k3_model` |
| FT-03 | AC-03 | Non-`"kimi"` provider on redirect switch → only the 3 base vars | `ft03_073_switch_to_redirect_non_kimi_provider_omits_tier_env_vars` |
| FT-04 | AC-04 | Switch kimi→anthropic removes all 10 vars | `ft04_073_switch_from_kimi_to_anthropic_clears_all_tier_env_vars` |
| FT-05 | AC-05 | Switch kimi→other-redirect clears the 7 stale tier vars | `ft05_073_switch_from_kimi_to_other_redirect_clears_stale_tier_env_vars` |
| FT-06 | AC-06 | `preset::kimi` fills `backend`/`base_url`/`inference_provider` defaults | `t14_save_preset_kimi_fills_backend_base_url_and_inference_provider` |
| FT-07 | AC-07 | Explicit `base_url::` overrides the preset default | `t15_save_preset_kimi_explicit_base_url_overrides_default` |
| FT-08 | AC-08 | `preset::kimi backend::anthropic` → ordinary OAuth path, no redirect fields | `t18_save_preset_kimi_with_explicit_backend_anthropic_does_not_force_redirect_fields` |
| FT-09 | AC-09 | `preset::bogus` → exit 1 naming the only recognized value | `t16_save_preset_unrecognized_value_exits_1` |
| FT-10 | AC-10 | End-to-end: preset save + `.account.use` writes all 10 vars via CLI | `t17_use_preset_kimi_account_writes_kimi_tier_env_vars` |

### Notes

- ✅ Implemented — domain-level cases (FT-01–FT-05) live in `claude_profile_core/tests/account_backend_test.rs` (`ft01`–`ft05_073` fn prefix); CLI-surface cases (FT-06–FT-10) in `tests/cli/account_redirect_backend_test.rs` (`t14`–`t18`).
- All FT cases use temporary isolated `$HOME`/credential stores; no real user environment.
- FT-01–FT-05 exercise `switch_account()`'s Kimi-tier branch directly (`KIMI_MODEL_TIER_ENV_VARS`, `kimi_auto_compact_window()`, `write_kimi_tier_env_vars()`/`clear_kimi_tier_env_vars()` in `claude_profile_core/src/account/switch.rs`); FT-06–FT-09 exercise `account_save_routine()`'s preset parsing/gating; FT-10 ties both through the CLI.
- FT-02's asymmetry rationale (default to the narrow window unless the model name confirms K3 capacity) is design-documented in the feature doc — both fns assert the exact string values `"1048576"`/`"262144"`.
- FT-08 pins the gating rule: preset defaults apply only when the *resolved* `backend` is `redirect`, never merely because `preset::kimi` was given.
- Base-3-variable write/clear mechanics are Feature 071's — see `tests/docs/feature/071_redirect_backend_accounts.md` FT-06/FT-07; this spec covers only the 7 additional Kimi-tier variables and the preset sugar.

---

### FT-01: All 10 env vars written on switch to kimi redirect

- **Given:** Redirect account with `inference_provider: "kimi"`, `redirect_model: "kimi-k3"`.
- **When:** `switch_account()` targets it.
- **Then:** `settings.json` `env` holds the 3 base vars plus all 7 tier vars; the 5 tier/subagent model vars each equal `redirect_model`; `CLAUDE_CODE_EFFORT_LEVEL` is `"max"`.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft01_073_switch_to_kimi_redirect_writes_all_tier_env_vars`
- **Source:** [073_kimi_provider_preset.md AC-01](../../../docs/feature/073_kimi_provider_preset.md)

---

### FT-02: Compact-window sizing by model name

- **Given:** Two kimi redirect accounts — `redirect_model` `kimi-k3` vs a non-K3 value.
- **When:** `switch_account()` targets each.
- **Then:** `CLAUDE_CODE_AUTO_COMPACT_WINDOW` is `"1048576"` for `kimi-k3*`, `"262144"` otherwise.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft01_073_switch_to_kimi_redirect_writes_all_tier_env_vars`, `ft02_073_switch_to_kimi_redirect_uses_narrow_compact_window_for_non_k3_model`
- **Source:** [073_kimi_provider_preset.md AC-02](../../../docs/feature/073_kimi_provider_preset.md)

---

### FT-03: Non-kimi provider omits tier vars

- **Given:** Redirect account whose `inference_provider` is absent or ≠ `"kimi"`.
- **When:** `switch_account()` targets it.
- **Then:** Only the 3 base vars written — none of the 7 tier vars appear.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft03_073_switch_to_redirect_non_kimi_provider_omits_tier_env_vars`
- **Source:** [073_kimi_provider_preset.md AC-03](../../../docs/feature/073_kimi_provider_preset.md)

---

### FT-04: Switch to anthropic clears all 10 vars

- **Given:** Active kimi redirect account (10 vars present).
- **When:** `switch_account()` targets a `backend: anthropic` account.
- **Then:** All 10 vars removed — no stale tier vars remain.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft04_073_switch_from_kimi_to_anthropic_clears_all_tier_env_vars`
- **Source:** [073_kimi_provider_preset.md AC-04](../../../docs/feature/073_kimi_provider_preset.md)

---

### FT-05: Switch to non-kimi redirect clears stale tier vars

- **Given:** Active kimi redirect account (10 vars present).
- **When:** `switch_account()` targets a different redirect account with `inference_provider` ≠ `"kimi"`.
- **Then:** The new account's own 3 base vars written; the 7 stale tier vars removed.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft05_073_switch_from_kimi_to_other_redirect_clears_stale_tier_env_vars`
- **Source:** [073_kimi_provider_preset.md AC-05](../../../docs/feature/073_kimi_provider_preset.md)

---

### FT-06: Preset fills the three defaults

- **Given:** No pre-existing account `kimi`.
- **When:** `clp .account.save name::kimi preset::kimi api_key::sk-test redirect_model::kimi-k3` (no explicit `backend::`/`base_url::`/`inference_provider::`)
- **Then:** `kimi.json` has `backend: "redirect"`, `base_url: "https://api.moonshot.ai/anthropic"`, `inference_provider: "kimi"`.
- **Exit:** 0
- **Source fn:** `t14_save_preset_kimi_fills_backend_base_url_and_inference_provider`
- **Source:** [073_kimi_provider_preset.md AC-06](../../../docs/feature/073_kimi_provider_preset.md)

---

### FT-07: Explicit `base_url::` overrides the preset

- **Given:** No pre-existing account.
- **When:** FT-06's command plus `base_url::https://custom.endpoint/anthropic`
- **Then:** The explicit value is stored — the preset default is never applied over it.
- **Exit:** 0
- **Source fn:** `t15_save_preset_kimi_explicit_base_url_overrides_default`
- **Source:** [073_kimi_provider_preset.md AC-07](../../../docs/feature/073_kimi_provider_preset.md)

---

### FT-08: `preset::kimi backend::anthropic` takes the ordinary path

- **Given:** Live credentials fixture.
- **When:** `clp .account.save name::alice@acme.com preset::kimi backend::anthropic`
- **Then:** Ordinary OAuth-capture path; no `base_url`/`inference_provider` defaults applied — resolved `backend` gates the preset.
- **Exit:** 0
- **Source fn:** `t18_save_preset_kimi_with_explicit_backend_anthropic_does_not_force_redirect_fields`
- **Source:** [073_kimi_provider_preset.md AC-08](../../../docs/feature/073_kimi_provider_preset.md)

---

### FT-09: Unrecognized preset value exits 1

- **Given:** Any state.
- **When:** `clp .account.save name::x preset::bogus api_key::sk-test redirect_model::m1`
- **Then:** Exits 1; stderr names the only recognized value (`kimi`); no files written.
- **Exit:** 1
- **Source fn:** `t16_save_preset_unrecognized_value_exits_1`
- **Source:** [073_kimi_provider_preset.md AC-09](../../../docs/feature/073_kimi_provider_preset.md)

---

### FT-10: End-to-end preset save + use writes all 10 vars

- **Given:** Account saved via `preset::kimi api_key::sk-test redirect_model::kimi-k3`.
- **When:** `clp .account.use name::kimi`
- **Then:** All 10 env vars land in `settings.json` through the CLI surface — AC-01's domain behavior reachable end-to-end.
- **Exit:** 0
- **Source fn:** `t17_use_preset_kimi_account_writes_kimi_tier_env_vars`
- **Source:** [073_kimi_provider_preset.md AC-10](../../../docs/feature/073_kimi_provider_preset.md)

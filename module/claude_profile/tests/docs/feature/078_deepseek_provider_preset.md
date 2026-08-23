# FT — Feature 078: DeepSeek Provider Preset

### Scope

- **Purpose**: Test cases for the `preset::deepseek` convenience param on `.account.save` (default-filling `backend::`/`base_url::`/`inference_provider::`) and the 6 DeepSeek-tier `settings.json` `env.*` variables `switch_account()` writes/clears for redirect accounts tagged `inference_provider: "deepseek"`, plus the cross-provider clearing between the Kimi-tier (Feature 073) and DeepSeek-tier bundles.
- **Source**: `docs/feature/078_deepseek_provider_preset.md`
- **Covers**: AC-01 through AC-11

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | Switch to deepseek redirect writes all 9 env vars; Opus/Sonnet mirror `redirect_model`; Haiku/subagent fixed `"deepseek-v4-flash"`; effort `"max"` | `ft06_078_switch_to_deepseek_redirect_writes_tier_env_vars` |
| FT-02 | AC-02 | Compact window is the flat `"786432"` regardless of `redirect_model` (both `deepseek-v4-pro` and `deepseek-v4-flash` as the saved model) | `ft07_078_switch_to_deepseek_redirect_uses_flat_compact_window_regardless_of_model` |
| FT-03 | AC-03 | Non-`"deepseek"` provider on redirect switch (absent or `"kimi"`) → none of the 6 DeepSeek-tier vars appear | `ft08_078_switch_to_redirect_non_deepseek_provider_omits_tier_env_vars` |
| FT-04 | AC-04 | Switch deepseek→anthropic removes all 9 vars | `ft09_078_switch_from_deepseek_to_anthropic_clears_tier_env_vars` |
| FT-05 | AC-05 | Switch deepseek→other-redirect (non-deepseek) clears the 6 stale tier vars | `ft10_078_switch_from_deepseek_to_other_redirect_clears_stale_tier_env_vars` |
| FT-06 | AC-06 | `preset::deepseek` fills `backend`/`base_url`/`inference_provider` defaults | `t24_save_preset_deepseek_fills_backend_base_url_and_inference_provider` |
| FT-07 | AC-07 | Explicit `base_url::` overrides the preset default | `t25_save_preset_deepseek_explicit_base_url_overrides_default` |
| FT-08 | AC-08 | `preset::deepseek backend::anthropic` → ordinary OAuth path, no redirect fields | `t26_save_preset_deepseek_with_explicit_backend_anthropic_does_not_force_redirect_fields` |
| FT-09 | AC-09 | `preset::bogus` → exit 1 naming both recognized values (`kimi`, `deepseek`) | `t16_save_preset_unrecognized_value_exits_1` (shared/updated — see Notes) |
| FT-10 | AC-10 | End-to-end: preset save + `.account.use` writes all 9 vars via CLI | `t27_use_preset_deepseek_account_writes_deepseek_tier_env_vars` |
| FT-11 | AC-11 | Switch kimi→deepseek clears all 7 Kimi-tier vars, writes all 6 DeepSeek-tier vars | `ft11_078_switch_from_kimi_to_deepseek_clears_kimi_writes_deepseek` |
| FT-12 | AC-11 | Switch deepseek→kimi clears all 6 DeepSeek-tier vars, writes all 7 Kimi-tier vars | `ft12_078_switch_from_deepseek_to_kimi_clears_deepseek_writes_kimi` |

### Notes

- ✅ Implemented — domain-level cases (FT-01–FT-05, FT-11, FT-12) live in `claude_profile_core/tests/account_backend_test.rs` (`ft06`–`ft12_078` fn prefix); CLI-surface cases (FT-06–FT-10) in `tests/cli/account_redirect_backend_test.rs` (`t24`–`t27`, plus the shared `t16`).
- All FT cases use temporary isolated `$HOME`/credential stores; no real user environment.
- FT-01–FT-05, FT-11, FT-12 exercise `switch_account()`'s DeepSeek-tier branch directly (`DEEPSEEK_PRO_TIER_ENV_VARS`/`DEEPSEEK_FLASH_TIER_ENV_VARS`, `DEEPSEEK_FLASH_MODEL`, `DEEPSEEK_AUTO_COMPACT_WINDOW`, `write_deepseek_tier_env_vars()`/`clear_deepseek_tier_env_vars()` in `claude_profile_core/src/account/switch.rs`); FT-06–FT-09 exercise `account_save_routine()`'s preset parsing/gating extended to `deepseek`; FT-10 ties both through the CLI.
- FT-09 shares its source fn with Feature 073's own AC-09 case (`t16_save_preset_unrecognized_value_exits_1`, [073_kimi_provider_preset.md](073_kimi_provider_preset.md) FT-09) — one test now asserts stderr names *both* recognized values (`kimi`, `deepseek`), rather than a second near-duplicate test.
- FT-02 pins the deliberate divergence from Kimi's AC-02 (`kimi-k3*`-conditional window): DeepSeek's compact window is a flat constant with no model-name branching, per DeepSeek's own integration guide — the test asserts the *same* `"786432"` value across two different `redirect_model` inputs, not a branch.
- FT-11/FT-12 are the only cases in either Feature 073 or Feature 078's own test-doc that switch directly between two *different* tier-var-bearing redirect providers in one hop — every other case switches redirect↔anthropic or redirect↔non-tier-provider-redirect. These pin the cross-provider clearing behavior documented in [078_deepseek_provider_preset.md](../../../docs/feature/078_deepseek_provider_preset.md)'s Design section.
- Base-3-variable write/clear mechanics are Feature 071's — see `tests/docs/feature/071_redirect_backend_accounts.md` FT-06/FT-07; this spec covers only the 6 additional DeepSeek-tier variables, the preset sugar, and the cross-provider clear.

---

### FT-01: All 9 env vars written on switch to deepseek redirect

- **Given:** Redirect account with `inference_provider: "deepseek"`, `redirect_model: "deepseek-v4-pro"`.
- **When:** `switch_account()` targets it.
- **Then:** `settings.json` `env` holds the 3 base vars plus all 6 DeepSeek-tier vars; `ANTHROPIC_DEFAULT_OPUS_MODEL`/`ANTHROPIC_DEFAULT_SONNET_MODEL` equal `redirect_model`; `ANTHROPIC_DEFAULT_HAIKU_MODEL`/`CLAUDE_CODE_SUBAGENT_MODEL` equal `"deepseek-v4-flash"`; `CLAUDE_CODE_EFFORT_LEVEL` is `"max"`.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft06_078_switch_to_deepseek_redirect_writes_tier_env_vars`
- **Source:** [078_deepseek_provider_preset.md AC-01](../../../docs/feature/078_deepseek_provider_preset.md)

---

### FT-02: Flat compact-window regardless of model

- **Given:** Two deepseek redirect accounts — `redirect_model` `deepseek-v4-pro` vs `deepseek-v4-flash`.
- **When:** `switch_account()` targets each.
- **Then:** `CLAUDE_CODE_AUTO_COMPACT_WINDOW` is `"786432"` for both — no model-name branching, unlike Kimi's AC-02.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft07_078_switch_to_deepseek_redirect_uses_flat_compact_window_regardless_of_model`
- **Source:** [078_deepseek_provider_preset.md AC-02](../../../docs/feature/078_deepseek_provider_preset.md)

---

### FT-03: Non-deepseek provider omits tier vars

- **Given:** Redirect account whose `inference_provider` is absent or ≠ `"deepseek"` (including `"kimi"`).
- **When:** `switch_account()` targets it.
- **Then:** Only the 3 base vars written (or the 7 Kimi-tier vars if `"kimi"`) — none of the 6 DeepSeek-tier vars appear.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft08_078_switch_to_redirect_non_deepseek_provider_omits_tier_env_vars`
- **Source:** [078_deepseek_provider_preset.md AC-03](../../../docs/feature/078_deepseek_provider_preset.md)

---

### FT-04: Switch to anthropic clears all 9 vars

- **Given:** Active deepseek redirect account (9 vars present).
- **When:** `switch_account()` targets a `backend: anthropic` account.
- **Then:** All 9 vars removed — no stale tier vars remain.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft09_078_switch_from_deepseek_to_anthropic_clears_tier_env_vars`
- **Source:** [078_deepseek_provider_preset.md AC-04](../../../docs/feature/078_deepseek_provider_preset.md)

---

### FT-05: Switch to non-deepseek redirect clears stale tier vars

- **Given:** Active deepseek redirect account (9 vars present).
- **When:** `switch_account()` targets a different redirect account with `inference_provider` ≠ `"deepseek"` and ≠ `"kimi"`.
- **Then:** The new account's own 3 base vars written; the 6 stale DeepSeek-tier vars removed.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft10_078_switch_from_deepseek_to_other_redirect_clears_stale_tier_env_vars`
- **Source:** [078_deepseek_provider_preset.md AC-05](../../../docs/feature/078_deepseek_provider_preset.md)

---

### FT-06: Preset fills the three defaults

- **Given:** No pre-existing account `deepseek`.
- **When:** `clp .account.save name::deepseek preset::deepseek api_key::sk-test redirect_model::deepseek-v4-pro` (no explicit `backend::`/`base_url::`/`inference_provider::`)
- **Then:** `deepseek.json` has `backend: "redirect"`, `base_url: "https://api.deepseek.com/anthropic"`, `inference_provider: "deepseek"`.
- **Exit:** 0
- **Source fn:** `t24_save_preset_deepseek_fills_backend_base_url_and_inference_provider`
- **Source:** [078_deepseek_provider_preset.md AC-06](../../../docs/feature/078_deepseek_provider_preset.md)

---

### FT-07: Explicit `base_url::` overrides the preset

- **Given:** No pre-existing account.
- **When:** FT-06's command plus `base_url::https://custom.endpoint/anthropic`
- **Then:** The explicit value is stored — the preset default is never applied over it.
- **Exit:** 0
- **Source fn:** `t25_save_preset_deepseek_explicit_base_url_overrides_default`
- **Source:** [078_deepseek_provider_preset.md AC-07](../../../docs/feature/078_deepseek_provider_preset.md)

---

### FT-08: `preset::deepseek backend::anthropic` takes the ordinary path

- **Given:** Live credentials fixture.
- **When:** `clp .account.save name::alice@acme.com preset::deepseek backend::anthropic`
- **Then:** Ordinary OAuth-capture path; no `base_url`/`inference_provider` defaults applied — resolved `backend` gates the preset.
- **Exit:** 0
- **Source fn:** `t26_save_preset_deepseek_with_explicit_backend_anthropic_does_not_force_redirect_fields`
- **Source:** [078_deepseek_provider_preset.md AC-08](../../../docs/feature/078_deepseek_provider_preset.md)

---

### FT-09: Unrecognized preset value exits 1 naming both values

- **Given:** Any state.
- **When:** `clp .account.save name::x preset::bogus api_key::sk-test redirect_model::m1`
- **Then:** Exits 1; stderr names both recognized values (`kimi`, `deepseek`); no files written.
- **Exit:** 1
- **Source fn:** `t16_save_preset_unrecognized_value_exits_1` (shared with Feature 073 AC-09 — see Notes)
- **Source:** [078_deepseek_provider_preset.md AC-09](../../../docs/feature/078_deepseek_provider_preset.md)

---

### FT-10: End-to-end preset save + use writes all 9 vars

- **Given:** Account saved via `preset::deepseek api_key::sk-test redirect_model::deepseek-v4-pro`.
- **When:** `clp .account.use name::deepseek`
- **Then:** All 9 env vars land in `settings.json` through the CLI surface — AC-01's domain behavior reachable end-to-end.
- **Exit:** 0
- **Source fn:** `t27_use_preset_deepseek_account_writes_deepseek_tier_env_vars`
- **Source:** [078_deepseek_provider_preset.md AC-10](../../../docs/feature/078_deepseek_provider_preset.md)

---

### FT-11: Switch kimi→deepseek clears Kimi bundle, writes DeepSeek bundle

- **Given:** Active kimi redirect account (7 Kimi-tier vars present).
- **When:** `switch_account()` targets a deepseek redirect account.
- **Then:** All 7 Kimi-tier vars removed; all 6 DeepSeek-tier vars written — no Kimi-tier var survives alongside the new bundle.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft11_078_switch_from_kimi_to_deepseek_clears_kimi_writes_deepseek`
- **Source:** [078_deepseek_provider_preset.md AC-11](../../../docs/feature/078_deepseek_provider_preset.md)

---

### FT-12: Switch deepseek→kimi clears DeepSeek bundle, writes Kimi bundle

- **Given:** Active deepseek redirect account (6 DeepSeek-tier vars present).
- **When:** `switch_account()` targets a kimi redirect account.
- **Then:** All 6 DeepSeek-tier vars removed; all 7 Kimi-tier vars written — no DeepSeek-tier var survives alongside the new bundle.
- **Exit:** n/a (domain fn)
- **Source fn:** `ft12_078_switch_from_deepseek_to_kimi_clears_deepseek_writes_kimi`
- **Source:** [078_deepseek_provider_preset.md AC-11](../../../docs/feature/078_deepseek_provider_preset.md)

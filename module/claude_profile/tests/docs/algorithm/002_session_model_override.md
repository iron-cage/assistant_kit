# Algorithm 002: Session Model Override

AC test cases for `docs/algorithm/002_session_model_override.md`. Tests `apply_model_override(quota, paths, false, "test", name, backend)` in `src/usage/api_switch.rs` (where `quota: &OauthUsageData`, `backend: AccountBackend`) and `recommended_model(aq)` in `src/usage/format.rs` (where `aq: &AccountQuota`).

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | Absent Sonnet tier + Opus session → Sonnet restored | Nominal (BUG-311 fix) | ✅ |
| AC-2 | Absent Sonnet tier + Sonnet session → no-op | Nominal | ✅ |
| AC-3 | Sufficient Sonnet + Opus session → Sonnet restored | Nominal (BUG-311 fix) | ✅ |
| AC-4 | Near-exhausted Sonnet + Sonnet session → Opus written | Nominal | ✅ |
| AC-5 | Near-exhausted Sonnet + Opus session → no-op | Boundary | ✅ |
| AC-6 | `recommended_model()` divergence: sufficient vs near-exhausted | Regression (BUG-300) | ✅ |
| AC-7 | Opus branch sets effort to `"max"` unconditionally | Fix BUG-322, TSK-335 | ✅ |
| AC-8 | Sonnet branch (sufficient quota) sets effort to `"high"` unconditionally | Fix BUG-322, TSK-335 | ✅ |
| AC-9 | Absent-tier path sets effort to `"high"` unconditionally | Fix BUG-322, TSK-335 | ✅ |
| AC-10 | Effort synced even when model is already at target (no-op path) | TSK-335 H2 always-sync | ✅ |
| AC-11 | BUG-312 fallback writes `"high"` when absent (unreachable after AC-7..AC-9) | TSK-335 | ✅ |
| AC-12 | Bare full model ID normalized to the shorthand in both directions | Regression (BUG-286, BUG-578) | ✅ |

---

### AC-1: Absent Sonnet tier with Opus session restores Sonnet

- **Given:** `OauthUsageData { seven_day_sonnet: None }`; `settings.json` model field = `"opus"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account", AccountBackend::Anthropic)` is called
- **Then:** `settings.json` model field is written to `"sonnet"` — tier absence is treated as a conservative restore signal, not as exhaustion
- **Note:** Fix BUG-311 — pre-fix code had a one-way ratchet that only wrote `"opus"` (the `None` row had no `else`-branch); `override_session_model_to_sonnet()` was added for the `None` + Opus combination
- **Source fn:** `ac1_absent_tier_with_opus_session_restores_sonnet` (api_tests_b.rs)

### AC-2: Absent Sonnet tier with Sonnet session is a no-op

- **Given:** `OauthUsageData { seven_day_sonnet: None }`; `settings.json` model field = `"sonnet"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account", AccountBackend::Anthropic)` is called
- **Then:** `settings.json` model field stays `"sonnet"` — already the shorthand target, so `override_session_model_to_sonnet()` returns `false` and writes no model (effort is still written — AC-10). A full Sonnet ID is not a no-op here — see AC-12
- **Source fn:** `ac2_absent_tier_with_sonnet_session_model_field_unchanged` (api_tests_b.rs)

### AC-3: Sufficient Sonnet quota with Opus session restores Sonnet

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 80.0, .. }) }` — 20% remaining, at or above the 10% threshold; `settings.json` model field = `"opus"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account", AccountBackend::Anthropic)` is called
- **Then:** `settings.json` model field is written to `"sonnet"` — capacity is sufficient; the override is reversed
- **Note:** Fix BUG-311 — the recovery path (`Some` + sufficient + Opus → Sonnet) was absent before the fix
- **Source fn:** `ac3_sufficient_quota_with_opus_session_restores_sonnet` (api_tests_b.rs)

### AC-4: Near-exhausted Sonnet quota with Sonnet session switches to Opus

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 91.0, resets_at: Some("...") }) }` — 9% remaining, below the 10% threshold; `settings.json` model field = `"sonnet"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account", AccountBackend::Anthropic)` is called
- **Then:** `settings.json` model field is written to `"opus"` — Sonnet is near-exhausted; switch to Opus to preserve remaining quota
- **Source fn:** `ac4_near_exhausted_quota_with_sonnet_session_switches_to_opus` (api_tests_b.rs)

### AC-5: Near-exhausted Sonnet quota with Opus session is a no-op

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 91.0, resets_at: Some("...") }) }` — 9% remaining; `settings.json` model field = `"opus"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account", AccountBackend::Anthropic)` is called
- **Then:** `settings.json` model field stays `"opus"` — already the shorthand target, so `override_session_model_to_opus()` returns `false` and writes no model (effort is still written — AC-10). A full Opus ID is not a no-op here — see AC-12
- **Source fn:** `ac5_near_exhausted_quota_with_opus_session_model_field_unchanged` (api_tests_b.rs)

### AC-6: `recommended_model()` divergence — sufficient vs near-exhausted

- **Given:** Two quota states: (A) `seven_day_sonnet = Some(PeriodUsage { utilization: 80.0 })` — 20% remaining; (B) `seven_day_sonnet = Some(PeriodUsage { utilization: 91.0 })` — 9% remaining
- **When:** `recommended_model(aq)` is called for each state independently
- **Then:** (A) returns `"sonnet"`; (B) returns `"opus"` — the two inputs produce divergent outputs, proving `recommended_model()` governs model selection rather than returning a constant string
- **Note:** Fix BUG-300 — pre-fix, `map_or(0.0, ...)` on `seven_day_sonnet = None` produced 0.0 < threshold, causing `recommended_model()` to return `"opus"` unconditionally for accounts without a Sonnet tier; `if let Some(ref sonnet)` guard prevents this

### AC-7: Opus branch sets effort to `"max"` unconditionally (Fix BUG-322, TSK-335)

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 91.0, resets_at: None }) }` — 9% remaining (< 10%); no `settings.json` initially
- **When:** `apply_model_override(&quota, &paths, false, "usage", "test-account", AccountBackend::Anthropic)` is called
- **Then:** `settings.json` contains `"model": "opus"` AND `"effortLevel": "max"` — effort written unconditionally in Opus branch regardless of `overrode` (TSK-335: was `"high"`, and only written when `overrode = true`)
- **Source fn:** `mre_bug322_opus_override_sets_effort_max` (api_tests_a.rs)

### AC-8: Sonnet branch sets effort to `"high"` unconditionally (Fix BUG-322, TSK-335)

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 4.0, resets_at: None }) }` — 96% remaining (≥ 10%); `settings.json` pre-seeded with `"model": "opus", "effortLevel": "max"`
- **When:** `apply_model_override(&quota, &paths, false, "usage", "test-account", AccountBackend::Anthropic)` is called
- **Then:** `settings.json` contains `"model": "sonnet"` AND `"effortLevel": "high"` — effort written unconditionally in Sonnet branch regardless of `overrode` (TSK-335: was `"low"`, only written when `overrode = true`)
- **Source fn:** `t11_opus_to_sonnet_sets_effort_high` (api_tests_a.rs)

### AC-9: Absent-tier path sets effort to `"high"` unconditionally (Fix BUG-322, TSK-335)

- **Given:** `OauthUsageData { seven_day_sonnet: None }` (absent tier); `settings.json` pre-seeded with `"model": "opus", "effortLevel": "max"`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account", AccountBackend::Anthropic)` is called
- **Then:** `settings.json` contains `"model": "sonnet"` AND `"effortLevel": "high"` — absent tier forces Sonnet + effort written unconditionally (TSK-335: was `"low"`)
- **Source fn:** `t12_absent_tier_with_opus_sets_effort_high` (api_tests_a.rs)

### AC-10: Effort synced even when model is already at target (TSK-335 H2 always-sync)

- **Given:** `settings.json` pre-seeded with `"model": "sonnet"` (no `effortLevel`); Sonnet left ≥ 10% — `override_session_model_to_sonnet()` returns `false` (already Sonnet, model does not change; `overrode = false`)
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account", AccountBackend::Anthropic)` is called
- **Then:** `settings.json` contains `"effortLevel": "high"` — effort written even though `overrode = false`; effort writes are unconditional, not gated on `if overrode`
- **Source fn:** `ft19_effort_synced_when_model_already_at_target` — new test for TSK-335

### AC-11: BUG-312 fallback writes `"high"` when effortLevel absent (effectively unreachable after AC-7..AC-9)

- **Given:** A scenario where neither Opus nor Sonnet branch writes effort (edge case; unreachable in current code since all branches write unconditionally)
- **When:** `apply_model_override` reaches the `get_session_effort().is_none()` guard
- **Then:** `settings.json` contains `"effortLevel": "high"` as safety fallback (TSK-335: was `"low"`)
- **Note:** This guard is retained for safety but is unreachable when any of AC-7..AC-9 fire first

### AC-12: Bare full model ID normalized to the shorthand in both directions (Fix BUG-286, BUG-578)

- **Given:** `settings.json` model field = a bare full ID — `OPUS_MODEL_ID` (`"claude-opus-5-5"`) or the legacy `"claude-opus-4-8"` with 9% Sonnet remaining; `SONNET_MODEL_ID` (`"claude-sonnet-5"`) with 50% remaining
- **When:** `apply_model_override(&quota, &paths, false, "account.use", "test-account", AccountBackend::Anthropic)` is called
- **Then:** the model field is rewritten to the target shorthand — `"opus"` for both Opus IDs, `"sonnet"` for the Sonnet ID — and the full ID is gone
- **Note:** Before BUG-578 the → Opus gate listed exact IDs (`"claude-opus-4-8"`, `"claude-opus-4-6"`), so `"claude-opus-5-5"` (clp's own `opus` value after the Opus 5.5 remap) was left as-is and the `sonnet→opus` trace never fired. The test reads the constants, so the next remap can't reopen this silently. `[`-suffixed IDs such as `"claude-opus-5-5[1m]"` stay untouched; that gate-level case lives in claude_profile_core's `mre_bug578_bare_full_opus_ids_normalized_to_shorthand`
- **Source fn:** `mre_bug286_full_opus_id_normalized_to_shorthand`, `mre_bug578_own_full_model_ids_normalized_both_directions` (api_tests_a.rs)

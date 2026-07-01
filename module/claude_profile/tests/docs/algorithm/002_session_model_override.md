# Algorithm 002: Session Model Override

AC test cases for `docs/algorithm/002_session_model_override.md`. Tests `apply_model_override(quota, paths, false, "test", name)` in `src/usage/api.rs` (where `quota: &OauthUsageData`) and `recommended_model(aq)` in `src/usage/format.rs` (where `aq: &AccountQuota`).

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

---

### AC-1: Absent Sonnet tier with Opus session restores Sonnet

- **Given:** `OauthUsageData { seven_day_sonnet: None }`; `settings.json` model field = `"claude-opus-4-6"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account")` is called
- **Then:** `settings.json` model field is written to `"claude-sonnet-4-6"` — tier absence is treated as a conservative restore signal, not as exhaustion
- **Note:** Fix BUG-311 — pre-fix code had a one-way ratchet that only wrote `"opus"` (the `None` row had no `else`-branch); `override_session_model_to_sonnet()` was added for the `None` + Opus combination

### AC-2: Absent Sonnet tier with Sonnet session is a no-op

- **Given:** `OauthUsageData { seven_day_sonnet: None }`; `settings.json` model field = `"claude-sonnet-4-6"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account")` is called
- **Then:** `settings.json` model field is unchanged — session is already in Sonnet form; no write occurs

### AC-3: Sufficient Sonnet quota with Opus session restores Sonnet

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 80.0, .. }) }` — 20% remaining, at or above the 10% threshold; `settings.json` model field = `"claude-opus-4-6"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account")` is called
- **Then:** `settings.json` model field is written to `"claude-sonnet-4-6"` — capacity is sufficient; the override is reversed
- **Note:** Fix BUG-311 — the recovery path (`Some` + sufficient + Opus → Sonnet) was absent before the fix

### AC-4: Near-exhausted Sonnet quota with Sonnet session switches to Opus

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 91.0, resets_at: Some("...") }) }` — 9% remaining, below the 10% threshold; `settings.json` model field = `"claude-sonnet-4-6"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account")` is called
- **Then:** `settings.json` model field is written to `"claude-opus-4-6"` — Sonnet is near-exhausted; switch to Opus to preserve remaining quota

### AC-5: Near-exhausted Sonnet quota with Opus session is a no-op

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 91.0, resets_at: Some("...") }) }` — 9% remaining; `settings.json` model field = `"claude-opus-4-6"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account")` is called
- **Then:** `settings.json` model field is unchanged — session is already in Opus form; redundant write avoided

### AC-6: `recommended_model()` divergence — sufficient vs near-exhausted

- **Given:** Two quota states: (A) `seven_day_sonnet = Some(PeriodUsage { utilization: 80.0 })` — 20% remaining; (B) `seven_day_sonnet = Some(PeriodUsage { utilization: 91.0 })` — 9% remaining
- **When:** `recommended_model(aq)` is called for each state independently
- **Then:** (A) returns `"sonnet"`; (B) returns `"opus"` — the two inputs produce divergent outputs, proving `recommended_model()` governs model selection rather than returning a constant string
- **Note:** Fix BUG-300 — pre-fix, `map_or(0.0, ...)` on `seven_day_sonnet = None` produced 0.0 < threshold, causing `recommended_model()` to return `"opus"` unconditionally for accounts without a Sonnet tier; `if let Some(ref sonnet)` guard prevents this

### AC-7: Opus branch sets effort to `"max"` unconditionally (Fix BUG-322, TSK-335)

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 91.0, resets_at: None }) }` — 9% remaining (< 10%); no `settings.json` initially
- **When:** `apply_model_override(&quota, &paths, false, "usage", "test-account")` is called
- **Then:** `settings.json` contains `"model": "opus"` AND `"effortLevel": "max"` — effort written unconditionally in Opus branch regardless of `overrode` (TSK-335: was `"high"`, and only written when `overrode = true`)
- **Source fn:** `mre_bug322_opus_override_sets_effort_max` (api_tests.rs)

### AC-8: Sonnet branch sets effort to `"high"` unconditionally (Fix BUG-322, TSK-335)

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 4.0, resets_at: None }) }` — 96% remaining (≥ 10%); `settings.json` pre-seeded with `"model": "opus", "effortLevel": "max"`
- **When:** `apply_model_override(&quota, &paths, false, "usage", "test-account")` is called
- **Then:** `settings.json` contains `"model": "sonnet"` AND `"effortLevel": "high"` — effort written unconditionally in Sonnet branch regardless of `overrode` (TSK-335: was `"low"`, only written when `overrode = true`)
- **Source fn:** `t11_opus_to_sonnet_sets_effort_high` (api_tests.rs)

### AC-9: Absent-tier path sets effort to `"high"` unconditionally (Fix BUG-322, TSK-335)

- **Given:** `OauthUsageData { seven_day_sonnet: None }` (absent tier); `settings.json` pre-seeded with `"model": "opus", "effortLevel": "max"`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account")` is called
- **Then:** `settings.json` contains `"model": "sonnet"` AND `"effortLevel": "high"` — absent tier forces Sonnet + effort written unconditionally (TSK-335: was `"low"`)
- **Source fn:** `t12_absent_tier_with_opus_sets_effort_high` (api_tests.rs)

### AC-10: Effort synced even when model is already at target (TSK-335 H2 always-sync)

- **Given:** `settings.json` pre-seeded with `"model": "sonnet"` (no `effortLevel`); Sonnet left ≥ 10% — `override_session_model_to_sonnet()` returns `false` (already Sonnet, model does not change; `overrode = false`)
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account")` is called
- **Then:** `settings.json` contains `"effortLevel": "high"` — effort written even though `overrode = false`; effort writes are unconditional, not gated on `if overrode`
- **Source fn:** `ft19_effort_synced_when_model_already_at_target` — new test for TSK-335

### AC-11: BUG-312 fallback writes `"high"` when effortLevel absent (effectively unreachable after AC-7..AC-9)

- **Given:** A scenario where neither Opus nor Sonnet branch writes effort (edge case; unreachable in current code since all branches write unconditionally)
- **When:** `apply_model_override` reaches the `get_session_effort().is_none()` guard
- **Then:** `settings.json` contains `"effortLevel": "high"` as safety fallback (TSK-335: was `"low"`)
- **Note:** This guard is retained for safety but is unreachable when any of AC-7..AC-9 fire first

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

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 80.0, .. }) }` — 20% remaining, at or above the 15% threshold; `settings.json` model field = `"claude-opus-4-6"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account")` is called
- **Then:** `settings.json` model field is written to `"claude-sonnet-4-6"` — capacity is sufficient; the override is reversed
- **Note:** Fix BUG-311 — the recovery path (`Some` + sufficient + Opus → Sonnet) was absent before the fix

### AC-4: Near-exhausted Sonnet quota with Sonnet session switches to Opus

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 86.0, resets_at: Some("...") }) }` — 14% remaining, below the 15% threshold; `settings.json` model field = `"claude-sonnet-4-6"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account")` is called
- **Then:** `settings.json` model field is written to `"claude-opus-4-6"` — Sonnet is near-exhausted; switch to Opus to preserve remaining quota

### AC-5: Near-exhausted Sonnet quota with Opus session is a no-op

- **Given:** `OauthUsageData { seven_day_sonnet: Some(PeriodUsage { utilization: 86.0, resets_at: Some("...") }) }` — 14% remaining; `settings.json` model field = `"claude-opus-4-6"` written to a temp `ClaudePaths`
- **When:** `apply_model_override(&quota, &paths, false, "test", "test-account")` is called
- **Then:** `settings.json` model field is unchanged — session is already in Opus form; redundant write avoided

### AC-6: `recommended_model()` divergence — sufficient vs near-exhausted

- **Given:** Two quota states: (A) `seven_day_sonnet = Some(PeriodUsage { utilization: 80.0 })` — 20% remaining; (B) `seven_day_sonnet = Some(PeriodUsage { utilization: 86.0 })` — 14% remaining
- **When:** `recommended_model(aq)` is called for each state independently
- **Then:** (A) returns `"sonnet"`; (B) returns `"opus"` — the two inputs produce divergent outputs, proving `recommended_model()` governs model selection rather than returning a constant string
- **Note:** Fix BUG-300 — pre-fix, `map_or(0.0, ...)` on `seven_day_sonnet = None` produced 0.0 < threshold, causing `recommended_model()` to return `"opus"` unconditionally for accounts without a Sonnet tier; `if let Some(ref sonnet)` guard prevents this

# Algorithm 001: Touch Model Selection

AC test cases for `docs/algorithm/001_touch_model_selection.md`. Tests `resolve_model(aq: &AccountQuota, imodel: SubprocessModel) -> IsolatedModel` in `src/usage/subprocess.rs`.

**Type note:** `imodel` is a `SubprocessModel` enum — `SubprocessModel::Haiku`, `::Sonnet`, `::Opus`, `::Keep`, `::Auto`. When clauses use shorthand names (`Haiku`, `Auto`, etc.) for brevity.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | `Haiku` param → Haiku regardless of quota | Nominal | ✅ |
| AC-2 | `Sonnet` param → Sonnet regardless of quota | Nominal | ✅ |
| AC-3 | Auto with no Sonnet tier → Haiku | Boundary | ✅ |
| AC-4 | Auto with idle Sonnet window → Sonnet | Nominal | ✅ |
| AC-5 | Auto with active Sonnet window and > 20% available → Sonnet | Regression (BUG-301) | ✅ |
| AC-6 | Auto with active Sonnet window and ≤ 20% available → Haiku | Boundary | ✅ |

---

### AC-1: `Haiku` param selects Haiku regardless of quota

- **Given:** An account quota with `seven_day_sonnet = Some(PeriodUsage { utilization: 0.0, resets_at: None })` — Sonnet is fully available and idle
- **When:** `resolve_model(aq, SubprocessModel::Haiku)` is called
- **Then:** Returns `Specific("claude-haiku-4-5-20251001")` — explicit `imodel` overrides all quota data; Sonnet availability is irrelevant

### AC-2: `Sonnet` param selects Sonnet regardless of quota

- **Given:** An account quota with `seven_day_sonnet = None` — no Sonnet tier present
- **When:** `resolve_model(aq, SubprocessModel::Sonnet)` is called
- **Then:** Returns `Specific("claude-sonnet-4-6")` — explicit `imodel` overrides all quota data; tier absence is irrelevant

### AC-3: Auto mode with no Sonnet tier selects Haiku

- **Given:** An account quota with `seven_day_sonnet = None`
- **When:** `resolve_model(aq, SubprocessModel::Auto)` is called
- **Then:** Returns `Specific("claude-haiku-4-5-20251001")` — `seven_day_sonnet` is absent; the `auto` branch falls through to the Haiku default
- **Note:** As of 2026-06-25, this is the always-taken path because `seven_day_sonnet` is permanently `null` in the API response until Feature 066 recovery populates it from the `limits` array

### AC-4: Auto mode with idle Sonnet window selects Sonnet

- **Given:** An account quota with `seven_day_sonnet = Some(PeriodUsage { utilization: 30.0, resets_at: None })` — Sonnet weekly window is idle (`son_idle = true`; no active tracking period)
- **When:** `resolve_model(aq, SubprocessModel::Auto)` is called
- **Then:** Returns `Specific("claude-sonnet-4-6")` — `son_idle = true`; a Haiku touch cannot open an idle Sonnet session window, so Sonnet is selected to hold the window open

### AC-5: Auto mode with active Sonnet window and capacity selects Sonnet

- **Given:** An account quota with `seven_day_sonnet = Some(PeriodUsage { utilization: 75.0, resets_at: Some("2026-06-28T04:00:00+00:00") })` — 25% remaining, which is > 20% threshold
- **When:** `resolve_model(aq, SubprocessModel::Auto)` is called
- **Then:** Returns `Specific("claude-sonnet-4-6")` — `son_available = (100.0 - 75.0) > 20.0 = true`; quota available, avoid wasting the expiring window
- **Note:** BUG-301 regression — before fix, the binary `son_idle` gate (only `resets_at == None`) ignored utilization entirely; `son_available` was not computed, causing Haiku to be selected when a Sonnet window was active with remaining quota

### AC-6: Auto mode with nearly-exhausted Sonnet window selects Haiku

- **Given:** An account quota with `seven_day_sonnet = Some(PeriodUsage { utilization: 85.0, resets_at: Some("2026-06-28T04:00:00+00:00") })` — 15% remaining, which is ≤ 20% threshold
- **When:** `resolve_model(aq, SubprocessModel::Auto)` is called
- **Then:** Returns `Specific("claude-haiku-4-5-20251001")` — `son_available = (100.0 - 85.0) > 20.0 = false`; Sonnet quota is nearly exhausted; conserve reserves for direct user sessions

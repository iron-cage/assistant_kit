# Pitfall: Quota Gate Pitfalls

### Scope

- **Purpose**: Document non-obvious failure modes in quota gate logic.
- **Responsibility**: Covers raw-vs-weighted metric confusion, absent-tier/exhaustion misclassification, window capacity independence, and cancelled-subscription classification errors.
- **In Scope**: `status_group_of()`, `apply_model_override()`, `resolve_model()` quota gate pitfalls; BUG-299, BUG-300, BUG-301, BUG-317.
- **Out of Scope**: Model override algorithm details (→ algorithm/002); subprocess model selection (→ algorithm/001).

### Pattern

Logic that gates on quota values has multiple non-obvious invariants: the correct metric (raw vs. strategy-weighted), the correct direction (absence ≠ exhaustion), and the correct utilization formula.

### Pitfall 1 — Status groups must use raw `seven_day_left`, not `prefer_weekly`

**Root cause (BUG-299):** `status_group_of()` used `prefer_weekly(aq, prefer)` for the 7d threshold. `prefer_weekly` is strategy-weighted — with `prefer::son`, accounts without a Sonnet tier got `prefer_weekly = 0.0 ≤ 5.0`, forcing them into the weekly-exhausted group regardless of actual 7d quota.

**Fix:** `status_group_of()` uses `seven_day_left(aq)` (raw, strategy-independent). `prefer_weekly` is only used for sort key computation (strategy-aware tiebreak contexts). Eligibility gate 7 uses raw `seven_day_left` (Fix BUG-324).

**Rule:** Status group partition and eligibility gates = raw per-dimension thresholds. Sort tiebreaks = strategy-weighted. Never use strategy-weighted values for group/gate boundary decisions.

### Pitfall 2 — Absent Sonnet tier is NOT exhaustion

**Root cause (BUG-300, BUG-311):** `apply_model_override()` used `map_or(0.0, |son| 100 - son.utilization)` to compute Sonnet remaining capacity. When `seven_day_sonnet = None`, this returned `0.0 < threshold`, triggering Opus override for ALL accounts without a Sonnet tier — even accounts with plenty of general quota.

**Fix:** `if let Some(ref sonnet)` guard before any threshold comparison. `None = no Sonnet tier` must be handled separately from `Some { utilization ≥ 85% } = exhausted Sonnet`. See [algorithm/002](../algorithm/002_session_model_override.md).

**Rule:** Always check `Option::is_some()` before comparing quota values. `None` and `0.0` are different states with opposite operational meanings.

### Pitfall 3 — `son_available` must check utilization, not just window state

**Root cause (BUG-301):** `resolve_model(Auto)` only checked `son_idle = resets_at.is_none()`. When the Sonnet window was active (`resets_at = Some`) with 40% remaining quota, the account was treated as Haiku-only — wasting Sonnet capacity as the window timer expired.

**Fix:** Added `son_available = (100.0 - son.utilization) > 20.0` gate. See [algorithm/001](../algorithm/001_touch_model_selection.md).

**Rule:** Session window presence (`resets_at = Some`) and session window capacity (`utilization`) are independent dimensions. Check both.

### Pitfall 4 — Cancelled subscription (`billing_type="none"`) is NOT a quota state

**Root cause (BUG-317):** `status_emoji()` and row filter functions (`only_valid`, `exclude_exhausted`) only inspected `result` (`Ok`/`Err`) to classify accounts. A cancelled account with `billing_type="none"` could still have `result = Ok(...)` (the API may return quota data for recently cancelled accounts), causing it to appear as 🟢/🟡 — misleading the user into thinking the account was temporarily exhausted rather than permanently dead.

**Fix:** Four sites patched: (A) `status_group_of()` in `sort.rs` gates on `billing_type` before quota thresholds → Red. (B) `find_first_eligible()` in `sort_next.rs` skips cancelled accounts (Gate 3b). (C) `status_emoji()` in `format.rs` changed from `&Result<OauthUsageData, String>` to `&AccountQuota`; `billing_type="none"` → 🔴. (D) `only_valid` filter in `api.rs` explicitly excludes `billing_type="none"`.

**Rule:** `billing_type` is a subscription-level signal independent of quota values. Check it before any quota threshold. `account = None` (API fetch failed) is NOT the same as `billing_type = "none"` (confirmed cancelled) — absent data is ambiguous and must not trigger the cancelled path.

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/001](../algorithm/001_touch_model_selection.md) | Touch model selection (Pitfall 3) |
| [algorithm/002](../algorithm/002_session_model_override.md) | Session model override (Pitfall 2) |
| [algorithm/003](../algorithm/003_quota_status_groups.md) | Status groups (Pitfall 1, Pitfall 4) |
| [algorithm/004](../algorithm/004_eligibility_gates.md) | Eligibility gates — Gate 3b (Pitfall 4) |

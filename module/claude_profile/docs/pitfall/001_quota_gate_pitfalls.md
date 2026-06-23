# Pitfall: Quota Gate Pitfalls

### Pattern

Logic that gates on quota values has multiple non-obvious invariants: the correct metric (raw vs. strategy-weighted), the correct direction (absence ≠ exhaustion), and the correct utilization formula.

### Pitfall 1 — Status groups must use raw `seven_day_left`, not `prefer_weekly`

**Root cause (BUG-299):** `status_group_of()` used `prefer_weekly(aq, prefer)` for the 7d threshold. `prefer_weekly` is strategy-weighted — with `prefer::son`, accounts without a Sonnet tier got `prefer_weekly = 0.0 ≤ 5.0`, forcing them into the weekly-exhausted group regardless of actual 7d quota.

**Fix:** `status_group_of()` uses `seven_day_left(aq)` (raw, strategy-independent). `prefer_weekly` is only used for eligibility gate 7 and sort key computation (strategy-aware contexts).

**Rule:** Status group partition = raw per-dimension thresholds. Eligibility gates = strategy-weighted thresholds. Never mix them.

### Pitfall 2 — Absent Sonnet tier is NOT exhaustion

**Root cause (BUG-300, BUG-311):** `apply_model_override()` used `map_or(0.0, |son| 100 - son.utilization)` to compute Sonnet remaining capacity. When `seven_day_sonnet = None`, this returned `0.0 < threshold`, triggering Opus override for ALL accounts without a Sonnet tier — even accounts with plenty of general quota.

**Fix:** `if let Some(ref sonnet)` guard before any threshold comparison. `None = no Sonnet tier` must be handled separately from `Some { utilization ≥ 85% } = exhausted Sonnet`. See [algorithm/002](../algorithm/002_session_model_override.md).

**Rule:** Always check `Option::is_some()` before comparing quota values. `None` and `0.0` are different states with opposite operational meanings.

### Pitfall 3 — `son_available` must check utilization, not just window state

**Root cause (BUG-301):** `resolve_model(Auto)` only checked `son_idle = resets_at.is_none()`. When the Sonnet window was active (`resets_at = Some`) with 40% remaining quota, the account was treated as Haiku-only — wasting Sonnet capacity as the window timer expired.

**Fix:** Added `son_available = (100.0 - son.utilization) > 20.0` gate. See [algorithm/001](../algorithm/001_touch_model_selection.md).

**Rule:** Session window presence (`resets_at = Some`) and session window capacity (`utilization`) are independent dimensions. Check both.

### Cross-References

| File | Relationship |
|------|-------------|
| [algorithm/001](../algorithm/001_touch_model_selection.md) | Touch model selection (Pitfall 3) |
| [algorithm/002](../algorithm/002_session_model_override.md) | Session model override (Pitfall 2) |
| [algorithm/003](../algorithm/003_quota_status_groups.md) | Status groups (Pitfall 1) |

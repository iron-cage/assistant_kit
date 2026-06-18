# Feature: Next Account Recommendation Strategies — DEPRECATED

> **DEPRECATED** — Absorbed into [020_usage_sort_strategies.md](020_usage_sort_strategies.md). The `next::` parameter has been removed; the `→` recommendation marker is now driven by [`sort::`](../cli/param/025_sort.md) — a single parameter controls both row ordering and the recommendation. The top eligible account in the active sort order receives `→`. The footer shows one recommendation line for the active `sort::` strategy (not three). See [param/032_next.md](../cli/param/032_next.md) for migration notes.

**Migration:**
- `next::renew` → `sort::renew` (default — no change needed)
- `next::endurance` → no replacement (strategy removed)
- `next::drain` → no replacement (strategy removed)
- 3-strategy footer → 1-line footer for the active `sort::` strategy

### Bugs

| File | Relationship |
|------|--------------|
| `task/claude_profile/bug/243_renew_strategy_missing_5h_tiebreaker.md` | BUG-243 ✅ Fixed: superseded by BUG-291 |
| `task/claude_profile/bug/260_renew_next_selection_nondeterministic_when_fully_tied.md` | BUG-260 ✅ Fixed: name tiebreaker added |
| `task/claude_profile/bug/287_endurance_missing_weekly_floor_gate.md` | BUG-287 ✅ Fixed: endurance weekly-floor gate (strategy now removed) |
| `task/claude_profile/bug/291_renew_next_uses_parallel_sort_instead_of_sort_indices.md` | BUG-291 ✅ Fixed: unified sort/recommendation algorithm |
| `task/claude_profile/bug/292_renew_next_recommends_weekly_exhausted_account.md` | BUG-292 ✅ Fixed: weekly-floor gate on renew |

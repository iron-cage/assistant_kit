# Feature: Next Account Recommendation Strategies — DEPRECATED

> **DEPRECATED** — Absorbed into [020_usage_sort_strategies.md](020_usage_sort_strategies.md). The `next::` parameter has been removed; the footer recommendation is now driven by [`sort::`](../cli/param/025_sort.md) — a single parameter controls both row ordering and the recommendation. The top eligible account in the active sort order is shown in the footer's `Next (strategy):` line. The footer shows one recommendation line for the active `sort::` strategy (not three). See [param/032_next.md](../cli/param/032_next.md) for migration notes.

**Migration:**
- `next::renew` → `sort::renew` (default — no change needed)
- `next::endurance` → no replacement (strategy removed)
- `next::drain` → no replacement (strategy removed)
- 3-strategy footer → 1-line footer for the active `sort::` strategy

### Bugs

| File | Relationship |
|------|--------------|
| BUG-243 | BUG-243 ✅ Fixed: superseded by BUG-291 |
| BUG-260 | BUG-260 ✅ Fixed: name tiebreaker added |
| BUG-287 | BUG-287 ✅ Fixed: endurance weekly-floor gate (strategy now removed) |
| BUG-291 | BUG-291 ✅ Fixed: unified sort/recommendation algorithm |
| BUG-292 | BUG-292 ✅ Fixed: weekly-floor gate on renew |

# Parameter :: 32. `next::` — REMOVED

Removed. The `→` recommendation marker is now driven by [`sort::`](025_sort.md) — a single parameter controls both row ordering and the recommendation. The top eligible account in the active sort order receives `→`.

**Migration:** `next::renew` → `sort::renew` (default). `next::endurance` and `next::drain` have no replacement (strategies removed).

**See Also:** [`sort::`](025_sort.md) for current behavior.

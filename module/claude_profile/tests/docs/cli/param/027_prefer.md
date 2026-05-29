# Test: `prefer::` Parameter

Edge case coverage for the `prefer::` parameter on `.usage`. See [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `prefer::any` accepted with empty store | Valid Value |
| EC-2 | `prefer::opus` accepted with empty store | Valid Value |
| EC-3 | `prefer::sonnet` accepted with empty store | Valid Value |
| EC-4 | `prefer::bogus` exits 1 and names valid values | Invalid Value |
| CC-1 | `prefer::` without `sort::` accepted (no-op with default sort::renew on empty store) | Isolation |
| CC-2 | `prefer::sonnet` vs `prefer::any` changes endurance qualification | Behavioral Divergence |

---

### EC-1: `prefer::any` accepted with empty store

- **Given:** Empty credential store.
- **When:** `clp .usage prefer::any`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md)

---

### EC-2: `prefer::opus` accepted with empty store

- **Given:** Empty credential store.
- **When:** `clp .usage prefer::opus`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md)

---

### EC-3: `prefer::sonnet` accepted with empty store

- **Given:** Empty credential store.
- **When:** `clp .usage prefer::sonnet`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source:** [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md)

---

### EC-4: `prefer::bogus` exits 1 and names valid values

- **Given:** Any environment.
- **When:** `clp .usage prefer::bogus`
- **Then:** Exits 1. Stderr contains the three valid values: "any", "opus", "sonnet".
- **Exit:** 1
- **Source:** [feature/020_usage_sort_strategies.md AC-10](../../../../docs/feature/020_usage_sort_strategies.md)

---

### CC-1: `prefer::` without `sort::` accepted and does not break default output

- **Behavioral Divergence:** `prefer::sonnet sort::endurance` uses `7d(Son)` for qualification; `prefer::any sort::endurance` uses `min(7d Left, 7d(Son))`. The divergence is in which weekly column governs endurance classification — not tested here (unit-level); this case just confirms `prefer::` is accepted with `sort::renew` (the default) without error.
- **Given:** Empty credential store.
- **When:** `clp .usage prefer::sonnet`
- **Then:** Exits 0 with "(no accounts configured)". `prefer::` is parsed silently — it only affects sort heuristics, not the default renew sort ordering with empty store.
- **Exit:** 0
- **Source:** [param/027_prefer.md](../../../../docs/cli/param/027_prefer.md)

---

### CC-2: `prefer::sonnet` vs `prefer::any` changes endurance qualification

- **Behavioral Divergence:** Same `AccountQuota` data under `sort::endurance prefer::sonnet` vs `sort::endurance prefer::any` produces a different qualified/unqualified tier assignment when `7d(Son) ≥ 30%` but `min(7d Left, 7d(Son)) < 30%`.
- **Given:** One `AccountQuota` struct: `seven_day.utilization=90%` (10% left), `seven_day_sonnet.utilization=65%` (35% left). 5h_reset within 30 min (reset window satisfied).
- **When-A:** `sort_indices(..., SortStrategy::Endurance, None, PreferStrategy::Sonnet, now_secs)` — `prefer_weekly` = 35% ≥ 30% → **qualified**.
- **When-B:** `sort_indices(..., SortStrategy::Endurance, None, PreferStrategy::Any, now_secs)` — `prefer_weekly` = min(10%, 35%) = 10% < 30% → **unqualified**.
- **Then-A:** Account placed in qualified tier (ranked above any unqualified accounts).
- **Then-B:** Account placed in unqualified tier (ranked below any qualified accounts).
- **Exit:** n/a (unit test — function return assertion)
- **Source fn:** `test_prefer_sonnet_qualifies_by_sonnet_quota` (in `src/usage/sort.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-07](../../../../docs/feature/020_usage_sort_strategies.md)

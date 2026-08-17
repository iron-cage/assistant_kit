# Test: `min_5h::` Parameter

Edge case coverage for the `min_5h::` parameter on `.usage`. See [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `min_5h::50` hides rows below threshold | Behavioral Divergence |
| EC-2 | `min_5h::50` with row exactly at threshold — row shown (inclusive) | Inclusive Boundary |
| EC-3 | `min_5h::0` (default) shows all rows | Behavioral Divergence |
| EC-4 | `min_5h::abc` exits 1 with type error | Invalid Value |
| EC-5 | `min_5h::101` exits 1 (above 100%) | Out of Range |
| EC-6 | `min_5h::50` with account having no session data — row passes filter | Absent Data |

---

### EC-1: `min_5h::50` hides rows below threshold

- **Given:** Three offline accounts with seeded quota caches (`seed_quota_cache`): A `5h Left = 80%`, B `5h Left = 50%`, C `5h Left = 30%`. Rows render via the G1 not-owned cache path (fetch.rs) — values exact, no token, no live API.
- **When:** `clp .usage min_5h::50`
- **Then:** Exits 0. A and B shown (B exactly at the threshold — inclusive `>=`); C hidden (30 < 50).
- **Exit:** 0
- **Live:** no
- **Source fn:** `it207_min_5h_50_hides_below_threshold` (in `usage_lim_it_test.rs`)
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

---

### EC-2: `min_5h::50` with row exactly at threshold — row shown (inclusive)

- **Given:** Two offline accounts with seeded quota caches: A `5h Left = 50%` (exactly at the threshold), B `5h Left = 49%` (just below).
- **When:** `clp .usage min_5h::50`
- **Then:** Exits 0. A shown (inclusive `>=`); B hidden — locks the comparison direction on both sides of the boundary.
- **Exit:** 0
- **Live:** no
- **Source fn:** `it208_min_5h_50_inclusive_boundary` (in `usage_lim_it_test.rs`)
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

---

### EC-3: `min_5h::0` shows all rows

- **Given:** Any environment.
- **When:** `clp .usage min_5h::0`
- **Then:** Exits 0. All rows shown (0 = no filter).
- **Exit:** 0
- **Source fn:** `it163_min_5h_0_shows_all_rows` (in `usage_filter_test.rs`)
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

---

### EC-4: `min_5h::abc` exits 1 with type error

- **Given:** Any environment.
- **When:** `clp .usage min_5h::abc`
- **Then:** Exits 1. Stderr contains a type error message.
- **Exit:** 1
- **Source fn:** `it164_min_5h_abc_exits_1` (in `usage_filter_test.rs`)
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

---

### EC-5: `min_5h::101` exits 1 (above 100%)

- **Given:** Any environment.
- **When:** `clp .usage min_5h::101`
- **Then:** Exits 1. Stderr indicates value out of valid range (0–100).
- **Exit:** 1
- **Source fn:** `it165_min_5h_101_exits_1` (in `usage_filter_test.rs`)
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

---

### EC-6: `min_5h::50` with account having no session quota data — row passes

- **Given:** One account whose `five_hour` quota field is absent (no session usage data available).
- **When:** `clp .usage min_5h::50`
- **Then:** Exits 0. Row shown. Absent session data is treated as 100% remaining (filter does not exclude rows for which threshold cannot be evaluated).
- **Exit:** 0
- **Source fn:** `it211_min_5h_absent_data_passes_filter` (in `usage_lim_it_test.rs`)
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

# Test: `min_7d::` Parameter

Edge case coverage for the `min_7d::` parameter on `.usage`. See [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `min_7d::20` hides rows below threshold | Behavioral Divergence |
| EC-2 | `min_7d::20` with row exactly at threshold — row shown (inclusive) | Inclusive Boundary |
| EC-3 | `min_7d::0` (default) shows all rows | Behavioral Divergence |
| EC-4 | `min_7d::abc` exits 1 with type error | Invalid Value |
| EC-5 | `min_7d::101` exits 1 (above 100%) | Out of Range |
| EC-6 | `min_7d::30` with account having no weekly data — row passes filter | Absent Data |

---

### EC-1: `min_7d::20` hides rows below threshold

- **Given:** Three offline accounts with seeded quota caches (`seed_quota_cache`): A `7d Left = 60%`, B `7d Left = 20%`, C `7d Left = 10%`. Rows render via the G1 not-owned cache path (fetch.rs) — values exact, no token, no live API.
- **When:** `clp .usage min_7d::20`
- **Then:** Exits 0. A and B shown (B exactly at the threshold — inclusive `>=`); C hidden (10 < 20).
- **Exit:** 0
- **Live:** no
- **Source fn:** `it209_min_7d_20_hides_below_threshold` (in `usage_lim_it_test.rs`)
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

---

### EC-2: `min_7d::20` with row exactly at threshold — row shown (inclusive)

- **Given:** Two offline accounts with seeded quota caches: A `7d Left = 20%` (exactly at the threshold), B `7d Left = 19%` (just below).
- **When:** `clp .usage min_7d::20`
- **Then:** Exits 0. A shown (inclusive `>=`); B hidden — locks the comparison direction on both sides of the boundary.
- **Exit:** 0
- **Live:** no
- **Source fn:** `it210_min_7d_20_inclusive_boundary` (in `usage_lim_it_test.rs`)
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

---

### EC-3: `min_7d::0` shows all rows

- **Given:** Any environment.
- **When:** `clp .usage min_7d::0`
- **Then:** Exits 0. All rows shown (0 = no filter).
- **Exit:** 0
- **Source fn:** `it166_min_7d_0_shows_all_rows` (in `usage_filter_test.rs`)
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

---

### EC-4: `min_7d::abc` exits 1 with type error

- **Given:** Any environment.
- **When:** `clp .usage min_7d::abc`
- **Then:** Exits 1. Stderr contains a type error message.
- **Exit:** 1
- **Source fn:** `it167_min_7d_abc_exits_1` (in `usage_filter_test.rs`)
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

---

### EC-5: `min_7d::101` exits 1 (above 100%)

- **Given:** Any environment.
- **When:** `clp .usage min_7d::101`
- **Then:** Exits 1. Stderr indicates value out of valid range (0–100).
- **Exit:** 1
- **Source fn:** `it168_min_7d_101_exits_1` (in `usage_filter_test.rs`)
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

---

### EC-6: `min_7d::30` with account having no weekly quota data — row passes

- **Given:** One account whose `seven_day` quota field is absent (no weekly usage data available).
- **When:** `clp .usage min_7d::30`
- **Then:** Exits 0. Row shown. Absent weekly data is treated as 100% remaining (filter does not exclude rows for which threshold cannot be evaluated).
- **Exit:** 0
- **Source fn:** `it212_min_7d_absent_data_passes_filter` (in `usage_lim_it_test.rs`)
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

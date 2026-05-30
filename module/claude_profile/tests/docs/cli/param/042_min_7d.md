# Test: `min_7d::` Parameter

Edge case coverage for the `min_7d::` parameter on `.usage`. See [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `min_7d::30` hides rows below threshold | Behavioral Divergence |
| EC-2 | `min_7d::30` with row exactly at threshold — row shown (inclusive) | Inclusive Boundary |
| EC-3 | `min_7d::0` (default) shows all rows | Behavioral Divergence |
| EC-4 | `min_7d::abc` exits 1 with type error | Invalid Value |
| EC-5 | `min_7d::101` exits 1 (above 100%) | Out of Range |
| EC-6 | `min_7d::30` with account having no weekly data — row passes filter | Absent Data |

---

### EC-1: `min_7d::30` hides rows below threshold

- **Given:** Two accounts: one with `7d Left = 60%`, one with `7d Left = 10%`.
- **When:** `clp .usage min_7d::30`
- **Then:** Exits 0. Only the 60% row shown; 10% row hidden.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it209_lim_it_min_7d_20_hides_below_threshold` (in `tests/cli/usage_test.rs`)
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

---

### EC-2: `min_7d::30` with row at exactly 30% — shown

- **Given:** One account with `7d Left = 30%`.
- **When:** `clp .usage min_7d::30`
- **Then:** Exits 0. Row shown (threshold is inclusive).
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it210_lim_it_min_7d_20_inclusive_boundary` (in `tests/cli/usage_test.rs`)
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

---

### EC-3: `min_7d::0` shows all rows

- **Given:** Any environment.
- **When:** `clp .usage min_7d::0`
- **Then:** Exits 0. All rows shown (0 = no filter).
- **Exit:** 0
- **Source fn:** `it166_min_7d_0_shows_all_rows` (in `tests/cli/usage_test.rs`)
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

---

### EC-4: `min_7d::abc` exits 1 with type error

- **Given:** Any environment.
- **When:** `clp .usage min_7d::abc`
- **Then:** Exits 1. Stderr contains a type error message.
- **Exit:** 1
- **Source fn:** `it167_min_7d_abc_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

---

### EC-5: `min_7d::101` exits 1 (above 100%)

- **Given:** Any environment.
- **When:** `clp .usage min_7d::101`
- **Then:** Exits 1. Stderr indicates value out of valid range (0–100).
- **Exit:** 1
- **Source fn:** `it168_min_7d_101_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

---

### EC-6: `min_7d::30` with account having no weekly quota data — row passes

- **Given:** One account whose `seven_day` quota field is absent (no weekly usage data available).
- **When:** `clp .usage min_7d::30`
- **Then:** Exits 0. Row shown. Absent weekly data is treated as 100% remaining (filter does not exclude rows for which threshold cannot be evaluated).
- **Exit:** 0
- **Source fn:** `it212_min_7d_absent_data_passes_filter` (in `tests/cli/usage_test.rs`)
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

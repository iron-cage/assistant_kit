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

- **Given:** Two accounts: one with `5h Left = 80%`, one with `5h Left = 30%`.
- **When:** `clp .usage min_5h::50`
- **Then:** Exits 0. Only the 80% row shown; 30% row hidden.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it207_lim_it_min_5h_50_hides_below_threshold` (in `tests/cli/usage_test.rs`)
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

---

### EC-2: `min_5h::50` with row at exactly 50% — shown

- **Given:** One account with `5h Left = 50%`.
- **When:** `clp .usage min_5h::50`
- **Then:** Exits 0. Row shown (threshold is inclusive).
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it208_lim_it_min_5h_50_inclusive_boundary` (in `tests/cli/usage_test.rs`)
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

---

### EC-3: `min_5h::0` shows all rows

- **Given:** Any environment.
- **When:** `clp .usage min_5h::0`
- **Then:** Exits 0. All rows shown (0 = no filter).
- **Exit:** 0
- **Source fn:** `it163_min_5h_0_shows_all_rows` (in `tests/cli/usage_test.rs`)
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

---

### EC-4: `min_5h::abc` exits 1 with type error

- **Given:** Any environment.
- **When:** `clp .usage min_5h::abc`
- **Then:** Exits 1. Stderr contains a type error message.
- **Exit:** 1
- **Source fn:** `it164_min_5h_abc_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

---

### EC-5: `min_5h::101` exits 1 (above 100%)

- **Given:** Any environment.
- **When:** `clp .usage min_5h::101`
- **Then:** Exits 1. Stderr indicates value out of valid range (0–100).
- **Exit:** 1
- **Source fn:** `it165_min_5h_101_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

---

### EC-6: `min_5h::50` with account having no session quota data — row passes

- **Given:** One account whose `five_hour` quota field is absent (no session usage data available).
- **When:** `clp .usage min_5h::50`
- **Then:** Exits 0. Row shown. Absent session data is treated as 100% remaining (filter does not exclude rows for which threshold cannot be evaluated).
- **Exit:** 0
- **Source fn:** `it211_min_5h_absent_data_passes_filter` (in `tests/cli/usage_test.rs`)
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

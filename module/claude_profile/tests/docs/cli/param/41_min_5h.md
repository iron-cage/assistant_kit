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

### EC-1: `min_5h::50` accepted with a live account — structural acceptance only

> **Semantic drift correction:** the cited test does not construct two accounts or verify any row-hiding behavior. It uses a single live account (shared token), calls `min_5h::50` once, and asserts only exit 0. Per the test's own doc comment: "With two live accounts sharing the same token the quota values are identical... Note: Exact threshold verification (80% shown / 30% hidden) requires two accounts with different quota levels — non-trivial to guarantee with shared tokens. This test verifies structural acceptance only." No test anywhere in the suite constructs the 80%/30% two-row hiding scenario this EC originally described.

- **Given:** One live account (`write_account_with_token`, shared token) — quota percentage is whatever the live API returns, not controlled by the test.
- **When:** `clp .usage min_5h::50`
- **Then:** Exits 0. The test verifies only that the flag is parsed and accepted without error — it does NOT verify that a row below 50% is hidden, nor does it construct a second account to compare against.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it207_lim_it_min_5h_50_hides_below_threshold` (in `usage_lim_it_test.rs`) — name and doc claim describe a hiding scenario the body does not implement; body is structural-acceptance-only
- **Source:** [param/041_min_5h.md](../../../../docs/cli/param/041_min_5h.md)

---

### EC-2: `min_5h::50` accepted with a live account — inclusive-boundary NOT independently controlled

> **Semantic drift correction:** the cited test does not set the account's quota to exactly 50%, nor does it assert a row is shown at that boundary. Per the test's own doc comment: "Verifies structural acceptance of the threshold flag with a live account. The inclusive-boundary semantic (≥ threshold) is verified by the offline unit logic; this test confirms the flag is parsed and applied." No offline unit test asserting the exact `min_5h::50` inclusive-boundary/row-shown case was found elsewhere in the suite — only `it163`/`it164`/`it165` (`usage_filter_test.rs`, cited below for EC-3/4/5) cover `0`/`abc`/`101`, not the inclusive-boundary value.

- **Given:** One live account (shared token) — quota percentage not set to 50% by the test.
- **When:** `clp .usage min_5h::50`
- **Then:** Exits 0. The test verifies only that the flag is parsed and applied without error — it does not control the account's quota to 50% and does not assert the row is shown at that exact boundary.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it208_lim_it_min_5h_50_inclusive_boundary` (in `usage_lim_it_test.rs`) — name and doc claim describe an inclusive-boundary verification the body does not implement; body is structural-acceptance-only
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

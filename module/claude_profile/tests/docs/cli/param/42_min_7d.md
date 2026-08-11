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

### EC-1: `min_7d::20` accepted with a live account — structural acceptance only

> **Semantic drift correction:** the cited test does not construct two accounts or verify any row-hiding behavior, and the doc's own claimed threshold value (30) does not match the test's actual parameter — the function name itself uses `min_7d::20`. The test uses a single live account (shared token) and asserts only exit 0. No test anywhere in the suite constructs the 60%/10% two-row hiding scenario this EC originally described, at any threshold value.

- **Given:** One live account (`write_account_with_token`, shared token) — quota percentage is whatever the live API returns, not controlled by the test.
- **When:** `clp .usage min_7d::20`
- **Then:** Exits 0. The test verifies only that the flag is parsed and accepted without error — it does NOT verify that a row below 20% is hidden, nor does it construct a second account to compare against.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it209_lim_it_min_7d_20_hides_below_threshold` (in `usage_lim_it_test.rs`) — name and doc claim describe a hiding scenario the body does not implement; body is structural-acceptance-only; the function's own name uses threshold 20, not the 30 this EC originally claimed
- **Source:** [param/042_min_7d.md](../../../../docs/cli/param/042_min_7d.md)

---

### EC-2: `min_7d::20` accepted with a live account — inclusive-boundary NOT independently controlled

> **Semantic drift correction:** the cited test does not set the account's quota to exactly 20% (nor 30%, the value this EC originally claimed), nor does it assert a row is shown at that boundary. It uses a single live account with an uncontrolled quota and asserts only exit 0. No offline unit test asserting the exact `min_7d` inclusive-boundary/row-shown case was found elsewhere in the suite.

- **Given:** One live account (shared token) — quota percentage not set to any specific value by the test.
- **When:** `clp .usage min_7d::20`
- **Then:** Exits 0. The test verifies only that the flag is parsed and applied without error — it does not control the account's quota to a specific percentage and does not assert the row is shown at that exact boundary.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it210_lim_it_min_7d_20_inclusive_boundary` (in `usage_lim_it_test.rs`) — name and doc claim describe an inclusive-boundary verification the body does not implement; body is structural-acceptance-only; the function's own name uses threshold 20, not the 30 this EC originally claimed
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

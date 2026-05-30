# Test: Feature 028 — Usage Row Filtering and Extraction

Feature behavioral requirement test cases for `docs/feature/028_usage_row_filtering.md`. Each FT case maps to one acceptance criterion.

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `count::3` shows at most 3 rows | AC-01 | Integration |
| FT-02 | `offset::2 count::3` skips first 2 rows then shows at most 3 | AC-02 | Integration |
| FT-03 | `only_active::1` shows exactly the active account row | AC-03 | Integration |
| FT-04 | `only_next::1` shows exactly the → row | AC-04 | Integration |
| FT-05 | `min_5h::50` hides rows below 50% threshold (inclusive boundary) | AC-05 | Integration |
| FT-06 | `min_7d::20` hides rows below 20% threshold (inclusive boundary) | AC-06 | Integration |
| FT-07 | `only_valid::1` hides 🔴 rows | AC-07 | Integration |
| FT-08 | `exclude_exhausted::1` hides 🟡 and 🔴 rows | AC-08 | Integration |
| FT-09 | Multiple filters combine with AND logic | AC-09 | Integration |
| FT-10 | `get::7d_left` extracts bare 7d Left value | AC-10 | Integration |
| FT-11 | `only_next::1 get::7d_left` extracts value for → account | AC-11 | Integration |
| FT-12 | `get::status` extracts status emoji | AC-12 | Integration |
| FT-13 | `format::tsv` produces tab-separated output with text status labels | AC-13 | Integration |
| FT-14 | `no_color::1` produces emoji-free output | AC-14 | Integration |
| FT-15 | Invalid `get::` field ID exits 1 listing valid IDs | AC-15 | Validation |
| FT-16 | Filters compose with `sort::`, `next::`, `prefer::`, `cols::` | AC-16 | Composability |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | count::3 truncates to 3 rows | AC-01 | Row Limit |
| FT-02 | offset::2 count::3 windows result set | AC-02 | Pagination |
| FT-03 | only_active::1 shows active account | AC-03 | Row Filter |
| FT-04 | only_next::1 shows → account | AC-04 | Row Filter |
| FT-05 | min_5h::50 inclusive threshold filter | AC-05 | Threshold Filter |
| FT-06 | min_7d::20 inclusive threshold filter | AC-06 | Threshold Filter |
| FT-07 | only_valid::1 hides 🔴 | AC-07 | Status Filter |
| FT-08 | exclude_exhausted::1 hides 🟡 and 🔴 | AC-08 | Status Filter |
| FT-09 | AND composition of multiple filters | AC-09 | Composability |
| FT-10 | get::7d_left extracts bare value | AC-10 | Extraction |
| FT-11 | only_next::1 get::7d_left targeted extraction | AC-11 | Extraction |
| FT-12 | get::status extracts emoji | AC-12 | Extraction |
| FT-13 | format::tsv tab-separated output | AC-13 | Format |
| FT-14 | no_color::1 plain output | AC-14 | Format |
| FT-15 | Invalid get:: field ID rejected | AC-15 | Validation |
| FT-16 | Filters compose with sort/next/prefer/cols | AC-16 | Composability |

**Total:** 16 FT cases

---

### FT-01: `count::3` shows at most 3 rows

- **Given:** Five accounts with valid quota data (quota fetched live).
- **When:** `clp .usage count::3`
- **Then:** Exits 0. Table body has exactly 3 data rows. Table header and footer are still shown.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it178_count_3_shows_first_3_rows` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-01](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-02: `offset::2 count::3` windows result set

- **Given:** Five or more accounts with valid quota data.
- **When-A:** `clp .usage count::0` (all rows, no offset)
- **When-B:** `clp .usage offset::2 count::3`
- **Then-B:** Exits 0. The rows shown in When-B match rows 3–5 (0-indexed) from When-A output.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it205_ft028_02_offset2_count3_windows_result` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-02](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-03: `only_active::1` shows exactly the active account row

- **Given:** Three accounts; one is the active account (per per-machine marker).
- **When:** `clp .usage only_active::1`
- **Then:** Exits 0. Exactly one data row shown — the active account. All other rows absent.
- **Exit:** 0
- **Source fn:** `it154_only_active_1_shows_active_account_row` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-03](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-04: `only_next::1` shows exactly the → account

- **Given:** Three accounts with valid quota; one receives → from the active `next::` strategy.
- **When:** `clp .usage only_next::1`
- **Then:** Exits 0. Exactly one data row shown — the → account. All other rows absent.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it206_lim_it_ft028_04_only_next_1_shows_arrow` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-04](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-05: `min_5h::50` inclusive threshold filter

- **Given:** Three accounts: A with `5h Left = 80%`, B with `5h Left = 50%`, C with `5h Left = 30%`.
- **When:** `clp .usage min_5h::50`
- **Then:** Exits 0. Rows A and B are shown; row C is hidden (30% < 50). B is shown (50% == threshold — inclusive).
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it207_lim_it_min_5h_50_hides_below_threshold` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-05](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-06: `min_7d::20` inclusive threshold filter

- **Given:** Three accounts: A with `7d Left = 60%`, B with `7d Left = 20%`, C with `7d Left = 10%`.
- **When:** `clp .usage min_7d::20`
- **Then:** Exits 0. Rows A and B shown; row C hidden (10% < 20). B shown (20% == threshold — inclusive).
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it209_lim_it_min_7d_20_hides_below_threshold` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-06](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-07: `only_valid::1` hides 🔴 rows

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴 (no valid token).
- **When:** `clp .usage only_valid::1`
- **Then:** Exits 0. 🟢 and 🟡 rows shown; 🔴 row hidden.
- **Exit:** 0
- **Source fn:** `it171_only_valid_1_all_red_shows_empty` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-07](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-08: `exclude_exhausted::1` hides 🟡 and 🔴 rows

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴.
- **When:** `clp .usage exclude_exhausted::1`
- **Then:** Exits 0. Only the 🟢 row shown; both 🟡 and 🔴 rows hidden.
- **Exit:** 0
- **Source fn:** `it176_exclude_exhausted_1_all_red_shows_empty` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-08](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-09: AND composition of multiple filters

- **Given:** Four accounts: A (🟢, 7d=40%), B (🟢, 7d=25%), C (🟡, 7d=40%), D (🔴).
- **When:** `clp .usage only_valid::1 min_7d::30`
- **Then:** Exits 0. Only A shown: must be non-🔴 AND 7d ≥ 30%. B (25% < 30%) excluded; C excluded (🟡 triggers `only_valid::1`); D excluded (🔴).
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it213_lim_it_ft028_09_and_composition` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-09](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-10: `get::7d_left` extracts bare 7d Left value

- **Given:** Two accounts with valid quota; `sort::name` so order is deterministic.
- **When:** `clp .usage sort::name get::7d_left`
- **Then:** Exits 0. Stdout is a single bare percentage string (e.g., `65%`) with no table headers, separator lines, or footer.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it214_lim_it_ft028_10_get_7d_left_bare` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-10](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-11: `only_next::1 get::7d_left` targeted extraction

- **Given:** Two accounts with valid quota; one receives →.
- **When:** `clp .usage only_next::1 get::7d_left`
- **Then:** Exits 0. Stdout is the 7d Left value for the → account as a bare string.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it215_lim_it_ft028_11_only_next_get_7d_left` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-11](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-12: `get::status` extracts status emoji

- **Given:** One 🟢 account with valid quota.
- **When:** `clp .usage get::status`
- **Then:** Exits 0. Stdout is exactly `🟢` (or `🟡` / `🔴` for other tier accounts). Single emoji, no newline except final.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it216_lim_it_ft028_12_get_status_green` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-12](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-13: `format::tsv` produces tab-separated output with text status labels

- **Given:** Two accounts with valid quota data.
- **When:** `clp .usage format::tsv`
- **Then:** Exits 0. Output has a header row with tab-separated column names. Data rows are tab-separated. Status column contains `ok`, `warn`, or `err` (no emoji).
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it217_lim_it_ft028_13_format_tsv_status_text` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-13](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-14: `no_color::1` produces emoji-free plain text output

- **Given:** One 🟢 account, one 🟡 account.
- **When:** `clp .usage no_color::1`
- **Then:** Exits 0. Stdout contains no emoji (`🟢`, `🟡`, `🔴`, `→`, `✓`, `*` absent). Status column shows plain text labels (`ok`, `warn`, `err`). Table structure preserved.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it218_lim_it_ft028_14_no_color_emoji_free` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-14](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-15: Invalid `get::` field ID exits 1 listing valid IDs

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage get::bogus_field`
- **Then:** Exits 1. Stderr contains a list of valid field IDs including `5h_left`, `7d_left`, `account`, `status`.
- **Exit:** 1
- **Source fn:** `ut_get_invalid_field_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-15](../../../../docs/feature/028_usage_row_filtering.md)

---

### FT-16: Filters compose with `sort::`, `next::`, `prefer::`, `cols::`

- **Given:** Four accounts with valid quota data.
- **When:** `clp .usage sort::name next::drain only_valid::1 count::2 cols::+sub`
- **Then:** Exits 0. Output shows at most 2 non-🔴 rows, sorted alphabetically, with Sub column present. Footer shows all three strategy recommendations.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it219_lim_it_ft028_16_filters_compose` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-16](../../../../docs/feature/028_usage_row_filtering.md)

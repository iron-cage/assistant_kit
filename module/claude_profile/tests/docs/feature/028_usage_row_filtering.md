# Test: Feature 028 — Usage Row Filtering and Extraction

### Scope

- **Purpose**: Test cases for `.usage` row filtering, pagination, threshold filters, and value extraction via `get::`.
- **Source**: `docs/feature/028_usage_row_filtering.md`
- **Covers**: AC-01 through AC-22

Feature behavioral requirement test cases for `docs/feature/028_usage_row_filtering.md`. Each FT case maps to one acceptance criterion.

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `count::3` shows at most 3 rows | AC-01 | Integration |
| FT-02 | `offset::2 count::3` skips first 2 rows then shows at most 3 | AC-02 | Integration |
| FT-03 | `only_active::1` shows exactly the active account row | AC-03 | Integration |
| FT-04 | `only_next::1` shows exactly the recommended account row | AC-04 | Integration |
| FT-05 | `min_5h::50` hides rows below 50% threshold (inclusive boundary) | AC-05 | Integration |
| FT-06 | `min_7d::20` hides rows below 20% threshold (inclusive boundary) | AC-06 | Integration |
| FT-07 | `only_valid::1` hides 🔴 rows | AC-07 | Integration |
| FT-08 | `exclude_exhausted::1` hides 🟡 and 🔴 rows | AC-08 | Integration |
| FT-09 | Multiple filters combine with AND logic | AC-09 | Integration |
| FT-10 | `get::7d_left` extracts bare 7d Left value | AC-10 | Integration |
| FT-11 | `only_next::1 get::7d_left` extracts value for recommended account | AC-11 | Integration |
| FT-12 | `get::status` extracts status emoji | AC-12 | Integration |
| FT-13 | `format::tsv` produces tab-separated output with text status labels | AC-13 | Integration |
| FT-14 | `no_color::1` produces emoji-free output | AC-14 | Integration |
| FT-15 | Invalid `get::` field ID exits 1 listing valid IDs | AC-15 | Validation |
| FT-16 | Filters compose with `sort::`, `prefer::`, `cols::` | AC-16 | Composability |
| FT-17 | `only_active::1` performs exactly 1 HTTP fetch on N-account store | AC-17 | Pipeline-Constraint |
| FT-18 | `stalest::K` selects the K oldest-cache accounts; all N rows render | AC-18 | Pre-Fetch Reducer |
| FT-19 | `max_age::S` eligibility threshold; fully-fresh fleet fetches nothing | AC-19 | Pre-Fetch Reducer |
| FT-20 | `stalest::0` and standalone `max_age::` exit 1 pre-HTTP | AC-20 | Validation |
| FT-21 | `stalest::K only_active::1` exits 1 — mutually exclusive reducers | AC-21 | Validation |
| FT-22 | `rotate::1` bypasses the reducer (full-fleet fetch) | AC-22 | Pre-Fetch Reducer |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | count::3 truncates to 3 rows | AC-01 | Row Limit |
| FT-02 | offset::2 count::3 windows result set | AC-02 | Pagination |
| FT-03 | only_active::1 shows active account | AC-03 | Row Filter |
| FT-04 | only_next::1 shows recommended account | AC-04 | Row Filter |
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
| FT-16 | Filters compose with sort/prefer/cols | AC-16 | Composability |
| FT-17 | only_active::1 performs exactly 1 HTTP fetch (N-account store) | AC-17 | Pipeline-Constraint |
| FT-18 | stalest::K selection: K oldest, missing cache oldest, list-order ties, full-fleet rows | AC-18 | Pre-Fetch Reducer |
| FT-19 | max_age::S drain: oldest-first eligibility, empty set when fleet fresh | AC-19 | Pre-Fetch Reducer |
| FT-20 | stalest::0 / negative / standalone max_age:: rejected | AC-20 | Validation |
| FT-21 | stalest + only_active mutual exclusion | AC-21 | Validation |
| FT-22 | rotate::1 bypasses reduction predicate | AC-22 | Pre-Fetch Reducer |

**Total:** 22 FT cases

---

### FT-01: `count::3` shows at most 3 rows

- **Given:** Five accounts in the credential store (no live token — `count::` truncation applies to rows regardless of quota validity).
- **When:** `clp .usage count::3`
- **Then:** Exits 0. Table body has exactly 3 data rows. Table header and footer are still shown.
- **Exit:** 0
- **Source fn:** `it178_count_3_shows_first_3_rows` (in `usage_filter_test_b.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-01](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-02: `offset::2 count::3` windows result set

- **Given:** Five accounts in the credential store (no live token; `sort::name` gives deterministic ordering).
- **When-A:** `clp .usage sort::name` (all rows; `count::` defaults to 0/unlimited)
- **When-B:** `clp .usage sort::name offset::2 count::3`
- **Then-B:** Exits 0. The rows shown in When-B match rows 3–5 (0-indexed) from When-A output.
- **Exit:** 0
- **Source fn:** `it205_ft028_02_offset2_count3_windows_result` (in `usage_lim_it_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-02](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-03: `only_active::1` shows exactly the active account row

- **Given:** Three accounts; one is the active account (per per-machine marker).
- **When:** `clp .usage only_active::1`
- **Then:** Exits 0. Exactly one data row shown — the active account. All other rows absent.
- **Exit:** 0
- **Source fn:** `it154_only_active_1_shows_active_account_row` (in `usage_filter_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-03](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-04: `only_next::1` shows exactly the recommended account

- **Given:** Three accounts with valid quota; one is the footer recommendation from the active sort strategy.
- **When:** `clp .usage only_next::1`
- **Then:** Exits 0. Exactly one data row shown — the footer-recommended account. All other rows absent.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it206_lim_it_ft028_04_only_next_1_shows_recommended` (in `tests/cli/usage_lim_it_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-04](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-05: `min_5h::50` inclusive threshold filter

- **Given:** Three accounts: A with `5h Left = 80%`, B with `5h Left = 50%`, C with `5h Left = 30%` — quota values seeded exactly via `seed_quota_cache`, rendered through the G1 not-owned cache path (no token, no HTTP).
- **When:** `clp .usage min_5h::50`
- **Then:** Exits 0. Rows A and B are shown; row C is hidden (30% < 50). B is shown (50% == threshold — inclusive `>=`).
- **Exit:** 0
- **Live:** no
- **Source fn:** `it207_min_5h_50_hides_below_threshold` (in `usage_lim_it_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-05](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-06: `min_7d::20` inclusive threshold filter

- **Given:** Three accounts: A with `7d Left = 60%`, B with `7d Left = 20%`, C with `7d Left = 10%` — quota values seeded exactly via `seed_quota_cache`, rendered through the G1 not-owned cache path (no token, no HTTP).
- **When:** `clp .usage min_7d::20`
- **Then:** Exits 0. Rows A and B shown; row C hidden (10% < 20). B shown (20% == threshold — inclusive `>=`).
- **Exit:** 0
- **Live:** no
- **Source fn:** `it209_min_7d_20_hides_below_threshold` (in `usage_lim_it_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-06](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-07: `only_valid::1` hides 🔴 rows

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴 (no valid token).
- **When:** `clp .usage only_valid::1`
- **Then:** Exits 0. 🟢 and 🟡 rows shown; 🔴 row hidden.
- **Exit:** 0
- **Source fn:** `it171_only_valid_1_all_red_shows_empty` (in `usage_filter_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-07](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-08: `exclude_exhausted::1` hides 🟡 and 🔴 rows

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴.
- **When:** `clp .usage exclude_exhausted::1`
- **Then:** Exits 0. Only the 🟢 row shown; both 🟡 and 🔴 rows hidden.
- **Exit:** 0
- **Source fn:** `it176_exclude_exhausted_1_all_red_shows_empty` (in `usage_filter_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-08](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-09: AND composition of multiple filters

- **Given:** Four accounts with seeded quota caches: A (🟢, `7d Left = 40%`), B (🟢, `7d Left = 25%`), C (🟡 — 5h exhausted, `7d Left = 40%`), D (🔴 — no cache, quota Err).
- **When:** `clp .usage only_valid::1 min_7d::30`
- **Then:** Exits 0. A and C shown — per AC-09 (`only_valid::1` keeps 🟢/🟡 rows, see AC-07) AND `7d Left ≥ 30%`. B hidden (25% < 30%); D hidden (🔴 fails `only_valid::1` — even though a bare `min_7d::30` alone would pass an Err row through, absent data ≠ exhausted).
- **Exit:** 0
- **Live:** no
- **Source fn:** `it213_ft028_09_and_composition` (in `usage_lim_it_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-09](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-10: `get::7d_left` extracts bare 7d Left value

- **Given:** Two accounts with valid quota; `sort::name` so order is deterministic.
- **When:** `clp .usage sort::name get::7d_left`
- **Then:** Exits 0. Stdout is a single bare percentage string (e.g., `65%`) with no table headers, separator lines, or footer.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it214_lim_it_ft028_10_get_7d_left_bare` (in `usage_lim_it_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-10](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-11: `only_next::1 get::7d_left` targeted extraction

- **Given:** Two accounts with valid quota; one is the footer recommendation.
- **When:** `clp .usage only_next::1 get::7d_left`
- **Then:** Exits 0. Stdout is the 7d Left value for the footer-recommended account as a bare string.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it215_lim_it_ft028_11_only_next_get_7d_left` (in `usage_lim_it_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-11](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-12: `get::status` extracts status emoji

- **Given:** One 🟢 account with valid quota.
- **When:** `clp .usage get::status`
- **Then:** Exits 0. Stdout is exactly `🟢` (or `🟡` / `🔴` for other tier accounts). Single emoji, no newline except final.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it216_lim_it_ft028_12_get_status_green` (in `usage_lim_it_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-12](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-13: `format::tsv` produces tab-separated output with text status labels

- **Given:** Two accounts with valid quota data.
- **When:** `clp .usage format::tsv`
- **Then:** Exits 0. Output has a header row with tab-separated column names. Data rows are tab-separated. Status column contains `ok`, `warn`, or `err` (no emoji).
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it217_lim_it_ft028_13_format_tsv_status_text` (in `usage_lim_it_test_b.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-13](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-14: `no_color::1` produces emoji-free plain text output

- **Given:** One 🟢 account, one 🟡 account.
- **When:** `clp .usage no_color::1`
- **Then:** Exits 0. Stdout contains no emoji (`🟢`, `🟡`, `🔴`, `→`, `✓`, `*` absent). Status column shows plain text labels (`ok`, `warn`, `err`). Table structure preserved.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it218_lim_it_ft028_14_no_color_emoji_free` (in `usage_lim_it_test_b.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-14](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-15: Invalid `get::` field ID exits 1 listing valid IDs

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage get::bogus_field`
- **Then:** Exits 1. Stderr contains a list of valid field IDs including `5h_left`, `7d_left`, `account`, `status`.
- **Exit:** 1
- **Source fn:** `ut_get_invalid_field_exits_1` (in `usage_model_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-15](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-16: Filters compose with `sort::`, `prefer::`, `cols::`

- **Given:** Four accounts with valid quota data.
- **When:** `clp .usage sort::name only_valid::1 count::2 cols::+sub`
- **Then:** Exits 0. Output shows at most 2 non-🔴 rows, sorted alphabetically, with Sub column present.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it219_lim_it_ft028_16_filters_compose` (in `usage_lim_it_test_b.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-16](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-17: `only_active::1` performs exactly 1 HTTP fetch on N-account store

- **Given:** Credential store with N ≥ 3 accounts; one account has the `_active_{hostname}_{user}` filesystem marker.
- **When:** `clp .usage only_active::1 get::status trace::1`
- **Then:** Exits 0. Trace output contains exactly 1 timestamped `... result:` line (one HTTP fetch). Non-active accounts produce no trace result lines. The single result line corresponds to the active account.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it_ft028_17_only_active_single_http_fetch` (in `usage_solo_test.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-17](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-18: `stalest::K` selects the K oldest-cache accounts; all N rows render

- **Given:** N-account store with per-account quota cache `fetched_at` timestamps of varying age; one account cacheless; two accounts with equal timestamps.
- **When:** `select_stalest( accounts, store, K, 0, now )` runs, and the fetch layer receives the resulting subset.
- **Then:** Exactly the K oldest-cache accounts are selected — a missing cache ranks oldest, equal ages tie-break by original list order, K > fleet selects all. Non-selected accounts still render rows (from cache via `approximate_quota()`); no row is removed.
- **Exit:** n/a (library-level; deterministic tempdir store)
- **Live:** no
- **Source fn:** `selection_picks_k_oldest`, `selection_missing_cache_ranks_oldest`, `selection_tie_breaks_by_list_order`, `selection_k_exceeding_fleet_selects_all`, `fetch_subset_preserves_full_fleet_rows` (in `usage/stalest_tests.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-18](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-19: `max_age::S` eligibility threshold; fully-fresh fleet fetches nothing

- **Given:** Four accounts with cache ages 3.0 h / 2.5 h / 2.2 h / 10 m against a 7200 s threshold; winners re-marked fresh between calls (as a successful fetch would).
- **When:** `select_stalest( accounts, store, 1, 7200, now )` runs repeatedly.
- **Then:** Successive calls drain oldest-first (3.0 h, then 2.5 h, then 2.2 h); the under-threshold account is never selected; once every cache is fresher than S the selection is empty — zero fetch-eligible accounts. Non-selected accounts take the cache path and their cache files stay untouched.
- **Exit:** n/a (library-level; deterministic tempdir store)
- **Live:** no
- **Source fn:** `selection_max_age_drains_oldest_first`, `fetch_gate_skips_non_selected`, `fetch_gate_leaves_non_selected_files_untouched` (in `usage/stalest_tests.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-19](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-20: `stalest::0` and standalone `max_age::` exit 1 pre-HTTP

- **Given:** Parsed `.usage` command parameters.
- **When:** `stalest::0`, `stalest::-1`, or `max_age::7200` without `stalest::` is supplied.
- **Then:** Parameter validation returns `Err` before any HTTP request or cache write; the error message names the offending parameter (and, for standalone `max_age::`, references `stalest`).
- **Exit:** 1 (validation error at parse time)
- **Live:** no
- **Source fn:** `stalest_zero_rejected`, `stalest_negative_rejected`, `max_age_without_stalest_rejected` (in `usage/params_tests.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-20](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-21: `stalest::K only_active::1` exits 1 — mutually exclusive reducers

- **Given:** Parsed `.usage` command parameters combining both pre-fetch reducers.
- **When:** `stalest::2 only_active::1` is supplied.
- **Then:** Parameter validation returns `Err`; the message names both `stalest` and `only_active`.
- **Exit:** 1 (validation error at parse time)
- **Live:** no
- **Source fn:** `stalest_and_only_active_mutual_exclusion` (in `usage/params_tests.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-21](../../../docs/feature/028_usage_row_filtering.md)

---

### FT-22: `rotate::1` bypasses the reducer (full-fleet fetch)

- **Given:** `stalest::2` with and without rotation.
- **When:** `reduction_applies( stalest, rotate )` gates the subset decision in `usage_routine`.
- **Then:** `reduction_applies( 2, false )` is true; `reduction_applies( 2, true )` is false — rotation needs a complete fresh ranking, so the reducer is bypassed. Source-order proof pins the predicate call before `fetch_quota_for_list` in `api.rs`.
- **Exit:** n/a (library-level + structural)
- **Live:** no
- **Source fn:** `reduction_predicate_rotate_bypasses`, `api_routes_reduction_through_predicate` (in `usage/stalest_tests.rs`)
- **Source:** [feature/028_usage_row_filtering.md AC-22](../../../docs/feature/028_usage_row_filtering.md)

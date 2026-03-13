# Test: `.usage`

Integration test planning for the `.usage` command. See [commands.md](../../commands.md#command--10-usage) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default invocation shows labelled table with Total row | Basic Invocation |
| IT-2 | `v::0` produces a single compact summary line | Verbosity |
| IT-3 | `v::2` includes Daily section with per-date breakdown | Verbosity |
| IT-4 | `format::json` produces valid JSON with required fields | Output Format |
| IT-5 | stats-cache.json missing exits 2 with error | Error Handling |
| IT-6 | Empty JSON file exits 2 with malformed error | Error Handling |
| IT-7 | Missing `dailyModelTokens` key exits 2 | Error Handling |
| IT-8 | Missing `lastComputedDate` exits 2 | Error Handling |
| IT-9 | HOME unset exits 2 | Error Handling |
| IT-10 | Models sorted descending by token count | Data Correctness |
| IT-11 | `claude-` prefix stripped from model names | Data Correctness |
| IT-12 | 8-digit date suffix stripped from model names | Data Correctness |
| IT-13 | Entries outside 7-day window are excluded | Data Correctness |
| IT-14 | Month boundary: Mar 3 − 6 days = Feb 25 | Date Arithmetic |
| IT-15 | Year boundary: Jan 3 − 6 days = Dec 28 | Date Arithmetic |
| IT-16 | Leap year: 2024 Mar 2 − 6 days = Feb 25 | Date Arithmetic |
| IT-17 | Token formatting: 999 → "999", 1000 → "1.0K", 999_950 → "1.0M" | Formatting |
| IT-18 | Token formatting: 999_949 → "999.9K" (boundary below promotion) | Formatting |
| IT-19 | Multi-day same-model tokens aggregate correctly | Data Correctness |
| IT-20 | Entries with missing `date` or `tokensByModel` skipped gracefully | Resilience |
| IT-21 | Empty `dailyModelTokens` array exits 0 with zero total | Edge Case |
| IT-22 | `format::json` with single model shows 100.0% | Percentage |
| IT-23 | `v::1` shows comma-formatted counts (e.g., `1,234,567`) | Formatting |
| IT-24 | `v::1` header shows period start and end dates | Display |

## Test Coverage Summary

- Basic Invocation: 1 test
- Verbosity: 2 tests
- Output Format: 1 test
- Error Handling: 5 tests
- Data Correctness: 4 tests
- Date Arithmetic: 3 tests
- Formatting: 3 tests
- Resilience: 1 test
- Edge Case: 1 test
- Percentage: 1 test
- Display: 1 test

**Total:** 24 integration tests

---

### IT-1: Default invocation shows labelled table with Total row

**Goal:** Verify that the default `v::1` output contains a period header, a Total row, and per-model rows with percentages.
**Setup:** Write `stats-cache.json` with `lastComputedDate = "2026-03-07"` and one model entry (e.g., 2 000 000 tokens for `claude-sonnet-4-6`).
**Command:** `clp .usage`
**Expected Output:** Stdout contains "Usage", "Total", a percentage sign. No "Daily" section. Exit 0.
**Verification:**
- Stdout contains "Usage" header
- Stdout contains "Total" row
- Stdout contains `%` character
- Stdout does NOT contain "Daily"
- Exit code is 0
**Pass Criteria:** Exit 0; table format with all required sections present.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u11_usage_v1_default_table`

---

### IT-2: `v::0` produces a single compact summary line

**Goal:** Verify `v::0` collapses all output to one line containing "total" with compact token notation.
**Setup:** Write `stats-cache.json` with one model and 2 000 000 tokens.
**Command:** `clp .usage v::0`
**Expected Output:** Exactly one non-empty line on stdout containing "total". Exit 0.
**Verification:**
- Stdout trimmed has exactly 1 line
- Line contains "total"
- Exit code is 0
**Pass Criteria:** Exit 0; single-line compact output.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u10_usage_v0_compact_single_line`

---

### IT-3: `v::2` includes Daily section with per-date breakdown

**Goal:** Verify `v::2` extends the default table with a per-date breakdown, newest date first.
**Setup:** Write `stats-cache.json` with two daily entries: `2026-03-06` and `2026-03-07`.
**Command:** `clp .usage v::2`
**Expected Output:** Stdout contains "Daily" section, both dates listed with `2026-03-07` appearing before `2026-03-06`. Exit 0.
**Verification:**
- Stdout contains "Daily"
- Stdout contains "2026-03-07" and "2026-03-06"
- Position of "2026-03-07" is before position of "2026-03-06"
- Exit code is 0
**Pass Criteria:** Exit 0; Daily section present, newest-first order.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u12_usage_v2_daily_breakdown`

---

### IT-4: `format::json` produces valid JSON with required fields

**Goal:** Verify `format::json` output is parseable JSON containing `period_days`, `period_start`, `period_end`, `total_tokens`, and `by_model` array.
**Setup:** Write `stats-cache.json` with two models on one day.
**Command:** `clp .usage format::json`
**Expected Output:** Valid JSON on stdout. Exit 0.
**Verification:**
- Stdout is parseable as JSON
- JSON contains `period_days` = 7
- JSON contains `total_tokens` = sum of all tokens
- JSON contains `by_model` array
- `by_model[0].model` is the model with most tokens
- Exit code is 0
**Pass Criteria:** Exit 0; valid structured JSON.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u13_usage_json_valid`

---

### IT-5: stats-cache.json missing exits 2 with error

**Goal:** Verify that when `~/.claude/stats-cache.json` does not exist, the command exits 2 with a descriptive error on stderr.
**Setup:** Create `~/.claude/` directory but do NOT create `stats-cache.json`.
**Command:** `clp .usage`
**Expected Output:** Error on stderr mentioning `stats-cache.json`. Exit 2.
**Verification:**
- Exit code is 2
- Stderr contains "stats-cache.json"
- Stdout is empty
**Pass Criteria:** Exit 2; error message identifies the missing file.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u01_usage_missing_stats_file_exits_2`

---

### IT-6: Empty JSON file exits 2 with malformed error

**Goal:** Verify that an empty `stats-cache.json` file (not valid JSON) produces exit 2 with a "malformed" error message.
**Setup:** Write `stats-cache.json` as an empty file.
**Command:** `clp .usage`
**Expected Output:** Error on stderr containing "malformed". Exit 2.
**Verification:**
- Exit code is 2
- Stderr contains "malformed"
**Pass Criteria:** Exit 2; "malformed" in error message.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u02_usage_empty_stats_file_exits_2`

---

### IT-7: Missing `dailyModelTokens` key exits 2

**Goal:** Verify that a JSON file without a `dailyModelTokens` key produces exit 2 with an error mentioning the key.
**Setup:** Write `stats-cache.json` as `{"lastComputedDate":"2026-03-07"}` (no `dailyModelTokens`).
**Command:** `clp .usage`
**Expected Output:** Error on stderr containing "dailyModelTokens". Exit 2.
**Verification:**
- Exit code is 2
- Stderr contains "dailyModelTokens"
**Pass Criteria:** Exit 2; error identifies missing key.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u03_usage_no_daily_model_tokens_exits_2`

---

### IT-8: Missing `lastComputedDate` exits 2

**Goal:** Verify that a JSON file without `lastComputedDate` produces exit 2, not silently-empty output.
**Setup:** Write `stats-cache.json` as `{"dailyModelTokens":[]}`.
**Command:** `clp .usage`
**Expected Output:** Error on stderr containing "lastComputedDate". Exit 2.
**Verification:**
- Exit code is 2
- Stderr contains "lastComputedDate"
**Pitfall:** Before the bug fix, missing `lastComputedDate` defaulted to "unknown" which caused all entries to be filtered out, silently producing zero output instead of an error.
**Pass Criteria:** Exit 2; error identifies missing field.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u04_usage_missing_last_computed_date_exits_2`

---

### IT-9: HOME unset exits 2

**Goal:** Verify that when `HOME` is unset, the command exits 2.
**Setup:** Unset `HOME` environment variable.
**Command:** `env -u HOME clp .usage`
**Expected Output:** Error on stderr. Exit 2.
**Verification:**
- Exit code is 2
- Stderr is non-empty
**Pass Criteria:** Exit 2; error reported.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u05_usage_home_unset_exits_2`

---

### IT-10: Models sorted descending by token count

**Goal:** Verify that models appear in descending order of total tokens in the output.
**Setup:** Write `stats-cache.json` with three models with different token counts: sonnet (5000), opus (3000), haiku (1000).
**Command:** `clp .usage`
**Expected Output:** sonnet appears before opus, opus before haiku. Exit 0.
**Verification:**
- Stdout position of "sonnet" < position of "opus" < position of "haiku"
- Exit code is 0
**Pass Criteria:** Exit 0; descending token-count order.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u08_usage_multiple_models_sorted_desc`

---

### IT-11: `claude-` prefix stripped from model names

**Goal:** Verify that full API model names like `claude-sonnet-4-6` are displayed as `sonnet-4-6`.
**Setup:** Write model key as `"claude-sonnet-4-6"` in `stats-cache.json`.
**Command:** `clp .usage v::0`
**Expected Output:** Stdout contains "sonnet-4-6", not "claude-sonnet-4-6". Exit 0.
**Verification:**
- Stdout contains "sonnet-4-6"
- Stdout does NOT contain "claude-sonnet-4-6"
- Exit code is 0
**Pass Criteria:** Exit 0; prefix stripped.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u09_usage_model_name_shortening`

---

### IT-12: 8-digit date suffix stripped from model names

**Goal:** Verify that model names with trailing 8-digit date suffixes (e.g., `claude-haiku-4-5-20251001`) are displayed without the date suffix (e.g., `haiku-4-5`).
**Setup:** Write model key as `"claude-haiku-4-5-20251001"` in `stats-cache.json`.
**Command:** `clp .usage v::0`
**Expected Output:** Stdout contains "haiku-4-5", not "20251001". Exit 0.
**Verification:**
- Stdout contains "haiku-4-5"
- Stdout does NOT contain "20251001"
- Exit code is 0
**Pass Criteria:** Exit 0; date suffix stripped.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u09_usage_model_name_shortening`

---

### IT-13: Entries outside 7-day window are excluded

**Goal:** Verify that daily entries before `period_start` (= `lastComputedDate` − 6 days) and after `lastComputedDate` are not included in totals.
**Setup:** `lastComputedDate = "2026-03-07"` → window `[2026-03-01, 2026-03-07]`. Include entries for `2026-02-28` (9 999 999), `2026-03-01` (1000), `2026-03-07` (2000), `2026-03-08` (9 999 999).
**Command:** `clp .usage format::json`
**Expected Output:** `total_tokens` = 3000 (only 03-01 + 03-07). Exit 0.
**Verification:**
- `total_tokens` = 3000
- Exit code is 0
**Pass Criteria:** Exit 0; out-of-window entries excluded.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u14_usage_filters_outside_7day_window`

---

### IT-14: Month boundary date arithmetic

**Goal:** Verify that `lastComputedDate = "2026-03-03"` produces `period_start = "2026-02-25"` (crossing Feb/Mar boundary).
**Setup:** `lastComputedDate = "2026-03-03"`. Include entries for `2026-02-24` (out-of-window, 999), `2026-02-25` (100), `2026-03-03` (200).
**Command:** `clp .usage format::json`
**Expected Output:** `period_start = "2026-02-25"`, `total_tokens` = 300. Exit 0.
**Verification:**
- `period_start` = "2026-02-25"
- `total_tokens` = 300
- Exit code is 0
**Pass Criteria:** Exit 0; correct month-boundary arithmetic.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u15_usage_month_boundary`

---

### IT-15: Year boundary date arithmetic

**Goal:** Verify that `lastComputedDate = "2026-01-03"` produces `period_start = "2025-12-28"` (crossing Dec/Jan boundary).
**Setup:** `lastComputedDate = "2026-01-03"`. Include entries for `2025-12-27` (999), `2025-12-28` (100), `2026-01-03` (200).
**Command:** `clp .usage format::json`
**Expected Output:** `period_start = "2025-12-28"`, `total_tokens` = 300. Exit 0.
**Verification:**
- `period_start` = "2025-12-28"
- `total_tokens` = 300
- Exit code is 0
**Pass Criteria:** Exit 0; correct year-boundary arithmetic.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u16_usage_year_boundary`

---

### IT-16: Leap year boundary date arithmetic

**Goal:** Verify that `lastComputedDate = "2024-03-02"` produces `period_start = "2024-02-25"` (2024 is a leap year, February has 29 days).
**Setup:** `lastComputedDate = "2024-03-02"`. Include entries for `2024-02-24` (999), `2024-02-25` (100), `2024-03-02` (200).
**Command:** `clp .usage format::json`
**Expected Output:** `period_start = "2024-02-25"`, `total_tokens` = 300. Exit 0.
**Verification:**
- `period_start` = "2024-02-25"
- `total_tokens` = 300
- Exit code is 0
**Pass Criteria:** Exit 0; leap-year February handled correctly.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u17_usage_leap_year_boundary`

---

### IT-17: Token formatting tier boundaries

**Goal:** Verify compact token display at and around tier boundaries, including the rounding promotion from K to M.
**Setup:** Test each threshold value separately.
**Commands and expected compact values:**
- 999 tokens → `"999 total"`
- 1000 tokens → `"1.0K total"`
- 999 950 tokens → `"1.0M total"` (not "1000.0K")
- 1 000 000 tokens → `"1.0M total"`
**Pitfall:** `{:.1}` formatting rounds 999.95 up to 1000.0, so the K→M promotion threshold must be 999 950, not 1 000 000.
**Pass Criteria:** Each case produces the expected compact string without cross-tier display artifacts.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u18_usage_token_format_boundaries`

---

### IT-18: Token formatting just below rounding threshold

**Goal:** Verify that 999 949 tokens displays as "999.9K" and is NOT promoted to M.
**Setup:** Single model entry with 999 949 tokens.
**Command:** `clp .usage v::0`
**Expected Output:** Stdout contains "999.9K total". Exit 0.
**Verification:**
- Stdout contains "999.9K total"
- Stdout does NOT contain "1.0M"
- Exit code is 0
**Pass Criteria:** Exit 0; stays in K tier below promotion threshold.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u18_usage_token_format_boundaries`

---

### IT-19: Multi-day same-model token aggregation

**Goal:** Verify that tokens for the same model across multiple days are summed correctly.
**Setup:** Three daily entries for `claude-sonnet-4-6`: 1000 + 2000 + 3000.
**Command:** `clp .usage format::json`
**Expected Output:** `total_tokens` = 6000. Exit 0.
**Verification:**
- `total_tokens` = 6000
- Exit code is 0
**Pass Criteria:** Exit 0; multi-day aggregation correct.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u19_usage_multi_day_aggregation`

---

### IT-20: Malformed entries skipped gracefully

**Goal:** Verify that daily entries missing `date` or `tokensByModel` are silently skipped without causing an error exit.
**Setup:** Raw JSON with one valid entry (500 tokens), one entry missing `date`, one entry missing `tokensByModel`.
**Command:** `clp .usage format::json`
**Expected Output:** `total_tokens` = 500 (only the valid entry). Exit 0.
**Verification:**
- `total_tokens` = 500
- Exit code is 0
- No error on stderr
**Pass Criteria:** Exit 0; only valid entries counted.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u20_usage_malformed_entries_skipped`

---

### IT-21: Empty `dailyModelTokens` array

**Goal:** Verify that an empty `dailyModelTokens` array produces exit 0 with a zero total.
**Setup:** `stats-cache.json` with `lastComputedDate` and `dailyModelTokens: []`.
**Command:** `clp .usage v::0`
**Expected Output:** Stdout contains "0 total". Exit 0.
**Verification:**
- Exit code is 0
- Stdout contains "0 total"
**Pass Criteria:** Exit 0; zero total reported cleanly.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u06_usage_empty_daily_array_shows_zero`

---

### IT-22: Single model shows 100.0%

**Goal:** Verify that when only one model is present, its percentage is shown as "100.0%".
**Setup:** Single model with any non-zero token count.
**Command:** `clp .usage`
**Expected Output:** Stdout contains "100.0%". Exit 0.
**Verification:**
- Stdout contains "100.0%"
- Exit code is 0
**Pass Criteria:** Exit 0; percentage arithmetic correct at 100%.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u22_usage_single_model_100_percent`

---

### IT-23: `v::1` uses comma-formatted token counts

**Goal:** Verify that `v::1` displays token counts with comma separators (e.g., `1,234,567`).
**Setup:** Single model with 1 234 567 tokens.
**Command:** `clp .usage`
**Expected Output:** Stdout contains "1,234,567". Exit 0.
**Verification:**
- Stdout contains "1,234,567"
- Exit code is 0
**Pass Criteria:** Exit 0; comma-formatted counts present.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u23_usage_v1_comma_formatted_tokens`

---

### IT-24: `v::1` header shows period start and end dates

**Goal:** Verify that the `v::1` header line includes both the `period_start` and `period_end` dates.
**Setup:** `lastComputedDate = "2026-03-07"` → window `2026-03-01 → 2026-03-07`.
**Command:** `clp .usage`
**Expected Output:** Stdout contains "2026-03-01" and "2026-03-07". Exit 0.
**Verification:**
- Stdout contains "2026-03-01"
- Stdout contains "2026-03-07"
- Exit code is 0
**Pass Criteria:** Exit 0; both period boundary dates shown.
**Source:** [commands.md — .usage](../../commands.md#command--10-usage)
**Implementation:** `usage_test.rs::u24_usage_v1_shows_period`

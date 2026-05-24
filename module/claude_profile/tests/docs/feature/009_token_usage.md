# Test: Feature 009 — All-Accounts Live Quota Reporting

Feature behavioral requirement test cases for `docs/feature/009_token_usage.md` (FR-14). Each FT case maps to one acceptance criterion. Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/009_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Command IT |
|----|-----------|-----|------------|
| FT-01 | Error reason shortened — missing accessToken | AC-03 | IT-3, IT-9 |
| FT-02 | HTTP 401 shortens to `(auth expired (401))` | AC-03 | IT-17 |
| FT-03 | All saved accounts fetched, not only `_active` | AC-01 | IT-1, IT-8 |
| FT-04 | Live token match governs `✓`, not `_active` marker | AC-02 | IT-2, IT-13 |
| FT-05 | Missing credential store → exit 2 | AC-06 | IT-6, IT-7 |
| FT-06 | Endurance strategy tiebreaker: expiry breaks 5h Left tie | AC-09 | IT-11 |
| FT-07 | Status emoji `🟢`/`🟡`/`🔴` correct per account state | AC-18 | IT-40, IT-41 |
| FT-08 | Strict 5% boundary: exactly 5% → `🟡`; 5.1% → `🟢` | AC-19 | IT-43 |
| FT-09 | `format::json` output contains no status emoji | AC-20 | IT-42 |
| FT-10 | After token refresh, `~Renews` shows actual date (not `?`) | BUG-171 | — |
| FT-11 | `5h Left` / `7d Left` values embed per-column emoji prefix | AC-21 | — |
| FT-12 | `Sub` / `7d Son Reset` hidden by default; `cols::+` reveals them | AC-22 | — |
| FT-13 | Invalid `cols::` column ID exits 1 with error | AC-23 | — |
| FT-14 | Three-tier grouping: 🟢 before 🟡 before 🔴 independent of sort | AC-24 | — |
| FT-15 | `format_duration_secs` capped to 2 significant time units | AC-25 | — |
| FT-16 | Within 🟡 tier: session-exhausted (`5h Left ≤ 5%`) before weekly-exhausted | AC-26 | — |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Missing accessToken shows short error, not verbose string | AC-03 | Error Shortening |
| FT-02 | HTTP 401 from usage API shortens to `(auth expired (401))` | AC-03 | Error Shortening |
| FT-03 | Both accounts appear regardless of `_active` marker | AC-01 | Complete Fetch |
| FT-04 | `✓` follows live token match, not `_active` marker | AC-02 | Live Detection |
| FT-05 | Unreadable credential store exits 2 | AC-06 | Error Handling |
| FT-06 | Tiebreaker: higher expiry wins when 5h Left tied | AC-09 | Recommendation |
| FT-07 | Status emoji correct for each of three account states | AC-18 | Status Emoji |
| FT-08 | 5% boundary is strict: 5.0% → yellow, 5.1% → green | AC-19 | Status Emoji |
| FT-09 | JSON output is emoji-free | AC-20 | Status Emoji |
| FT-10 | ~Renews shows actual date after refresh (BUG-171) | BUG-171 | Account After Refresh |
| FT-11 | Per-column emoji in 5h Left and 7d Left column values | AC-21 | Per-Column Emoji |
| FT-12 | Sub and 7d Son Reset columns hidden by default; shown via cols::+ | AC-22 | Column Visibility |
| FT-13 | Invalid cols:: column ID exits 1 | AC-23 | Column Modifiers |
| FT-14 | Three-tier grouping preserved regardless of sort strategy | AC-24 | Three-Tier Grouping |
| FT-15 | format_duration_secs shows at most 2 time components | AC-25 | Duration Format |
| FT-16 | Session-exhausted 🟡 before weekly-exhausted 🟡 regardless of sort | AC-26 | Yellow Sub-Grouping |

**Total:** 16 FT cases

---

### FT-01: Missing accessToken shows short error, not verbose string

- **Given:** One saved account whose credential file exists but has no `accessToken` field.
- **When:** `clp .usage`
- **Then:** The account's row appears in the table; the last visible column shows a short error reason in parentheses (e.g., `(missing accessToken)`); the string does NOT begin with `HTTP transport error:`; all other accounts (none here) are still processed. Exit 0.
- **Exit:** 0
- **Source fn:** `ft01_missing_access_token_short_error`
- **Source:** [009_token_usage.md AC-03](../../../../docs/feature/009_token_usage.md)

---

### FT-02: HTTP 401 from usage API shortens to `(auth expired (401))`

- **Given:** One saved account whose `expiresAt` in the credential file is a past timestamp (PAST_MS); the usage API rejects the account's `accessToken` with HTTP 401.
- **When:** `clp .usage`
- **Then:** That account's row shows `EXPIRED` in the Expires column; the 7d Reset column shows `(auth expired (401))` — NOT the verbose string `(HTTP transport error: HTTP 401)`. Exit 0.
- **Exit:** 0
- **Source fn:** `ft02_lim_it_http_401_shortens_to_auth_expired`
- **Note:** Fix for BUG-152; implemented by TSK-153 (`shorten_error` HTTP 401 branch).
- **Source:** [009_token_usage.md AC-03](../../../../docs/feature/009_token_usage.md)

---

### FT-03: Both accounts appear regardless of `_active` marker

- **Given:** Two saved accounts `alice@a.com` and `bob@a.com`; neither is stored as `_active`; both credential files exist.
- **When:** `clp .usage`
- **Then:** Stdout contains both `alice@a.com` and `bob@a.com`. Exit 0.
- **Exit:** 0
- **Source fn:** `ft03_both_accounts_appear_regardless_of_active`
- **Source:** [009_token_usage.md AC-01](../../../../docs/feature/009_token_usage.md)

---

### FT-04: `✓` follows live token match, not `_active` marker

- **Given:** Two saved accounts: `alice@a.com` (stored as `_active`) and `work@a.com`. The live `~/.claude/.credentials.json` has an `accessToken` matching `work@a.com`'s stored token.
- **When:** `clp .usage`
- **Then:** A line in stdout contains `✓` and `work@a.com`; no line contains `✓` and `alice@a.com`. Exit 0.
- **Exit:** 0
- **Source fn:** `ft04_check_mark_follows_live_token_not_active`
- **Source:** [009_token_usage.md AC-02](../../../../docs/feature/009_token_usage.md)

---

### FT-05: Unreadable credential store exits 2

- **Given:** `HOME` is set to a directory that exists but whose `~/.persistent/claude/credential/` path is chmod 000 (unreadable).
- **When:** `clp .usage`
- **Then:** Exits 2; stderr contains a non-empty error message.
- **Exit:** 2
- **Source fn:** `ft05_unreadable_credential_store_exits_2`
- **Source:** [009_token_usage.md AC-06](../../../../docs/feature/009_token_usage.md)

---

### FT-06: Tiebreaker — higher expiry wins when `5h Left` is tied (endurance strategy)

- **Given:** Two `AccountQuota` structs (unit test): `a@x.com` (`five_hour.utilization=50.0`, `expires_at_ms=now+7200000` — 2h expiry) and `b@x.com` (`five_hour.utilization=50.0`, `expires_at_ms=now+3600000` — 1h expiry). Neither is current. Both `result = Ok(...)`. `next::endurance`.
- **When:** `find_next_for_strategy(&[a, b], NextStrategy::Endurance, PreferStrategy::Any, now_secs)`
- **Then:** Returns the index of `a@x.com` (higher expiry wins the tiebreaker when 5h utilization is equal). `b@x.com` is NOT returned.
- **Exit:** n/a (unit test — function return assertion)
- **Note:** TSK-184 deleted `find_recommendation()`; tiebreaker now verified via `find_next_for_strategy()` with `NextStrategy::Endurance`.
- **Source fn:** `test_ft06_009_endurance_tiebreaker_higher_expiry_wins` (in `src/usage.rs`)
- **Source:** [009_token_usage.md AC-09](../../../../docs/feature/009_token_usage.md)

---

### FT-07: Status emoji correct for each of three account states

- **Given:** Unit test. Three `AccountQuota` variants:
  - Variant A: `result = Err("missing accessToken".to_string())` → expected `🔴`
  - Variant B: `result = Ok(data)` where `five_hour.utilization = 10.0` (90% left) → expected `🟢`
  - Variant C: `result = Ok(data)` where `five_hour.utilization = 97.0` (3% left) → expected `🟡`
- **When:** `status_emoji(&aq.result)` called for each variant.
- **Then:** Returns `"🔴"` for A, `"🟢"` for B, `"🟡"` for C.
- **Exit:** n/a (unit test)
- **Source fn:** `test_status_emoji_red`, `test_status_emoji_green`, `test_status_emoji_yellow`
- **Source:** [009_token_usage.md AC-18](../../../../docs/feature/009_token_usage.md)

---

### FT-08: 5% boundary is strict — exactly 5% → `🟡`; 5.1% → `🟢`

- **Given:** Unit test. Two `AccountQuota` variants:
  - Variant A: `five_hour.utilization = 95.0` → exactly 5.0% left → expected `🟡`
  - Variant B: `five_hour.utilization = 94.9` → 5.1% left → expected `🟢`
- **When:** `status_emoji(&aq.result)` for each.
- **Then:** A returns `"🟡"`; B returns `"🟢"`. The boundary is `left > 5.0` (strict greater-than).
- **Exit:** n/a (unit test)
- **Source fn:** `test_status_emoji_boundary`
- **Source:** [009_token_usage.md AC-19](../../../../docs/feature/009_token_usage.md)

---

### FT-09: `format::json` output is emoji-free

- **Given:** One saved account whose credential file has no `accessToken` field.
- **When:** `clp .usage format::json`
- **Then:** Exits 0. The output string does NOT contain `🔴`, `🟡`, or `🟢`. The JSON array is present and valid.
- **Exit:** 0
- **Source fn:** `test_status_emoji_absent_in_json`
- **Source:** [009_token_usage.md AC-20](../../../../docs/feature/009_token_usage.md)

---

### FT-10: After token refresh, `~Renews` shows actual billing date (not `?`)

- **Given:** One saved account whose OAuth token is expired; `OauthAccountData` was populated on the initial fetch (so `~Renews` showed a date before expiry). Token expires → `apply_refresh()` succeeds → quota re-fetched.
- **When:** `clp .usage refresh::1`
- **Then:** The `~Renews` column for the refreshed account shows a concrete date (e.g., `"Jun  5"`) — NOT `"?"`. `aq.account` is re-populated by the BUG-171 fix (`fetch_oauth_account()` called inside `apply_refresh()` after quota re-fetch).
- **Exit:** 0
- **Live:** yes (lim_it — requires expired token + live account with billing data)
- **Note:** Fix(BUG-171): `apply_refresh()` previously left `aq.account` stale after refresh; `~Renews` reverted to `?` even though the account had valid billing data.
- **Source fn:** `mre_bug_171_account_populated_after_refresh` (MRE test in `tests/cli/usage_test.rs`)
- **Source:** [009_token_usage.md](../../../../docs/feature/009_token_usage.md)

---

### FT-11: Per-column emoji in `5h Left` and `7d Left` column values

- **Given:** Unit test. Two percentage values:
  - Pct A: `90.0` (> 5% — expected prefix `🟢`)
  - Pct B: `3.0` (≤ 5% — expected prefix `🟡`)
  - Boundary Pct C: `5.0` (exactly 5% — inclusive for `🟡`, expected prefix `🟡`)
- **When:** Per-column emoji formatting applied to each value.
- **Then:** Pct A produces a string with `🟢` prefix (e.g., `🟢 90%`). Pct B produces a string with `🟡` prefix (e.g., `🟡 3%`). Pct C produces `🟡` (boundary is inclusive for `🟡`).
- **Exit:** n/a (unit test — string return assertion)
- **Source fn:** `test_ft11_009_per_column_emoji_prefix_three_cases` (in `src/usage.rs`)
- **Source:** [009_token_usage.md AC-21](../../../../docs/feature/009_token_usage.md)

---

### FT-12: `Sub` and `7d Son Reset` columns hidden by default; `cols::+` reveals them

- **Given:** One saved account with an expired token (🔴 state; credential file present so the table header renders).
- **When:** `clp .usage` (no `cols::` param) and `clp .usage cols::+sub` and `clp .usage cols::+7d_son_reset`
- **Then:**
  - Default: stdout does NOT contain `Sub` or `7d Son Reset` in the table header. Exit 0.
  - `cols::+sub`: stdout contains `Sub` in the table header. Exit 0.
  - `cols::+7d_son_reset`: stdout contains `7d Son Reset` in the table header. Exit 0.
- **Exit:** 0
- **Source fn:** `it107_ft12_cols_plus_reveals_sub_and_7d_son_reset_columns` (in `tests/cli/usage_test.rs`)
- **Source:** [009_token_usage.md AC-22](../../../../docs/feature/009_token_usage.md)

---

### FT-13: Invalid `cols::` column ID exits 1

- **Given:** Any credential setup (param validation occurs before credential reads).
- **When:** `clp .usage cols::+not_a_real_column`
- **Then:** Exit 1. Stderr contains an error message naming valid column IDs (e.g., `sub`, `7d_son_reset`).
- **Exit:** 1
- **Source fn:** `it072_cols_unknown_id_exit_1` (in `tests/cli/usage_test.rs`)
- **Source:** [009_token_usage.md AC-23](../../../../docs/feature/009_token_usage.md)

---

### FT-14: Three-tier grouping: `🟢` before `🟡` before `🔴` regardless of sort strategy

- **Given:** Unit test. Three `AccountQuota` entries (alphabetical order: alice → bob → carol):
  - `alice@x.com`: `result = Err(...)` → 🔴 tier
  - `bob@x.com`: `result = Ok(data)` where `five_hour.utilization = 97.0` (3% left) → 🟡 tier
  - `carol@x.com`: `result = Ok(data)` where `five_hour.utilization = 10.0` (90% left) → 🟢 tier
- **When:** Three-tier grouping applied with alphabetical sort within each tier.
- **Then:** Output order is `carol@x.com` (🟢) → `bob@x.com` (🟡) → `alice@x.com` (🔴). The tier ordering 🟢 → 🟡 → 🔴 is preserved regardless of alphabetical order.
- **Exit:** n/a (unit test — order assertion on sorted list)
- **Source fn:** `test_three_tier_grouping_green_before_yellow_before_red` (in `src/usage.rs`)
- **Source:** [009_token_usage.md AC-24](../../../../docs/feature/009_token_usage.md)

---

### FT-15: `format_duration_secs` capped to 2 significant time units

- **Given:** Unit test. Three input durations:
  - `90120` seconds (1 day + 1 hour + 2 minutes)
  - `11970` seconds (3 hours + 19 minutes + 30 seconds)
  - `1380` seconds (23 minutes)
- **When:** `format_duration_secs(n)` for each input.
- **Then:** Returns `"1d 1h"` (minutes dropped; 2 units shown), `"3h 19m"` (seconds dropped; 2 units shown), `"23m"` (1 unit — within the cap). No input produces a 3-component string.
- **Exit:** n/a (unit test — string return assertion)
- **Source fn:** `test_format_duration_secs_caps_at_two_units` (in `src/output.rs`)
- **Source:** [009_token_usage.md AC-25](../../../../docs/feature/009_token_usage.md)

---

### FT-16: Session-exhausted 🟡 before weekly-exhausted 🟡 regardless of sort

- **Given:** Unit test. Three `AccountQuota` structs all in 🟡 tier (plus one 🟢 as anchor). Input order: alphabetical.
  - `a@x.com`: `five_hour.utilization=10.0` (90% left), `seven_day.utilization=98.0` (2% left) → tier 🟡, **weekly-exhausted** sub-group
  - `b@x.com`: `five_hour.utilization=99.0` (1% left), `seven_day.utilization=30.0` (70% left) → tier 🟡, **session-exhausted** sub-group
  - `c@x.com`: `five_hour.utilization=97.0` (3% left), `seven_day.utilization=50.0` (50% left) → tier 🟡, **session-exhausted** sub-group (5h ≤ 5%)
  - `d@x.com`: `five_hour.utilization=10.0` (90% left), `seven_day.utilization=10.0` (90% left) → tier 🟢
  - Alpha sort would produce: a → b → c → d. Three-tier would place d (🟢) first, then a, b, c (all 🟡), then any 🔴.
- **When:** `render_text(&accounts, SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &ColsVisibility::default_set())`
- **Then:** Output row order is: `d@x.com` (🟢), then among 🟡 — `b@x.com` and `c@x.com` (session-exhausted, in alpha order), then `a@x.com` (weekly-exhausted). `a@x.com` must appear AFTER both `b@x.com` and `c@x.com` despite being alpha-first.
- **Edge case:** An account with both `5h Left ≤ 5%` AND `7d Left ≤ 5%` falls in the session-exhausted sub-group (verified by `c@x.com` if `seven_day.utilization` is set ≥ 95%).
- **Exit:** n/a (unit test — position assertion via `output.find()`)
- **Source fn:** `test_ft16_009_yellow_tier_session_before_weekly` (in `src/usage.rs`)
- **Source:** [009_token_usage.md AC-26](../../../../docs/feature/009_token_usage.md)

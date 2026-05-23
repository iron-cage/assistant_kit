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
| FT-06 | `find_recommendation()` tiebreaker: expiry breaks 5h Left tie | AC-09 | IT-11 |
| FT-07 | Status emoji `🟢`/`🟡`/`🔴` correct per account state | AC-18 | IT-40, IT-41 |
| FT-08 | Strict 5% boundary: exactly 5% → `🟡`; 5.1% → `🟢` | AC-19 | IT-43 |
| FT-09 | `format::json` output contains no status emoji | AC-20 | IT-42 |

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

**Total:** 9 FT cases

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

### FT-06: Tiebreaker — higher expiry wins when `5h Left` is tied

- **Given:** Two `AccountQuota` structs (unit test): `a@x.com` (`five_hour.utilization=50.0`, `expires_at_ms=now+7200000` — 2h expiry) and `b@x.com` (`five_hour.utilization=50.0`, `expires_at_ms=now+3600000` — 1h expiry). Neither is current. Both `result = Ok(...)`.
- **When:** `find_recommendation(&[a, b], /*current_name=*/None)`
- **Then:** Returns the index of `a@x.com` (higher expiry wins the tiebreaker at level 2). `b@x.com` is NOT returned despite alphabetical precedence.
- **Exit:** n/a (unit test — function return assertion)
- **Source fn:** `test_find_recommendation_tiebreaks_by_expiry`
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

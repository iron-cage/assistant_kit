# Test: `.usage`

Integration test planning for the `.usage` command. See [commands.md](../../../../docs/cli/commands.md#command--9-usage) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default invocation shows quota table with new column headers | Basic Invocation |
| IT-2 | Current account (live-token match) has `✓` in flag column; others do not | Current Marker |
| IT-3 | Account with missing accessToken shows `—` columns and error reason | Error Inline |
| IT-4 | `format::json` produces valid JSON array with `expires_in_secs` and `_left_pct` fields | Output Format |
| IT-5 | Empty credential store exits 0 with `(no accounts configured)` | Edge Case |
| IT-6 | Credential store unreadable exits 2 | Error Handling |
| IT-7 | HOME unset exits 2 | Error Handling |
| IT-8 | Multiple accounts displayed in alphabetical order | Ordering |
| IT-9 | Account with missing token file shows `—` with error reason | Error Inline |
| IT-10 | Account with expired token shows `EXPIRED` in Expires column | Expires Column |
| IT-11 | Best non-current account is marked with `→` in flag column | Recommendation |
| IT-12 | Footer line shows valid count and recommended next account | Footer |
| IT-13 | `*` marks `_active` account when it differs from the current account | Active Divergence |
| IT-14 | When credentials file unreadable: no `✓`; `*` still marks `_active` account | Active Divergence |
| IT-15 | When current = active, only `✓` appears; no `*` on any row | Active Divergence |
| IT-16 | JSON output uses `is_current` (not `active`) and includes `is_active` per object | JSON Schema |
| IT-17 | HTTP 401 from usage API shows `(auth expired (401))` in 7d Reset column | Error Shortening |
| IT-18 | `.usage format::table` exits 1 (`ArgumentTypeMismatch`) | Argument Rejection |
| IT-19 | Live token unmatched → synthetic `(current session)` row | Synthetic Row |
| IT-20 | `refresh::0` accepted; empty store exits 0 | Token Refresh |
| IT-21 | `refresh::1` accepted; no retry triggered without HTTP | Token Refresh |
| IT-22 | `live::1 interval::30 jitter::0` — live loop shows countdown (lim_it) | Live Monitor |
| IT-23 | `live::1 interval::60 jitter::70` — jitter > interval → exit 1 | Live Guards |
| IT-24 | `live::1 interval::5` — interval < 30 → exit 1, error mentions "30" | Live Guards |
| IT-25 | `live::1 format::json` — incompatible with live mode → exit 1 | Live Guards |
| IT-26 | Live token unmatched + `.claude.json` email → synthetic row shows email | Synthetic Row |
| IT-27 | `live::1 interval::30 jitter::30` — jitter = interval accepted → exit 2 | Live Guards |
| IT-28 | `format::json` for failed account → JSON has `"error"` field | JSON Output |
| IT-29 | `interval::5 jitter::70` without `live::1` → guards not triggered, exit 0 | Live Guards |
| IT-30 | `live::1` alone — default interval 30 satisfies >= 30 guard | Live Guards |
| IT-31 | SIGINT in live mode → clean exit 0; stdout contains "Monitor stopped." | Live Monitor |
| IT-32 | `.usage.help` lists `live`, `interval`, `jitter` params | Help Output |
| IT-33 | `refresh::1` per-account refresh loop — no panic, exit 0 (lim_it) | Token Refresh |
| IT-34 | `.usage.help` refresh description includes "401/403" but NOT "401/403/429" | Help Output |
| IT-35 | `trace::1` with no-token account → stderr contains `[trace]` lines | Trace |
| IT-36 | Empty store + `format::json` → output is `[]` | Output Format |
| IT-37 | Single failed account → no "Valid:" footer line emitted | Footer |
| IT-38 | `.usage.help` shows `refresh::` default as `1` (enabled) | Help Output |
| IT-39 | `.usage.help` refresh description mentions `429` and locally-expired case | Help Output |

### Test Coverage Summary

- Basic Invocation: 1 test (IT-1)
- Current Marker: 1 test (IT-2)
- Error Inline: 2 tests (IT-3, IT-9)
- Output Format: 2 tests (IT-4, IT-36)
- Edge Case: 1 test (IT-5)
- Error Handling: 2 tests (IT-6, IT-7)
- Ordering: 1 test (IT-8)
- Expires Column: 1 test (IT-10)
- Recommendation: 1 test (IT-11)
- Footer: 2 tests (IT-12, IT-37)
- Active Divergence: 3 tests (IT-13, IT-14, IT-15)
- JSON Schema: 1 test (IT-16)
- Error Shortening: 1 test (IT-17)
- Argument Rejection: 1 test (IT-18)
- Synthetic Row: 2 tests (IT-19, IT-26)
- Token Refresh: 3 tests (IT-20, IT-21, IT-33)
- Live Monitor: 2 tests (IT-22, IT-31)
- Live Guards: 6 tests (IT-23, IT-24, IT-25, IT-27, IT-29, IT-30)
- JSON Output: 1 test (IT-28)
- Help Output: 4 tests (IT-32, IT-34, IT-38, IT-39)
- Trace: 1 test (IT-35)

**Total:** 38 integration tests in `usage_test.rs`; source functions it17–it33 map to spec IT-18–IT-34; it34/it35/it36 map to IT-35/IT-36/IT-37; it37 maps to IT-38; it38 maps to IT-39; IT-17 covered by `ft02_lim_it_http_401_shortens_to_auth_expired` in `usage_feature_test.rs` (live network test; kept in feature test file to avoid duplication with FT-02)

---

### IT-1: Default invocation shows quota table with new column headers

- **Given:** At least one saved account with a valid token exists in the credential store.
- **When:** `clp .usage`
- **Then:** Stdout contains a table with "Quota" heading and rows showing columns: "Expires", "5h Left", "5h Reset", "7d Left", "7d Reset". Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-2: Current account (live-token match) has `✓` in flag column

- **Given:** Two saved accounts; live `~/.claude/.credentials.json` has an `accessToken` matching `work@acme.com`'s stored token. `_active` also points to `work@acme.com` (current = active, normal case).
- **When:** `clp .usage`
- **Then:** A line in stdout contains both `✓` and `work@acme.com`; no line contains `✓` and any other account name; no `*` appears (current = active). Exit 0.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-05](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-3: Account with missing accessToken shows `—` columns and error reason

- **Given:** One account whose credential file has no `accessToken` field (but has a future `expiresAt`).
- **When:** `clp .usage`
- **Then:** That account's row shows `—` for 5h Left and 7d Left; Status column shows an inline error reason. Expires column shows "in" (not "EXPIRED") because token has a future expiry. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-4: `format::json` produces valid JSON array with `expires_in_secs`, `is_current`, `is_active`

- **Given:** At least one saved account with a valid token.
- **When:** `clp .usage format::json`
- **Then:** Valid JSON array on stdout. Each element has `account` (string), `is_current` (boolean), `is_active` (boolean), and `expires_in_secs` (number). Successful elements have `session_5h_left_pct` and `weekly_7d_left_pct` (not `session_5h_pct` or `weekly_7d_pct`). No element has a top-level `active` key. Exit 0.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-08](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-5: Empty credential store shows empty table

- **Given:** Credential store exists but contains no `*.credentials.json` files.
- **When:** `clp .usage`
- **Then:** Stdout contains `(no accounts configured)`. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-6: Credential store unreadable exits 2

- **Given:** `HOME` is set but credential store directory cannot be read (permissions error).
- **When:** `clp .usage`
- **Then:** Error on stderr. Exit 2.
- **Exit:** 2
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-7: HOME unset exits 2

- **Given:** `HOME` environment variable is unset.
- **When:** `env -u HOME clp .usage`
- **Then:** Error on stderr. Exit 2.
- **Exit:** 2
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-8: Multiple accounts displayed in alphabetical order

- **Given:** Three saved accounts: `c@x.com`, `a@x.com`, `b@x.com`.
- **When:** `clp .usage`
- **Then:** Rows appear in order `a@x.com`, `b@x.com`, `c@x.com`. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-9: Account with missing token file shows `—` with error reason

- **Given:** Credential store entry exists but the `.credentials.json` file for that account is missing.
- **When:** `clp .usage`
- **Then:** That account's row shows `—` for quota columns and a missing-token error reason in Status. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-10: Account with expired token shows `EXPIRED` in Expires column

- **Given:** One saved account whose `expiresAt` in the credential file is a past timestamp (e.g., `PAST_MS`).
- **When:** `clp .usage`
- **Then:** That account's row shows `EXPIRED` in the Expires column. The quota columns show `—`. Exit 0.
- **Exit:** 0
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-11: Best non-current account is marked with `→` in flag column

- **Given:** Two accounts — one active with quota data, one non-active with valid token and quota data showing lower session usage than the active account.
- **When:** `clp .usage`
- **Then:** A line in stdout contains both `→` and the non-active account name. No line contains both `→` and the active account name. Exit 0.
- **Exit:** 0
- **Live:** yes (requires real tokens for both accounts to return quota data)
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-12: Footer line shows valid count and recommended next account

- **Given:** At least two accounts with valid tokens that return quota data.
- **When:** `clp .usage`
- **Then:** Stdout contains a footer line matching "Valid: N / M" and "Next:" with the recommended account name. Exit 0.
- **Exit:** 0
- **Live:** yes (requires ≥2 accounts with live quota headers)
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-13: `*` marks `_active` account when it differs from current

- **Given:** Two saved accounts: `alice@acme.com` (stored as `_active`) and `work@acme.com`. Live `~/.claude/.credentials.json` `accessToken` matches `work@acme.com`'s stored token (not `alice`'s).
- **When:** `clp .usage`
- **Then:** A line contains `✓` and `work@acme.com`; a different line contains `*` and `alice@acme.com`. No line contains both `✓` and `alice`, or both `*` and `work`.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-06](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-14: Credentials file unreadable — no `✓`; `*` still marks `_active`

- **Given:** Two saved accounts: `alice@acme.com` (stored as `_active`) and `work@acme.com`. `~/.claude/.credentials.json` is absent or unreadable.
- **When:** `clp .usage`
- **Then:** No line contains `✓`; a line contains `*` and `alice@acme.com`. All saved accounts are still shown. Exit 0.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-07](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-15: When current = active, only `✓` appears; no `*` on any row

- **Given:** Two saved accounts: `alice@acme.com` (stored as `_active`) and `work@acme.com`. Live `~/.claude/.credentials.json` `accessToken` matches `alice@acme.com`'s stored token (current = active).
- **When:** `clp .usage`
- **Then:** A line contains `✓` and `alice@acme.com`; no line contains `*`.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-06](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-16: JSON output uses `is_current` and `is_active`; no `active` key

- **Given:** Two saved accounts; live credentials match one of them; `_active` points to the other (divergence case).
- **When:** `clp .usage format::json`
- **Then:** Valid JSON array; the current account object has `"is_current":true` and `"is_active":false`; the `_active` account object has `"is_current":false` and `"is_active":true`; no object has a top-level `"active"` key.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-08](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-17: HTTP 401 from usage API shows `(auth expired (401))` in 7d Reset column

- **Given:** One saved account whose `expiresAt` in the credential file is a past timestamp and whose `accessToken` the usage API rejects with HTTP 401.
- **When:** `clp .usage`
- **Then:** That account's row shows `EXPIRED` in Expires and `—` for all quota columns (5h Left, 5h Reset, 7d Left, 7d(Son)); the 7d Reset column shows `(auth expired (401))` — NOT `(HTTP transport error: HTTP 401)`. Exit 0.
- **Exit:** 0
- **Fix:** BUG-152 (`task/claude_profile/bug/152_shorten_error_omits_401.md`)
- **Source fn:** `ft02_lim_it_http_401_shortens_to_auth_expired` (in `usage_feature_test.rs`)
- **Source:** [009_token_usage.md AC-03](../../../../docs/feature/009_token_usage.md)

---

### IT-18: `.usage format::table` exits 1 (`ArgumentTypeMismatch`)

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage format::table`
- **Then:** Exits 1. `format::table` is valid only for `.accounts`; all other commands reject it.
- **Exit:** 1
- **Source fn:** `it17_format_table_rejected`
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-19: Live token unmatched → synthetic `(current session)` row prepended

- **Given:** One saved account `alice@acme.com` with token `tok-alice`; live `~/.claude/.credentials.json` uses a different token `tok-unsaved`.
- **When:** `clp .usage`
- **Then:** Table contains a `(current session)` row with `✓` in the flag column; `alice@acme.com` does NOT have `✓`. Exit 0.
- **Exit:** 0
- **Source fn:** `it18_synthetic_row_when_no_saved_match`
- **Source:** [009_token_usage.md AC-11](../../../../docs/feature/009_token_usage.md)

---

### IT-20: `refresh::0` accepted; empty store exits 0

- **Given:** Empty credential store; `refresh::0` param passed.
- **When:** `clp .usage refresh::0`
- **Then:** Exits 0 with "no accounts configured" message. `refresh::0` explicitly disables the default refresh behavior without breaking baseline output.
- **Exit:** 0
- **Source fn:** `it19_refresh_disabled_param_accepted`
- **Source:** [017_token_refresh.md AC-18](../../../../docs/feature/017_token_refresh.md)

---

### IT-21: `refresh::1` accepted; no retry triggered when HTTP is not reached

- **Given:** One account with no `accessToken` in the credential file (read_token returns Err without any HTTP call); `refresh::1` param.
- **When:** `clp .usage refresh::1`
- **Then:** Exits 0; account name appears in output. No HTTP call is made, so no 401 is triggered and no retry loop fires.
- **Exit:** 0
- **Source fn:** `it20_refresh_enabled_offline_no_retry_triggered`
- **Source:** [017_token_refresh.md AC-19](../../../../docs/feature/017_token_refresh.md)

---

### IT-22: `live::1 interval::30 jitter::0` — live loop emits countdown footer (lim_it)

- **Given:** One saved account with a valid live token; `live::1 interval::30 jitter::0`; process killed after 10 s.
- **When:** `clp .usage live::1 interval::30 jitter::0`
- **Then:** stdout (captured from raw bytes) contains "Next update". Exit determined by kill signal.
- **Live:** yes (lim_it — requires live credentials)
- **Source fn:** `it21_lim_it_live_mode`
- **Source:** [018_live_monitor.md AC-28](../../../../docs/feature/018_live_monitor.md)

---

### IT-23: `live::1 interval::60 jitter::70` — jitter > interval → exit 1

- **Given:** Any environment; guard fires before any fetch.
- **When:** `clp .usage live::1 interval::60 jitter::70`
- **Then:** Exits 1; stderr is non-empty (validation error).
- **Exit:** 1
- **Source fn:** `it22_live_jitter_exceeds_interval`
- **Source:** [018_live_monitor.md AC-27](../../../../docs/feature/018_live_monitor.md)

---

### IT-24: `live::1 interval::5` — interval below 30 → exit 1, error mentions "30"

- **Given:** Any environment; guard fires before any fetch.
- **When:** `clp .usage live::1 interval::5 jitter::0`
- **Then:** Exits 1; stderr contains "30" (the minimum interval).
- **Exit:** 1
- **Source fn:** `it23_live_interval_below_minimum`
- **Source:** [018_live_monitor.md AC-26](../../../../docs/feature/018_live_monitor.md)

---

### IT-25: `live::1 format::json` — incompatible with live mode → exit 1

- **Given:** Any environment; guard fires before any fetch.
- **When:** `clp .usage live::1 format::json`
- **Then:** Exits 1; stderr is non-empty.
- **Exit:** 1
- **Source fn:** `it24_live_incompatible_with_json`
- **Source:** [018_live_monitor.md AC-25](../../../../docs/feature/018_live_monitor.md)

---

### IT-26: Live token unmatched + `.claude.json` email → synthetic row shows email

- **Given:** One saved account `alice@acme.com` with `tok-alice`; live credentials use `tok-unsaved`; `~/.claude.json` has `emailAddress = "unsaved@example.com"`.
- **When:** `clp .usage`
- **Then:** Table shows `unsaved@example.com` with `✓` in the flag column; does NOT show `(current session)` fallback. Exit 0.
- **Exit:** 0
- **Source fn:** `it25_synthetic_row_uses_claude_json_email`
- **Source:** [009_token_usage.md AC-11](../../../../docs/feature/009_token_usage.md)

---

### IT-27: `live::1 interval::30 jitter::30` — jitter equal to interval is accepted

- **Given:** Credential store directory chmod 000 (unreadable); `live::1 interval::30 jitter::30`. Guard uses strict greater-than (`jitter > interval`), so equal values must not fire.
- **When:** `clp .usage live::1 interval::30 jitter::30`
- **Then:** Exits 2 (store unreadable — proves `execute_live_mode()` was entered; guards passed); stderr does NOT contain "jitter".
- **Exit:** 2
- **Source fn:** `it26_live_jitter_equals_interval_accepted`
- **Source:** [018_live_monitor.md AC-27](../../../../docs/feature/018_live_monitor.md)

---

### IT-28: `format::json` for failed account → JSON has `"error"` field

- **Given:** One account with no `accessToken` in the credential file (read_token returns Err).
- **When:** `clp .usage format::json`
- **Then:** Exits 0; JSON contains `"error":` key; does NOT contain `"session_5h_left_pct"`; does contain `"is_current"`, `"is_active"`, `"expires_in_secs"`.
- **Exit:** 0
- **Source fn:** `it27_json_error_field_on_failed_account`
- **Source:** [009_token_usage.md AC-05](../../../../docs/feature/009_token_usage.md)

---

### IT-29: `interval::5 jitter::70` without `live::1` → guards not triggered, exits 0

- **Given:** Empty credential store; `interval::5 jitter::70` without `live::1`.
- **When:** `clp .usage interval::5 jitter::70`
- **Then:** Exits 0 with "no accounts" message; live-mode guards (interval minimum, jitter ceiling) do NOT fire.
- **Exit:** 0
- **Source fn:** `it28_interval_jitter_ignored_when_not_live`
- **Source:** [018_live_monitor.md AC-31](../../../../docs/feature/018_live_monitor.md)

---

### IT-30: `live::1` alone — default interval 30 satisfies >= 30 guard

- **Given:** Credential store directory chmod 000; `live::1` with no explicit interval or jitter. Defaults: `interval=30`, `jitter=0`. Guard is `interval < 30` (strict less-than).
- **When:** `clp .usage live::1`
- **Then:** Exits 2 (store unreadable — proves interval guard did NOT fire); stderr does NOT contain "interval".
- **Exit:** 2
- **Source fn:** `it29_live_default_interval_accepted`
- **Source:** [018_live_monitor.md AC-28](../../../../docs/feature/018_live_monitor.md)

---

### IT-31: SIGINT in live mode → clean exit 0; stdout contains "Monitor stopped."

- **Given:** One account with no `accessToken` (fetch fails instantly without HTTP, ensuring render + countdown start within 3 s); `live::1 interval::30 jitter::0`; SIGINT sent via `kill -INT` after 3 s.
- **When:** `clp .usage live::1 interval::30 jitter::0` (then SIGINT)
- **Then:** Process exits with code 0; stdout contains "Monitor stopped."
- **Exit:** 0
- **Source fn:** `it30_live_sigint_exits_0`
- **Source:** [018_live_monitor.md AC-30](../../../../docs/feature/018_live_monitor.md)

---

### IT-32: `.usage.help` lists `live`, `interval`, `jitter` params

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0; stdout contains "live", "interval", and "jitter".
- **Exit:** 0
- **Source fn:** `it31_usage_help_shows_live_params`
- **Source:** [018_live_monitor.md AC-32](../../../../docs/feature/018_live_monitor.md)

---

### IT-33: `refresh::1` per-account refresh loop — no panic, exit 0 (lim_it)

- **Given:** One saved account with a valid live token (from `live_active_token()`); `refresh::1`.
- **When:** `clp .usage refresh::1`
- **Then:** Exits 0; no panic; per-account refresh loop runs (happy-path: quota fetch succeeds on first pass, no retry needed).
- **Exit:** 0
- **Live:** yes (lim_it — requires live credentials)
- **Source fn:** `it32_lim_it_refresh_per_account`
- **Source:** [017_token_refresh.md AC-19](../../../../docs/feature/017_token_refresh.md)

---

### IT-34: `.usage.help` refresh description includes "401/403" but NOT "401/403/429"

- **Given:** Standard environment. Task 150 removed HTTP 429 from the refresh retry guard; the parameter description must no longer mention it.
- **When:** `clp .usage.help`
- **Then:** Exits 0; stdout contains "401/403"; stdout does NOT contain the substring "401/403/429".
- **Exit:** 0
- **Source fn:** `it33_mre_refresh_help_excludes_429`
- **Source:** [017_token_refresh.md AC-23](../../../../docs/feature/017_token_refresh.md)

---

### IT-35: `trace::1` with no-token account → stderr contains `[trace]` lines

- **Given:** One saved account whose credential file has no `accessToken` field.
- **When:** `clp .usage trace::1`
- **Then:** Exits 0; stderr contains `[trace]` lines including the account name; stdout still shows the account row.
- **Exit:** 0
- **Source fn:** `it34_trace_param_writes_to_stderr`
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-36: Empty store + `format::json` → output is `[]`

- **Given:** Credential store directory exists but contains no `*.credentials.json` files.
- **When:** `clp .usage format::json`
- **Then:** Exits 0; stdout (trimmed) equals `[]`; no text-format "no accounts configured" message.
- **Exit:** 0
- **Source fn:** `it35_empty_store_json_format`
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-37: Single failed account → no "Valid:" footer line emitted

- **Given:** One saved account whose credential file has no `accessToken` (quota fetch fails; `valid_count = 0`).
- **When:** `clp .usage`
- **Then:** Exits 0; stdout does NOT contain "Valid:" (footer is suppressed when `valid_count < 2`).
- **Exit:** 0
- **Source fn:** `it36_no_footer_when_no_valid_accounts`
- **Source:** [commands.md — .usage](../../../../docs/cli/commands.md#command--9-usage)

---

### IT-38: `.usage.help` shows `refresh::` default as `1` (enabled)

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0; stdout contains `"1 = enabled, default"` (indicating `refresh::1` is the default); stdout does NOT contain `"0 = disabled, default"`.
- **Exit:** 0
- **Fix:** BUG-155 (`task/claude_profile/bug/155_refresh_wrong_default.md`)
- **Source fn:** `it37_mre_bug155_refresh_defaults_to_1`
- **Source:** [017_token_refresh.md AC-23](../../../../docs/feature/017_token_refresh.md)

---

### IT-39: `.usage.help` refresh description mentions `429` and locally-expired case

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0; stdout contains `"429"` (the conditional 429+locally-expired refresh case is documented in the parameter description); stdout does NOT contain the old combined string `"401/403/429"`.
- **Exit:** 0
- **Fix:** BUG-156 (`task/claude_profile/bug/156_refresh_429_expired_not_refreshed.md`)
- **Source fn:** `it38_mre_bug156_refresh_help_mentions_429_expired`
- **Source:** [017_token_refresh.md AC-24](../../../../docs/feature/017_token_refresh.md)

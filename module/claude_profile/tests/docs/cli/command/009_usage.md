# Test: `.usage`

Integration test planning for the `.usage` command. See [command/namespace.md](../../../../docs/cli/command/006_usage.md#command--9-usage) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default invocation shows quota table with new column headers | Basic Invocation |
| IT-2 | Current account (live-token match) has `✓` in flag column; others do not | Current Marker |
| IT-3 | Account with missing accessToken shows `—` columns and error reason | Error Inline |
| IT-4 | `format::json` produces valid JSON array with core fields (`expires_in_secs`, `is_current`, `is_active`); no `next_renewal_est` | Output Format |
| IT-5 | Empty credential store exits 0 with `(no accounts configured)` | Edge Case |
| IT-6 | Credential store unreadable exits 2 | Error Handling |
| IT-7 | HOME unset exits 2 | Error Handling |
| IT-8 | Multiple accounts displayed in alphabetical order | Ordering |
| IT-9 | Account with missing token file shows `—` with error reason | Error Inline |
| IT-10 | Account with expired token shows `EXPIRED` in Expires column | Expires Column |
| IT-11 | Best non-current account is marked with `→` in flag column | Recommendation |
| IT-12 | Footer line shows valid count and recommended next account | Footer |
| IT-13 | `*` marks active account when it differs from the current account | Active Divergence |
| IT-14 | When credentials file unreadable: no `✓`; `*` still marks active account | Active Divergence |
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
| IT-40 | Table header row contains `●` column label | Status Emoji |
| IT-41 | Account with missing token → `🔴` in table row | Status Emoji |
| IT-42 | `format::json` output does not contain `🔴`, `🟡`, or `🟢` | Status Emoji |
| IT-43 | Exact boundary: 5h at 15% (`utilization=85.0` → `🟡`), 7d at 5% | Status Emoji |
| IT-44 | `sort::name` accepted with empty store → exit 0 | Sort Acceptance |
| IT-45 | `sort::endurance` accepted with empty store → exit 0 | Sort Acceptance |
| IT-46 | `sort::drain` accepted with empty store → exit 0 | Sort Acceptance |
| IT-47 | `sort::renew` accepted with empty store → exit 0 | Sort Acceptance |
| IT-48 | `sort::bogus` → exit 1, stderr names all five valid values | Sort Rejection |
| IT-65 | `sort::next` accepted with empty store → exit 0 | Sort Acceptance |
| IT-49 | `prefer::bogus` → exit 1, stderr names valid values | Sort Rejection |
| IT-50 | `.usage.help` lists `sort`, `desc`, `prefer` params | Help Output |
| IT-51 | `next::drain` (default) places `→` on drain winner | Next Strategy |
| IT-52 | `next::drain` places `→` on drain winner | Next Strategy |
| IT-53 | `next::bogus` exits 1 naming both valid values | Next Rejection |
| IT-54 | Footer always shows both strategy lines regardless of `next::` value | Next Footer |
| IT-55 | `cols::+sub` shows Sub column in output | Column Visibility |
| IT-56 | `cols::+bogus` exits 1 naming valid column IDs | Column Rejection |
| IT-57 | Composite `●` uses AND(5h, 7d): 5h >15% and 7d >5% → 🟢, either below → 🟡 | Composite Emoji |
| IT-58 | Per-column emoji in `5h Left` value: `🟢 86%` / `🟡 3%` | Per-Column Emoji |
| IT-59 | Duration format capped to 2 units: no 3-unit durations in output | Duration Format |
| IT-60 | Three-tier grouping: 🟢 accounts above 🟡 above 🔴 regardless of sort | Tier Grouping |
| IT-61 | `.usage.help` lists `next`, `cols` params | Help Output |
| IT-62 | `touch::0` accepted; empty store exits 0 | Touch Param |
| IT-63 | `touch::1` with no-token accounts — errored accounts never touched | Touch Param |
| IT-64 | `.usage.help` lists `touch` param with default `1` | Help Output |
| IT-65 | `sort::next` accepted with empty store → exit 0 | Sort Acceptance |
| IT-66 | `imodel::auto` accepted; empty store exits 0 | imodel Param |
| IT-67 | `imodel::bogus` → exit 1, stderr names all five valid values | imodel Param |
| IT-68 | `effort::auto` accepted; empty store exits 0 | effort Param |
| IT-69 | `effort::bogus` → exit 1, stderr names all five valid values | effort Param |
| IT-70 | `.usage.help` lists `imodel` and `effort` params with default `auto` | Help Output |
| IT-71 | `→ Next` column shows soonest upcoming event label + duration | Next Event Column |
| IT-72 | `format::json` new fields: `renewal_secs`, `renewal_is_estimate`, `next_event_type`, `next_event_secs` | JSON Schema |
| IT-73 | 429 rate-limit error — `~Renews` retains billing date (not error reason) | ~Renews Preservation |

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
- Help Output: 8 tests (IT-32, IT-34, IT-38, IT-39, IT-50, IT-61, IT-64, IT-70)
- Trace: 1 test (IT-35)
- Status Emoji: 4 tests (IT-40, IT-41, IT-42, IT-43)
- Sort Acceptance: 5 tests (IT-44, IT-45, IT-46, IT-47, IT-65)
- Sort Rejection: 2 tests (IT-48, IT-49)
- Next Strategy: 2 tests (IT-51, IT-52)
- Next Rejection: 1 test (IT-53)
- Next Footer: 1 test (IT-54)
- Column Visibility: 1 test (IT-55)
- Column Rejection: 1 test (IT-56)
- Composite Emoji: 1 test (IT-57)
- Per-Column Emoji: 1 test (IT-58)
- Duration Format: 1 test (IT-59)
- Tier Grouping: 1 test (IT-60)
- Touch Param: 2 tests (IT-62, IT-63)
- imodel Param: 2 tests (IT-66, IT-67)
- effort Param: 2 tests (IT-68, IT-69)
- Next Event Column: 1 test (IT-71)
- ~Renews Preservation: 1 test (IT-73)

**Total:** 85 spec entries (IT-1 through IT-73); IT-65 added for `sort::next`; IT-66–IT-70 added by TSK-191 (`imodel::`/`effort::` params and `touch::` default `1`); IT-71–IT-72 added by Plan 012 (`→ Next` column and JSON new fields); IT-73 added by Plan 013 (BUG-220 `~Renews` preservation); source functions it17–it33 map to spec IT-18–IT-34; it34/it35/it36 map to IT-35/IT-36/IT-37; it37 maps to IT-38; it38 maps to IT-39; IT-17 covered by `ft002_lim_it_http_401_shortens_to_auth_expired` in `usage_feature_test.rs` (live network test; kept in feature test file to avoid duplication with FT-02); it39–it52 covered by param spec docs `tests/docs/cli/param/019_refresh.md`–`023_trace.md` (param EC edge cases, not command spec)

---

### IT-1: Default invocation shows quota table with new column headers

- **Given:** At least one saved account with a valid token exists in the credential store.
- **When:** `clp .usage`
- **Then:** Stdout contains a table with "Quota" heading and rows showing columns: "5h Left", "5h Reset", "7d Left", "7d Reset", "Expires", "~Renews", "→ Next". Column order: quota columns (5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset) appear before billing-metadata columns (Expires, ~Renews, → Next). Exit 0.
- **Exit:** 0
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-2: Current account (live-token match) has `✓` in flag column

- **Given:** Two saved accounts; live `~/.claude/.credentials.json` has an `accessToken` matching `work@acme.com`'s stored token. Per-machine active marker also points to `work@acme.com` (current = active, normal case).
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
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-4: `format::json` produces valid JSON array with `expires_in_secs`, `is_current`, `is_active`

- **Given:** At least one saved account with a valid token.
- **When:** `clp .usage format::json`
- **Then:** Valid JSON array on stdout. Each element has `account` (string), `is_current` (boolean), `is_active` (boolean), `expires_in_secs` (number), `billing_type` (string or null), `has_max` (boolean or null), `renewal_secs` (number or null), `renewal_is_estimate` (boolean or null), `next_event_type` (string or null), and `next_event_secs` (number or null). No element has a `next_renewal_est` key (deprecated). Successful elements have `session_5h_left_pct` and `weekly_7d_left_pct`. No element has a top-level `active` key. Exit 0.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-08](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-5: Empty credential store shows empty table

- **Given:** Credential store exists but contains no `*.credentials.json` files.
- **When:** `clp .usage`
- **Then:** Stdout contains `(no accounts configured)`. Exit 0.
- **Exit:** 0
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-6: Credential store unreadable exits 2

- **Given:** `HOME` is set but credential store directory cannot be read (permissions error).
- **When:** `clp .usage`
- **Then:** Error on stderr. Exit 2.
- **Exit:** 2
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-7: HOME unset exits 2

- **Given:** `HOME` environment variable is unset.
- **When:** `env -u HOME clp .usage`
- **Then:** Error on stderr. Exit 2.
- **Exit:** 2
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-8: Multiple accounts displayed in alphabetical order

- **Given:** Three saved accounts: `c@x.com`, `a@x.com`, `b@x.com`.
- **When:** `clp .usage`
- **Then:** Rows appear in order `a@x.com`, `b@x.com`, `c@x.com`. Exit 0.
- **Exit:** 0
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-9: Account with missing token file shows `—` with error reason

- **Given:** Credential store entry exists but the `.credentials.json` file for that account is missing.
- **When:** `clp .usage`
- **Then:** That account's row shows `—` for quota columns and a missing-token error reason in Status. Exit 0.
- **Exit:** 0
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-10: Account with expired token shows `EXPIRED` in Expires column

- **Given:** One saved account whose `expiresAt` in the credential file is a past timestamp (e.g., `PAST_MS`).
- **When:** `clp .usage`
- **Then:** That account's row shows `EXPIRED` in the Expires column. The quota columns show `—`. Exit 0.
- **Exit:** 0
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-11: Best non-current account is marked with `→` in flag column

- **Given:** Two accounts — one active with quota data, one non-active with valid token and quota data showing lower session usage than the active account.
- **When:** `clp .usage`
- **Then:** A line in stdout contains both `→` and the non-active account name. No line contains both `→` and the active account name. Exit 0.
- **Exit:** 0
- **Live:** yes (requires real tokens for both accounts to return quota data)
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-12: Footer line shows valid count and recommended next account

- **Given:** At least two accounts with valid tokens that return quota data.
- **When:** `clp .usage`
- **Then:** Stdout contains a footer line matching "Valid: N / M" and "Next:" with the recommended account name. Exit 0.
- **Exit:** 0
- **Live:** yes (requires ≥2 accounts with live quota headers)
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-13: `*` marks active account when it differs from current

- **Given:** Two saved accounts: `alice@acme.com` (active account) and `work@acme.com`. Live `~/.claude/.credentials.json` `accessToken` matches `work@acme.com`'s stored token (not `alice`'s).
- **When:** `clp .usage`
- **Then:** A line contains `✓` and `work@acme.com`; a different line contains `*` and `alice@acme.com`. No line contains both `✓` and `alice`, or both `*` and `work`.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-06](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-14: Credentials file unreadable — no `✓`; `*` still marks active account

- **Given:** Two saved accounts: `alice@acme.com` (active account) and `work@acme.com`. `~/.claude/.credentials.json` is absent or unreadable.
- **When:** `clp .usage`
- **Then:** No line contains `✓`; a line contains `*` and `alice@acme.com`. All saved accounts are still shown. Exit 0.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-07](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-15: When current = active, only `✓` appears; no `*` on any row

- **Given:** Two saved accounts: `alice@acme.com` (active account) and `work@acme.com`. Live `~/.claude/.credentials.json` `accessToken` matches `alice@acme.com`'s stored token (current = active).
- **When:** `clp .usage`
- **Then:** A line contains `✓` and `alice@acme.com`; no line contains `*`.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-06](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-16: JSON output uses `is_current` and `is_active`; no `active` key

- **Given:** Two saved accounts; live credentials match one of them; per-machine active marker points to the other (divergence case).
- **When:** `clp .usage format::json`
- **Then:** Valid JSON array; the current account object has `"is_current":true` and `"is_active":false`; the active account object has `"is_current":false` and `"is_active":true`; no object has a top-level `"active"` key.
- **Exit:** 0
- **Source:** [016_current_account_awareness.md AC-08](../../../../docs/feature/016_current_account_awareness.md)

---

### IT-17: HTTP 401 from usage API shows `(auth expired (401))` in 7d Reset column

- **Given:** One saved account whose `expiresAt` in the credential file is a past timestamp and whose `accessToken` the usage API rejects with HTTP 401.
- **When:** `clp .usage`
- **Then:** That account's row shows `EXPIRED` in Expires and `—` for all quota columns (5h Left, 5h Reset, 7d Left, 7d(Son)); the 7d Reset column shows `(auth expired (401))` — NOT `(HTTP transport error: HTTP 401)`. Exit 0.
- **Exit:** 0
- **Fix:** BUG-152 (`task/claude_profile/bug/152_shorten_error_omits_401.md`)
- **Source fn:** `ft002_lim_it_http_401_shortens_to_auth_expired` (in `usage_feature_test.rs`)
- **Source:** [009_token_usage.md AC-03](../../../../docs/feature/009_token_usage.md)

---

### IT-18: `.usage format::table` exits 1 (`ArgumentTypeMismatch`)

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage format::table`
- **Then:** Exits 1. `format::table` is valid only for `.accounts`; all other commands reject it.
- **Exit:** 1
- **Source fn:** `it017_format_table_rejected`
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-19: Live token unmatched → synthetic `(current session)` row prepended

- **Given:** One saved account `alice@acme.com` with token `tok-alice`; live `~/.claude/.credentials.json` uses a different token `tok-unsaved`.
- **When:** `clp .usage`
- **Then:** Table contains a `(current session)` row with `✓` in the flag column; `alice@acme.com` does NOT have `✓`. Exit 0.
- **Exit:** 0
- **Source fn:** `it018_synthetic_row_when_no_saved_match`
- **Source:** [009_token_usage.md AC-11](../../../../docs/feature/009_token_usage.md)

---

### IT-20: `refresh::0` accepted; empty store exits 0

- **Given:** Empty credential store; `refresh::0` param passed.
- **When:** `clp .usage refresh::0`
- **Then:** Exits 0 with "no accounts configured" message. `refresh::0` explicitly disables the default refresh behavior without breaking baseline output.
- **Exit:** 0
- **Source fn:** `it019_refresh_disabled_param_accepted`
- **Source:** [017_token_refresh.md AC-18](../../../../docs/feature/017_token_refresh.md)

---

### IT-21: `refresh::1` accepted; no retry triggered when HTTP is not reached

- **Given:** One account with no `accessToken` in the credential file (read_token returns Err without any HTTP call); `refresh::1` param.
- **When:** `clp .usage refresh::1`
- **Then:** Exits 0; account name appears in output. No HTTP call is made, so no 401 is triggered and no retry loop fires.
- **Exit:** 0
- **Source fn:** `it020_refresh_enabled_offline_no_retry_triggered`
- **Source:** [017_token_refresh.md AC-19](../../../../docs/feature/017_token_refresh.md)

---

### IT-22: `live::1 interval::30 jitter::0` — live loop emits countdown footer (lim_it)

- **Given:** One saved account with a valid live token; `live::1 interval::30 jitter::0`; process killed after 10 s.
- **When:** `clp .usage live::1 interval::30 jitter::0`
- **Then:** stdout (captured from raw bytes) contains "Next update". Exit determined by kill signal.
- **Live:** yes (lim_it — requires live credentials)
- **Source fn:** `it021_lim_it_live_mode`
- **Source:** [018_live_monitor.md AC-28](../../../../docs/feature/018_live_monitor.md)

---

### IT-23: `live::1 interval::60 jitter::70` — jitter > interval → exit 1

- **Given:** Any environment; guard fires before any fetch.
- **When:** `clp .usage live::1 interval::60 jitter::70`
- **Then:** Exits 1; stderr is non-empty (validation error).
- **Exit:** 1
- **Source fn:** `it022_live_jitter_exceeds_interval`
- **Source:** [018_live_monitor.md AC-27](../../../../docs/feature/018_live_monitor.md)

---

### IT-24: `live::1 interval::5` — interval below 30 → exit 1, error mentions "30"

- **Given:** Any environment; guard fires before any fetch.
- **When:** `clp .usage live::1 interval::5 jitter::0`
- **Then:** Exits 1; stderr contains "30" (the minimum interval).
- **Exit:** 1
- **Source fn:** `it023_live_interval_below_minimum`
- **Source:** [018_live_monitor.md AC-26](../../../../docs/feature/018_live_monitor.md)

---

### IT-25: `live::1 format::json` — incompatible with live mode → exit 1

- **Given:** Any environment; guard fires before any fetch.
- **When:** `clp .usage live::1 format::json`
- **Then:** Exits 1; stderr is non-empty.
- **Exit:** 1
- **Source fn:** `it024_live_incompatible_with_json`
- **Source:** [018_live_monitor.md AC-25](../../../../docs/feature/018_live_monitor.md)

---

### IT-26: Live token unmatched + `.claude.json` email → synthetic row shows email

- **Given:** One saved account `alice@acme.com` with `tok-alice`; live credentials use `tok-unsaved`; `~/.claude.json` has `emailAddress = "unsaved@example.com"`.
- **When:** `clp .usage`
- **Then:** Table shows `unsaved@example.com` with `✓` in the flag column; does NOT show `(current session)` fallback. Exit 0.
- **Exit:** 0
- **Source fn:** `it025_synthetic_row_uses_claude_json_email`
- **Source:** [009_token_usage.md AC-11](../../../../docs/feature/009_token_usage.md)

---

### IT-27: `live::1 interval::30 jitter::30` — jitter equal to interval is accepted

- **Given:** Credential store directory chmod 000 (unreadable); `live::1 interval::30 jitter::30`. Guard uses strict greater-than (`jitter > interval`), so equal values must not fire.
- **When:** `clp .usage live::1 interval::30 jitter::30`
- **Then:** Exits 2 (store unreadable — proves `execute_live_mode()` was entered; guards passed); stderr does NOT contain "jitter".
- **Exit:** 2
- **Source fn:** `it026_live_jitter_equals_interval_accepted`
- **Source:** [018_live_monitor.md AC-27](../../../../docs/feature/018_live_monitor.md)

---

### IT-28: `format::json` for failed account → JSON has `"error"` field

- **Given:** One account with no `accessToken` in the credential file (read_token returns Err).
- **When:** `clp .usage format::json`
- **Then:** Exits 0; JSON contains `"error":` key; does NOT contain `"session_5h_left_pct"`; does contain `"is_current"`, `"is_active"`, `"expires_in_secs"`, `"billing_type"` (null — token read failed, no account fetch ran), `"has_max"` (null), `"renewal_secs"` (null), `"renewal_is_estimate"` (null), `"next_event_type"` (null), `"next_event_secs"` (null); does NOT contain `"next_renewal_est"` (deprecated field removed).
- **Exit:** 0
- **Source fn:** `it027_json_error_field_on_failed_account`
- **Source:** [009_token_usage.md AC-05](../../../../docs/feature/009_token_usage.md)

---

### IT-29: `interval::5 jitter::70` without `live::1` → guards not triggered, exits 0

- **Given:** Empty credential store; `interval::5 jitter::70` without `live::1`.
- **When:** `clp .usage interval::5 jitter::70`
- **Then:** Exits 0 with "no accounts" message; live-mode guards (interval minimum, jitter ceiling) do NOT fire.
- **Exit:** 0
- **Source fn:** `it028_interval_jitter_ignored_when_not_live`
- **Source:** [018_live_monitor.md AC-31](../../../../docs/feature/018_live_monitor.md)

---

### IT-30: `live::1` alone — default interval 30 satisfies >= 30 guard

- **Given:** Credential store directory chmod 000; `live::1` with no explicit interval or jitter. Defaults: `interval=30`, `jitter=0`. Guard is `interval < 30` (strict less-than).
- **When:** `clp .usage live::1`
- **Then:** Exits 2 (store unreadable — proves interval guard did NOT fire); stderr does NOT contain "interval".
- **Exit:** 2
- **Source fn:** `it029_live_default_interval_accepted`
- **Source:** [018_live_monitor.md AC-28](../../../../docs/feature/018_live_monitor.md)

---

### IT-31: SIGINT in live mode → clean exit 0; stdout contains "Monitor stopped."

- **Given:** One account with no `accessToken` (fetch fails instantly without HTTP, ensuring render + countdown start within 3 s); `live::1 interval::30 jitter::0`; SIGINT sent via `kill -INT` after 3 s.
- **When:** `clp .usage live::1 interval::30 jitter::0` (then SIGINT)
- **Then:** Process exits with code 0; stdout contains "Monitor stopped."
- **Exit:** 0
- **Source fn:** `it030_live_sigint_exits_0`
- **Source:** [018_live_monitor.md AC-30](../../../../docs/feature/018_live_monitor.md)

---

### IT-32: `.usage.help` lists `live`, `interval`, `jitter` params

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0; stdout contains "live", "interval", and "jitter".
- **Exit:** 0
- **Source fn:** `it031_usage_help_shows_live_params`
- **Source:** [018_live_monitor.md AC-32](../../../../docs/feature/018_live_monitor.md)

---

### IT-33: `refresh::1` per-account refresh loop — no panic, exit 0 (lim_it)

- **Given:** One saved account with a valid live token (from `live_active_token()`); `refresh::1`.
- **When:** `clp .usage refresh::1`
- **Then:** Exits 0; no panic; per-account refresh loop runs (happy-path: quota fetch succeeds on first pass, no retry needed).
- **Exit:** 0
- **Live:** yes (lim_it — requires live credentials)
- **Source fn:** `it032_lim_it_refresh_per_account`
- **Source:** [017_token_refresh.md AC-19](../../../../docs/feature/017_token_refresh.md)

---

### IT-34: `.usage.help` refresh description includes "401/403" but NOT "401/403/429"

- **Given:** Standard environment. Task 150 removed HTTP 429 from the refresh retry guard; the parameter description must no longer mention it.
- **When:** `clp .usage.help`
- **Then:** Exits 0; stdout contains "401/403"; stdout does NOT contain the substring "401/403/429".
- **Exit:** 0
- **Source fn:** `it033_mre_refresh_help_excludes_429`
- **Source:** [017_token_refresh.md AC-23](../../../../docs/feature/017_token_refresh.md)

---

### IT-35: `trace::1` with no-token account → stderr contains `[trace]` lines

- **Given:** One saved account whose credential file has no `accessToken` field.
- **When:** `clp .usage trace::1`
- **Then:** Exits 0; stderr contains `[trace]` lines including the account name; stdout still shows the account row.
- **Exit:** 0
- **Source fn:** `it034_trace_param_writes_to_stderr`
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-36: Empty store + `format::json` → output is `[]`

- **Given:** Credential store directory exists but contains no `*.credentials.json` files.
- **When:** `clp .usage format::json`
- **Then:** Exits 0; stdout (trimmed) equals `[]`; no text-format "no accounts configured" message.
- **Exit:** 0
- **Source fn:** `it035_empty_store_json_format`
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-37: Single failed account → no "Valid:" footer line emitted

- **Given:** One saved account whose credential file has no `accessToken` (quota fetch fails; `valid_count = 0`).
- **When:** `clp .usage`
- **Then:** Exits 0; stdout does NOT contain "Valid:" (footer is suppressed when `valid_count < 2`).
- **Exit:** 0
- **Source fn:** `it036_no_footer_when_no_valid_accounts`
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command--9-usage)

---

### IT-38: `.usage.help` shows `refresh::` default as `1` (enabled)

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0; stdout contains `"1 = enabled, default"` (indicating `refresh::1` is the default); stdout does NOT contain `"0 = disabled, default"`.
- **Exit:** 0
- **Fix:** BUG-155 (`task/claude_profile/bug/155_refresh_wrong_default.md`)
- **Source fn:** `it037_mre_bug155_refresh_defaults_to_1`
- **Source:** [017_token_refresh.md AC-23](../../../../docs/feature/017_token_refresh.md)

---

### IT-39: `.usage.help` refresh description mentions `429` and locally-expired case

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0; stdout contains `"429"` (the conditional 429+locally-expired refresh case is documented in the parameter description); stdout does NOT contain the old combined string `"401/403/429"`.
- **Exit:** 0
- **Fix:** BUG-156 (`task/claude_profile/bug/156_refresh_429_expired_not_refreshed.md`)
- **Source fn:** `it038_mre_bug156_refresh_help_mentions_429_expired`
- **Source:** [017_token_refresh.md AC-24](../../../../docs/feature/017_token_refresh.md)

---

### IT-40: Table header row contains `●` column label

- **Given:** One saved account with a valid credential file (no accessToken — produces error row, but table is still rendered).
- **When:** `clp .usage`
- **Then:** Exits 0. Stdout contains `"●"` (the status emoji column header).
- **Exit:** 0
- **Source fn:** `it148_status_emoji_column_header_present`
- **Source:** [009_token_usage.md AC-18](../../../../docs/feature/009_token_usage.md)

---

### IT-41: Account with missing token → `🔴` in table row

- **Given:** One saved account whose credential file exists but has no `accessToken` field (result is Err).
- **When:** `clp .usage`
- **Then:** Exits 0. Stdout contains `"🔴"`.
- **Exit:** 0
- **Source fn:** `it149_status_emoji_red_on_token_error`
- **Source:** [009_token_usage.md AC-18](../../../../docs/feature/009_token_usage.md)

---

### IT-42: `format::json` output does not contain status emoji

- **Given:** One saved account whose credential file has no `accessToken` field.
- **When:** `clp .usage format::json`
- **Then:** Exits 0. Stdout does NOT contain `"🔴"`, `"🟡"`, or `"🟢"`.
- **Exit:** 0
- **Source fn:** `it150_status_emoji_absent_from_json`
- **Source:** [009_token_usage.md AC-20](../../../../docs/feature/009_token_usage.md)

---

### IT-43: Exact boundary — composite AND: 5h at 15%, 7d at 5%

- **Given:** Unit test of `status_emoji()`. Three `OauthUsageData` variants:
  - A: `five_hour.utilization = 85.0` (15.0% left), `seven_day.utilization = 50.0` (50% left) → expected `🟡` (5h at boundary)
  - B: `five_hour.utilization = 84.9` (15.1% left), `seven_day.utilization = 50.0` (50% left) → expected `🟢` (both above threshold)
  - C: `five_hour.utilization = 50.0` (50% left), `seven_day.utilization = 95.0` (5.0% left) → expected `🟡` (7d at boundary)
- **When:** `status_emoji(&Ok(data_a))`, `status_emoji(&Ok(data_b))`, `status_emoji(&Ok(data_c))`
- **Then:** A returns `"🟡"`; B returns `"🟢"`; C returns `"🟡"`. Composite AND: `5h Left > 15.0%` and `7d Left > 5.0%` required for `🟢`.
- **Exit:** n/a (unit test)
- **Source fn:** `it151_status_emoji_boundary_precision`
- **Source:** [009_token_usage.md AC-19](../../../../docs/feature/009_token_usage.md)

---

### IT-44: `sort::name` accepted with empty store → exit 0

- **Given:** Empty credential store.
- **When:** `clp .usage sort::name`
- **Then:** Exits 0 with "(no accounts configured)". No unknown-parameter error.
- **Exit:** 0
- **Source fn:** `it053_sort_name_accepted`
- **Source:** [feature/020_usage_sort_strategies.md AC-01](../../../../docs/feature/020_usage_sort_strategies.md)

---

### IT-45: `sort::endurance` accepted with empty store → exit 0

- **Given:** Empty credential store.
- **When:** `clp .usage sort::endurance`
- **Then:** Exits 0 with "(no accounts configured)". No unknown-parameter error.
- **Exit:** 0
- **Source fn:** `it054_sort_endurance_accepted`
- **Source:** [feature/020_usage_sort_strategies.md AC-02](../../../../docs/feature/020_usage_sort_strategies.md)

---

### IT-46: `sort::drain` accepted with empty store → exit 0

- **Given:** Empty credential store.
- **When:** `clp .usage sort::drain`
- **Then:** Exits 0 with "(no accounts configured)". No unknown-parameter error.
- **Exit:** 0
- **Source fn:** `it055_sort_drain_accepted`
- **Source:** [feature/020_usage_sort_strategies.md AC-03](../../../../docs/feature/020_usage_sort_strategies.md)

---

### IT-47: `sort::renew` accepted with empty store → exit 0

- **Given:** Empty credential store.
- **When:** `clp .usage sort::renew`
- **Then:** Exits 0 with "(no accounts configured)". No unknown-parameter error.
- **Exit:** 0
- **Source fn:** `it056_sort_renew_accepted`
- **Source:** [feature/020_usage_sort_strategies.md AC-04](../../../../docs/feature/020_usage_sort_strategies.md)

---

### IT-48: `sort::bogus` → exit 1, stderr names all five valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage sort::bogus`
- **Then:** Exits 1. Stderr contains each of the five valid values: `name`, `endurance`, `drain`, `renew`, `next`.
- **Exit:** 1
- **Source fn:** `it057_sort_invalid_value_exit_1`
- **Source:** [feature/020_usage_sort_strategies.md AC-09](../../../../docs/feature/020_usage_sort_strategies.md)

---

### IT-49: `prefer::bogus` → exit 1, stderr names valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage prefer::bogus`
- **Then:** Exits 1. Stderr contains each of the three valid values: `any`, `opus`, `sonnet`.
- **Exit:** 1
- **Source fn:** `it058_prefer_invalid_value_exit_1`
- **Source:** [feature/020_usage_sort_strategies.md AC-10](../../../../docs/feature/020_usage_sort_strategies.md)

---

### IT-50: `.usage.help` lists `sort`, `desc`, `prefer` params

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains `"sort"`, `"desc"`, and `"prefer"`.
- **Exit:** 0
- **Source fn:** `it059_usage_help_shows_sort_params`
- **Source:** [feature/020_usage_sort_strategies.md](../../../../docs/feature/020_usage_sort_strategies.md)

---

### IT-51: `next::drain` (default) places `→` on drain winner

- **Given:** Two saved accounts with valid tokens and quota data; default `next::drain`.
- **When:** `clp .usage`
- **Then:** Exits 0. Exactly one line contains `→` in the flag column — the account selected by the drain strategy (lowest non-exhausted `prefer_weekly` / 7d Left). Footer contains "Next by strategy:" with two lines (endurance and drain).
- **Exit:** 0
- **Live:** yes (requires ≥2 accounts with live quota)
- **Source fn:** `it103_lim_it_next_drain_places_arrow_on_winner` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-04](../../../../docs/feature/023_next_account_strategies.md)

---

### IT-52: `next::drain` places `→` on drain winner

- **Given:** Two saved accounts with valid tokens and quota data; `next::drain`.
- **When:** `clp .usage next::drain`
- **Then:** Exits 0. Exactly one line contains `→` — the account selected by the drain strategy (lowest non-exhausted `prefer_weekly` / 7d Left). Footer still contains "Next by strategy:" with both strategy lines.
- **Exit:** 0
- **Live:** yes (requires ≥2 accounts with live quota)
- **Source fn:** `it103_lim_it_next_drain_places_arrow_on_winner` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-04](../../../../docs/feature/023_next_account_strategies.md)

---

### IT-53: `next::bogus` exits 1 naming both valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage next::bogus`
- **Then:** Exits 1. Stderr contains each of the two valid values: `endurance`, `drain`. Does NOT contain `all`, `session`, or `reset`.
- **Exit:** 1
- **Source fn:** `it092_next_all_rejected_exit_1`, `it094_next_session_rejected_exit_1` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-05](../../../../docs/feature/023_next_account_strategies.md)

---

### IT-54: Footer always shows both strategy lines regardless of `next::` value

- **Given:** At least two accounts with valid tokens and quota data; `next::drain` (non-default).
- **When:** `clp .usage next::drain`
- **Then:** Exits 0. Footer contains "Next by strategy:" followed by a line starting "endurance" AND a line starting "drain". Both appear regardless of which strategy is active.
- **Exit:** 0
- **Live:** yes (requires ≥2 accounts with live quota)
- **Source fn:** `it104_lim_it_footer_always_shows_both_strategy_lines` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/023_next_account_strategies.md AC-01](../../../../docs/feature/023_next_account_strategies.md)

---

### IT-55: `cols::+sub` shows Sub column in output

- **Given:** One saved account with valid credentials.
- **When:** `clp .usage cols::+sub`
- **Then:** Exits 0. Table header contains `Sub`.
- **Exit:** 0
- **Source fn:** `it081_cols_sub_shows_sub_column` (in `tests/cli/usage_test.rs`)
- **Source:** [009_token_usage.md AC-22](../../../../docs/feature/009_token_usage.md)

---

### IT-56: `cols::+bogus` exits 1 naming valid column IDs

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage cols::+bogus`
- **Then:** Exits 1. Stderr names valid column IDs.
- **Exit:** 1
- **Source fn:** `it082_cols_unknown_id_exit_1` (in `tests/cli/usage_test.rs`)
- **Source:** [009_token_usage.md AC-23](../../../../docs/feature/009_token_usage.md)

---

### IT-57: Composite `●` uses AND(5h, 7d): 5h >15% and 7d >5% → 🟢, either below → 🟡

- **Given:** Unit test of `status_emoji()`. Two `OauthUsageData` variants:
  - A: `five_hour.utilization = 50%` (50% left), `seven_day.utilization = 96%` (4% left) → expected `🟡` (7d exhausted)
  - B: `five_hour.utilization = 50%` (50% left), `seven_day.utilization = 50%` (50% left) → expected `🟢` (both healthy)
- **When:** `status_emoji(&Ok(data_a))` and `status_emoji(&Ok(data_b))`
- **Then:** A returns `"🟡"`; B returns `"🟢"`.
- **Exit:** n/a (unit test)
- **Source fn:** `test_status_emoji_and_both_ample_green`, `test_status_emoji_and_7d_low_yellow` (in `src/usage/format.rs`)
- **Source:** [009_token_usage.md AC-18](../../../../docs/feature/009_token_usage.md)

---

### IT-58: Per-column emoji in `5h Left` value: `🟢 86%` / `🟡 3%`

- **Given:** Two accounts: one with `five_hour.utilization=14%` (86% left), one with `five_hour.utilization=97%` (3% left).
- **When:** `clp .usage`
- **Then:** Exits 0. The first account's `5h Left` column contains `🟢`; the second contains `🟡`.
- **Exit:** 0
- **Live:** yes (requires real tokens)
- **Source fn:** `it105_lim_it_per_column_emoji_in_5h_left` (in `tests/cli/usage_test.rs`)
- **Source:** [009_token_usage.md AC-21](../../../../docs/feature/009_token_usage.md)

---

### IT-59: Duration format capped to 2 units: no 3-unit durations in output

- **Given:** Unit test of `format_duration_secs()`. Input: `90061` seconds (1d 1h 1m 1s).
- **When:** `format_duration_secs(90061)`
- **Then:** Returns `"1d 1h"` (not `"1d 1h 1m"` or `"1d 1h 1m 1s"`).
- **Exit:** n/a (unit test)
- **Source fn:** `test_format_duration_secs_caps_at_two_units` (in `src/output.rs`)
- **Source:** [009_token_usage.md AC-25](../../../../docs/feature/009_token_usage.md)

---

### IT-60: Three-tier grouping: 🟢 accounts above 🟡 above 🔴 regardless of sort

- **Given:** Three `AccountQuota` structs: `A` (5h > 15% and 7d > 5% — 🟢), `B` (5h ≤ 15% — 🟡), `C` (error — 🔴). Any sort strategy.
- **When:** `render_text(...)` with any sort strategy.
- **Then:** In the output, `A` appears before `B`, and `B` appears before `C`.
- **Exit:** n/a (unit test)
- **Source fn:** `test_three_tier_grouping_green_before_yellow_before_red` (in `src/usage/mod.rs`)
- **Source:** [009_token_usage.md AC-24](../../../../docs/feature/009_token_usage.md)

---

### IT-61: `.usage.help` lists `next`, `cols` params

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains `"next"` and `"cols"`.
- **Exit:** 0
- **Source fn:** `it083_usage_help_shows_next_cols_params` (in `tests/cli/usage_test.rs`)
- **Source:** [009_token_usage.md AC-09](../../../../docs/feature/009_token_usage.md)

---

### IT-62: `touch::0` accepted; empty store exits 0

- **Given:** Empty credential store; `touch::0` param passed (explicit default).
- **When:** `clp .usage touch::0`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned.
- **Exit:** 0
- **Source fn:** `it106_touch_0_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-01](../../../../docs/feature/024_session_touch.md)

---

### IT-63: `touch::1` with no-token accounts — errored accounts never touched

- **Given:** One saved account whose credential file has no `accessToken` (quota fetch returns Err); `touch::1`.
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. Account row shows original error state. No subprocess spawned — touch trigger requires `result = Ok(...)`.
- **Exit:** 0
- **Source fn:** `it098_touch_1_errored_account_skipped` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-04](../../../../docs/feature/024_session_touch.md)

---

### IT-64: `.usage.help` lists `touch` param with default `1`

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains `"touch"` with default value `1` (on).
- **Exit:** 0
- **Source fn:** `it101_usage_help_shows_touch_param` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-10](../../../../docs/feature/024_session_touch.md)

---

### IT-65: `sort::next` accepted with empty store → exit 0

- **Given:** Empty credential store.
- **When:** `clp .usage sort::next`
- **Then:** Exits 0 with "(no accounts configured)". `sort::next` resolves to `sort::drain` (default `next::drain`) and is accepted without error.
- **Exit:** 0
- **Source fn:** `it121_sort_next_accepted` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/020_usage_sort_strategies.md AC-15](../../../../docs/feature/020_usage_sort_strategies.md)

---

### IT-66: `imodel::auto` accepted; empty store exits 0

- **Given:** Empty credential store.
- **When:** `clp .usage imodel::auto`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. `auto` is the default; no subprocess spawned (no accounts).
- **Exit:** 0
- **Source fn:** `it122_imodel_auto_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../../docs/feature/026_subprocess_model_effort.md)

---

### IT-67: `imodel::bogus` → exit 1, stderr names all five valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage imodel::bogus`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `sonnet`, `opus`, `haiku`, `keep`.
- **Exit:** 1
- **Source fn:** `it123_imodel_bogus_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-10](../../../../docs/feature/026_subprocess_model_effort.md)

---

### IT-68: `effort::auto` accepted; empty store exits 0

- **Given:** Empty credential store.
- **When:** `clp .usage effort::auto`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. `auto` is the default; no subprocess spawned (no accounts).
- **Exit:** 0
- **Source fn:** `it124_effort_auto_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../../docs/feature/026_subprocess_model_effort.md)

---

### IT-69: `effort::bogus` → exit 1, stderr names all five valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage effort::bogus`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `low`, `normal`, `high`, `max`.
- **Exit:** 1
- **Source fn:** `it125_effort_bogus_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-11](../../../../docs/feature/026_subprocess_model_effort.md)

---

### IT-70: `.usage.help` lists `imodel` and `effort` params with default `auto`

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains `"imodel"` and `"effort"`, each showing default value `auto`.
- **Exit:** 0
- **Source fn:** `it126_usage_help_shows_imodel_effort_params` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-12](../../../../docs/feature/026_subprocess_model_effort.md)

---

### IT-71: `→ Next` column shows soonest upcoming strategic event label and duration

- **Given:** One account with live quota data: `seven_day.resets_at` set to ~2 days in the future; no `_renewal_at`. The `+7d` reset is soonest.
- **When:** `clp .usage`
- **Then:** Exits 0. The `→ Next` column header appears in the table header row. That account's `→ Next` cell contains `+7d in` followed by a duration string (e.g., `+7d in 2d 0m`). No `!tok` or `+5h` label appears — token expiry and 5h resets are not candidates for `→ Next`.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it225_lim_it_it71_next_event_cell_shows_label_and_duration` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/009_token_usage.md AC-28](../../../../docs/feature/009_token_usage.md)

---

### IT-72: `format::json` output contains new fields; `next_renewal_est` absent

- **Given:** One account with `_renewal_at` set to a future timestamp (~6 days away). Token expiry is 8h away; `five_hour.resets_at` is 3h away; no `seven_day.resets_at` present.
- **When:** `clp .usage format::json`
- **Then:** Exits 0. JSON array contains one element with:
  - `renewal_secs`: positive integer (~518400 for 6 days)
  - `renewal_is_estimate`: `false` (sourced from `_renewal_at`, not estimate)
  - `next_event_type`: `"ren"` (soonest strategic event is `$ren`; sigil stripped in JSON output)
  - `next_event_secs`: positive integer (~518400 for 6 days)
  - No `next_renewal_est` key present in the object.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it222_lim_it_it72_json_new_renewal_fields` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/009_token_usage.md AC-29](../../../../docs/feature/009_token_usage.md)

---

### IT-73: 429 rate-limit error — `~Renews` retains billing date (not error reason)

- **Given (unit test):** An `AccountQuota` constructed with:
  - `result = Err("rate limited (429)")` — quota API failed with 429
  - `account = Some(OauthAccountData { org_created_at: "1970-01-15T00:00:00Z", billing_type: "stripe_subscription", has_max: true })` — account data fetched independently, unaffected by the 429 on the usage API
  - `expires_at_ms = FAR_FUTURE_MS` — non-expired token
  - Default `ColsVisibility` (renews ON; host and role OFF)
  - `now_secs` fixed such that `next_billing_label("1970-01-15T00:00:00Z", now_secs)` returns a known date string (not `"?"`)
- **When:** The `AccountQuota` is rendered via both `render_text()` and `render_tsv()`.
- **Then:**
  - `render_tsv()`: the `~Renews` tab-separated field is NOT `(rate limited (429))` and is NOT `"?"`. It contains the billing date string from `OauthAccountData.org_created_at`.
  - `render_text()`: the output contains `(rate limited (429))` in at least one quota column (`5h Left`–`7d Reset`); the `~Renews` cell does NOT contain `(rate limited (429))`.
- **Exit:** n/a (unit test)
- **Note:** Fix for BUG-220. `render_text()` used `last_mut()` positional overwrite that hit `~Renews` as the last pushed column; `render_tsv()` explicitly pushed `error_str` for the renews cell. Both were incorrect — `~Renews` is sourced from `OauthAccountData` independently of the quota fetch result.
- **Source fn:** `mre_bug_220_renews_preserved_for_429_accounts` (in `src/usage/render.rs`)
- **Source:** [feature/009_token_usage.md AC-03](../../../../docs/feature/009_token_usage.md)

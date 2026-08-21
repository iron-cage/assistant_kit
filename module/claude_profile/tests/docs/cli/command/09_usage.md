# Test: `.usage`

Integration test planning for the `.usage` command. See [command/namespace.md](../../../../docs/cli/command/006_usage.md#command-9-usage) for specification.

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
| IT-11 | Recommended account appears in footer `Next (<strategy>)` line with `·` delimiter; no `→` in table rows | Recommendation |
| IT-12 | Footer `Current` line shows `✓` account with `·`-delimited model and valid count | Footer |
| IT-13 | `*` marks active account when it differs from the current account | Active Divergence |
| IT-14 | When credentials file unreadable: no `✓`; `*` still marks active account | Active Divergence |
| IT-15 | When current = active, only `✓` appears; no `*` on any row | Active Divergence |
| IT-16 | JSON output uses `is_current` (not `active`) and includes `is_active` per object | JSON Schema |
| IT-17 | HTTP error from usage API never shows verbose `HTTP transport error:` string (exact short form not deterministic) | Error Shortening |
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
| IT-35 | `trace::1` with no-token account → stderr contains timestamped diagnostic lines | Trace |
| IT-36 | Empty store + `format::json` → output is `[]` | Output Format |
| IT-37 | Single failed account → no `Valid:` footer line emitted | Footer |
| IT-38 | `.usage.help` shows `refresh::` default as `1` (enabled) | Help Output |
| IT-39 | `.usage.help` refresh description mentions `429` and locally-expired case | Help Output |
| IT-40 | Table header row contains `●` column label | Status Emoji |
| IT-41 | Account with missing token → `🔴` in table row | Status Emoji |
| IT-42 | `format::json` output does not contain `🔴`, `🟡`, or `🟢` | Status Emoji |
| IT-44 | `sort::name` accepted with empty store → exit 0 | Sort Acceptance |
| IT-45 | ~~`sort::endurance` accepted~~ → REMOVED (now rejected — see `it249`) | Sort Rejection |
| IT-46 | ~~`sort::drain` accepted~~ → REMOVED (now rejected — see `it250`) | Sort Rejection |
| IT-47 | `sort::renew` accepted with empty store → exit 0 | Sort Acceptance |
| IT-48 | `sort::bogus` → exit 1, stderr names all three valid values | Sort Rejection |
| IT-49 | `prefer::bogus` → exit 1, stderr names valid values | Sort Rejection |
| IT-50 | `.usage.help` lists `sort`, `desc`, `prefer` params | Help Output |
| IT-51 | ~~`next::drain` default~~ → REMOVED (`next::` parameter removed) | Next Strategy |
| IT-52 | ~~`next::drain` explicit~~ → REMOVED (`next::` parameter removed) | Next Strategy |
| IT-53 | ~~`next::bogus` rejection~~ → REMOVED (`next::` parameter removed — see `it253`) | Next Rejection |
| IT-54 | ~~Footer shows both strategy lines~~ → REMOVED (single-strategy footer) | Next Footer |
| IT-55 | `cols::+sub` shows Sub column in output | Column Visibility |
| IT-56 | `cols::+bogus` exits 1 naming valid column IDs | Column Rejection |
| IT-58 | Per-column emoji appears somewhere in output — any of `🟢`/`🟡`/`🔴`, one live account, uncontrolled | Per-Column Emoji |
| IT-61 | `.usage.help` lists `cols` params (`next` removed) | Help Output |
| IT-62 | `touch::0` accepted; empty store exits 0 | Touch Param |
| IT-63 | `touch::1` with no-token accounts — errored accounts never touched | Touch Param |
| IT-64 | `.usage.help` lists `touch` param with default `1` | Help Output |
| IT-65 | ~~`sort::next` accepted~~ → REMOVED (now rejected — see `it251`) | Sort Rejection |
| IT-66 | `imodel::auto` accepted; empty store exits 0 | imodel Param |
| IT-67 | `imodel::bogus` → exit 1, stderr names all five valid values | imodel Param |
| IT-68 | `effort::auto` accepted; empty store exits 0 | effort Param |
| IT-69 | `effort::bogus` → exit 1, stderr names all five valid values | effort Param |
| IT-70 | `.usage.help` lists `imodel` and `effort` params with default `auto` | Help Output |
| IT-71 | `→ Next` column shows soonest upcoming event label + duration | Next Event Column |
| IT-72 | `format::json` new fields: `renewal_secs`, `renewal_is_estimate`, `next_event_type`, `next_event_secs` | JSON Schema |
| IT-74 | Owner column visible by default; `cols::-owner` hides it | Owner Column |
| IT-75 | `rotate::1 live::1` exits 1 with mutual exclusion message | Rotate Param |
| IT-76 | `rotate::1` — all accounts lack `accessToken` → exits 1 (not an absent-candidate scenario) | Rotate Param |
| IT-77 | `rotate::1 dry::1` previews target; no switch executed; exit 0 | Rotate Param |
| IT-78 | `rotate::1` — exits 0 (switched) or 1 (no eligible); `switched to` only checked on exit 0 | Rotate Param |
| IT-79 | `rotate::1 sort::renews` — exits 0 (switched) or 1 (no eligible); winner account not verified | Rotate Param |
| IT-80 | `rotate::1 force::1` — exit 0 unverified; only checks absence of `"ownership"` when exit is 1 | Rotate Param |
| IT-81 | `who::0` accepted; empty store exits 0 | Who Param |
| IT-82 | `who::2` rejected; exit 1; error mentions valid values `0` and `1` | Who Param |
| IT-83 | `.usage.help` lists `who` param with sessions table description | Help Output |
| IT-84 | `assignee::USER@MACHINE name::X` writes active marker on `.usage` (Feature 065) | Feature 065 — assignee mutation |
| IT-85 | `owner::0 name::X` clears owner field when G8 passes on `.usage` (Feature 064) | Feature 064 — owner mutation |
| IT-86 | `assign::1` REMOVED_TOGGLE exits 1 on `.usage` (Feature 064) | Feature 064 — REMOVED_TOGGLE |
| IT-87 | `unclaim::1` REMOVED_TOGGLE exits 1 on `.usage` (Feature 064) | Feature 064 — REMOVED_TOGGLE |

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
- Help Output: 9 tests (IT-32, IT-34, IT-38, IT-39, IT-50, IT-61, IT-64, IT-70, IT-83)
- Trace: 1 test (IT-35)
- Status Emoji: 3 tests (IT-40, IT-41, IT-42)
- Sort Acceptance: 2 tests (IT-44, IT-47)
- Sort Rejection: 5 tests (IT-45, IT-46, IT-48, IT-49, IT-65)
- Next Strategy: 2 tests (IT-51, IT-52)
- Next Rejection: 1 test (IT-53)
- Next Footer: 1 test (IT-54)
- Column Visibility: 1 test (IT-55)
- Column Rejection: 1 test (IT-56)
- Per-Column Emoji: 1 test (IT-58)
- Touch Param: 2 tests (IT-62, IT-63)
- imodel Param: 2 tests (IT-66, IT-67)
- effort Param: 2 tests (IT-68, IT-69)
- Next Event Column: 1 test (IT-71)
- Owner Column: 1 test (IT-74)
- Rotate Param: 6 tests (IT-75, IT-76, IT-77, IT-78, IT-79, IT-80)
- Who Param: 2 tests (IT-81, IT-82)
- Feature 064 — active mutation: 1 test (IT-84)
- Feature 064 — owner mutation: 1 test (IT-85)
- Feature 064 — REMOVED_TOGGLE: 2 tests (IT-86, IT-87)

**Total:** 94 spec entries (IT-43, IT-57, IT-59, IT-60, IT-73 removed — unit tests not observable via clp output); IT-65 added for `sort::next`; IT-66–IT-70 added by TSK-191 (`imodel::`/`effort::` params and `touch::` default `1`); IT-71–IT-72 added by Plan 012 (`→ Next` column and JSON new fields); IT-74 added by Feature 037 (Owner column default-visible in `.usage`); IT-75–IT-80 added by Feature 038 (`rotate::` parameter group); IT-81–IT-83 added by Plan 022 (`who::` parameter and sessions table); source functions it17–it33 map to spec IT-18–IT-34; it34/it35/it36 map to IT-35/IT-36/IT-37; it37 maps to IT-38; it38 maps to IT-39; IT-17 covered by `ft002_lim_it_http_401_shortens_to_auth_expired` in `usage_feature_test.rs` (live network test; kept in feature test file to avoid duplication with FT-02); it39–it52 covered by param spec docs `tests/docs/cli/param/19_refresh.md`–`23_trace.md` (param EC edge cases, not command spec)

---

### IT-1: Default invocation shows quota table with new column headers

- **Given:** At least one saved account with a valid token exists in the credential store.
- **When:** `clp .usage`
- **Then:** Stdout contains a table with "Quota" heading and rows showing columns: "5h Left", "5h Reset", "7d Left", "7d Reset", "Expires", "~Renews", "→ Next". Column order: quota columns (5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset) appear before billing-metadata columns (Expires, ~Renews, → Next). Exit 0.
- **Exit:** 0
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

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
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

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
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

---

### IT-6: Credential store unreadable exits 2

- **Given:** `HOME` is set but credential store directory cannot be read (permissions error).
- **When:** `clp .usage`
- **Then:** Error on stderr. Exit 2.
- **Exit:** 2
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

---

### IT-7: HOME unset exits 2

- **Given:** `HOME` environment variable is unset.
- **When:** `env -u HOME clp .usage`
- **Then:** Error on stderr. Exit 2.
- **Exit:** 2
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

---

### IT-8: Multiple accounts displayed in alphabetical order

- **Given:** Three saved accounts: `c@x.com`, `a@x.com`, `b@x.com`.
- **When:** `clp .usage`
- **Then:** Rows appear in order `a@x.com`, `b@x.com`, `c@x.com`. Exit 0.
- **Exit:** 0
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

---

### IT-9: Account with missing token file shows `—` with error reason

- **Given:** Credential store entry exists but the `.credentials.json` file for that account is missing.
- **When:** `clp .usage`
- **Then:** That account's row shows `—` for quota columns and a missing-token error reason in Status. Exit 0.
- **Exit:** 0
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

---

### IT-10: Account with expired token shows `EXPIRED` in Expires column

- **Given:** One saved account whose `expiresAt` in the credential file is a past timestamp (e.g., `PAST_MS`).
- **When:** `clp .usage`
- **Then:** That account's row shows `EXPIRED` in the Expires column. The quota columns show `—`. Exit 0.
- **Exit:** 0
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

---

### IT-11: Recommended account appears in footer `Next (<strategy>)` line with `·` delimiter; no `→` in table rows

- **Given:** Two accounts — one active with quota data, one non-active with valid token and quota data showing lower session usage than the active account.
- **When:** `clp .usage`
- **Then:** Stdout contains a `·`-delimited footer line matching `Next (renew) ·` and the non-active account name. No table data row contains a bare `→` marker in the flag column. Exit 0.
- **Exit:** 0
- **Live:** yes (requires real tokens for both accounts to return quota data)
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

---

### IT-12: Footer `Current` line shows `✓` account with `·`-delimited model and valid count

- **Given:** At least two accounts with valid tokens that return quota data.
- **When:** `clp .usage`
- **Then:** Stdout contains a `·`-delimited footer line matching `Current · <name> · <model> · N/N` identifying the `✓` account, followed by a `Next (renew) ·` line with the recommended account. Exit 0.
- **Exit:** 0
- **Live:** yes (requires ≥2 accounts with live quota headers)
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

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

### IT-17: HTTP error from usage API is never shown as verbose `HTTP transport error:` (exact short form not deterministic)

> `ft02_lim_it_http_401_shortens_to_auth_expired`'s own doc comment states the literal `(auth expired (401))` string is no longer guaranteed: `apply_refresh` now intercepts 401 responses and attempts an OAuth refresh first; when that refresh itself fails, the error becomes `"token refresh failed"` instead (cause-neutral label per BUG-539), and a 429 (rate-limited) response would show `"rate limited (429)"`. The durable invariant the test actually enforces is: no verbose `HTTP transport error: HTTP NNN` string appears. The fixture also uses a locally-**valid** `expiresAt` (`FAR_FUTURE_MS`), not a past/expired one — the account's `Expires` column would show a valid date, not `EXPIRED`. Assertions check only: exit 0, the account row (`invalid@test.com`) appears in stdout, and stdout never contains `"HTTP transport error:"`. None of `EXPIRED`, the `—` quota-column placeholders, or `(auth expired (401))` are asserted.

- **Given:** One saved account with a locally-**valid** (far-future) `expiresAt` but an `accessToken` the live usage API rejects (invalid token).
- **When:** `clp .usage`
- **Then:** Exit 0. The account row (`invalid@test.com`) appears in stdout. Stdout never contains the verbose `"HTTP transport error:"` string — the actual short-form message shown (e.g. `(auth expired (401))`, `(token refresh failed)`, or `(rate limited (429))`) depends on live API behavior at test time and is not asserted.
- **Exit:** 0
- **Live:** yes (requires network access; exact error text depends on live API response)
- **Fix:** BUG-152
- **Source fn:** `ft02_lim_it_http_401_shortens_to_auth_expired` (in `usage_feature_test.rs`)
- **Source:** [009_token_usage.md AC-03](../../../../docs/feature/009_token_usage.md)

---

### IT-18: `.usage format::table` exits 1 (`ArgumentTypeMismatch`)

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage format::table`
- **Then:** Exits 1. `format::table` is valid only for `.accounts`; all other commands reject it.
- **Exit:** 1
- **Source fn:** `it017_format_table_rejected`
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

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

> Test checks 5 things only: the `"error":` key, the absence of `"session_5h_left_pct"`, and the presence of `"is_current"`, `"is_active"`, `"expires_in_secs"`. It does not check `billing_type`, `has_max`, `renewal_secs`, `renewal_is_estimate`, `next_event_type`, `next_event_secs`, or the absence of `next_renewal_est` — none of these fields (null or otherwise) are asserted.

- **Given:** One account with no `accessToken` in the credential file (read_token returns Err).
- **When:** `clp .usage format::json`
- **Then:** Exits 0; JSON contains `"error":` key; does NOT contain `"session_5h_left_pct"`; does contain `"is_current"`, `"is_active"`, `"expires_in_secs"`.
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

### IT-35: `trace::1` with no-token account → stderr contains timestamped diagnostic lines

> Test never calls `stdout(&out)` at all — its own doc comment states it confirms trace output "without affecting exit code or stdout," meaning stdout is deliberately not inspected, not that it was checked and found unaffected.

- **Given:** One saved account whose credential file has no `accessToken` field.
- **When:** `clp .usage trace::1`
- **Then:** Exits 0; stderr contains a ` · ` separator and the account name (`trace-acct`). Stdout is not read or asserted on by this test.
- **Exit:** 0
- **Source fn:** `it034_trace_param_writes_to_stderr`
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

---

### IT-36: Empty store + `format::json` → output is `[]`

- **Given:** Credential store directory exists but contains no `*.credentials.json` files.
- **When:** `clp .usage format::json`
- **Then:** Exits 0; stdout (trimmed) equals `[]`; no text-format "no accounts configured" message.
- **Exit:** 0
- **Source fn:** `it035_empty_store_json_format`
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

---

### IT-37: Single failed account → no `Valid:` footer line emitted

> Test checks `!text.contains("Valid:")`, not `"Current ·"`. `src/usage/render.rs` emits `"Current · ... · valid_count/total"` only when an account is marked `is_current`; when none is (this fixture's case), rendering falls back to the legacy `"Valid: {valid_count} / {total}"` line — the string the test actually checks for absence of.

- **Given:** One saved account whose credential file has no `accessToken` (quota fetch fails; `valid_count = 0`).
- **When:** `clp .usage`
- **Then:** Exits 0; stdout does NOT contain `Valid:` (footer is suppressed when `valid_count < 2`).
- **Exit:** 0
- **Source fn:** `it036_no_footer_when_no_valid_accounts`
- **Source:** [command/006_usage.md — .usage](../../../../docs/cli/command/006_usage.md#command-9-usage)

---

### IT-38: `.usage.help` shows `refresh::` default as `1` (enabled)

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0; stdout contains `"1 = enabled, default"` (indicating `refresh::1` is the default); stdout does NOT contain `"0 = disabled, default"`.
- **Exit:** 0
- **Fix:** BUG-155
- **Source fn:** `it037_mre_bug155_refresh_defaults_to_1`
- **Source:** [017_token_refresh.md AC-23](../../../../docs/feature/017_token_refresh.md)

---

### IT-39: `.usage.help` refresh description mentions `429` and locally-expired case

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0; stdout contains `"429"` (the conditional 429+locally-expired refresh case is documented in the parameter description); stdout does NOT contain the old combined string `"401/403/429"`.
- **Exit:** 0
- **Fix:** BUG-156
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

> **Note:** IT-43 removed — unit test of `status_emoji()` not directly observable via clp output — behavior only verifiable at unit-test level. Unit test lives in `tests/cli/usage_test.rs` as `it151_status_emoji_boundary_precision`.

---

### IT-44: `sort::name` accepted with empty store → exit 0

- **Given:** Empty credential store.
- **When:** `clp .usage sort::name`
- **Then:** Exits 0 with "(no accounts configured)". No unknown-parameter error.
- **Exit:** 0
- **Source fn:** `it053_sort_name_accepted`
- **Source:** [feature/020_usage_sort_strategies.md AC-01](../../../../docs/feature/020_usage_sort_strategies.md)

---

### IT-45: ~~`sort::endurance` accepted~~ → REMOVED

> `sort::endurance` is now rejected (exits 1). Replaced by `it249_sort_endurance_rejected_exit_1`.

---

### IT-46: ~~`sort::drain` accepted~~ → REMOVED

> `sort::drain` is now rejected (exits 1). Replaced by `it250_sort_drain_rejected_exit_1`.

---

### IT-47: `sort::renew` accepted with empty store → exit 0

- **Given:** Empty credential store.
- **When:** `clp .usage sort::renew`
- **Then:** Exits 0 with "(no accounts configured)". No unknown-parameter error.
- **Exit:** 0
- **Source fn:** `it056_sort_renew_accepted`
- **Source:** [feature/020_usage_sort_strategies.md AC-04](../../../../docs/feature/020_usage_sort_strategies.md)

---

### IT-48: `sort::bogus` → exit 1, stderr names all three valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage sort::bogus`
- **Then:** Exits 1. Stderr contains each of the three valid values: `name`, `renew`, `renews`.
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

### IT-51: ~~`next::drain` default~~ → REMOVED

> `next::` parameter removed entirely. `sort::` now drives the footer recommendation.
> Replaced by single-strategy footer tests. See `it253_next_param_removed_exit_1`.

---

### IT-52: ~~`next::drain` explicit~~ → REMOVED

> See IT-51. `next::` parameter removed.

---

### IT-53: ~~`next::bogus` rejection~~ → REMOVED

> `next::` parameter removed. Any `next::` value exits 1 with "next:: parameter has been removed".
> Replaced by `it253_next_param_removed_exit_1`.

---

### IT-54: ~~Footer shows both strategy lines~~ → REMOVED

> Footer now shows a single recommendation line for the active `sort::` strategy.
> Covered by single-strategy footer tests in `020_usage_sort_strategies.md`.

---

### IT-55: `cols::+sub` shows Sub column in output

- **Given:** One saved account with valid credentials.
- **When:** `clp .usage cols::+sub`
- **Then:** Exits 0. Table header contains `Sub`.
- **Exit:** 0
- **Source fn:** `it081_cols_sub_shows_sub_column` (in `usage_sort_test.rs`)
- **Source:** [009_token_usage.md AC-22](../../../../docs/feature/009_token_usage.md)

---

### IT-56: `cols::+bogus` exits 1 naming valid column IDs

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage cols::+bogus`
- **Then:** Exits 1. Stderr names valid column IDs.
- **Exit:** 1
- **Source fn:** `it082_cols_unknown_id_exit_1` (in `usage_sort_test.rs`)
- **Source:** [009_token_usage.md AC-23](../../../../docs/feature/009_token_usage.md)

---

> **Note:** IT-57 removed — unit test of `status_emoji()` not directly observable via clp output — behavior only verifiable at unit-test level. Unit tests live in `tests/usage/format_tests.rs` as `test_status_emoji_and_both_ample_green` and `test_status_emoji_and_7d_low_yellow`.

---

### IT-58: Per-column emoji appears somewhere in usage output (uncontrolled — one live account)

> Test uses one live-token account, not two, and does not control its quota utilization — the assertion is `text.contains("🟢") || text.contains("🟡") || text.contains("🔴")`, satisfied by any of the three colors appearing anywhere in stdout, not specifically in the `5h Left` column, and not a specific color or percentage.

- **Given:** One live-token account (`acct-a@test.com`); its real quota utilization at test time is not controlled by the test.
- **When:** `clp .usage`
- **Then:** Exits 0; stdout contains at least one of `🟢`, `🟡`, `🔴` (which color, and where, is not asserted).
- **Exit:** 0
- **Live:** yes (requires a live OAuth token; `lim_it`)
- **Source fn:** `it105_lim_it_per_column_emoji_in_5h_left` (in `usage_touch_test.rs`)
- **Source:** [009_token_usage.md AC-21](../../../../docs/feature/009_token_usage.md)

---

> **Note:** IT-59 removed — unit test of `format_duration_secs()` not directly observable via clp output — behavior only verifiable at unit-test level. Unit test lives in `tests/cli_adapter_test.rs` module `format_duration` (D-11: `dur_90060s_shows_1d_1h_capped`).

---

> **Note:** IT-60 removed — unit test of `render_text()` not directly observable via clp output — behavior only verifiable at unit-test level. Unit test lives in `tests/usage/mod_tests.rs` as `test_three_tier_grouping_green_before_yellow_before_red`.

---

### IT-61: `.usage.help` lists `cols` param (and `next` column ID)

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains `"next"` (as a `cols::` column ID for `→ Next`) and `"cols"`. Note: `next::` parameter was removed; `next` here refers to the column name.
- **Exit:** 0
- **Source fn:** `it083_usage_help_shows_next_cols_params` (in `usage_sort_test.rs`)
- **Source:** [009_token_usage.md AC-09](../../../../docs/feature/009_token_usage.md)

---

### IT-62: `touch::0` accepted; empty store exits 0

- **Given:** Empty credential store; `touch::0` param passed (explicit default).
- **When:** `clp .usage touch::0`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned.
- **Exit:** 0
- **Source fn:** `it106_touch_0_accepted_empty_store_exits_0` (in `usage_touch_test.rs`)
- **Source:** [feature/024_session_touch.md AC-01](../../../../docs/feature/024_session_touch.md)

---

### IT-63: `touch::1` with no-token accounts — errored accounts never touched

- **Given:** One saved account whose credential file has no `accessToken` (quota fetch returns Err); `touch::1`.
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. Account row shows original error state. No subprocess spawned — touch trigger requires `result = Ok(...)`.
- **Exit:** 0
- **Source fn:** `it098_touch_1_errored_account_skipped` (in `usage_touch_test.rs`)
- **Source:** [feature/024_session_touch.md AC-04](../../../../docs/feature/024_session_touch.md)

---

### IT-64: `.usage.help` lists `touch` param with default `1`

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains `"touch"` with default value `1` (on).
- **Exit:** 0
- **Source fn:** `it101_usage_help_shows_touch_param` (in `usage_touch_test.rs`)
- **Source:** [feature/024_session_touch.md AC-10](../../../../docs/feature/024_session_touch.md)

---

### IT-65: ~~`sort::next` accepted~~ → REMOVED

> `sort::next` is now rejected (exits 1). Replaced by `it251_sort_next_rejected_exit_1`.

---

### IT-66: `imodel::auto` accepted; empty store exits 0

- **Given:** Empty credential store.
- **When:** `clp .usage imodel::auto`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. `auto` is the default; no subprocess spawned (no accounts).
- **Exit:** 0
- **Source fn:** `it122_imodel_auto_accepted_empty_store_exits_0` (in `usage_model_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../../docs/feature/026_subprocess_model_effort.md)

---

### IT-67: `imodel::bogus` → exit 1, stderr names all five valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage imodel::bogus`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `sonnet`, `opus`, `haiku`, `keep`.
- **Exit:** 1
- **Source fn:** `it123_imodel_bogus_exits_1` (in `usage_model_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-10](../../../../docs/feature/026_subprocess_model_effort.md)

---

### IT-68: `effort::auto` accepted; empty store exits 0

- **Given:** Empty credential store.
- **When:** `clp .usage effort::auto`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. `auto` is the default; no subprocess spawned (no accounts).
- **Exit:** 0
- **Source fn:** `it124_effort_auto_accepted_empty_store_exits_0` (in `usage_model_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-05](../../../../docs/feature/026_subprocess_model_effort.md)

---

### IT-69: `effort::bogus` → exit 1, stderr names all five valid values

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage effort::bogus`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `low`, `normal`, `high`, `max`.
- **Exit:** 1
- **Source fn:** `it125_effort_bogus_exits_1` (in `usage_model_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-11](../../../../docs/feature/026_subprocess_model_effort.md)

---

### IT-70: `.usage.help` lists `imodel` and `effort` params with default `auto`

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains `"imodel"` and `"effort"`, each showing default value `auto`.
- **Exit:** 0
- **Source fn:** `it126_usage_help_shows_imodel_effort_params` (in `usage_model_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-12](../../../../docs/feature/026_subprocess_model_effort.md)

---

### IT-71: `→ Next` column shows soonest upcoming strategic event label and duration (uncontrolled — either `+7d` or `$ren` accepted)

> Test uses a live account with uncontrolled quota state — it does not construct a specific `seven_day.resets_at`/`_renewal_at` fixture to force `+7d` as soonest. The assertion tolerates either label: `text.contains(" +7d") || text.contains(" $ren")`. Absence of `!tok`/`+5h` labels is not asserted at all.

- **Given:** One account with a live token; its real quota state (which strategic event is soonest) is not controlled by the test.
- **When:** `clp .usage`
- **Then:** Exits 0. The `→ Next` column header appears in the table header row. That account's `→ Next` cell contains either `" +7d"` or `" $ren"` (whichever event the live account's real quota state makes soonest), preceded by `in <duration>`.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it225_lim_it_it71_next_event_cell_shows_label_and_duration` (in `usage_lim_it_test_b.rs`)
- **Source:** [feature/009_token_usage.md AC-28](../../../../docs/feature/009_token_usage.md)

---

### IT-72: `format::json` contains renewal/next-event field NAMES; values not checked; `next_renewal_est` absent

> Test uses a live account with uncontrolled quota state — no specific `_renewal_at`/`five_hour.resets_at`/`seven_day.resets_at` fixture is constructed; the live account's real quota data governs the JSON values. Every positive assertion checks only that a field-name substring appears in the JSON text (e.g., `text.contains("renewal_secs")`) — none check the field's actual value. `renewal_is_estimate` could be `true` or `false`, `next_event_type` could be `"ren"` or another sigil, and `renewal_secs`/`next_event_secs` could be any number — the test passes regardless, as long as the key name string is present somewhere. Only the absence of `next_renewal_est` is a genuine (negative) check.

- **Given:** One account with a live token; its real quota state (renewal timing, 5h/7d resets) is not controlled by the test.
- **When:** `clp .usage format::json`
- **Then:** Exits 0. JSON output contains the field-name substrings `renewal_secs`, `renewal_is_estimate`, `next_event_type`, `next_event_secs` (actual values not checked), and does NOT contain `next_renewal_est`.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it222_lim_it_it72_json_new_renewal_fields` (in `usage_lim_it_test_b.rs`)
- **Source:** [feature/009_token_usage.md AC-29](../../../../docs/feature/009_token_usage.md)

---

> **Note:** IT-73 removed — unit test of `render_text()` / `render_tsv()` not directly observable via clp output — behavior only verifiable at unit-test level. Unit test lives in `tests/usage/render_tests_a.rs` as `mre_bug_220_renews_preserved_for_429_accounts`. Fix for BUG-220 (`~Renews` was overwritten by the 429 error reason in both render functions).

---

### IT-74: Owner column visible by default; `cols::-owner` hides it

- **Given:** Two accounts: `alice@acme.com` with owner `testuser@testmachine`; `bob@acme.com` with empty owner. Neither has a live token.
- **When (Case A):** `clp .usage`
- **Then (Case A):** Exit 0. Stdout contains `Owner` column header. Contains `testuser@testmachine`. Contains `—` (em dash U+2014) for bob's unowned slot.
- **When (Case B):** `clp .usage cols::-owner`
- **Then (Case B):** Exit 0. Stdout does NOT contain `Owner` column header.
- **Exit:** 0
- **Source fn:** `it248_owner_column_visible_by_default` (in `usage_solo_test.rs`)
- **Source:** [feature/037_accounts_usage_param_unification.md AC-19](../../../../docs/feature/037_accounts_usage_param_unification.md)

---

### IT-75: `rotate::1 live::1` exits 1 with mutual exclusion message

- **Given:** Any environment (empty credential store is sufficient).
- **When:** `clp .usage rotate::1 live::1`
- **Then:** Exits 1 before any account fetch. Stderr contains a message indicating `rotate::1` and `live::1` are mutually exclusive. No table is rendered.
- **Exit:** 1
- **Source fn:** `ft04_rotate_live_mutual_exclusion` (in `tests/cli/usage_rotate_test.rs`)
- **Source:** [feature/038_usage_strategy_rotate.md AC-04](../../../../docs/feature/038_usage_strategy_rotate.md)

---

### IT-76: `rotate::1` with all accounts lacking `accessToken` exits 1

> Not an absent-candidate scenario: the test's own doc comment states the real cause is "all accounts fail API fetch (no `accessToken`)" — `write_account`'s `credential_json()` helper never writes an `accessToken` field. Two accounts exist (`current@test.com`, active; `other@test.com`, not active); both lack a token, so both fail quota fetch and neither is eligible, regardless of ownership/currency. The assertion is also looser than previously documented: `combined.contains("eligible") || combined.contains("rotate")` — an OR of two single-word substrings, not the literal phrase `"no eligible account to rotate to"`. Table rendering and absence of a `switched to` line are not asserted at all.

- **Given:** Two accounts saved: `current@test.com` (active) and `other@test.com` (not active). Neither has an `accessToken` — both fail quota fetch.
- **When:** `clp .usage rotate::1`
- **Then:** Exits 1. Combined stdout+stderr contains `"eligible"` or `"rotate"`. Table rendering and absence of a `switched to` line are not checked by this test.
- **Exit:** 1
- **Source fn:** `ft03_no_eligible_account_exits_1` (in `tests/cli/usage_rotate_test.rs`)
- **Source:** [feature/038_usage_strategy_rotate.md AC-03](../../../../docs/feature/038_usage_strategy_rotate.md)

---

### IT-77: `rotate::1 dry::1` previews target; no switch executed; exit 0

- **Given:** Two accounts: `primary@acme.com` (current, active) and `secondary@acme.com` (owned, non-current, non-active, not h-exhausted, not expired, has quota). `sort::renew` (default) selects `secondary@acme.com` as the footer recommendation.
- **When:** `clp .usage rotate::1 dry::1`
- **Then:** Exits 0. Table is rendered; footer `Next:` line shows `secondary@acme.com`. Output ends with `[dry-run] would switch to 'secondary@acme.com'`. Credential store is NOT modified (credentials file unchanged).
- **Exit:** 0
- **Source fn:** `cc07_rotate_dry_offline_no_credential_change` (in `tests/cli/usage_rotate_test.rs`)
- **Source:** [feature/038_usage_strategy_rotate.md AC-02](../../../../docs/feature/038_usage_strategy_rotate.md)

---

### IT-78: `rotate::1` — exits 0 (switched) or 1 (no eligible); `switched to` only checked on exit 0

> Test's own comment: "Must exit 0 (switched) or 1 (no eligible — rate limited/both same token)." There is no unconditional `assert_exit` call — assertions run only inside `if out.status.code() == Some(0) { ... }`; on exit 1, nothing is checked and the test passes trivially. Even on exit 0, only `text.contains("switched to")` is checked — not the specific account name, not "ends with," not footer/table content, and not the on-disk active-marker file. Account names in the fixture are `active@test.com` (current) / `rotate_target@test.com` (target), not `primary@acme.com`/`secondary@acme.com`.

- **Given:** Two accounts sharing the same live token: `active@test.com` (current, active) and `rotate_target@test.com` (not active). Live test environment.
- **When:** `clp .usage rotate::1`
- **Then:** Exits 0 or 1, depending on live quota/rate-limit state. Only when exit 0 is observed: stdout contains `"switched to"` (the target account name, table/footer content, and the on-disk active marker are not checked either way).
- **Exit:** 0 or 1 (not deterministic)
- **Live:** yes
- **Source fn:** `ft01_lim_it_rotates_to_next_winner` (in `tests/cli/usage_rotate_test.rs`)
- **Source:** [feature/038_usage_strategy_rotate.md AC-01](../../../../docs/feature/038_usage_strategy_rotate.md)

---

### IT-79: `rotate::1 sort::renews` — exits 0 (switched) or 1 (no eligible); winner account not verified

> Test's own comment: "exit 0 = switched; exit 1 = no eligible account (fine — strategy just found none)." No unconditional `assert_exit`; the `if out.status.code() == Some(0)` branch only checks `text.contains("switched to")` — it never checks which account was selected, so the "soonest billing renewal" strategy outcome is not independently verified. Fixture is one current account (`active@test.com`) plus one non-current candidate (`candidate@test.com`) sharing the same live token — not "two eligible non-current accounts."

- **Given:** One current account (`active@test.com`, active) and one non-current candidate (`candidate@test.com`), sharing the same live token. Live test environment.
- **When:** `clp .usage rotate::1 sort::renews`
- **Then:** Exits 0 or 1, depending on live quota/rate-limit state. Only when exit 0 is observed: stdout contains `"switched to"` — which account was actually selected is not asserted.
- **Exit:** 0 or 1 (not deterministic)
- **Live:** yes
- **Source fn:** `ft07_lim_it_sort_renews` (in `tests/cli/usage_rotate_test.rs`)
- **Source:** [feature/038_usage_strategy_rotate.md AC-07](../../../../docs/feature/038_usage_strategy_rotate.md)

---

### IT-80: `rotate::1 force::1` — exit 0 unverified; only checks absence of `"ownership"` when exit is 1

> This test's structure is inverted relative to ft01/ft07: it checks `if out.status.code() == Some(1)` (not `Some(0)`), and even then only asserts `!combined.contains("ownership")`. The exit-0 (success) path has zero assertions — if the command exits 0, the test passes without checking anything (no `"switched to"`, no account name, no ownership-bypass confirmation). Fixture is two accounts, not three: `active@test.com` (active/current, implicitly owned) and `foreign@test.com` (not active, explicitly owned by `"other@remotemachine"` via `write_account_owner`) — not `owned@acme.com`/`foreign@acme.com`, and not a current+owned-non-current+foreign-non-owned 3-account scenario.

- **Given:** Two accounts sharing the same live token: `active@test.com` (active/current) and `foreign@test.com` (not active, explicitly owned by a different machine). Live test environment.
- **When:** `clp .usage rotate::1 force::1`
- **Then:** Exits 0 or 1 (not deterministic). Only when exit 1 is observed: combined stdout+stderr does NOT contain `"ownership"`. When exit 0 is observed, nothing is asserted at all.
- **Exit:** 0 or 1 (not deterministic; only the exit-1 path is checked, and only for absence of `"ownership"`)
- **Live:** yes
- **Source fn:** `ft06_lim_it_force_bypasses_g5` (in `tests/cli/usage_rotate_test.rs`)
- **Source:** [feature/038_usage_strategy_rotate.md AC-06](../../../../docs/feature/038_usage_strategy_rotate.md)

---

### IT-81: `who::0` accepted; empty store exits 0

- **Given:** Empty credential store (no accounts, no `_active_*` markers).
- **When:** `clp .usage who::0`
- **Then:** Exits 0 with `(no accounts configured)`. The `who::0` parameter is accepted without error.
- **Exit:** 0
- **Source:** [cli/param/061_who.md](../../../../docs/cli/param/061_who.md)

---

### IT-82: `who::2` rejected; exit 1; error mentions valid values

- **Given:** Any environment (empty credential store is sufficient).
- **When:** `clp .usage who::2`
- **Then:** Exits 1. Stderr contains error indicating `who::` must be `0` or `1`.
- **Exit:** 1
- **Source:** [cli/param/061_who.md](../../../../docs/cli/param/061_who.md)

---

### IT-83: `.usage.help` lists `who` param with sessions table description

- **Given:** Any environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains `who` in the parameter listing. Description mentions sessions table visibility.
- **Exit:** 0
- **Source:** [cli/param/061_who.md](../../../../docs/cli/param/061_who.md)

---

### IT-84: `assignee::USER@MACHINE name::X` writes active marker on `.usage` (Feature 065)

- **Given:** `alice@acme.com` exists in credential store. Record mtime of `alice.json`, `alice.credentials.json`.
- **When:** `clp .usage assignee::testuser@testmachine name::alice@acme.com`
- **Then:** Exit 0. `_active_testmachine_testuser` in credential store contains `alice@acme.com`. mtime of `alice.json` and `alice.credentials.json` unchanged. Same behavior as `.accounts` IT-43.
- **Exit:** 0
- **Source:** [feature/065_assignee_param_redesign.md AC-01](../../../../docs/feature/065_assignee_param_redesign.md)

---

### IT-85: `owner::0 name::X` clears owner field when G8 passes on `.usage` (Feature 064)

- **Given:** `alice@acme.com` with `alice.json` containing `"owner": "testuser@testmachine"`. Current identity = `testuser@testmachine`.
- **When:** `clp .usage owner::0 name::alice@acme.com`
- **Then:** Exit 0. `alice.json` contains `"owner": ""`. `alice.credentials.json` mtime unchanged. Same behavior as `.accounts` IT-44.
- **Exit:** 0
- **Source:** [feature/064_active_marker_and_owner_redesign.md AC-08](../../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### IT-86: `assign::1` REMOVED_TOGGLE exits 1 on `.usage` (Feature 064)

- **Given:** Any environment.
- **When:** `clp .usage assign::1 name::alice@acme.com`
- **Then:** Exit 1. Migration message: "REMOVED — use `assignee::USER@MACHINE name::X` instead". No files modified.
- **Exit:** 1
- **Source:** [feature/064_active_marker_and_owner_redesign.md AC-05](../../../../docs/feature/064_active_marker_and_owner_redesign.md)

---

### IT-87: `unclaim::1` REMOVED_TOGGLE exits 1 on `.usage` (Feature 064)

- **Given:** Any environment.
- **When:** `clp .usage unclaim::1 name::alice@acme.com`
- **Then:** Exit 1. Migration message: "REMOVED — use `owner::0 name::X` instead (or `owner::0` alone to batch-clear)". No files modified.
- **Exit:** 1
- **Source:** [feature/064_active_marker_and_owner_redesign.md AC-07](../../../../docs/feature/064_active_marker_and_owner_redesign.md)

# Test: Feature 009 — All-Accounts Live Quota Reporting

Feature behavioral requirement test cases for `docs/feature/009_token_usage.md` (FR-14). Each FT case maps to one acceptance criterion. Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/09_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Command IT |
|----|-----------|-----|------------|
| FT-01 | Error reason shortened — missing accessToken | AC-03 | IT-3, IT-9 |
| FT-02 | HTTP 401 shortens to `(auth expired (401))` | AC-03 | IT-17 |
| FT-03 | All saved accounts fetched, not only `_active` | AC-01 | IT-1, IT-8 |
| FT-04 | Live token match governs `✓`, not `_active` marker | AC-02 | IT-2, IT-13 |
| FT-05 | Missing credential store → exit 2 | AC-06 | IT-6, IT-7 |
| ~~FT-06~~ | ~~Endurance strategy tiebreaker: expiry breaks 5h Left tie~~ (REMOVED — endurance strategy deleted) | ~~AC-09~~ | ~~IT-11~~ |
| FT-07 | Status emoji `🟢`/`🟡`/`🔴` correct per account state (4 variants incl. both-exhausted → 🟡) | AC-18 | IT-40, IT-41 |
| FT-08 | Strict boundary: 5h at 15%, 7d at 5% — at boundary → `🟡`; above → `🟢` | AC-19 | — |
| FT-09 | `format::json` output contains no status emoji | AC-20 | IT-42 |
| FT-10 | After token refresh, `~Renews` shows actual date (not `?`) | BUG-171 | — |
| FT-11 | `5h Left` / `7d Left` values embed per-column emoji prefix | AC-21 | — |
| FT-12 | `Sub` / `7d Son Reset` hidden by default; `cols::+` reveals them | AC-22 | — |
| FT-13 | Invalid `cols::` column ID exits 1 with error | AC-23 | — |
| FT-14 | Four-group outer ordering: 🟢 before 🟡 (G2+G3) before 🔴 independent of sort | AC-24 | — |
| FT-15 | `format_duration_secs` capped to 2 significant time units | AC-25 | — |
| FT-16 | Within 🟡 group: h-exhausted (`5h Left ≤ 15%`) before weekly-exhausted | AC-26 | — |
| FT-17 | `~Renews` shows exact `in Xh Ym` (no `~`) when `_renewal_at` set | AC-27 | — |
| FT-18 | `→ Next` column shows soonest upcoming event label and duration | AC-28 | — |
| FT-19 | JSON includes `renewal_secs`, `renewal_is_estimate`, `next_event_type`, `next_event_secs` | AC-29 | — |
| FT-20 | `~Renews` shows renewal date (not error reason) for 429 accounts when `OauthAccountData` is available | AC-03 | — |
| FT-21 | `@` in flag column for accounts active on another machine's `_active_*` marker | AC-30 | — |
| FT-22 | Cancelled subscription (`billing_type == "none"`) shows `(no subscription)` in last quota column | AC-03, AC-31 | — |
| FT-23 | `~Renews` shows `"—"` for cancelled subscription accounts (`billing_type == "none"`) | AC-27, AC-31 | — |
| FT-24 | Trace result line emitted AFTER Class A billing_type override — trace matches stored result | AC-31 | — |
| FT-25 | `.usage` applies model override for current account when `7d(Son) < 15%` | AC-32 | — |
| FT-26 | `format::json` output includes `"is_owned"` bool per account object | AC-05 | — |
| FT-27 | `.usage` model override writes `"sonnet"` conservatively when `seven_day_sonnet` is absent (`None`) — absent tier is unknown, not exhausted (BUG-300 + BUG-311) | AC-32 | — |
| FT-28 | Footer `Current` line identifies `✓` account with model and valid count; `Next` line shows recommendation with model and metric; both use `·` delimiter and aligned columns | AC-10 | — |
| FT-29 | Footer `Current` line shows `model/effort` combined when effort present; `model` only when effort absent | AC-10 | — |
| FT-30 | Sessions table shown after footer when >1 `_active_*` marker exists; each marker rendered as `{user}@{host}` + account; own session has `✓` | AC-33 | — |
| FT-31 | Sessions table hidden when ≤1 `_active_*` marker (single-session default) | AC-33 | — |
| FT-32 | `who::0` suppresses sessions table; `who::1` forces it on | AC-34 | — |
| FT-33 | Cancelled account (`billing_type="none"`) gets `🔴` in `●` column | AC-18 | — |
| FT-34 | Both-exhausted account (5h ≤ 15% AND 7d ≤ 5%) gets `🟡`, not `🔴` (BUG-321) | AC-18, AC-26 | — |
| FT-35 | Both-exhausted (🟡) sorts in G3 weekly-exhausted group, before G4 Dead (🔴) | AC-24, AC-26 | — |
| — | Table output rendered by `data_fmt` crate (`use data_fmt::…` in `render.rs`) | AC-04 | Structural (code review — all render paths use `data_fmt`) |
| — | `Expires` column: `"in Xh Ym"` / `"EXPIRED"` from `compute_expires_cell()` | AC-07 | IT-003, IT-010 (command-level coverage) |
| — | `5h Left`, `7d Left`, `7d(Son)`, `5h Reset`, `7d Reset` from `OauthUsageData` | AC-08 | Indirect — FT-07/FT-08/FT-11/FT-14/FT-15/FT-16 all depend on these columns |
| — | Footer appended when ≥2 valid accounts; absent when 0 or 1 | AC-10 | Live-only (IT-012); IT-036 covers no-footer offline |
| — | Synthetic row for unsaved live credentials | AC-11 | Cross-feature: Feature 016 FT-09 |
| — | Credentials unreadable → no `✓` on any row | AC-12 | Cross-feature: Feature 016 FT-07 |
| — | `*` marks active-marker divergence from current | AC-13 | Cross-feature: Feature 016 FT-06 |
| — | Current = active → only `✓`; no `*` emitted | AC-14 | Cross-feature: Feature 016 FT-05 (implicit) |
| — | Credentials unreadable → no `✓`; `*` still emitted for active | AC-15 | Cross-feature: Feature 016 FT-07 |
| — | `format::json` uses `is_current` + `is_active`; no `active` field | AC-16 | Cross-feature: Feature 016 FT-08 |
| — | `7d(Son)` populated when `seven_day_sonnet` is `Some`; `—` when `None` | AC-17 | IT-004 (JSON field); structural (table `—` default) |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Missing accessToken shows short error, not verbose string | AC-03 | Error Shortening |
| FT-02 | HTTP 401 from usage API shortens to `(auth expired (401))` | AC-03 | Error Shortening |
| FT-03 | Both accounts appear regardless of `_active` marker | AC-01 | Complete Fetch |
| FT-04 | `✓` follows live token match, not `_active` marker | AC-02 | Live Detection |
| FT-05 | Unreadable credential store exits 2 | AC-06 | Error Handling |
| ~~FT-06~~ | ~~Tiebreaker: higher expiry wins when 5h Left tied~~ (REMOVED) | ~~AC-09~~ | ~~Recommendation~~ |
| FT-07 | Status emoji correct for each of three account states | AC-18 | Status Emoji |
| FT-08 | Exhaustion boundary is strict: 5h at 15%, 7d at 5% | AC-19 | Status Emoji |
| FT-09 | JSON output is emoji-free | AC-20 | Status Emoji |
| FT-10 | ~Renews shows actual date after refresh (BUG-171) | BUG-171 | Account After Refresh |
| FT-11 | Per-column emoji in 5h Left and 7d Left column values | AC-21 | Per-Column Emoji |
| FT-12 | Sub and 7d Son Reset columns hidden by default; shown via cols::+ | AC-22 | Column Visibility |
| FT-13 | Invalid cols:: column ID exits 1 | AC-23 | Column Modifiers |
| FT-14 | Four-group outer ordering preserved regardless of sort strategy | AC-24 | Group Ordering |
| FT-15 | format_duration_secs shows at most 2 time components | AC-25 | Duration Format |
| FT-16 | h-exhausted (`5h Left ≤ 15%`) 🟡 before weekly-exhausted 🟡 regardless of sort | AC-26 | Yellow Sub-Grouping |
| FT-17 | `~Renews` exact `in Xh Ym` (no `~`) when `_renewal_at` is set | AC-27 | `~Renews` Format |
| FT-18 | `→ Next` column shows soonest event label and duration | AC-28 | `→ Next` Column |
| FT-19 | JSON `renewal_secs`, `renewal_is_estimate`, `next_event_type`, `next_event_secs` | AC-29 | JSON Fields |
| FT-20 | `~Renews` shows billing renewal date (not error reason) for 429 accounts with valid `OauthAccountData` | AC-03 | `~Renews` Error Preservation |
| FT-21 | `@` in flag column when account is active on another machine's `_active_*` marker | AC-30 | Occupied Elsewhere |
| FT-22 | Cancelled subscription (`billing_type == "none"`) shows `(no subscription)` in last quota column | AC-03, AC-31 | Subscription State |
| FT-23 | `~Renews` shows `"—"` for cancelled subscription accounts (`billing_type == "none"`) | AC-27, AC-31 | Subscription State |
| FT-24 | Trace result line emitted AFTER Class A billing_type override — trace matches stored result | AC-31 | Trace Ordering |
| FT-25 | `.usage` applies model override for current account when `7d(Son) < 15%` | AC-32 | Model Override |
| FT-26 | `format::json` includes `"is_owned": bool` per account object | AC-05 | JSON Fields |
| FT-27 | `.usage` model override writes `"sonnet"` conservatively when `seven_day_sonnet = None` (BUG-300 + BUG-311) | AC-32 | Model Override |
| FT-28 | Footer `Current` + `Next` lines with `·` delimiter and column alignment | AC-10 | Footer |
| FT-29 | Footer `Current` line: `model/effort` when effort present; `model` only when absent | AC-10 | Footer Session/Effort |
| FT-30 | Sessions table shown when >1 marker; own session has `✓` | AC-33 | Sessions Table |
| FT-31 | Sessions table hidden when ≤1 marker | AC-33 | Sessions Table |
| FT-32 | `who::0` suppresses sessions table; `who::1` forces it on | AC-34 | Sessions Table |
| FT-33 | Cancelled account (`billing_type="none"`) gets `🔴` in `●` column regardless of quota values | AC-18 | Status Emoji |
| FT-34 | Both-exhausted account (5h ≤ 15% AND 7d ≤ 5%) gets `🟡` in `●` column, not `🔴` (BUG-321) | AC-18, AC-26 | Status Emoji |
| FT-35 | Both-exhausted (🟡) sorts in G3 weekly-exhausted group, before G4 Dead (🔴) | AC-24, AC-26 | Sort Order |

**Total:** 35 FT cases

---

### FT-01: Missing accessToken shows short error, not verbose string

- **Given:** One saved account whose credential file exists but has no `accessToken` field.
- **When:** `clp .usage`
- **Then:** The account's row appears in the table; the last visible column shows a short error reason in parentheses (e.g., `(missing accessToken)`); the string does NOT begin with `HTTP transport error:`; all other accounts (none here) are still processed. Exit 0.
- **Exit:** 0
- **Source fn:** `ft01_missing_access_token_short_error`
- **Source:** [009_token_usage.md AC-03](../../../docs/feature/009_token_usage.md)

---

### FT-02: HTTP 401 from usage API shortens to `(auth expired (401))`

- **Given:** One saved account whose `expiresAt` in the credential file is a past timestamp (PAST_MS); the usage API rejects the account's `accessToken` with HTTP 401.
- **When:** `clp .usage`
- **Then:** That account's row shows `EXPIRED` in the Expires column; the 7d Reset column shows `(auth expired (401))` — NOT the verbose string `(HTTP transport error: HTTP 401)`. Exit 0.
- **Exit:** 0
- **Source fn:** `ft02_lim_it_http_401_shortens_to_auth_expired`
- **Note:** Fix for BUG-152; implemented by TSK-153 (`shorten_error` HTTP 401 branch).
- **Source:** [009_token_usage.md AC-03](../../../docs/feature/009_token_usage.md)

---

### FT-03: Both accounts appear regardless of `_active` marker

- **Given:** Two saved accounts `alice@a.com` and `bob@a.com`; neither is stored as `_active`; both credential files exist.
- **When:** `clp .usage`
- **Then:** Stdout contains both `alice@a.com` and `bob@a.com`. Exit 0.
- **Exit:** 0
- **Source fn:** `ft03_both_accounts_appear_regardless_of_active`
- **Source:** [009_token_usage.md AC-01](../../../docs/feature/009_token_usage.md)

---

### FT-04: `✓` follows live token match, not `_active` marker

- **Given:** Two saved accounts: `alice@a.com` (stored as `_active`) and `work@a.com`. The live `~/.claude/.credentials.json` has an `accessToken` matching `work@a.com`'s stored token.
- **When:** `clp .usage`
- **Then:** A line in stdout contains `✓` and `work@a.com`; no line contains `✓` and `alice@a.com`. Exit 0.
- **Exit:** 0
- **Source fn:** `ft04_check_mark_follows_live_token_not_active`
- **Source:** [009_token_usage.md AC-02](../../../docs/feature/009_token_usage.md)

---

### FT-05: Unreadable credential store exits 2

- **Given:** `HOME` is set to a directory that exists but whose `~/.persistent/claude/credential/` path is chmod 000 (unreadable).
- **When:** `clp .usage`
- **Then:** Exits 2; stderr contains a non-empty error message.
- **Exit:** 2
- **Source fn:** `ft05_unreadable_credential_store_exits_2`
- **Source:** [009_token_usage.md AC-06](../../../docs/feature/009_token_usage.md)

---

### FT-06: ~~Tiebreaker — higher expiry wins when `5h Left` is tied (endurance strategy)~~ REMOVED

> Test `test_ft06_009_endurance_tiebreaker_higher_expiry_wins` removed — endurance strategy deleted (Feature 037/038).
> Tiebreaker behavior for remaining strategies is covered by `sort::renew` tests in `020_usage_sort_strategies.md`.

---

### FT-07: Status emoji correct for each of four account states

- **Given:** Unit test. Four `AccountQuota` variants:
  - Variant A: `result = Err("missing accessToken".to_string())` → expected `🔴` (dead: error)
  - Variant B: `result = Ok(data)` where `five_hour.utilization = 10.0` (90% left), `seven_day.utilization = 10.0` (90% left) → expected `🟢`
  - Variant C: `result = Ok(data)` where `five_hour.utilization = 97.0` (3% left), `seven_day.utilization = 10.0` (90% left) → expected `🟡` (h-exhausted only)
  - Variant D: `result = Ok(data)` where `five_hour.utilization = 97.0` (3% left), `seven_day.utilization = 97.0` (3% left) → expected `🟡` (both-exhausted → G3 weekly-exhausted; recoverable; Fix BUG-321)
- **When:** `status_emoji(&aq)` called for each variant.
- **Then:** Returns `"🔴"` for A, `"🟢"` for B, `"🟡"` for C, `"🟡"` for D.
- **Exit:** n/a (unit test)
- **Note:** Variant D was previously `"🔴"` (BUG-319 fix). BUG-321 reverses this — both-exhausted is recoverable and must show 🟡, not 🔴. Dead (🔴) is only for error or cancelled.
- **Source fn:** `test_status_emoji_red`, `test_status_emoji_green`, `test_status_emoji_yellow`, `mre_bug321_both_exhausted_status_emoji_is_yellow`
- **Source:** [009_token_usage.md AC-18](../../../docs/feature/009_token_usage.md)

---

### FT-08: Exhaustion boundary is strict — 5h at 15%, 7d at 5%

- **Given:** Unit test. Three `AccountQuota` variants:
  - Variant A: `five_hour.utilization = 85.0` (15.0% left), `seven_day.utilization = 50.0` (50% left) → expected `🟡` (5h at boundary)
  - Variant B: `five_hour.utilization = 84.9` (15.1% left), `seven_day.utilization = 50.0` (50% left) → expected `🟢` (both above threshold)
  - Variant C: `five_hour.utilization = 50.0` (50% left), `seven_day.utilization = 95.0` (5.0% left) → expected `🟡` (7d at boundary)
- **When:** `status_emoji(&aq)` for each.
- **Then:** A returns `"🟡"`; B returns `"🟢"`; C returns `"🟡"`. The 5h boundary is `left > 15.0`; the 7d boundary is `left > 5.0` (both strict greater-than).
- **Exit:** n/a (unit test)
- **Source fn:** `test_status_emoji_boundary`
- **Source:** [009_token_usage.md AC-19](../../../docs/feature/009_token_usage.md)

---

### FT-09: `format::json` output is emoji-free

- **Given:** One saved account whose credential file has no `accessToken` field.
- **When:** `clp .usage format::json`
- **Then:** Exits 0. The output string does NOT contain `🔴`, `🟡`, or `🟢`. The JSON array is present and valid.
- **Exit:** 0
- **Source fn:** `test_status_emoji_absent_in_json`
- **Source:** [009_token_usage.md AC-20](../../../docs/feature/009_token_usage.md)

---

### FT-10: After token refresh, `~Renews` shows actual billing date (not `?`)

- **Given:** One saved account whose OAuth token is expired; `OauthAccountData` was populated on the initial fetch (so `~Renews` showed a date before expiry). Token expires → `apply_refresh()` succeeds → quota re-fetched.
- **When:** `clp .usage refresh::1`
- **Then:** The `~Renews` column for the refreshed account shows a concrete date (e.g., `"Jun  5"`) — NOT `"?"`. `aq.account` is re-populated by the BUG-171 fix (`fetch_oauth_account()` called inside `apply_refresh()` after quota re-fetch).
- **Exit:** 0
- **Live:** yes (lim_it — requires expired token + live account with billing data)
- **Note:** Fix(BUG-171): `apply_refresh()` previously left `aq.account` stale after refresh; `~Renews` reverted to `?` even though the account had valid billing data.
- **Source fn:** `mre_bug_171_account_populated_after_refresh` (MRE test in `tests/cli/usage_test.rs`)
- **Source:** [009_token_usage.md](../../../docs/feature/009_token_usage.md)

---

### FT-11: Per-column emoji in `5h Left` and `7d Left` column values

- **Given:** Unit test. Per-column emoji uses dimension-specific thresholds: `5h Left` at 15%, `7d Left` at 5%.
  - 5h dimension: `86.0` (> 15% → `🟢`), `12.0` (≤ 15% → `🟡`), boundary `15.0` (exactly 15% → `🟡`)
  - 7d dimension: `65.0` (> 5% → `🟢`), `3.0` (≤ 5% → `🟡`), boundary `5.0` (exactly 5% → `🟡`)
- **When:** Per-column emoji formatting applied to each value with its dimension's threshold.
- **Then:** Values above threshold produce `🟢` prefix; values at or below produce `🟡` prefix. Each dimension uses its own threshold independently.
- **Exit:** n/a (unit test — string return assertion)
- **Source fn:** `test_ft11_009_per_column_emoji_prefix_three_cases` (in `tests/usage/format_tests.rs`)
- **Source:** [009_token_usage.md AC-21](../../../docs/feature/009_token_usage.md)

---

### FT-12: `Sub` and `7d Son Reset` columns hidden by default; `cols::+` reveals them

- **Given:** One saved account with an expired token (🔴 state; credential file present so the table header renders).
- **When:** `clp .usage` (no `cols::` param) and `clp .usage cols::+sub` and `clp .usage cols::+7d_son_reset`
- **Then:**
  - Default: stdout does NOT contain `Sub` or `7d Son Reset` in the table header. Exit 0.
  - `cols::+sub`: stdout contains `Sub` in the table header. Exit 0.
  - `cols::+7d_son_reset`: stdout contains `7d Son Reset` in the table header. Exit 0.
- **Exit:** 0
- **Source fn:** `it117_ft12_cols_plus_reveals_sub_and_7d_son_reset_columns` (in `tests/cli/usage_test.rs`)
- **Source:** [009_token_usage.md AC-22](../../../docs/feature/009_token_usage.md)

---

### FT-13: Invalid `cols::` column ID exits 1

- **Given:** Any credential setup (param validation occurs before credential reads).
- **When:** `clp .usage cols::+not_a_real_column`
- **Then:** Exit 1. Stderr contains an error message naming valid column IDs (e.g., `sub`, `7d_son_reset`).
- **Exit:** 1
- **Source fn:** `it082_cols_unknown_id_exit_1` (in `tests/cli/usage_test.rs`)
- **Source:** [009_token_usage.md AC-23](../../../docs/feature/009_token_usage.md)

---

### FT-14: Four-group outer ordering: `🟢` before `🟡` (G2+G3) before `🔴` regardless of sort strategy

- **Given:** Unit test. Three `AccountQuota` entries (alphabetical order: alice → bob → carol):
  - `alice@x.com`: `result = Err(...)` → 🔴 Dead group (G4)
  - `bob@x.com`: `result = Ok(data)` where `five_hour.utilization = 97.0` (3% left) → 🟡 group (G2 h-exhausted)
  - `carol@x.com`: `result = Ok(data)` where `five_hour.utilization = 10.0` (90% left) → 🟢 group (G1)
- **When:** Four-group status partition applied; alphabetical sort within each group.
- **Then:** Output order is `carol@x.com` (🟢) → `bob@x.com` (🟡) → `alice@x.com` (🔴). Outer group ordering 🟢 → 🟡 → 🔴 is preserved regardless of alphabetical order. (Within-🟡 sub-grouping — h-exhausted (G2) before weekly-exhausted (G3, including both-exhausted) — is tested by FT-16.)
- **Exit:** n/a (unit test — order assertion on sorted list)
- **Source fn:** `test_three_tier_grouping_green_before_yellow_before_red` (in `src/usage/mod.rs`)
- **Source:** [009_token_usage.md AC-24](../../../docs/feature/009_token_usage.md)

---

### FT-15: `format_duration_secs` capped to 2 significant time units

- **Given:** Unit test. Three input durations:
  - `90120` seconds (1 day + 1 hour + 2 minutes)
  - `11970` seconds (3 hours + 19 minutes + 30 seconds)
  - `1380` seconds (23 minutes)
- **When:** `format_duration_secs(n)` for each input.
- **Then:** Returns `"1d 1h"` (minutes dropped; 2 units shown), `"3h 19m"` (seconds dropped; 2 units shown), `"23m"` (1 unit — within the cap). No input produces a 3-component string.
- **Exit:** n/a (unit test — string return assertion)
- **Source fn:** `dur_90060s_shows_1d_1h_capped` (in `tests/cli_adapter_test.rs` — module `format_duration`, D-11: 2-unit cap drops minutes)
- **Source:** [009_token_usage.md AC-25](../../../docs/feature/009_token_usage.md)

---

### FT-16: h-exhausted 🟡 before weekly-exhausted 🟡 regardless of sort

- **Given:** Unit test. Three `AccountQuota` structs all in 🟡 group (plus one 🟢 as anchor). Input order: alphabetical.
  - `a@x.com`: `five_hour.utilization=10.0` (90% left), `seven_day.utilization=98.0` (2% left) → 🟡 group, **weekly-exhausted** sub-group
  - `b@x.com`: `five_hour.utilization=99.0` (1% left), `seven_day.utilization=30.0` (70% left) → 🟡 group, **h-exhausted** sub-group
  - `c@x.com`: `five_hour.utilization=97.0` (3% left), `seven_day.utilization=50.0` (50% left) → 🟡 group, **h-exhausted** sub-group (5h ≤ 15%)
  - `d@x.com`: `five_hour.utilization=10.0` (90% left), `seven_day.utilization=10.0` (90% left) → 🟢 group
  - Alpha sort would produce: a → b → c → d. Four-group partition would place d (🟢) first, then a, b, c (all 🟡), then any 🔴 Dead.
- **When:** `render_text(&accounts, SortStrategy::Name, None, PreferStrategy::Any, &ColsVisibility::default_set(), None, None)`
- **Then:** Output row order is: `d@x.com` (🟢), then among 🟡 — `b@x.com` and `c@x.com` (h-exhausted, in alpha order), then `a@x.com` (weekly-exhausted). `a@x.com` must appear AFTER both `b@x.com` and `c@x.com` despite being alpha-first.
- **Edge case:** An account with both `5h Left ≤ 15%` AND `7d Left ≤ 5%` falls in **G3 weekly-exhausted** (🟡), NOT G4 Dead (🔴) (Fix BUG-321). The 7d constraint is binding — when 7d resets, 5h will have long since reset too. Both-exhausted and weekly-exhausted have identical recovery behavior. FT-35 tests this boundary.
- **Exit:** n/a (unit test — position assertion via `output.find()`)
- **Source fn:** `test_ft16_009_yellow_tier_session_before_weekly` (in `src/usage/mod.rs`)
- **Source:** [009_token_usage.md AC-26](../../../docs/feature/009_token_usage.md)

---

### FT-17: `~Renews` shows exact `in Xh Ym` (no `~`) when `_renewal_at` is set

- **Given (unit test):** `renews_label(renewal_at_opt, org_created_at_opt, now_secs)` with `renewal_at_opt = Some("2026-06-29T21:00:00Z")` and `now_secs` set such that the timestamp is 3h47m in the future.
- **When:** `renews_label()` called with the above inputs.
- **Then:** Returns `"in 3h 47m"` — no `~` prefix, exact duration format.
- **Exit:** n/a (unit test)
- **Source fn:** `rl_exact_from_renewal_at`, `rl_estimate_from_org_created_at`, `rl_auto_advance_past_renewal_at`, `rl_absent_returns_question` (in `tests/usage/format_tests.rs`)
- **Source:** [009_token_usage.md AC-27](../../../docs/feature/009_token_usage.md)

---

### FT-18: `→ Next` column shows soonest strategic event label and duration

- **Given (unit test):** `next_event_label(seven_day_resets_secs, renewal_secs, renewal_is_estimate)` — only `+7d` and `$ren` are candidates; `!tok` (token expiry) and `+5h` (5h reset) are not candidates since they are already shown in `Expires` and `5h Reset` columns.
- **When:** `next_event_label()` called with `seven_day_resets_secs = Some(7200)` (2h), `renewal_secs = None`.
- **Then:** Returns `"in 2h +7d"` — weekly reset is soonest strategic event.
- **Exit:** n/a (unit test)
- **Source fn:** `ne_tok_excluded_after_tsk228`, `ne_7d_soonest`, `ne_renewal_soonest_exact`, `ne_renewal_soonest_estimate`, `ne_all_none_returns_dash` (in `tests/usage/format_tests.rs`)
- **Source:** [009_token_usage.md AC-28](../../../docs/feature/009_token_usage.md)

---

### FT-19: JSON includes `renewal_secs`, `renewal_is_estimate`, `next_event_type`, `next_event_secs`

- **Given:** One saved account with `_renewal_at` set to a future timestamp.
- **When:** `clp .usage format::json`
- **Then:** Exits 0. The JSON object for that account contains: `renewal_secs` (u64 integer), `renewal_is_estimate: false`, `next_event_type` (string), `next_event_secs` (u64 integer). No `next_renewal_est` field present (deprecated field removed).
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it147_json_renewal_secs_present`, `it153_json_renewal_fields_with_renewal_at` (in `tests/cli/usage_test.rs`)
- **Source:** [009_token_usage.md AC-29](../../../docs/feature/009_token_usage.md)

---

### FT-20: `~Renews` shows billing renewal date (not error reason) for 429 accounts with valid `OauthAccountData`

- **Given (unit test):** An `AccountQuota` where:
  - `result = Err("rate limited (429)")` (quota API failed with 429)
  - `account = Some(OauthAccountData { org_created_at: <ISO date string for a known billing anchor>, ... })` (account data fetched independently — unaffected by 429 on usage API)
  - `now_secs` fixed such that `next_billing_label(&a.org_created_at, now_secs)` produces a known date string (e.g., `Jun  6`)
  - Default `ColsVisibility` (host and role OFF, renews ON)
- **When:** The `AccountQuota` is rendered via both `render_text()` and `render_tsv()`.
- **Then:**
  - In the `render_text()` output: the `~Renews` cell contains the expected renewal date string (e.g., `Jun  6`); the error reason `(rate limited (429))` appears in at least one quota-data column (`5h Left` through `7d Reset`) and does NOT appear in the `~Renews` cell.
  - In the `render_tsv()` output: the `~Renews` field contains the expected renewal date string; the TSV renews cell is NOT `(rate limited (429))`.
- **Exit:** n/a (unit test)
- **Note:** Fix for BUG-220. The defect had `render_text()` using `last_mut()` positional overwrite (hitting `~Renews` as the last non-host/role column) and `render_tsv()` explicitly pushing `error_str` for the renews cell. Both renderers must preserve `renews_str` (from `OauthAccountData`) regardless of `result` error state.
- **Source fn:** `mre_bug_220_renews_preserved_for_429_accounts` (in `tests/usage/render_tests_a.rs`)
- **Source:** [009_token_usage.md AC-03](../../../docs/feature/009_token_usage.md)

---

### FT-21: `@` in flag column when account is active on another machine's `_active_*` marker

- **Given (unit test):** Two `AccountQuota` structs rendered via `render_text()`:
  - `alice@x.com`: `is_current = true`, `is_active = true`, `is_occupied_elsewhere = false`, `result = Ok(...)`.
  - `bob@x.com`: `is_current = false`, `is_active = false`, `is_occupied_elsewhere = true`, `result = Ok(...)`.
  - `bob@x.com` is NOT the recommended next account.
- **When:** `render_text()` called with these two accounts and default `ColsVisibility`.
- **Then:**
  - The line containing `alice@x.com` starts with `✓` in the flag column.
  - The line containing `bob@x.com` starts with `@` in the flag column.
  - No line contains both `@` and `✓` or both `@` and `*`.
- **Exit:** n/a (unit test — string content assertion)
- **Note:** `is_occupied_elsewhere = true` sets `@` only when neither `is_current` nor `is_active` is true (priority: `✓` > `*` > `@` > blank).
- **Source fn:** `test_ft21_009_occupied_elsewhere_at_flag` (in `tests/usage/render_tests_a.rs`)
- **Source:** [009_token_usage.md AC-30](../../../docs/feature/009_token_usage.md)

---

### FT-22: Cancelled subscription shows `(no subscription)` in last quota column

- **Given (unit test):** One `AccountQuota`:
  - `result = Err("no subscription")` (overridden at fetch time when `billing_type == "none"`)
  - `account = Some(OauthAccountData { billing_type: "none", has_max: false, org_created_at: "..." })`
  - Default `ColsVisibility` (7d Reset is last visible quota column)
- **When:** Rendered via `render_text()` and `render_tsv()`.
- **Then:**
  - The `7d Reset` column cell contains `(no subscription)`.
  - No cell contains `(rate limited (429))`.
  - `Sub` column (when visible via `cols::+sub`) shows `"—"` (from `sub_label` with `billing_type="none"`).
- **Exit:** n/a (unit test)
- **Note:** Fix(BUG-233) Class A: fetch layer now overrides `result` to `Err("no subscription")` after `account_handle.join()` when `billing_type == "none"`. The previous BUG-231 display-layer workaround (`error_label` in `format.rs`) is deleted — superseded by this data-layer fix (AC-31).
- **Source fn:** `test_ft23_009_renews_dash_for_cancelled_subscription` (in `tests/usage/render_tests_a.rs`); `test_class_a_billing_none_override_predicate` (in `tests/usage/fetch_tests.rs`)
- **Source:** [009_token_usage.md AC-03, AC-31](../../../docs/feature/009_token_usage.md)

---

### FT-23: `~Renews` shows `"—"` for cancelled subscription accounts

- **Given (unit test):** One `AccountQuota`:
  - `result = Err("no subscription")`
  - `account = Some(OauthAccountData { billing_type: "none", has_max: false, org_created_at: "2024-01-15T00:00:00Z" })`
  - `renewal_at = None` (no override)
  - Default `ColsVisibility` (`~Renews` visible)
- **When:** Rendered via `render_text()` and `render_tsv()`.
- **Then:**
  - The `~Renews` column cell contains `"—"` (em dash, not `"?"`, not `"~in Nd"`).
  - Despite `org_created_at` being present and parseable, no billing estimate is shown — the subscription is cancelled.
- **Exit:** n/a (unit test)
- **Source fn:** `test_ft23_009_renews_dash_for_cancelled_subscription` (in `tests/usage/render_tests_a.rs`)
- **Source:** [009_token_usage.md AC-27, AC-31](../../../docs/feature/009_token_usage.md)

---

### FT-24: Trace result line emitted AFTER Class A billing_type override

- **Given (structural test):** Source file `src/usage/fetch.rs` as a string (via `include_str!`).
- **When:** Position of the Class A override pattern (`a.billing_type == "none" ) { Err( "no subscription"`) and position of the trace emission pattern (`eprintln!( "{}{}  result: OK"`) are extracted from the source string.
- **Then:**
  - The Class A override pattern is found (non-None) — the override is present.
  - The trace emission pattern is found (non-None) — the trace line is present.
  - `override_pos < trace_pos` — the override precedes the trace emission in source order.
  - Observable consequence: for `billing_type="none"` accounts, the trace result line emits `Err(no subscription)`, matching the table — no `result: OK` / `(no subscription)` contradiction.
- **Exit:** n/a (structural unit test — assertion failure if positions violate ordering)
- **Note:** BUG-234 fix. The bug was introduced when the BUG-233 Class A override was added after the trace block rather than before it. Structural test prevents regression without requiring live API calls. Source ordering is the correctness invariant — at runtime, any `billing_type="none"` override applied before the trace emission guarantees trace-result consistency.
- **Source fn:** `mre_bug234_result_trace_after_billing_type_override` (in `tests/usage/fetch_tests.rs`)
- **Source:** [009_token_usage.md AC-31](../../../docs/feature/009_token_usage.md)

---

### FT-25: `.usage` applies model override for current account when `7d(Son) < 15%`

- **Given (unit test):** One `AccountQuota` for the current account (`is_current = true`):
  - `result = Ok(OauthUsageData)` with `seven_day_sonnet = Some(PeriodUsage { utilization: 90.0, resets_at: Some("...") })` — 10% left (< 15% threshold)
  - `~/.claude/settings.json` contains `"model": "claude-sonnet-4-6"`
  - `ClaudePaths` pointing to a temp directory
- **When:** `apply_model_override(&data, &paths, false, "usage", "test@example.com")` is called with the current account's quota data.
- **Then:**
  - `~/.claude/settings.json` now contains `"model": "claude-opus-4-6"` — the override fired.
  - The structural test verifies that `usage_routine()` calls `apply_model_override` after the touch loop for the current account (source position assertion or direct call test).
- **Exit:** n/a (unit test)
- **Note:** Fix for BUG-244. The model override was previously only reachable from `.account.use` (`account_ops.rs`). This test verifies the `.usage` path also applies it. Reuses the existing `apply_model_override()` function (tested by BUG-238 MRE) but validates it is called from the `.usage` pipeline.
- **Source fn:** `mre_bug244_usage_routine_never_calls_apply_model_override` (in `tests/usage/api_tests_b.rs`)
- **Source:** [009_token_usage.md AC-32](../../../docs/feature/009_token_usage.md)

---

### FT-27: `.usage` model override writes `"sonnet"` conservatively when `seven_day_sonnet = None` (BUG-300 + BUG-311)

- **Given (unit test):** One `AccountQuota` for the current account (`is_current = true`):
  - `result = Ok(OauthUsageData)` with `seven_day_sonnet = None` (absent tier — account has no Sonnet weekly quota)
  - `~/.claude/settings.json` not present (or empty)
  - `ClaudePaths` pointing to a temp directory
- **When:** `apply_model_override(&data, &paths, false, "usage", "test@example.com")` is called.
- **Then:**
  - `~/.claude/settings.json` contains `"model": "sonnet"` — `override_session_model_to_sonnet()` fires conservatively (Fix BUG-311). `"opus"` does NOT appear. Absent tier is treated as unknown, not exhausted.
- **Exit:** n/a (unit test)
- **Note:** Fix(BUG-300): `map_or(0.0, ...)` treated `None` as 0% remaining (fully exhausted), causing unconditional Opus override for accounts without a Sonnet tier. `None` must be treated as absent/unknown — not as exhaustion. Guard changed to `if let Some(ref sonnet) = quota.seven_day_sonnet { ... }`. Fix(BUG-311): the else-branch (tier absent) now conservatively calls `override_session_model_to_sonnet()` — absent tier ≠ exhausted, so Sonnet is the safe default.
- **Source fn:** `mre_bug300_model_override_absent_sonnet_no_override` (in `tests/usage/api_tests_a.rs`)
- **Source:** [009_token_usage.md AC-32](../../../docs/feature/009_token_usage.md)

---

### FT-28: Footer `Current` + `Next` lines with `·` delimiter and column alignment

- **Given (unit test):** Two `AccountQuota` entries rendered via `render_text()`: `a@x.com` (`is_current=true`, valid quota), `b@x.com` (valid quota, `seven_day_sonnet.utilization = 50.0`). `session_model = Some("sonnet")`, `session_effort = Some("low")`.
- **When:** `render_text(&accounts, SortStrategy::Renew, ...)` is called.
- **Then:** Footer contains two lines: (1) a line containing `Current` and `·` and `a@x.com` and `sonnet/low`; (2) a line containing `Next (renew)` and `·` and `b@x.com` and `sonnet/high` — effort is model-derived (TSK-335 H3: always shown unconditionally; `"high"` for Sonnet regardless of `session_effort`). The `·` delimiters in both lines are vertically aligned (same column positions).
- **Exit:** n/a (unit test)
- **Source fn:** `test_ft28_009_footer_model_label` (in `src/usage/mod.rs`)
- **Source:** [009_token_usage.md AC-10](../../../docs/feature/009_token_usage.md)

---

### FT-26: `format::json` output includes `"is_owned"` bool per account object

- **Given (unit test):** Two `AccountQuota` entries rendered via `render_json()`:
  - `alice@x.com`: `is_owned = true` (owned by current identity), `result = Ok(...)`.
  - `bob@x.com`: `is_owned = false` (owned by a different identity — G1 non-owned path), `result = Err("cached")` (from cache).
- **When:** `render_json(&[alice_aq, bob_aq])` is called (or equivalent JSON rendering pipeline).
- **Then:**
  - The JSON array contains two objects.
  - The object for `alice@x.com` contains `"is_owned": true`.
  - The object for `bob@x.com` contains `"is_owned": false`.
  - `"is_owned"` is present in every account object regardless of result state.
- **Exit:** n/a (unit test — JSON string assertion)
- **Source fn:** `ft12_json_output_includes_is_owned` (in `tests/usage/render_tests_a.rs`)
- **Note:** Feature 036 FT-12 is the definitive test for this behavior. This FT-26 records the same coverage from Feature 009's perspective (AC-05 lists `is_owned (bool — Feature 036)` as a required JSON field). Both specs reference the same source function.
- **Source:** [009_token_usage.md AC-05](../../../docs/feature/009_token_usage.md)

---

### FT-29: Footer `Current` line shows `model/effort` when effort present; `model` only when absent

- **Given (unit test):** Two scenarios, each with two valid `AccountQuota` entries rendered via `render_text()`:
  - **Scenario 1 (effort present):** `session_model = Some("sonnet")`, `session_effort = Some("low")`.
  - **Scenario 2 (effort absent):** `session_model = Some("sonnet")`, `session_effort = None`.
- **When:** `render_text(...)` is called for each scenario.
- **Then:**
  - Scenario 1: footer `Current` line contains `sonnet/low` (model + slash + effort combined).
  - Scenario 2: footer `Current` line contains `sonnet` but NOT `sonnet/` (no trailing slash when effort absent).
  - In both scenarios, footer `Next` line is present and unaffected.
- **Exit:** n/a (unit test — string content assertions)
- **Source fn:** `test_ft29_009_footer_session_effort_display` (in `tests/usage/render_tests_b.rs`)
- **Source:** [009_token_usage.md AC-10](../../../docs/feature/009_token_usage.md)

---

### FT-30: Sessions table shown when >1 `_active_*` marker exists; own session has `✓`

- **Given:** Credential store contains 2 `_active_*` files: `_active_laptop_user1` → `alice@x.com` (current machine's own marker), `_active_desktop_user1` → `bob@x.com` (another machine). Two saved accounts with valid quota.
- **When:** `clp .usage` is run.
- **Then:** Output after the footer contains a `Sessions` table with two rows: `user1@laptop` → `alice@x.com ✓` and `user1@desktop` → `bob@x.com`. The `✓` marks the current machine's own session.
- **Exit:** 0
- **Source:** [009_token_usage.md AC-33](../../../docs/feature/009_token_usage.md)

---

### FT-31: Sessions table hidden when ≤1 `_active_*` marker (single-session default)

- **Given:** Credential store contains only the current machine's own `_active_*` marker. Two saved accounts with valid quota.
- **When:** `clp .usage` is run (no `who::` parameter).
- **Then:** Output does NOT contain a `Sessions` heading or table. Footer is present but no sessions section follows it.
- **Exit:** 0
- **Source:** [009_token_usage.md AC-33](../../../docs/feature/009_token_usage.md)

---

### FT-32: `who::0` suppresses sessions table; `who::1` forces it on

- **Given:** Credential store contains 2 `_active_*` marker files (>1 session). Two saved accounts with valid quota.
- **When (suppress):** `clp .usage who::0`
- **Then:** Output does NOT contain a `Sessions` table despite >1 marker.
- **When (force):** Credential store contains only 1 `_active_*` marker. `clp .usage who::1`
- **Then:** Output contains a `Sessions` table with 1 row (the current machine's own session with `✓`), despite ≤1 marker.
- **Exit:** 0
- **Source:** [009_token_usage.md AC-34](../../../docs/feature/009_token_usage.md)

---

### FT-33: Cancelled account (`billing_type="none"`) gets `🔴` in `●` column regardless of quota values

- **Given (unit test):** One `AccountQuota` created via `mk_aq_cancelled("dead@test.com", 20.0, 20.0)`:
  - `result = Ok(OauthUsageData)` with `five_hour.utilization = 20.0` (80% left) and `seven_day.utilization = 20.0` (80% left) — both well above thresholds
  - `account = Some(OauthAccountData { billing_type: "none", ... })` — confirmed cancelled subscription
- **When:** `status_emoji(&aq)` is called.
- **Then:** Returns `"🔴"` — the `billing_type = "none"` gate fires before quota thresholds. Without Fix(BUG-317), this would return `"🟢"` (both quotas healthy, `result = Ok`).
- **Exit:** n/a (unit test)
- **Source fn:** `mre_bug317_cancelled_status_emoji_is_red` (in `tests/usage/format_tests.rs`)
- **Source:** [009_token_usage.md AC-18](../../../docs/feature/009_token_usage.md)

---

### FT-34: Both-exhausted account (5h ≤ 15% AND 7d ≤ 5%) gets `🟡`, not `🔴` (BUG-321)

- **Given (unit test):** One `AccountQuota`:
  - `result = Ok(OauthUsageData)` with `five_hour.utilization = 94.0` (6% left) and `seven_day.utilization = 96.0` (4% left) — both dimensions below their exhaustion thresholds
  - Active subscription (`billing_type` not `"none"`)
- **When:** `status_emoji(&aq)` is called.
- **Then:** Returns `"🟡"` — both-exhausted (G3 weekly-exhausted) is recoverable by waiting; it is NOT dead. `"🔴"` (Dead) is reserved for `result = Err` or `billing_type="none"` only.
- **Exit:** n/a (unit test)
- **Note:** Fix(BUG-321): `( _, false ) => StatusGroup::WeeklyExhausted` in `status_group_of()` (merged `(true,false)` and `(false,false)` arms; no new variant); `_ => "🟡"` catch-all in `status_emoji()` (dead gate via `billing_type` early return fires before the match). The BUG-319 fix (`(false,false)→🔴`) was premise-incorrect — both-exhausted and weekly-exhausted have identical recovery behavior (7d is the binding constraint). `mre_bug319_both_exhausted_status_emoji_is_red` assertion flipped 🔴→🟡 by this fix.
- **Source fn:** `mre_bug321_both_exhausted_status_emoji_is_yellow` (in `tests/usage/format_tests.rs`)
- **Source:** [009_token_usage.md AC-18, AC-26](../../../docs/feature/009_token_usage.md)

---

### FT-35: Both-exhausted (🟡) sorts in G3 weekly-exhausted group, before G4 Dead (🔴)

- **Given (unit test):** Two `AccountQuota` entries:
  - `both@x.com`: `result = Ok(data)` with `five_hour.utilization = 94.0` (6% left) and `seven_day.utilization = 96.0` (4% left) — both-exhausted → G3 weekly-exhausted (🟡)
  - `dead@x.com`: `result = Err("missing accessToken")` — G4 Dead (🔴)
  - Input order: dead first (alpha order `both` > `dead` reversed for test clarity)
- **When:** Four-group status partition applied; `sort::name` applied.
- **Then:** `both@x.com` (G3 🟡) appears before `dead@x.com` (G4 🔴) in output. G3 < G4 partition index enforces this regardless of alphabetical order. Fix(BUG-321): `(false,false)` maps to `StatusGroup::WeeklyExhausted` (G3), not `StatusGroup::Red` (G4).
- **Exit:** n/a (unit test — order assertion)
- **Note:** Both-exhausted accounts WILL recover when quota resets (7d resets, 5h will also have reset); dead accounts require external action. The 7d constraint is binding for both both-exhausted and weekly-exhausted — identical recovery behavior.
- **Source fn:** `mre_bug321_both_exhausted_sorts_in_weekly_group` (in `src/usage/sort.rs` or `src/usage/mod.rs`)
- **Source:** [009_token_usage.md AC-24, AC-26](../../../docs/feature/009_token_usage.md)

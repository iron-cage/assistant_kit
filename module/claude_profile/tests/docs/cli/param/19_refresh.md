# Parameter :: `refresh::`

Edge case tests for the `refresh::` parameter. Tests validate boolean enforcement, default-on behavior, and conditional 429 trigger logic. Used by `.usage` to silently retry expired OAuth tokens before reporting auth errors, and by `.account.use` to attempt token refresh before refusing with exit 3 on locally-expired tokens.

**Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `refresh::1` accepted — default-on behavior active | Default On |
| EC-2 | `refresh::0` accepted — auth errors shown as rows without retry | Opt-out |
| EC-3 | `refresh::2` rejected (out of range) | Boundary Values |
| EC-4 | `refresh::yes` rejected (type validation) | Type Validation |
| EC-5 | Default value is `1` (refresh on by default) | Default |
| EC-6 | 429 + non-expired local token — NOT retried even with `refresh::1` | Conditional 429 |
| EC-7 | 429 + expired local token — refresh triggered with `refresh::1` | Conditional 429 |
| EC-8 | `.account.use refresh::0` + expired `expiresAt` — exits 3 immediately, no refresh attempt | `.account.use` Opt-out |
| EC-9 | `.account.use refresh::1` (default) + expired `expiresAt` — refresh attempted, exits 3 on failure | `.account.use` Default-on |

## Test Coverage Summary

- Default On: 1 test (EC-1)
- Opt-out: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Type Validation: 1 test (EC-4)
- Default: 1 test (EC-5)
- Conditional 429: 2 tests (EC-6, EC-7)
- `.account.use` Opt-out: 1 test (EC-8)
- `.account.use` Default-on: 1 test (EC-9)

**Total:** 9 edge cases

**Behavioral Divergence Pair:** EC-2 (explicit opt-out — auth errors shown) ↔ EC-5 (default on — auth errors silently retried)

## Test Cases
---

### EC-1: `refresh::1` — accepted; offline no-token account never reaches retry logic

- **Given:** One saved account `test-acct` with no `accessToken` in its credential file (no live credentials — the quota fetch never reaches HTTP).
- **When:** `clp .usage refresh::1`
- **Then:** Exits 0. Account name `test-acct` appears in output. Because there's no `accessToken`, no HTTP call and no 401 ever occur — `refresh_account_token()` is never invoked. The test only confirms `refresh::1` is accepted and doesn't crash this offline/no-token path; it does not exercise or assert a silent-retry code path.
- **Exit:** 0
- **Source fn:** `it020_refresh_enabled_offline_no_retry_triggered`
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md)
---

### EC-2: `refresh::0` — explicit disable accepted on empty store

- **Given:** Empty credential store (no accounts saved).
- **When:** `clp .usage refresh::0`
- **Then:** Exits 0. stdout contains a "no accounts" message. This is a parser-acceptance/TDD-guard test — it confirms `refresh::0` is accepted on an empty store; it does not set up an expired-credential/401 account or assert an `auth expired (401)` row.
- **Exit:** 0
- **Source fn:** `it019_refresh_disabled_param_accepted`
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md)
---

### EC-3: `refresh::2` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage refresh::2`
- **Then:** Exit 1 with error referencing `refresh::`; must be 0 or 1.
- **Exit:** 1
- **Source fn:** `it039_refresh_2_rejected`
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md)
---

### EC-4: `refresh::yes` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage refresh::yes`
- **Then:** Exit 1 with type validation error referencing `refresh::`.
- **Exit:** 1
- **Source fn:** `it040_refresh_yes_rejected`
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md)
---

### EC-5: Default value is `1` (refresh on by default) — verified via help text

- **Given:** None — no account or credential setup.
- **When:** `clp .usage.help`
- **Then:** Exits 0. stdout contains the exact phrase `1 = enabled, default` and does NOT contain `0 = disabled, default`. This is a `bug_reproducer(BUG-155)` regression test verifying the documented default-value wording in help text; it does not invoke `.usage` at runtime or assert any silent-retry behavior.
- **Exit:** 0
- **Source fn:** `it037_mre_bug155_refresh_defaults_to_1`
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md)
---

### EC-6: 429 + non-expired local token — NOT retried

- **Given:** One saved account with a non-expired `expiresAt` in its per-account credential file (`expiresAt / 1000 > now`); the usage API returns HTTP 429 for that account.
- **When:** `clp .usage refresh::1`
- **Then:** The account's row shows the rate-limit error (`rate limited (429)`); `refresh_account_token` is NOT called for this account; the 429 is passed through unchanged.
- **Exit:** 0
- **Source fn:** `test_apply_refresh_ft4_429_valid_token_not_retried`
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md)
---

### EC-7: 429 + expired local token, no credential file — refresh path entered but hard-fails

- **Given:** An in-memory `AccountQuota` with `expires_at_ms: 0` (locally expired) and `result: Err("HTTP transport error: HTTP 429")`. No per-account credential file exists in the (empty) store directory. This is a direct unit-level call to `apply_refresh()` via `test_bridge`, not a `clp` CLI invocation.
- **When:** `apply_refresh(&mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false)` called directly.
- **Then:** `should_refresh` returns `true` for the locally-expired 429, so the refresh path IS entered — but with no credential file present, `refresh_account_token` returns `None`, the account is skipped, and `accounts[0].result` becomes `Err("token refresh failed")` (BUG-297 fix; cause-neutral label per BUG-539) — a hard failure, NOT a retried/successful quota fetch. Contrast: `test_apply_refresh_ft4_429_valid_token_not_retried` (FT-04) covers the non-expired 429 case, where the refresh path is never entered at all.
- **Exit:** N/A (unit-level function call — no CLI process, no exit code)
- **Source fn:** `test_apply_refresh_ft5_429_expired_refresh_path_entered_no_cred`
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md)
---

### EC-8: `.account.use refresh::0` + expired token — exits 3 immediately, no refresh attempt

- **Given:** Account `alice@home.com` saved with `expiresAt` in the past (locally expired) and no `accessToken`. Per-machine active marker set to a different account.
- **When:** `clp .account.use name::alice@home.com refresh::0 trace::1`
- **Then:** Exits 3. Stderr contains `account credentials expired: alice@home.com`. Does NOT contain `"and refresh failed"` (no refresh was attempted). Trace contains `refused (refresh::0)`. `~/.claude/.credentials.json` unchanged.
- **Exit:** 3
- **Source fn:** `aw33_refresh_disabled_exits_3_immediately` (in `account_relogin_test_b.rs`)
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md), [feature/027_account_use_post_switch_touch.md AC-20](../../../../docs/feature/027_account_use_post_switch_touch.md)
---

### EC-9: `.account.use refresh::1` (default) + expired token — refresh attempted, exits 3 on failure

- **Given:** Account `alice@home.com` saved with `expiresAt` in the past (locally expired) and no `accessToken` (refresh will fail because there is no valid credential to run the subprocess with). Default `refresh::1` applies.
- **When:** `clp .account.use name::alice@home.com` (default `refresh::1`)
- **Then:** Exits 3. Stderr contains `account credentials expired and refresh failed: alice@home.com`. The refresh was attempted (no `accessToken` → subprocess fails immediately). `~/.claude/.credentials.json` unchanged.
- **Exit:** 3
- **Source fn:** `mre_bug230_account_use_refresh_fails_exits_3_with_updated_message` (in `account_relogin_test_b.rs`)
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md), [feature/027_account_use_post_switch_touch.md AC-17](../../../../docs/feature/027_account_use_post_switch_touch.md)

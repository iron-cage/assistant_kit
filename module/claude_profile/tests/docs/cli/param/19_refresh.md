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

### EC-1: `refresh::1` — default-on value accepted

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage refresh::1`
- **Then:** Command accepted; on auth error accounts silently retry via `account::refresh_account_token()`; behavior identical to omitting `refresh::`.
- **Exit:** 0
- **Source fn:** `it020_refresh_enabled_offline_no_retry_triggered`
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md)
---

### EC-2: `refresh::0` — explicit disable accepted; auth errors shown as rows

- **Given:** One saved account whose credential is expired (returns 401 on fetch).
- **When:** `clp .usage refresh::0`
- **Then:** The account's row shows the auth error string (e.g., `auth expired (401)`); `refresh_account_token` is never called; exit 0.
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

### EC-5: Default value is `1` (refresh on by default)

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage` (no `refresh::` param)
- **Then:** Refresh behavior is active — identical to `refresh::1`; on auth error accounts silently retry; exit 0.
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

### EC-7: 429 + expired local token — refresh triggered

- **Given:** One saved account with an expired `expiresAt` in its per-account credential file (`expiresAt / 1000 <= now`); the usage API returns HTTP 429 for that account.
- **When:** `clp .usage refresh::1`
- **Then:** `refresh_account_token` is called for that account (expired local token indicates stale per-account copy); if updated credentials are returned, the account quota fetch is retried once.
- **Exit:** 0
- **Source fn:** `test_apply_refresh_ft5_429_expired_refresh_path_entered_no_cred`
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md)
---

### EC-8: `.account.use refresh::0` + expired token — exits 3 immediately, no refresh attempt

- **Given:** Account `alice@home.com` saved with `expiresAt` in the past (locally expired) and no `accessToken`. Per-machine active marker set to a different account.
- **When:** `clp .account.use name::alice@home.com refresh::0 trace::1`
- **Then:** Exits 3. Stderr contains `account credentials expired: alice@home.com`. Does NOT contain `"and refresh failed"` (no refresh was attempted). Trace contains `refused (refresh::0)`. `~/.claude/.credentials.json` unchanged.
- **Exit:** 3
- **Source fn:** `aw33_refresh_disabled_exits_3_immediately` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md), [feature/027_account_use_post_switch_touch.md AC-20](../../../../docs/feature/027_account_use_post_switch_touch.md)
---

### EC-9: `.account.use refresh::1` (default) + expired token — refresh attempted, exits 3 on failure

- **Given:** Account `alice@home.com` saved with `expiresAt` in the past (locally expired) and no `accessToken` (refresh will fail because there is no valid credential to run the subprocess with). Default `refresh::1` applies.
- **When:** `clp .account.use name::alice@home.com` (default `refresh::1`)
- **Then:** Exits 3. Stderr contains `account credentials expired and refresh failed: alice@home.com`. The refresh was attempted (no `accessToken` → subprocess fails immediately). `~/.claude/.credentials.json` unchanged.
- **Exit:** 3
- **Source fn:** `mre_bug230_account_use_refresh_fails_exits_3_with_updated_message` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [params.md#parameter--19-refresh](../../../../docs/cli/param/019_refresh.md), [feature/027_account_use_post_switch_touch.md AC-17](../../../../docs/feature/027_account_use_post_switch_touch.md)

# Test: Feature 017 — Expired Token Refresh via Isolated Subprocess

Feature behavioral requirement test cases for `docs/feature/017_token_refresh.md`. Each FT case maps to one acceptance criterion. Command-level tests (IT-N) are in [cli/command/09_usage.md](../cli/command/09_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Command IT |
|----|-----------|-----|------------|
| FT-01 | `refresh::0` — no `run_isolated` calls; errors shown as rows | AC-18 | it19, it20 |
| FT-02 | HTTP 401 triggers refresh attempt | AC-19 | — |
| FT-03 | HTTP 403 triggers refresh attempt | AC-19 | — |
| FT-04 | 429 + non-expired local token — NOT retried | AC-19 | — |
| FT-05 | 429 + expired local token — refresh triggered | AC-19 | — |
| FT-06 | Credential file updated on disk before retry fetch | AC-20 | it32 |
| FT-07 | Refresh failure shown in row; other accounts still rendered | AC-21 | — |
| FT-08 | `format::json` output structure unchanged by refresh | AC-22 | — |
| FT-09 | `refresh::` in `.usage --help` with default `1` | AC-23 | it37 |
| FT-10 | Help text documents conditional 429 case | AC-24 | it33, it38 |
| FT-11 | `expires_at_ms` derived from JWT `exp` after refresh | AC-25 | test_jwt_exp_ms_mre_bug162 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `refresh::0` produces no retry — auth error shown in row | AC-18 | Disable Refresh |
| FT-02 | HTTP 401 triggers `run_isolated` + retry | AC-19 | Auth Error Trigger |
| FT-03 | HTTP 403 triggers `run_isolated` + retry | AC-19 | Auth Error Trigger |
| FT-04 | 429 + non-expired local token passes through unchanged | AC-19 | Conditional 429 |
| FT-05 | 429 + expired local token triggers `run_isolated` | AC-19 | Conditional 429 |
| FT-06 | Credential file written to disk before retry fetch | AC-20 | Write-back |
| FT-07 | Refresh failure in row; remaining accounts processed | AC-21 | Non-aborting |
| FT-08 | `format::json` structure contains refreshed data unchanged | AC-22 | Format Interaction |
| FT-09 | `refresh::` appears in `.usage --help` with default `1` | AC-23 | Help Output |
| FT-10 | Help documents conditional 429 case (not unconditionally excluded) | AC-24 | Help Output |
| FT-11 | Post-refresh `expires_at_ms` from JWT `exp`; not from file `expiresAt` | AC-25 | JWT Expiry |

**Total:** 11 FT cases

---

### FT-01: `refresh::0` produces no retry — auth error shown in row

- **Given:** One saved account whose credential is expired; the usage API returns HTTP 401 for that account.
- **When:** `clp .usage refresh::0`
- **Then:** The account's row shows the auth error (e.g., `auth expired (401)`); no retry is attempted; exit 0.
- **Exit:** 0
- **Source fn:** `it19_refresh_disabled_param_accepted`
- **Source:** [017_token_refresh.md AC-18](../../../docs/feature/017_token_refresh.md)

---

### FT-02: HTTP 401 triggers `run_isolated` + retry

- **Given:** One saved account whose credential is expired; the usage API returns HTTP 401 for that account.
- **When:** `clp .usage refresh::1`
- **Then:** A `run_isolated` call is attempted for that account; if updated credentials are returned, the quota fetch is retried and the row shows live data; exit 0.
- **Exit:** 0
- **Source fn:** `it32_lim_it_refresh_per_account` [live — requires credentials]
- **Source:** [017_token_refresh.md AC-19](../../../docs/feature/017_token_refresh.md)

---

### FT-03: HTTP 403 triggers `run_isolated` + retry

- **Given:** One saved account whose credential returns HTTP 403 from the usage API.
- **When:** `clp .usage refresh::1`
- **Then:** A `run_isolated` call is attempted for that account (403 is treated identically to 401); exit 0.
- **Exit:** 0
- **Source fn:** `TBD — no dedicated test`
- **Source:** [017_token_refresh.md AC-19](../../../docs/feature/017_token_refresh.md)

---

### FT-04: 429 + non-expired local token passes through unchanged

- **Given:** One saved account with a valid (non-expired) `expiresAt` in its per-account credential file; the usage API returns HTTP 429 for that account.
- **When:** `clp .usage refresh::1`
- **Then:** The account's row shows `rate limited (429)`; no refresh is attempted (local token is valid — the 429 is a genuine rate limit); exit 0.
- **Exit:** 0
- **Source fn:** `TBD — no dedicated test`
- **Source:** [017_token_refresh.md AC-19](../../../docs/feature/017_token_refresh.md)

---

### FT-05: 429 + expired local token triggers `run_isolated`

- **Given:** One saved account with an expired `expiresAt` in its per-account credential file (`expiresAt / 1000 <= now`); the usage API returns HTTP 429.
- **When:** `clp .usage refresh::1`
- **Then:** `run_isolated` is invoked for that account; the 429 is treated as a stale-credential condition; exit 0.
- **Exit:** 0
- **Source fn:** `TBD — no dedicated test`
- **Source:** [017_token_refresh.md AC-19](../../../docs/feature/017_token_refresh.md)

---

### FT-06: Credential file written to disk before retry fetch

- **Given:** One saved account whose credential is expired; `run_isolated` returns updated credentials.
- **When:** `clp .usage refresh::1`
- **Then:** The per-account credential file at `{credential_store}/{name}.credentials.json` is overwritten with the new JSON before the retry fetch; subsequent reads of that file yield the refreshed token.
- **Exit:** 0
- **Source fn:** `it32_lim_it_refresh_per_account` [live — requires credentials]
- **Source:** [017_token_refresh.md AC-20](../../../docs/feature/017_token_refresh.md)

---

### FT-07: Refresh failure shown in row; remaining accounts processed

- **Given:** Two saved accounts: one whose refresh fails (e.g., `run_isolated` times out), one whose fetch succeeds normally.
- **When:** `clp .usage refresh::1`
- **Then:** The failing account's row shows the final error reason; the succeeding account's row shows normal quota data; both rows are present; the table is rendered; exit 0.
- **Exit:** 0
- **Source fn:** `TBD — no dedicated test`
- **Source:** [017_token_refresh.md AC-21](../../../docs/feature/017_token_refresh.md)

---

### FT-08: `format::json` structure unchanged by refresh

- **Given:** One saved account whose token is refreshed; one whose token is valid without refresh.
- **When:** `clp .usage format::json refresh::1`
- **Then:** JSON output is a valid array; refreshed accounts appear as normal data objects with quota fields; failed-refresh accounts appear as error objects; output structure matches the baseline `.usage format::json` schema; exit 0.
- **Exit:** 0
- **Source fn:** `TBD — no dedicated test`
- **Source:** [017_token_refresh.md AC-22](../../../docs/feature/017_token_refresh.md)

---

### FT-09: `refresh::` appears in `.usage --help` with default `1`

- **Given:** clean environment.
- **When:** `clp .usage --help`
- **Then:** stdout or stderr contains `refresh::` with a default value of `1`; the parameter is documented in the help output.
- **Exit:** 0
- **Source fn:** `it37_mre_bug155_refresh_defaults_to_1`
- **Source:** [017_token_refresh.md AC-23](../../../docs/feature/017_token_refresh.md)

---

### FT-10: Help documents conditional 429 case (not unconditionally excluded)

- **Given:** clean environment.
- **When:** `clp .usage --help`
- **Then:** The `refresh::` help text references the conditional 429 case (e.g., `429 when token is locally expired` or similar); it does NOT describe 429 as unconditionally excluded from refresh.
- **Exit:** 0
- **Source fn:** `it38_mre_bug156_refresh_help_mentions_429_expired`
- **Source:** [017_token_refresh.md AC-24](../../../docs/feature/017_token_refresh.md)

---

### FT-11: Post-refresh `expires_at_ms` derived from JWT `exp`; not from file `expiresAt`

- **Given:** One saved account whose credential is expired; `run_isolated` returns a new `accessToken` with a future JWT `exp` claim; the credential file's `expiresAt` field is NOT updated by the subprocess.
- **When:** `clp .usage refresh::1`
- **Then:** After refresh, the account's Expires column shows a future time (not `EXPIRED`); the expiry is derived from the JWT `exp` claim of the new `accessToken`, not from the stale `expiresAt` field in the credential file.
- **Exit:** 0
- **Source fn:** `test_jwt_exp_ms_mre_bug162`
- **Note:** Fix for BUG-162; implemented by TSK-163 (`jwt_exp_ms()` in `src/usage.rs`).
- **Source:** [017_token_refresh.md AC-25](../../../docs/feature/017_token_refresh.md)

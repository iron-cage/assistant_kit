# State Machine 002: OAuth Token Lifecycle

AC test cases for `docs/state_machine/002_oauth_token_lifecycle.md`. Tests the
`valid/at_expired/rt_expired/refreshed` state transitions for OAuth access tokens,
including the `expiresAt=1` forcing mechanism and the no-valid→refreshed boundary.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | `at_expired` — expiresAt ≤ now_ms, token expired locally | State | ✅ |
| AC-2 | `valid` — expiresAt > now_ms; ExpiringSoon classification at threshold | State | ✅ |
| AC-3 | `at_expired` detected via 401 from API | Detection | ✅ |
| AC-4 | `at_expired` detected via 403 from API | Detection | ✅ |
| AC-5 | `expiresAt=1` forced before run_isolated — prevents silent RT decay | Invariant | ✅ |
| AC-6 | `rt_expired` — refresh_account_token() returns None → aq.result = Err | State | ✅ |
| AC-7 | No `valid→refreshed` — valid AT subprocess exits without credential update | Boundary | ✅ |

---

### AC-1: `at_expired` — expiresAt ≤ now_ms, token expired locally

- **Given:** A `{name}.credentials.json` file with `expiresAt` set to a timestamp in the past.
- **When:** `token::status()` is called.
- **Then:** Returns `TokenStatus::Expired`. The access token is classified as expired based on
  the stored `expiresAt` field, independently of any API call.
- **Source fn:** `status_returns_expired_when_expires_at_in_past` in
  `tests/token_tests.rs`
- **Source:** [state_machine/002_oauth_token_lifecycle.md](../../../docs/state_machine/002_oauth_token_lifecycle.md)

---

### AC-2: `valid` — expiresAt > now_ms; ExpiringSoon at threshold

- **Given:** A credentials file with `expiresAt` within the expiry threshold window (token is
  technically valid but approaching expiry).
- **When:** `token::status()` is called with the default expiry threshold.
- **Then:** Returns `TokenStatus::ExpiringSoon { expires_in }`. Token is in `valid` state
  (usable), but the classification signals approaching expiry for caller awareness.
- **Source fn:** `status_returns_expiring_soon_within_default_threshold` in
  `tests/token_tests.rs`
- **Source:** [state_machine/002_oauth_token_lifecycle.md](../../../docs/state_machine/002_oauth_token_lifecycle.md)

---

### AC-3: `at_expired` detected via 401 from API

- **Given:** An account quota fetch returns HTTP 401 (Unauthorized).
- **When:** `apply_refresh()` evaluates the `should_refresh()` predicate for this account.
- **Then:** The 401 triggers the refresh path — `refresh_account_token()` is called. This is
  the server-side detection of `at_expired` state (complements local `expiresAt` detection).
- **Source fn:** `test_apply_refresh_401_no_cred_file` in
  `tests/usage/refresh_tests_a.rs`
- **Source:** [state_machine/002_oauth_token_lifecycle.md](../../../docs/state_machine/002_oauth_token_lifecycle.md)

---

### AC-4: `at_expired` detected via 403 from API

- **Given:** An account quota fetch returns HTTP 403 (Forbidden).
- **When:** `apply_refresh()` evaluates the `should_refresh()` predicate.
- **Then:** The 403 triggers the refresh path — `refresh_account_token()` is called. Both 401
  and 403 are treated as auth failures indicating the access token has expired server-side.
- **Source fn:** `test_apply_refresh_403_no_cred_file` in
  `tests/usage/refresh_tests_a.rs`
- **Source:** [state_machine/002_oauth_token_lifecycle.md](../../../docs/state_machine/002_oauth_token_lifecycle.md)

---

### AC-5: `expiresAt=1` forced before run_isolated — prevents silent RT decay

- **Given:** `refresh_account_token()` is about to call `run_isolated()` for any account.
- **When:** The code path to `run_isolated()` is traversed.
- **Then:** `expiresAt` is set to `"1"` in the in-memory credential copy BEFORE passing it to
  `run_isolated()`. This forces Claude CLI to treat the AT as expired, performing a full
  RT→AT+RT exchange on every call. Without this, a still-valid AT causes the subprocess to
  exit without refresh, silently aging the RT toward server-side expiry.
- **Source fn:** `single_token_refresh_entry_in3_expires_manipulation_before_run_isolated` in
  `tests/cli/invariant_test.rs`
- **Source:** [state_machine/002_oauth_token_lifecycle.md](../../../docs/state_machine/002_oauth_token_lifecycle.md)

---

### AC-6: `rt_expired` — refresh_account_token() returns None → aq.result set to Err

- **Given:** An account whose refresh token has expired server-side. `refresh_account_token()`
  is called (e.g., for a 401 response) but the subprocess exits without updating credentials.
- **When:** `refresh_account_token()` returns `None`.
- **Then:** `aq.result` is set to `Err("refresh token expired")`. This prevents downstream
  `apply_touch()` from firing on the unrecoverable account (Fix BUG-297). The account transitions
  to `rt_expired` state — only browser relogin (`.account.relogin`) can recover it.
- **Source fn:** `mre_bug297_refresh_none_sets_aq_result_err` in
  `tests/usage/refresh_tests_b.rs`
- **Source:** [state_machine/002_oauth_token_lifecycle.md](../../../docs/state_machine/002_oauth_token_lifecycle.md)

---

### AC-7: No `valid→refreshed` — valid AT subprocess exits without credential update

- **Given:** An account with a valid (non-expired) access token. A 429 (rate-limit) response
  is received, but the locally-stored `expiresAt` is still in the future.
- **When:** `apply_refresh()` evaluates `should_refresh()`.
- **Then:** The refresh path is NOT entered — `credentials = None`. A valid AT causes the Claude
  subprocess to use it as-is without performing OAuth refresh. The `valid→refreshed` transition
  does not exist: proactive/approaching-expiry refresh via `run_isolated` is permanently out of
  scope (`feature/017` line 8; BUG-323 invalidated).
- **Source fn:** `test_apply_refresh_ft4_429_valid_token_not_retried` in
  `tests/usage/refresh_tests_b.rs`
- **Source:** [state_machine/002_oauth_token_lifecycle.md](../../../docs/state_machine/002_oauth_token_lifecycle.md)

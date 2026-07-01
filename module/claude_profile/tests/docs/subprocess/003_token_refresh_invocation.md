# Subprocess 003: Token Refresh Invocation

AC test cases for `docs/subprocess/003_token_refresh_invocation.md`. Tests the
`should_refresh()` predicate — which error types trigger refresh, which are blocked
by ownership/occupation gates, and the approaching-expiry prohibition.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | 401 auth error triggers `refresh_account_token()` | Predicate | ✅ |
| AC-2 | 403 auth error triggers `refresh_account_token()` | Predicate | ✅ |
| AC-3 | 429 + expired AT triggers refresh path | Predicate | ✅ |
| AC-4 | 429 + valid AT does NOT trigger refresh | Predicate | ✅ |
| AC-5 | Non-owned account skipped by G2 gate | Gate | ✅ |
| AC-6 | Occupied-elsewhere account skipped by G2 gate | Gate | ✅ |
| AC-7 | `refresh_account_token()` returns `None` → aq.result set to Err | Result | ✅ |

---

### AC-1: 401 auth error triggers `refresh_account_token()`

- **Given:** An account's quota fetch returned HTTP 401 (Unauthorized). Account is owned and
  not occupied elsewhere.
- **When:** `should_refresh()` evaluates the predicate for this account.
- **Then:** Returns `true`. `refresh_account_token()` is called. 401 is a direct auth failure —
  the access token is rejected server-side and must be rotated.
- **Source fn:** `test_apply_refresh_401_no_cred_file` in
  `tests/usage/refresh_tests_a.rs`
- **Source:** [subprocess/003_token_refresh_invocation.md](../../../docs/subprocess/003_token_refresh_invocation.md)

---

### AC-2: 403 auth error triggers `refresh_account_token()`

- **Given:** An account's quota fetch returned HTTP 403 (Forbidden). Account is owned and not
  occupied elsewhere.
- **When:** `should_refresh()` evaluates the predicate.
- **Then:** Returns `true`. `refresh_account_token()` is called. Both 401 and 403 indicate
  an expired or revoked access token requiring rotation.
- **Source fn:** `test_apply_refresh_403_no_cred_file` in
  `tests/usage/refresh_tests_a.rs`
- **Source:** [subprocess/003_token_refresh_invocation.md](../../../docs/subprocess/003_token_refresh_invocation.md)

---

### AC-3: 429 + expired AT triggers refresh path

- **Given:** An account's quota fetch returned HTTP 429 (rate-limited). The locally-stored
  `expiresAt_ms / 1000 ≤ now_secs` (AT is locally expired).
- **When:** `should_refresh()` evaluates the combined 429 + expired predicate.
- **Then:** Returns `true`. `refresh_account_token()` is called. A stale per-account credential
  copy can cause 429 responses; refreshing also rotates the token to a fresh copy.
- **Source fn:** `test_apply_refresh_ft5_429_expired_refresh_path_entered_no_cred` in
  `tests/usage/refresh_tests_b.rs`
- **Source:** [subprocess/003_token_refresh_invocation.md](../../../docs/subprocess/003_token_refresh_invocation.md)

---

### AC-4: 429 + valid AT does NOT trigger refresh

- **Given:** An account's quota fetch returned HTTP 429. The AT is locally valid
  (`expiresAt_ms / 1000 > now_secs`).
- **When:** `should_refresh()` evaluates the predicate.
- **Then:** Returns `false`. 429 alone does not trigger refresh when the AT is still valid —
  a valid AT would cause the subprocess to exit without credential update, making the refresh
  a costly 35-second no-op.
- **Source fn:** `test_apply_refresh_ft4_429_valid_token_not_retried` in
  `tests/usage/refresh_tests_b.rs`
- **Source:** [subprocess/003_token_refresh_invocation.md](../../../docs/subprocess/003_token_refresh_invocation.md)

---

### AC-5: Non-owned account skipped by G2 gate

- **Given:** An account with `is_owned=false` (owned by another machine) that received a
  401/403 auth error.
- **When:** `should_refresh()` evaluates the G2 ownership gate.
- **Then:** Returns `false`. `reason: not_owned` is emitted in trace mode. Refreshing a
  foreign-owned account's token would invalidate the other machine's active session.
  Fix BUG-295.
- **Source fn:** `mre_bug295_apply_refresh_trace_reason_not_owned` in
  `tests/usage/refresh_tests_b.rs`
- **Source:** [subprocess/003_token_refresh_invocation.md](../../../docs/subprocess/003_token_refresh_invocation.md)

---

### AC-6: Occupied-elsewhere account skipped by G2 gate

- **Given:** An account with `is_occupied_elsewhere=true` (currently active on another machine)
  that received a 401 auth error.
- **When:** `should_refresh()` evaluates the G2 occupation gate.
- **Then:** Returns `false`. `reason: occupied_elsewhere` is emitted in trace mode. The G2
  gate checks BOTH `!is_owned` AND `is_occupied_elsewhere` — either condition independently
  blocks refresh. Fix BUG-298 (cached-expired-occupied trace).
- **Source fn:** `mre_bug306_refresh_trace_reason_occupied_elsewhere` in
  `tests/usage/refresh_tests_b.rs`
- **Source:** [subprocess/003_token_refresh_invocation.md](../../../docs/subprocess/003_token_refresh_invocation.md)

---

### AC-7: `refresh_account_token()` returns `None` → aq.result set to Err

- **Given:** `refresh_account_token()` is called for a 401 account but the subprocess exits
  without credential update (i.e., the refresh token is expired server-side).
- **When:** `refresh_account_token()` returns `None`.
- **Then:** `aq.result` is set to `Err("refresh token expired")`. This prevents downstream
  `apply_touch()` from firing on the unrecoverable account — touch after a failed refresh
  would trigger a new subprocess call that cannot succeed. Fix BUG-297.
- **Source fn:** `mre_bug297_refresh_none_sets_aq_result_err` in
  `tests/usage/refresh_tests_b.rs`
- **Source:** [subprocess/003_token_refresh_invocation.md](../../../docs/subprocess/003_token_refresh_invocation.md)

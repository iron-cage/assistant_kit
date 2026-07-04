# Subprocess 002: Credential Write-Back Protocol

AC test cases for `docs/subprocess/002_credential_writeback.md`. Tests the write-back
safety rules — never-write-to-live, `expiresAt=1` forcing, `credentials=None` for valid
AT, and post-rotation live sync (BUG-310 fix).

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | Bulk touch never writes to live `~/.claude/.credentials.json` | Safety | ✅ |
| AC-2 | `expiresAt=1` forces AT-expired before subprocess — prevents silent RT decay | Safety | ✅ |
| AC-3 | Valid AT subprocess produces `credentials=None` (no refresh performed) | Boundary | ✅ |
| AC-4 | Post-rotation live sync: after `apply_touch`, store copied to live (BUG-310) | Safety | ✅ |

---

### AC-1: Bulk touch never writes to live `~/.claude/.credentials.json`

- **Given:** A batch `.usage touch::1` operation that calls `apply_touch()` across multiple
  accounts, potentially including the currently-active account.
- **When:** `apply_touch()` calls `refresh_account_token()` which invokes `run_isolated()`.
- **Then:** `~/.claude/.credentials.json` (the live session file) is NEVER written. Only
  `{name}.credentials.json` in the credential store is updated. Writing to the live session
  file during batch touch was BUG-221 (fixed TSK-230). The credential write path writes to
  the store only.
- **Source fn:** `reach_bulk_touch_does_not_write_live_credentials` in
  `tests/usage/touch_tests.rs`
- **Source:** [subprocess/002_credential_writeback.md](../../../docs/subprocess/002_credential_writeback.md)

---

### AC-2: `expiresAt=1` forces AT-expired before subprocess — prevents silent RT decay

- **Given:** `refresh_account_token()` is about to invoke `run_isolated()`.
- **When:** The code path from `refresh_account_token()` to `run_isolated()` is traversed.
- **Then:** `expiresAt` is set to `"1"` in the in-memory credential copy before the call.
  This forces Claude CLI to classify the AT as expired, performing a full RT→AT+RT exchange
  on every invocation. Without this trick, a valid AT causes the subprocess to exit without
  refresh (`credentials=None`), silently aging the refresh token toward server-side expiry.
- **Source fn:** `single_token_refresh_entry_in3_expires_manipulation_before_run_isolated` in
  `tests/cli/invariant_test.rs`
- **Source:** [subprocess/002_credential_writeback.md](../../../docs/subprocess/002_credential_writeback.md)

---

### AC-3: Valid AT subprocess produces `credentials=None` (no refresh performed)

- **Given:** An account that received a 429 rate-limit response but has a locally-valid AT
  (`expiresAt` is still in the future, not manipulated to `"1"`).
- **When:** `apply_refresh()` evaluates `should_refresh()`.
- **Then:** Refresh is not triggered — `credentials=None`. A valid AT causes Claude subprocess
  to use it as-is and exit without OAuth exchange. This is the boundary case showing that
  `expiresAt=1` manipulation is essential: without it, valid tokens never produce a fresh
  credential write-back.
- **Source fn:** `test_apply_refresh_ft4_429_valid_token_not_retried` in
  `tests/usage/refresh_tests_b.rs`
- **Source:** [subprocess/002_credential_writeback.md](../../../docs/subprocess/002_credential_writeback.md)

---

### AC-4: Post-rotation live sync: after `apply_touch`, store copied to live (BUG-310)

- **Given:** A `.usage rotate::1` operation. `switch_account(winner)` copies stored creds to
  live. Then `apply_touch(winner)` refreshes the token, writing updated creds to the STORE
  only (not to live).
- **When:** Rotation completes.
- **Then:** After `apply_touch()`, the credential store is copied back to the live
  `~/.claude/.credentials.json` for the rotated account. Without this sync, the live session
  retains stale `token_A` while the store holds `token_B` — the user's active Claude session
  would use an outdated token. Fix BUG-310 (AC-11, Feature 038).
- **Source fn:** `mre_bug310_rotation_touch_resyncs_live_credentials` in
  `tests/usage/api_tests_b.rs`
- **Source:** [subprocess/002_credential_writeback.md](../../../docs/subprocess/002_credential_writeback.md)

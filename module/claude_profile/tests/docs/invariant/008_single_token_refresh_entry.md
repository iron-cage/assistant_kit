# Test: Invariant 008 — Single Token Refresh Entry Point

Property assertion cases for `docs/invariant/008_single_token_refresh_entry.md`. Verifies that
all token refresh operations go through `refresh_account_token()` and that no direct
`run_isolated()` calls exist outside `account.rs`.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | `claude_profile/src/` contains zero `run_isolated` calls | Invariant holds (normal) |
| IN-2 | Automated grep test catches any future direct `run_isolated` call | Invariant holds (boundary) |
| IN-3 | `refresh_account_token` sets `expiresAt=1` before `run_isolated` (Change A) | RT rotation enforced |
| IN-4 | `refresh_account_token` syncs live credentials for current account (Change B) | Live sync enforced |

**Total:** 4 IN cases

---

### IN-1: `claude_profile/src/` contains zero `run_isolated` calls

- **Given:** The `src/` directory of `claude_profile` crate at the current HEAD
- **When:** `grep -r "run_isolated(" src/` is run from `module/claude_profile/`
- **Then:** The command returns empty output (exit 1 from grep); no file in `src/` calls
  `run_isolated` directly — all token refresh goes through `refresh_account_token()` in
  `claude_profile_core/src/account.rs`
- **Source:** [docs/invariant/008_single_token_refresh_entry.md](../../../docs/invariant/008_single_token_refresh_entry.md)

---

### IN-2: Automated grep test catches any future direct `run_isolated` call

- **Given:** A grep-based invariant test exists in `tests/` and is executed as part of the
  standard test suite
- **When:** The test is run via `cargo nextest run`
- **Then:** The test passes; if `run_isolated(` were introduced into `claude_profile/src/`,
  this test would fail the CI build, enforcing the invariant automatically
- **Source:** [docs/invariant/008_single_token_refresh_entry.md](../../../docs/invariant/008_single_token_refresh_entry.md)

---

### IN-3: `refresh_account_token` sets `expiresAt=1` before `run_isolated` (Change A)

- **Given:** `refresh_account_token` is called with stored credentials containing
  `"expiresAt":"1719000000000"` (a far-future value — access token is valid)
- **When:** The function prepares credentials for `run_isolated`
- **Then:** The credential JSON passed to `run_isolated` has `expiresAt` set to `"1"` (or
  equivalent past timestamp), forcing CLI to treat AT as expired and use RT to obtain
  fresh AT+RT pair. The original stored credential file is NOT modified.
- **Source:** [docs/invariant/008_single_token_refresh_entry.md](../../../docs/invariant/008_single_token_refresh_entry.md)

---

### IN-4: `refresh_account_token` syncs live credentials for current account (Change B)

- **Given:** `refresh_account_token` is called for the current account with `paths = Some(...)`;
  live credentials at `~/.claude/.credentials.json` differ from stored credentials
  (the live Claude Code session has already refreshed)
- **When:** The function compares live credentials with stored credentials
- **Then:** Live credentials are written to the store; `Some(live_creds)` is returned; no
  `run_isolated` subprocess is spawned. The live session's fresh RT is preserved in the store.
- **Source:** [docs/invariant/008_single_token_refresh_entry.md](../../../docs/invariant/008_single_token_refresh_entry.md)

# Subprocess 005: Browser Relogin Invocation

AC test cases for `docs/subprocess/005_relogin_invocation.md`. Tests `.account.relogin`
behavior — when it is used (RT-expired only), name resolution from active marker,
and the TTY-inherited mechanism distinct from `run_isolated()`.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | Relogin is non-idempotent — each invocation triggers a fresh OAuth browser flow | Behavior | ✅ |
| AC-2 | Relogin updates credentials in-place — account state preserved | Behavior | ✅ |
| AC-3 | Absent account exits 1 — account must exist before relogin | Guard | ✅ |
| AC-4 | No `name::` argument — active account resolved from marker | Resolution | ✅ |
| AC-5 | No `name::` + no active marker → exits 2 | Guard | ✅ |

---

### AC-1: Relogin is non-idempotent — each invocation triggers a fresh OAuth browser flow

- **Given:** An account whose refresh token has expired (RT-expired state). `.account.relogin`
  is the only remedy since `run_isolated()` cannot recover from server-side RT expiry.
- **When:** `.account.relogin name::X` is executed.
- **Then:** A `claude` subprocess is spawned with the real TTY inherited (not piped), allowing
  the browser OAuth flow to complete interactively. Each invocation is non-idempotent: it
  opens a new browser session. The TTY-inherited mechanism is structurally different from
  `run_isolated()` (which pipes stdout/stderr to an isolated temp HOME).
- **Source fn:** `relogin_bv1_lim_it_non_idempotent_oauth_flow` in
  `tests/cli/command_verb_test.rs`
- **Source:** [subprocess/005_relogin_invocation.md](../../../docs/subprocess/005_relogin_invocation.md)

---

### AC-2: Relogin updates credentials in-place — account state preserved

- **Given:** An account in RT-expired state with existing `{name}.json` metadata.
- **When:** `.account.relogin name::X` completes successfully (browser OAuth flow finishes).
- **Then:** The new credentials are written to `{name}.credentials.json` in the credential
  store. The account's `{name}.json` metadata (billing type, org identity, renewal fields) is
  preserved and updated in-place — the account is not deleted and recreated. The previously-
  active account's credentials are restored to `~/.claude/.credentials.json` after relogin.
- **Source fn:** `relogin_bv2_lim_it_updates_in_place_state_preserved` in
  `tests/cli/command_verb_test.rs`
- **Source:** [subprocess/005_relogin_invocation.md](../../../docs/subprocess/005_relogin_invocation.md)

---

### AC-3: Absent account exits 1 — account must exist before relogin

- **Given:** `.account.relogin name::ghost` is invoked for an account that does not exist in
  the credential store.
- **When:** The relogin command looks up the account.
- **Then:** Command exits 1 (usage error). The account must exist (`saved` or `active` state)
  before relogin can be performed. Relogin is an in-place credential update, not account
  creation.
- **Source fn:** `relogin_bv3_absent_account_exits_1` in
  `tests/cli/command_verb_test.rs`
- **Source:** [subprocess/005_relogin_invocation.md](../../../docs/subprocess/005_relogin_invocation.md)

---

### AC-4: No `name::` argument — active account resolved from marker

- **Given:** `.account.relogin` is invoked without a `name::` argument. An active account
  marker `_active_{host}_{user}` exists with a valid account name.
- **When:** The relogin command resolves the target account.
- **Then:** The active account name is read from the marker file and used as the target.
  Relogin without a name argument operates on the currently-active account.
- **Source fn:** `relogin_mre_no_name_uses_active` in
  `tests/cli/account_relogin_test.rs`
- **Source:** [subprocess/005_relogin_invocation.md](../../../docs/subprocess/005_relogin_invocation.md)

---

### AC-5: No `name::` + no active marker → exits 2

- **Given:** `.account.relogin` is invoked without a `name::` argument AND no active account
  marker exists (no account has been activated on this machine).
- **When:** The relogin command attempts to resolve the target from the marker.
- **Then:** Command exits 2 (not-found). Without a marker, no default account can be inferred.
  The caller must supply an explicit `name::` argument.
- **Source fn:** `relogin_mre_no_name_no_active_exits2` in
  `tests/cli/account_relogin_test.rs`
- **Source:** [subprocess/005_relogin_invocation.md](../../../docs/subprocess/005_relogin_invocation.md)

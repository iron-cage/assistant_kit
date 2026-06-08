# tests/manual/

| File | Responsibility |
|------|----------------|
| `readme.md` | Manual testing plan: live Claude Code account switching. |

## Manual Testing Plan — Account Rotation

**Trigger:** After any change to `account::save`, `account::switch_account`,
`account::list`, or `account::delete`.

### Prerequisites

- At least two active Claude Code accounts (personal + work, or two `max` subscriptions)
- Claude Code installed at `~/.claude/`
- Valid `~/.claude/.credentials.json` with an active session

### Steps

1. **Verify starting state**
   ```
   ls ~/.persistent/claude/credential/   # may be empty on first run
   cat ~/.claude/.credentials.json | python3 -m json.tool | grep subscriptionType
   ```

2. **Save current account**
   ```rust
   let persist = claude_profile::PersistPaths::new().expect("HOME set");
   let credential_store = persist.credential_store();
   let paths = claude_profile::ClaudePaths::new().expect("HOME set");
   claude_profile::account::save("work@acme.com", &credential_store, &paths, true, None, None, None).expect("save");
   ```
   Verify: `~/.persistent/claude/credential/work@acme.com.credentials.json` exists and matches active credentials.

3. **List accounts**
   ```rust
   let accounts = claude_profile::account::list(&credential_store).expect("list");
   for a in &accounts { println!("{} active={}", a.name, a.is_active); }
   ```
   Verify: `work@acme.com` appears with `is_active = true` (`save()` writes the `_active` marker).

4. **Switch to saved account**
   ```rust
   claude_profile::account::switch_account("work@acme.com", &credential_store, &paths).expect("switch");
   ```
   Verify:
   - `~/.claude/.credentials.json` content matches `work@acme.com.credentials.json`
   - `~/.persistent/claude/credential/_active_{hostname}_{user}` (use `active_marker_filename()`) contains `"work@acme.com"`
   - Running `claude --version` or a minimal `claude` invocation succeeds

5. **Token status after switch**
   ```rust
   let status = claude_profile::token::status().expect("status");
   println!("{status:?}");
   ```
   Verify: Returns `Valid` or `ExpiringSoon` (not `Expired`) after a fresh switch.

6. **Delete inactive account**
   - Save a second account: `account::save("temp@test.com", &credential_store, &paths, true, None, None, None).expect("save")`
   - Delete it: `account::delete("temp@test.com", &credential_store).expect("delete")`
   - Verify: `~/.persistent/claude/credential/temp@test.com.credentials.json` is gone

7. **Active-account deletion**
   - Ensure the per-machine active marker (`_active_{hostname}_{user}`) points to `"work@acme.com"`
   - Delete: `account::delete("work@acme.com", &credential_store).expect("active account deletion succeeds")`
   - Verify: `work@acme.com.credentials.json` is gone; the per-machine active marker is also removed
   - System is now in "no active account" state; next use of `.account.use` or `.account.save` restores it

### Expected Outcome

All steps succeed without panics. `~/.claude/.credentials.json` is intact after each
step. No partial writes or missing files.

---

## Manual Testing Plan — `.account.limits` Happy Path

**Trigger:** After any change to `fetch_rate_limits()`, `account_limits_routine()`,
or the format helpers in `src/commands.rs`.

**Automated tests (do not re-run manually):** IT-1 (default text) and IT-3 (`format::json`)
are automated live tests in `tests/cli/account_limits_test.rs` (lim_it1, lim_it3).
They require real credentials and will fail without `claude auth login`.

**Manual-only tests (require additional setup):**

### Prerequisites

- Valid `~/.claude/.credentials.json` with an active Claude Max session
- `clp` binary compiled with `--features enabled`
- Network access to `api.anthropic.com`
- A saved named account (run `clp .account.save name::work@acme.com` first)

### IT-4: Named account resolves credentials

```
clp .account.save name::work@acme.com   # save current as "work@acme.com"
clp .account.limits name::work@acme.com
```

Expected exit: 0 — uses `work@acme.com.credentials.json` (not active `.credentials.json`).
Expected: same utilization output as default (uses the named account's API key).

---

## Manual Testing Plan — `.account.relogin` Happy Path

**Trigger:** After any change to `account::relogin`, the credential-capture subprocess path,
or the active-account save/restore logic in `apply_refresh`.

**Automated tests:** None — all relogin scenarios require an interactive TTY `claude` spawn
that cannot be mocked. These must be run manually.

### Prerequisites

- Valid `~/.claude/.credentials.json` with a Claude Max session
- `clp` binary compiled with `--features enabled`
- A saved named account: `clp .account.save name::carol@example.com`
- A second account active: `clp .account.use name::alice@acme.com`

### IT-5: Successful relogin updates credential store (FT-07)

```
clp .account.relogin name::carol@example.com
```

Expected: interactive `claude` TTY prompt appears; after successful login,
`{credential_store}/carol@example.com.credentials.json` is updated. Exit 0.

### IT-6: Active account restored after relogin (FT-08)

```
# alice@acme.com is active
clp .account.relogin name::carol@example.com
clp .usage   # verify alice@acme.com still shows as active (✓)
```

Expected: after relogin completes, active account marker points back to
`alice@acme.com` — not `carol@example.com`. Exit 0.

### IT-7: Abandoned login → exit 3 diagnostic (FT-09)

```
clp .account.relogin name::carol@example.com
# Press Ctrl-C or close TTY without completing login
```

Expected: stderr diagnostic "credentials unchanged"; exit 3 (not 0 or 2).

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
   claude_profile::account::save("work@acme.com", &credential_store, &paths).expect("save");
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
   - `~/.persistent/claude/credential/_active` contains `"work@acme.com"`
   - Running `claude --version` or a minimal `claude` invocation succeeds

5. **Token status after switch**
   ```rust
   let status = claude_profile::token::status().expect("status");
   println!("{status:?}");
   ```
   Verify: Returns `Valid` or `ExpiringSoon` (not `Expired`) after a fresh switch.

6. **Delete inactive account**
   - Save a second account: `account::save("temp@test.com", &credential_store, &paths).expect("save")`
   - Delete it: `account::delete("temp@test.com", &credential_store).expect("delete")`
   - Verify: `~/.persistent/claude/credential/temp@test.com.credentials.json` is gone

7. **Active-account guard**
   - Ensure `_active` marker points to `"work@acme.com"`
   - Try: `account::delete("work@acme.com", &credential_store).expect_err("must fail")`
   - Verify: returns `PermissionDenied` error

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

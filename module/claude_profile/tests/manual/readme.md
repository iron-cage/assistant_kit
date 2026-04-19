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
   ls ~/.claude/accounts/          # may be empty on first run
   cat ~/.claude/.credentials.json | python3 -m json.tool | grep subscriptionType
   ```

2. **Save current account**
   ```rust
   claude_profile::account::save("work").expect("save");
   ```
   Verify: `~/.claude/accounts/work.credentials.json` exists and matches active credentials.

3. **List accounts**
   ```rust
   let accounts = claude_profile::account::list().expect("list");
   for a in &accounts { println!("{} active={}", a.name, a.is_active); }
   ```
   Verify: `work` appears with `is_active = false` (no `_active` marker written yet).

4. **Switch to saved account**
   ```rust
   claude_profile::account::switch_account("work").expect("switch");
   ```
   Verify:
   - `~/.claude/.credentials.json` content matches `work.credentials.json`
   - `~/.claude/accounts/_active` contains `"work"`
   - Running `claude --version` or a minimal `claude` invocation succeeds

5. **Token status after switch**
   ```rust
   let status = claude_profile::token::status().expect("status");
   println!("{status:?}");
   ```
   Verify: Returns `Valid` or `ExpiringSoon` (not `Expired`) after a fresh switch.

6. **Delete inactive account**
   - Save a second account: `account::save("temp").expect("save")`
   - Delete it: `account::delete("temp").expect("delete")`
   - Verify: `~/.claude/accounts/temp.credentials.json` is gone

7. **Active-account guard**
   - Ensure `_active` marker points to `"work"`
   - Try: `account::delete("work")`
   - Verify: returns `PermissionDenied` error

### Expected Outcome

All steps succeed without panics. `~/.claude/.credentials.json` is intact after each
step. No partial writes or missing files.

---

## Manual Testing Plan — `.account.limits` Happy Path

**Trigger:** After any change to `fetch_rate_limits()`, `account_limits_routine()`,
or the format helpers in `src/commands.rs`.

**Automated tests (do not re-run manually):** IT-1, IT-2, IT-3, IT-5 are automated
live tests in `tests/cli/account_limits_test.rs` (lim_it1, lim_it2, lim_it3,
lim_it5). They require real credentials and will fail without `claude auth login`.

**Manual-only tests (require additional setup):**

### Prerequisites

- Valid `~/.claude/.credentials.json` with an active Claude Max session
- `clp` binary compiled with `--features enabled`
- Network access to `api.anthropic.com`
- A saved named account `work` (run `clp .account.save name::work` first)

### IT-4: Named account resolves credentials

```
clp .account.save name::work   # save current as "work"
clp .account.limits name::work
```

Expected exit: 0 — uses `work.credentials.json` (not active `.credentials.json`).
Expected: same utilization output as default (uses the named account's API key).

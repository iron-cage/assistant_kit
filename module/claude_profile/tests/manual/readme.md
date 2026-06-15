# tests/manual/

| File | Responsibility |
|------|----------------|
| `readme.md` | Manual testing plan: account switching, limits, relogin, and ownership. |

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
   claude_profile::account::save("work@acme.com", &credential_store, &paths, true, None, None, None, None).expect("save");
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
   - Save a second account: `account::save("temp@test.com", &credential_store, &paths, true, None, None, None, None).expect("save")`
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

---

## Manual Testing Plan — Account Ownership (Feature 036)

**Trigger:** After any change to `account::save` owner handling, `account::read_owner`,
`account::is_owned`, `account::current_identity`, G1–G8 enforcement gates, or
the `.account.unclaim` command.

**Automated tests:** FT-01..FT-17, CC-1..CC-9 cover all code paths
mechanically. The scenarios below require two physical machines (or two user accounts
on the same machine) sharing a credential store — cannot be automated.

### Prerequisites

- Two machines (A and B) sharing `~/.persistent/claude/credential/` via file sync
- Valid `~/.claude/.credentials.json` on machine A with a Claude Max session
- `clp` binary compiled with `--features enabled` on both machines
- Machine A identity: `userA@hostA` (verify: `echo "$USER@$(hostname)"`)
- Machine B identity: `userB@hostB`

### IT-8: Save on machine A → owned by A

```
# On machine A:
clp .account.save name::shared@team.com
cat ~/.persistent/claude/credential/shared@team.com.json | python3 -c "import sys,json; print(json.load(sys.stdin).get('owner','MISSING'))"
```

Expected: owner field shows `userA@hostA` (stamped by `.account.save`). Exit 0.

### IT-9: Machine B blocked from `.account.use` on A's account

```
# On machine B (credential store synced):
clp .account.use name::shared@team.com
```

Expected: exit 1. Stderr: `"ownership violation: this account is owned by userA@hostA"`.

### IT-10: Machine B blocked from `.account.delete` on A's account

```
# On machine B:
clp .account.delete name::shared@team.com
```

Expected: exit 1. Stderr: `"ownership violation: this account is owned by userA@hostA"`.
Credential files remain intact.

### IT-11: `.usage` on machine B shows cached quota for A's account

```
# On machine B:
clp .usage
```

Expected: exit 0. `shared@team.com` row shows `~` prefixed quota values with
`(Xm ago)` age indicator (from cache). No HTTP call made for this account.

### IT-12: Unclaim on machine A → machine B can use

```
# On machine A:
clp .account.unclaim name::shared@team.com
cat ~/.persistent/claude/credential/shared@team.com.json | python3 -c "import sys,json; print(json.load(sys.stdin).get('owner','MISSING'))"
# On machine B (after sync):
clp .account.use name::shared@team.com
```

Expected: owner shows empty string after unclaim. Machine B `.account.use` succeeds (exit 0). Note: credentials are NOT re-saved by `.account.unclaim` (unlike the old `.account.save unclaim::1` approach).

### IT-13: Save on machine B → ownership transfers to B

```
# On machine B:
clp .account.save name::shared@team.com
cat ~/.persistent/claude/credential/shared@team.com.json | python3 -c "import sys,json; print(json.load(sys.stdin).get('owner','MISSING'))"
```

Expected: owner field now shows `userB@hostB`. Machine A is now blocked.
Note: `.account.save` stamps `current_identity()` as owner on every interactive save. To claim ownership on behalf of another identity, re-save from that machine.

### Expected Outcome

All scenarios succeed with correct exit codes. Ownership enforcement prevents
cross-machine credential mutation while allowing cache reads. Unclaim correctly
disables all enforcement.

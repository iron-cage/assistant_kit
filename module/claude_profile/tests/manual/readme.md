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

## Manual Testing Plan — `.account.limits` Happy Path (IT-1–IT-5)

**Trigger:** When HTTP client infrastructure is added to support `fetch_rate_limits()`.
Until then, these tests cannot be automated (no local cache exists for rate-limit headers).

**Blocker:** Requires a live `POST /v1/messages` call. See TSK-086 for the data source
investigation and HTTP blocker details.

### Prerequisites

- Valid `~/.claude/.credentials.json` with an active Claude Max session
- `clp` binary compiled with `--features enabled`
- Network access to `api.anthropic.com`

### IT-1: Default output shows utilization lines

```
clp .account.limits
```

Expected exit: 0
Expected stdout contains:
- `5h` or `session` utilization line with a percentage (e.g., `Session (5h): 12%`)
- `7d` or `weekly` utilization line with a percentage
- Reset time in a human-readable format

### IT-2: Verbose flag shows full header values

```
clp .account.limits v::2
```

Expected: all available header fields displayed (status, reset timestamps, utilization decimals).

### IT-3: JSON format returns valid object

```
clp .account.limits format::json | jq .
```

Expected exit: 0
Expected: valid JSON object with numeric utilization fields.

### IT-4: Named account resolves credentials

```
clp .account.save name::work   # save current as "work"
clp .account.limits name::work
```

Expected exit: 0 — uses `work.credentials.json` (not active `.credentials.json`).

### IT-5: `allowed_warning` status shows advisory

Make enough API calls to trigger a warning utilization level, then:
```
clp .account.limits
```

Expected: output mentions the warning status or elevated utilization.

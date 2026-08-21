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

## Manual Testing Plan — Burn-Rate Alert & Telemetry Attribution (Tasks 544/547, BUG-540)

Fully sandboxed — no live store, no external HTTP. `PRO` points the default
credential store at a temp fixture; `CLAUDE_QUOTA_BASE_URL=http://127.0.0.1:9`
makes every quota fetch fail instantly (connection refused), forcing the
cache-fallback path.

### Prerequisites

```sh
SB=$(mktemp -d)
mkdir -p "$SB"/{home,wd1,journal2} "$SB/pro/.persistent/claude/credential/cache/mthost_mtuser"
CS=$SB/pro/.persistent/claude/credential
printf '{"claudeAiOauth":{"accessToken":"mt-fake-token"}}\n' > "$CS/manual.acct.credentials.json"
printf 'manual.acct\n' > "$CS/_active_mthost_mtuser"
NOW=$(date -u +%s); iso() { date -u -d "@$1" +%Y-%m-%dT%H:%M:%SZ; }
RESET=$((NOW+3600))
cat > "$CS/cache/mthost_mtuser/manual.acct.json" <<EOF
{
  "fetched_at": "$(iso $((NOW-300)))",
  "five_hour": {"utilization": 70.0, "resets_at": "$(iso $RESET)"},
  "seven_day": {"utilization": 20.0, "resets_at": "$(iso $((NOW+500000)))"},
  "history": [
    {"t": $((NOW-900)), "h5": [40.0, "$(iso $RESET)"]},
    {"t": $((NOW-600)), "h5": [55.0, "$(iso $RESET)"]},
    {"t": $((NOW-300)), "h5": [70.0, "$(iso $RESET)"]}
  ]
}
EOF
E="env HOME=$SB/home USER=mtuser HOSTNAME=mthost PRO=$SB/pro CLAUDE_QUOTA_BASE_URL=http://127.0.0.1:9"
CLP=${CARGO_TARGET_DIR:-target}/debug/clp
```

The ring encodes a linear burn: 40→55→70% over 10 minutes (3.0 %/min,
last-two-sample slope), all samples sharing the current window's `resets_at`.

### IT-14: `.usage` Text Format Renders the Burn Footer (Task 544)

```sh
( cd "$SB/wd1" && $E "$CLP" .usage alert::99999 format::text </dev/null )
```

Expected: exit 0. Below the table, a footer line
`⚠ 5h burn · manual.acct · ~4m to exhaustion (≈3.0%/min)` (minutes vary with
run timing; rate ≈3.0). The account row itself shows the refresh-failure state
(`token refresh failed` — the fake token cannot refresh) — the footer is
computed from the cache ring regardless. Verified 2026-08-20.

### IT-15: `alert::0` Disables the Footer

```sh
( cd "$SB/wd1" && $E "$CLP" .usage alert::0 format::text </dev/null )
```

Expected: exit 0, table renders, NO `⚠ 5h burn` line anywhere. Verified 2026-08-20.

### IT-16: `format::json` Stays Byte-Clean and Records Cache Fallback (BUG-540)

```sh
( cd "$SB/wd1" && $E "$CLP" .usage alert::99999 format::json </dev/null ) | jq -r 'type, .[0].cached, .[0].cache_age_secs'
```

Expected: `array`, `true`, and a plausible age in seconds — valid JSON with no
footer contamination. The fixture's `"utilization"` cache key is BUG-540's fixed
writer key; renaming both period keys to legacy `"left_pct"` and re-running must
produce identical results (dual-key reader compatibility). Verified 2026-08-20
both ways.

### IT-17: `clp` Telemetry Command Event Carries Full Attribution (Task 547)

```sh
( cd "$SB/wd1" && $E CLR_JOURNAL=full CLR_JOURNAL_DIR="$SB/journal2" "$CLP" .usage alert::0 format::text </dev/null )
head -1 "$SB/journal2/"*.jsonl | jq '{type, args, user, host, account, dir, agent_id, duration_ms, exit_code}'
```

Expected: a `command` event with `args` = the full invocation tokens,
`user:"mtuser"`, `host:"mthost"`, `dir` = invocation cwd,
`agent_id:"mtuser@mthost<abs-dir>/"`, numeric `duration_ms`, `exit_code:0`, and
`account:"manual.acct"` resolved from the `_active_mthost_mtuser` marker (no
`CLR_ACCOUNT` env set). Verified 2026-08-20.

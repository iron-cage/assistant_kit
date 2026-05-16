# Workflows

Common usage patterns showing how `clp` CLI commands compose for real operational tasks.

### 1. Account Rotation on Token Expiry

The primary use case: detect an expired or expiring token and switch to a fresh account.

```bash
# Check current token
clp .token.status
# expiring soon — 12m remaining

# See what's available
clp .accounts
# alice@acme.com
#   Active:  yes
#   Sub:     max
#   Tier:    default_claude_max_20x
#   Expires: in 12m
#   Email:   N/A
#
# alice@home.com
#   Active:  no
#   Sub:     pro
#   Tier:    default_claude_pro
#   Expires: in 4h 2m
#   Email:   N/A

# Switch to the account with more time
clp .account.use name::alice@home.com
# switched to 'alice@home.com'

# Verify
clp .token.status
# valid — 4h2m remaining
```

**When to use:** Token is `Expired` or `ExpiringSoon` and automation or manual work must continue.

### 2. Onboarding a New Account

Save the current session's credentials before they're lost, then verify the account store.

```bash
# Save current credentials as a named profile
clp .account.save name::alice@acme.com
# saved current credentials as 'alice@acme.com'

# Log into a different Claude account (external step)
# claude auth login  ← done outside clp

# Save the new credentials too
clp .account.save name::alice@home.com
# saved current credentials as 'alice@home.com'

# Verify both are stored
clp .accounts
# alice@home.com
#   Active:  yes
#   Sub:     pro
#   Tier:    default_claude_pro
#   Expires: in 5h 59m
#   Email:   N/A
#
# alice@acme.com
#   Active:  no
#   Sub:     max
#   Tier:    default_claude_max_20x
#   Expires: in 3h 41m
#   Email:   N/A
```

**When to use:** First time setting up multi-account rotation on a machine.

### 3. Scripted Health Check

Use JSON output for pipeline integration in monitoring scripts.

```bash
#!/bin/bash
# health_check.sh — exit non-zero if token is not valid

status=$( clp .token.status format::json )
state=$( echo "$status" | jq -r '.status' )

case "$state" in
  valid)
    echo "ok"
    exit 0
    ;;
  expiring_soon)
    echo "warning: token expiring soon"
    exit 0
    ;;
  expired)
    echo "error: token expired"
    exit 1
    ;;
esac
```

**When to use:** CI/CD pipelines, cron monitoring, pre-flight checks before batch operations.

### 4. Account Cleanup

Remove stale accounts that are no longer needed.

```bash
# List all accounts with full metadata
clp .accounts
# alice@acme.com
#   Active:  yes
#   Sub:     max
#   Tier:    default_claude_max_20x
#   Expires: in 2h 10m
#   Email:   N/A
#
# alice@home.com
#   Active:  no
#   Sub:     pro
#   Tier:    default_claude_pro
#   Expires: expired
#   Email:   N/A
#
# alice@oldco.com
#   Active:  no
#   Sub:     free
#   Tier:    default_claude_free
#   Expires: expired
#   Email:   N/A

# Preview what delete would do
clp .account.delete name::alice@oldco.com dry::1
# [dry-run] would delete account 'alice@oldco.com'

# Execute deletion
clp .account.delete name::alice@oldco.com
# deleted account 'alice@oldco.com'

# Cannot delete active account
clp .account.delete name::alice@acme.com
# error: cannot delete active account 'alice@acme.com' — switch to another account first
```

**When to use:** Periodic maintenance to remove expired or unused accounts.

### 5. Diagnostics and Support

Collect environment information for troubleshooting.

```bash
# Show all file paths
clp .paths
# credentials:      /home/user/.claude/.credentials.json
# credential_store: /home/user/.persistent/claude/credential/
# projects:         /home/user/.claude/projects/
# stats:            /home/user/.claude/stats-cache.json
# settings:         /home/user/.claude/settings.json
# session-env:      /home/user/.claude/session-env/
# sessions:         /home/user/.claude/sessions/

# Check token state
clp .token.status
# valid — 2h47m remaining

# List all accounts with full metadata
clp .accounts
# alice@acme.com
#   Active:  yes
#   Sub:     max
#   Tier:    default_claude_max_20x
#   Expires: in 2h 47m
#   Email:   N/A
#
# alice@home.com
#   Active:  no
#   Sub:     pro
#   Tier:    default_claude_pro
#   Expires: in 1h 3m
#   Email:   N/A

# Machine-readable snapshot for support tickets
clp .paths format::json > /tmp/diag-paths.json
clp .accounts format::json > /tmp/diag-accounts.json
clp .token.status format::json > /tmp/diag-token.json
```

**When to use:** Filing support tickets, debugging environment issues, verifying correct `~/.claude/` layout.

### 6. Dry-Run Preview Before Destructive Operations

Preview all mutation operations before executing in unfamiliar or production environments.

```bash
# Preview save
clp .account.save name::alice@acme.com dry::1
# [dry-run] would save current credentials as 'alice@acme.com'

# Preview switch
clp .account.use name::alice@home.com dry::1
# [dry-run] would switch to 'alice@home.com'

# Preview delete
clp .account.delete name::alice@oldco.com dry::1
# [dry-run] would delete account 'alice@oldco.com'

# All look correct — execute for real
clp .account.save name::alice@acme.com
clp .account.use name::alice@home.com
clp .account.delete name::alice@oldco.com
```

**When to use:** Shared machines, production environments, or any context where credential file changes must be verified before execution.

### 7. Fresh Installation Credential Check

Inspect live credentials on a machine where account management has not been initialized — `.accounts` shows `(no accounts configured)` without error.

```bash
# .accounts shows empty store gracefully
clp .accounts
# (no accounts configured)

# .credentials.status works without a credential store — shows 6 default-on fields
clp .credentials.status
# Account: N/A
# Sub:     pro
# Tier:    standard
# Token:   valid
# Expires: in 3h 42m
# Email:   user@example.com

# Compact view — suppress the email line
clp .credentials.status email::0
# Account: N/A
# Sub:     pro
# Tier:    standard
# Token:   valid
# Expires: in 3h 42m

# Initialize account management from live credentials
clp .account.save name::alice@example.com
# saved current credentials as 'alice@example.com'
clp .account.use name::alice@example.com
# switched to 'alice@example.com'

# Now .accounts shows the new account
clp .accounts
# alice@example.com
#   Active:  yes
#   Sub:     pro
#   Tier:    standard
#   Expires: in 3h 42m
#   Email:   alice@example.com
```

**When to use:** Fresh Claude Code installations, CI/CD machines, or any environment where the credential store has never been initialized.

### 8. Live Quota Monitoring Dashboard

Run `.usage` as a continuous ambient display that auto-refreshes without re-invoking the command.

```bash
# Start the live monitor (default: refresh every 30 seconds)
clp .usage live::1
# Quota
#
#   Account          Expires     5h Left  5h Reset    7d Left  7d(Son)  7d Reset
# ✓ i12@wbox.pro    in 7h 24m  86%      in 3h 19m  65%      35%      in 4d 23h
# → i6@wbox.pro     in 5h 02m  100%     in 4h 58m  88%      28%      in 6d 14h
#   i7@wbox.pro     EXPIRED    —        —           —        —        (missing accessToken)
#
# Valid: 2 / 3   →  Next: i6@wbox.pro  (100% session left, token expires in 5h 02m)
#
#   Next update in 0:29 (at 14:32:07 UTC)  [Ctrl-C to exit]
# (refreshes every 30 seconds; press Ctrl-C to exit cleanly)

# Slower refresh with jitter to spread out API calls across time
clp .usage live::1 interval::120 jitter::15

# Combine with auto token refresh for long-running sessions
clp .usage live::1 refresh::1 interval::60

# Incompatible: live mode with JSON output exits 1 before any fetch
clp .usage live::1 format::json
# error: live monitor mode is incompatible with format::json
```

**When to use:** Long-running work sessions where you want an always-visible quota dashboard in a side terminal. Set `interval::` to 120+ and `jitter::` to 10–30 to reduce API call frequency over many hours.

### 9. Quota Fetch with Auto Token Refresh

Use `refresh::1` to silently refresh expired tokens so every account shows current quota data rather than auth error rows.

```bash
# Without refresh::1 — expired accounts show error rows
clp .usage
#   Account          Expires   5h Left  ...
# ✓ i12@wbox.pro    in 7h     86%      ...
#   i6@wbox.pro     EXPIRED   —        (auth error: 401)

# With refresh::1 — expired tokens silently refreshed before the fetch
clp .usage refresh::1
#   Account          Expires     5h Left  ...
# ✓ i12@wbox.pro    in 7h 24m  86%      ...
#   i6@wbox.pro     in 5h 02m  100%     ...
# (i6's token was refreshed in-place; credential file updated on disk)

# Combine with live mode for sessions where tokens may expire mid-session
clp .usage live::1 refresh::1 interval::60

# JSON output is also supported; refresh is invisible in JSON output
clp .usage refresh::1 format::json
# [
#   {"account":"i12@wbox.pro","session_5h_left_pct":86,...},
#   {"account":"i6@wbox.pro","session_5h_left_pct":100,...}
# ]
```

**When to use:** When accounts have expired tokens and you want quota data for all of them without manually triggering a re-login. The credential file is updated on disk so subsequent `.usage` calls (without `refresh::1`) also use the fresh token.

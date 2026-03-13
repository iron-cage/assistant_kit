# Workflows

Common usage patterns showing how `clp` CLI commands compose for real operational tasks.

## 1. Account Rotation on Token Expiry

The primary use case: detect an expired or expiring token and switch to a fresh account.

```bash
# Check current token
clp .token.status
# expiring soon — 12m remaining

# See what's available
clp .account.list
# work <- active (max, standard, expires in 12m)
# personal (pro, standard, expires in 4h2m)

# Switch to the account with more time
clp .account.switch name::personal
# switched to 'personal'

# Verify
clp .token.status
# valid — 4h2m remaining
```

**When to use:** Token is `Expired` or `ExpiringSoon` and automation or manual work must continue.

## 2. Onboarding a New Account

Save the current session's credentials before they're lost, then verify the account store.

```bash
# Save current credentials as a named profile
clp .account.save name::work
# saved current credentials as 'work'

# Log into a different Claude account (external step)
# claude auth login  ← done outside clp

# Save the new credentials too
clp .account.save name::personal
# saved current credentials as 'personal'

# Verify both are stored
clp .account.list
# personal <- active (pro, standard, expires in 5h59m)
# work (max, standard, expires in 3h41m)
```

**When to use:** First time setting up multi-account rotation on a machine.

## 3. Scripted Health Check

Use JSON output for pipeline integration in monitoring scripts.

```bash
#!/bin/bash
# health_check.sh — exit non-zero if token is not valid

status=$( clp .token.status format::json v::0 )
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

## 4. Account Cleanup

Remove stale accounts that are no longer needed.

```bash
# List all accounts
clp .account.list v::2
# work <- active (max, standard, expires in 2h10m, saved: 2026-03-15)
# personal (pro, standard, expired)
# old-trial (free, standard, expired)

# Preview what delete would do
clp .account.delete name::old-trial dry::1
# [dry-run] would delete account 'old-trial'

# Execute deletion
clp .account.delete name::old-trial
# deleted account 'old-trial'

# Cannot delete active account
clp .account.delete name::work
# error: cannot delete active account 'work' — switch to another account first
```

**When to use:** Periodic maintenance to remove expired or unused accounts.

## 5. Diagnostics and Support

Collect environment information for troubleshooting.

```bash
# Show all file paths
clp .paths
# credentials: /home/user/.claude/.credentials.json
# accounts:    /home/user/.claude/accounts/
# projects:    /home/user/.claude/projects/
# stats:       /home/user/.claude/stats-cache.json
# settings:    /home/user/.claude/settings.json
# session-env: /home/user/.claude/session-env/
# sessions:    /home/user/.claude/sessions/

# Check token state
clp .token.status v::2
# valid — 2h47m remaining (expiresAt: 1711234567000, threshold: 3600s)

# List all accounts with full metadata
clp .account.list v::2
# work <- active (max, standard, expires in 2h47m)
# personal (pro, standard, expires in 1h3m)

# Machine-readable snapshot for support tickets
clp .paths format::json > /tmp/diag-paths.json
clp .account.list format::json > /tmp/diag-accounts.json
clp .token.status format::json > /tmp/diag-token.json
```

**When to use:** Filing support tickets, debugging environment issues, verifying correct `~/.claude/` layout.

## 6. Dry-Run Preview Before Destructive Operations

Preview all mutation operations before executing in unfamiliar or production environments.

```bash
# Preview save
clp .account.save name::backup dry::1
# [dry-run] would save current credentials as 'backup'

# Preview switch
clp .account.switch name::personal dry::1
# [dry-run] would switch to 'personal'

# Preview delete
clp .account.delete name::old dry::1
# [dry-run] would delete account 'old'

# All look correct — execute for real
clp .account.save name::backup
clp .account.switch name::personal
clp .account.delete name::old
```

**When to use:** Shared machines, production environments, or any context where credential file changes must be verified before execution.

## 7. Fresh Installation Credential Check

Inspect live credentials on a machine where account management has not been initialized — `.account.status` would fail with "no active account linked".

```bash
# .account.status fails on machines with no account management set up
clp .account.status
# error: no active account linked — see `.credentials.status` for live credentials

# .credentials.status works without any accounts/ setup
clp .credentials.status
# Sub:     pro
# Tier:    standard
# Token:   valid
# Email:   user@example.com
# Org:     Acme Corp

# Check in detail
clp .credentials.status v::2
# Sub:     pro
# Tier:    standard
# Token:   valid
# Expires: in 3h 42m
# Email:   user@example.com
# Org:     Acme Corp

# Initialize account management from live credentials
clp .account.save name::main
# saved current credentials as 'main'
clp .account.switch name::main
# switched to 'main'

# Now .account.status also works
clp .account.status
# Account: main
# Token:   valid
```

**When to use:** Fresh Claude Code installations, CI/CD machines, or any environment where `~/.claude/accounts/` was never created.

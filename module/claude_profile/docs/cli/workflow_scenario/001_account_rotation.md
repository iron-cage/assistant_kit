# Workflow Scenario :: 1. Account Rotation on Token Expiry

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

**Shorthand:** When you don't need to pick a specific account, use [`.usage rotate::1`](../command/006_usage.md) to auto-select the best account using the active strategy (default: `renew`) in one command:

```bash
clp .usage rotate::1
# switched to 'alice@home.com'
```

Strategy variants: `clp .usage rotate::1 next::endurance` (most 5h quota), `clp .usage rotate::1 next::drain` (least non-zero weekly quota). Preview with `dry::1`: `clp .usage rotate::1 dry::1`.

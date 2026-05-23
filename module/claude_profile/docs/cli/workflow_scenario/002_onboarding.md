# Workflow Scenario :: 2. Onboarding a New Account

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

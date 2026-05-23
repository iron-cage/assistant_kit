# Workflow Scenario :: 4. Account Cleanup

Remove stale accounts that are no longer needed.

```bash
# List all accounts with full metadata
clp .accounts
# alice@acme.com
#   Active:  yes
#   Expires: in 2h 10m
#
# alice@home.com
#   Active:  no
#   Expires: expired
#
# alice@oldco.com
#   Active:  no
#   Expires: expired

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

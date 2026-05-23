# Workflow Scenario :: 7. Fresh Installation Credential Check

Inspect live credentials on a machine where account management has not been initialized.

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

# Workflow Scenario :: 6. Dry-Run Preview Before Destructive Operations

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

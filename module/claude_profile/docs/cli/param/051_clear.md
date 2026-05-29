# Parameter :: 51. `clear::`

Removes the `_renewal_at` billing renewal override from `{name}.claude.json`. After clearing, `.usage` reverts to the `~`-prefixed estimate derived from `org_created_at`.

- **Type:** `bool`
- **Default:** `0`
- **Mutually exclusive with:** `at::`, `from_now::`
- **Commands:** [`.account.renewal`](../command/001_account.md#command--14-accountrenewal)
- **Purpose:** Revert a previously set billing renewal override, restoring the estimated `~Renews` display in `.usage`.

**Usage:**

```bash
clp .account.renewal name::alice@acme.com clear::1
clp .account.renewal name::all clear::1
clp .account.renewal name::alice@acme.com clear::1 dry::1
```

**See Also:** [feature/030_account_renewal_override.md](../../feature/030_account_renewal_override.md) for full `_renewal_at` lifecycle.

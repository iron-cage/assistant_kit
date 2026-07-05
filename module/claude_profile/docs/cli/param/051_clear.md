# Parameter: 51. `clear::`

Removes the `_renewal_at` billing renewal override from `{name}.json`. After clearing, `.usage` reverts to the `~`-prefixed estimate derived from `org_created_at`.

- **Default:** `0`
- **Mutually exclusive with:** `at::`, `from_now::`
- **Purpose:** Revert a previously set billing renewal override, restoring the estimated `~Renews` display in `.usage`.

**Usage:**

```bash
clp .account.renewal name::alice@acme.com clear::1
clp .account.renewal name::all clear::1
clp .account.renewal name::alice@acme.com clear::1 dry::1
```

**See Also:** [feature/030_account_renewal_override.md](../../feature/030_account_renewal_override.md) for full `_renewal_at` lifecycle.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.renewal`](../command/001_account.md#command-14-accountrenewal) | Remove billing renewal override |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Revert renewal override during account profile management |

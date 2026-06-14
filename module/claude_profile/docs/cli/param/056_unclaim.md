# Parameter :: 56. `unclaim::`

Clears the `owner` field in the account's `{name}.json` profile, returning the account to unowned (shared) mode where all enforcement gates are disabled.

- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Remove ownership from a saved account so any host/user identity may operate on its credentials.

**Behavior:** When `unclaim::1` is passed to `.account.save`, the `owner` field in `{name}.json` is written as an empty string `""`. An empty `owner` disables all seven enforcement gates (G1–G7) from Feature 036, restoring the account to the pre-ownership behavior where any machine can refresh, touch, switch, delete, or re-authenticate it. All other fields in `{name}.json` are preserved via read-merge. `unclaim::1` takes precedence over the normal auto-capture of `current_identity()` — ownership is cleared, not updated.

**When to use:** Use when an account was mistakenly saved from the wrong machine, or when you intentionally want the account to be accessible from all machines without ownership restrictions.

**Interaction with `name::`:** `unclaim::1` applies to the account named by `name::` (or the account resolved from a positional arg). It does NOT affect other accounts.

**Examples:**

```text
clp .account.save unclaim::1                  -> clears owner on the current credentials
clp .account.save name::alice@corp.com unclaim::1  -> clears owner on alice@corp.com
```

**See Also:** [feature/036_account_ownership.md](../../feature/036_account_ownership.md) for the full ownership model and all enforcement gates.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command--4-accountsave) | Clear ownership on a saved account profile |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Unclaim a mistakenly-owned account for shared use |

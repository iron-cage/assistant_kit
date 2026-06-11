# Parameter :: 1. `name::`

Identifies the target account. Accepted as an explicit `name::EMAIL` pair, as a bare positional argument after the command name (no `name::` prefix required), or as a prefix shortcut (no `@`) that resolves to the first saved account whose name starts with that value.

- **Default:** **(required)** on `.account.use`, `.account.delete`; **inferred** on `.account.save` (reads `oauthAccount.emailAddress` from `~/.claude.json` as primary source; falls back to per-machine `_active` marker; exits 1 if neither present); **optional** on `.accounts` (omit to list all), `.account.limits` (omit for active account), `.account.relogin` (omit for active account), and `.account.inspect` (omit for active account)
- **Constraints:** Resolved value must be a valid email address (non-empty, must contain `@`, non-empty local part and domain); local part must not contain `/`, `\`, or `*` (path-unsafe characters rejected before any filesystem operation). Prefix input (no `@`) must be unambiguous — exits 1 when multiple saved accounts share the prefix.
- **Positional syntax:** On `.accounts`, `.account.use`, `.account.delete`, `.account.limits`, `.account.relogin`, and `.account.inspect` a bare argument after the command name is treated as the `name::` value. `clp .account.use alice@home.com` is equivalent to `clp .account.use name::alice@home.com`.
- **Prefix resolution:** When the supplied value contains no `@`, it is matched as a prefix against saved account names. The first alphabetically sorted match is used. If zero or multiple accounts match, the command exits 1 with a disambiguation error.
- **Purpose:** Selects the target credential file at `{credential_store}/{email}.credentials.json`. Name validation matches the library's `account::validate_name()` rules. An invalid name exits 1; a valid but unknown name exits 2.

**Examples:**

```text
name::alice@acme.com   → explicit form → {credential_store}/alice@acme.com.credentials.json
alice@acme.com         → positional form (bare arg after command) → same as above
alice                  → prefix form → resolves to first saved account starting with "alice"
car                    → prefix form → resolves to e.g. carol@example.com
```

### Referenced Type

| # | Type | Role |
|---|------|------|
| 1 | [AccountName](../type/001_account_name.md) | Post-resolution value type |
| 2 | [AccountSelector](../type/004_account_selector.md) | Pre-resolution adapter type |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command--3-accounts) | Optional — filter to named account |
| 2 | [`.account.save`](../command/001_account.md#command--4-accountsave) | Optional/inferred — profile name from email |
| 3 | [`.account.use`](../command/001_account.md#command--5-accountuse) | Required — target account to activate |
| 4 | [`.account.delete`](../command/001_account.md#command--6-accountdelete) | Required — account to remove |
| 5 | [`.account.limits`](../command/001_account.md#command--11-accountlimits) | Optional — defaults to active account |
| 6 | [`.account.relogin`](../command/001_account.md#command--12-accountrelogin) | Optional — defaults to active account |
| 7 | [`.account.renewal`](../command/001_account.md#command--14-accountrenewal) | Required — target account(s) for renewal |
| 8 | [`.account.inspect`](../command/001_account.md#command--15-accountinspect) | Optional — defaults to active account |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Identifies target account for switch and rotation |
| 2 | [Account Onboarding](../user_story/002_onboarding.md) | Identifies profiles during save, delete, and management |

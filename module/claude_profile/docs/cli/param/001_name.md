# Parameter :: 1. `name::`

Identifies the target account. Accepted as an explicit `name::EMAIL` pair, as a bare positional argument after the command name (no `name::` prefix required), or as a prefix shortcut (no `@`) that resolves to the first saved account whose name starts with that value.

- **Type:** [`AccountName`](../type/001_account_name.md) (post-resolution); [`AccountSelector`](../type/004_account_selector.md) (pre-resolution, adapter-layer)
- **Default:** **(required)** on `.account.use`, `.account.delete`; **inferred** on `.account.save` (reads `emailAddress` from `~/.claude.json`; exits 1 if absent); **optional** on `.accounts` (omit to list all), `.account.limits` (omit for active account), and `.account.relogin` (omit for active account)
- **Constraints:** Resolved value must be a valid email address (non-empty, must contain `@`, non-empty local part and domain); local part must not contain `/`, `\`, or `*` (path-unsafe characters rejected before any filesystem operation). Prefix input (no `@`) must be unambiguous — exits 1 when multiple saved accounts share the prefix.
- **Positional syntax:** On `.accounts`, `.account.use`, `.account.delete`, `.account.limits`, and `.account.relogin` a bare argument after the command name is treated as the `name::` value. `clp .account.use alice@home.com` is equivalent to `clp .account.use name::alice@home.com`.
- **Prefix resolution:** When the supplied value contains no `@`, it is matched as a prefix against saved account names. The first alphabetically sorted match is used. If zero or multiple accounts match, the command exits 1 with a disambiguation error.
- **Commands:** [`.accounts`](../command/001_account.md#command--3-accounts) *(optional)*, [`.account.save`](../command/001_account.md#command--4-accountsave) *(optional/inferred)*, [`.account.use`](../command/001_account.md#command--5-accountuse) **(required)**, [`.account.delete`](../command/001_account.md#command--6-accountdelete) **(required)**, [`.account.limits`](../command/001_account.md#command--11-accountlimits) *(optional/active)*, [`.account.relogin`](../command/001_account.md#command--12-accountrelogin) *(optional/active)*
- **Purpose:** Selects the target credential file at `{credential_store}/{email}.credentials.json`. Name validation matches the library's `account::validate_name()` rules. An invalid name exits 1; a valid but unknown name exits 2.

**Examples:**

```text
name::alice@acme.com   → explicit form → {credential_store}/alice@acme.com.credentials.json
alice@acme.com         → positional form (bare arg after command) → same as above
alice                  → prefix form → resolves to first saved account starting with "alice"
i3                     → prefix form → resolves to e.g. i3@wbox.pro
```

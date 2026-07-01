# Parameter: 5. `account::`

Controls whether the active account name line appears in `.credentials.status` output. Reads the per-machine active marker file; shows `N/A` when no account store has been initialised.

- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Lets callers suppress the account name line when it is irrelevant (e.g., scripting that only needs the token state).

**Examples:**

```text
account::1   → Account: alice@acme.com  (default)
account::0   → line omitted
```

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Field Presence](../param_group/002_field_presence.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Active account name line visibility |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Suppress account name for focused token output |

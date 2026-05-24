# Parameter :: 5. `account::`

Controls whether the active account name line appears in `.credentials.status` output. Reads the per-machine active marker file; shows `N/A` when no account store has been initialised.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus)
- **Purpose:** Lets callers suppress the account name line when it is irrelevant (e.g., scripting that only needs the token state).
- **Group:** [Field Presence](../param_group/002_field_presence.md)

**Examples:**

```text
account::1   → Account: alice@acme.com  (default)
account::0   → line omitted
```

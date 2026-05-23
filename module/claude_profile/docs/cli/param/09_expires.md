# Parameter :: 9. `expires::`

Controls whether the token expiry duration line appears in output. Used by both `.accounts` (per stored credential) and `.credentials.status` (from live credentials).

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](../command/account.md#command--3-accounts), [`.credentials.status`](../command/credentials.md#command--10-credentialsstatus)
- **Purpose:** Allows suppression of the expiry countdown when exact timing is not needed.
- **Group:** [Field Presence](../param_group/02_field_presence.md)

**Examples:**

```text
expires::1   → Expires: in 7h 24m  (default)
expires::0   → line omitted
```

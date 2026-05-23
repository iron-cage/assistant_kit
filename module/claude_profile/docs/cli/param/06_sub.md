# Parameter :: 6. `sub::`

Controls whether the subscription type line appears in output. Used by both `.accounts` (per stored credential) and `.credentials.status` (from live credentials).

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](../command/account.md#command--3-accounts), [`.credentials.status`](../command/credentials.md#command--10-credentialsstatus)
- **Purpose:** Allows suppression of the subscription type when only token validity or account name matters.
- **Group:** [Field Presence](../param_group/02_field_presence.md)

**Examples:**

```text
sub::1   → Sub:     max  (default)
sub::0   → line omitted
```

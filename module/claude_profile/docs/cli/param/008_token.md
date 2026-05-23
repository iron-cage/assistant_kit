# Parameter :: 8. `token::`

Controls whether the token validity status line appears in `.credentials.status` output.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus)
- **Purpose:** Allows suppression of the token status line (rare; usually the most important field).
- **Group:** [Field Presence](../param_group/002_field_presence.md)

**Examples:**

```text
token::1   → Token:   valid  (default)
token::0   → line omitted
```

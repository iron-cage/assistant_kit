# Parameter :: 7. `tier::`

Controls whether the rate-limit tier line appears in output. Used by both `.accounts` (per stored credential) and `.credentials.status` (from live credentials).

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](../command/001_account.md#command--3-accounts), [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus)
- **Purpose:** Allows suppression of the tier when only core token state is needed.
- **Group:** [Field Presence](../param_group/002_field_presence.md)

**Examples:**

```text
tier::1   → Tier:    default_claude_max_20x  (default)
tier::0   → line omitted
```

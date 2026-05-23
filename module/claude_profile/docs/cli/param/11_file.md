# Parameter :: 11. `file::`

Controls whether the credentials file path line appears in `.credentials.status` output. Opt-in (default `0`).

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](../command/credentials.md#command--10-credentialsstatus)
- **Purpose:** Exposes the resolved path to `~/.claude/.credentials.json` for diagnostics and tooling integration.
- **Group:** [Field Presence](../param_group/02_field_presence.md)

**Examples:**

```text
file::0   → line omitted  (default)
file::1   → File:    /home/user/.claude/.credentials.json
```

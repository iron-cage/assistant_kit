# Parameter :: 12. `saved::`

Controls whether the saved account count line appears in `.credentials.status` output. Opt-in (default `0`). Counts `*.credentials.json` files in the credential store.

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus)
- **Purpose:** Shows how many credential profiles are saved in the credential store; shows `0` when the credential store is absent.
- **Group:** [Field Presence](../param_group/002_field_presence.md)

**Examples:**

```text
saved::0   → line omitted  (default)
saved::1   → Saved:   3 account(s)
```

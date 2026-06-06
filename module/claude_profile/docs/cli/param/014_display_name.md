# Parameter :: 14. `display_name::`

Controls whether the display name line appears in output. Opt-in (default `0`). Source: `displayName` field in `oauthAccount` — read from live `~/.claude.json` (`.credentials.status`) or from the saved `{name}.json` snapshot (`.accounts`).

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](../command/001_account.md#command--3-accounts), [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus)
- **Purpose:** Exposes the human-readable display name set by the OAuth account. Shows `N/A` when the source file is absent or the field is missing.
- **Group:** [Field Presence](../param_group/002_field_presence.md)

**Examples:**

```text
display_name::0   → line omitted  (default)
display_name::1   → Display: alice
```

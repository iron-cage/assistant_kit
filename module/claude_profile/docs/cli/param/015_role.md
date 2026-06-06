# Parameter :: 15. `role::`

Controls whether the organisation role line appears in output. Opt-in (default `0`). Source: `organizationRole` field in `oauthAccount` — read from live `~/.claude.json` (`.credentials.status`) or from the saved `{name}.json` snapshot (`.accounts`).

- **Type:** `bool`
- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](../command/001_account.md#command--3-accounts), [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus)
- **Purpose:** Shows the OAuth account's role within its organisation (e.g., `admin`, `member`). Shows `N/A` when the source file is absent or the field is missing.
- **Group:** [Field Presence](../param_group/002_field_presence.md)

**Examples:**

```text
role::0   → line omitted  (default)
role::1   → Role:    admin
```

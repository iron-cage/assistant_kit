# Parameter :: 10. `email::`

Controls whether the email address line appears in output. Source for `.credentials.status`: `emailAddress` field in live `~/.claude.json`. Source for `.accounts`: `emailAddress` field in saved `{name}.json` snapshot.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](../command/001_account.md#command--3-accounts), [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus)
- **Purpose:** Allows suppression of the email line; shows `N/A` when the source file is absent or `emailAddress` is empty.
- **Group:** [Field Presence](../param_group/002_field_presence.md)

**Examples:**

```text
email::1   → Email:   alice@acme.com  (default; N/A when absent)
email::0   → line omitted
```

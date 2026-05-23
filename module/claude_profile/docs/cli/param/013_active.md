# Parameter :: 13. `active::`

Controls whether the active/inactive status line appears in `.accounts` output for each account entry.

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Commands:** [`.accounts`](../command/001_account.md#command--3-accounts)
- **Purpose:** Shows whether each listed account is currently active. When listing multiple accounts, `active::0` suppresses the status lines to show only the remaining fields.
- **Group:** [Field Presence](../param_group/002_field_presence.md)

**Examples:**

```text
active::1   → Active:  yes  (default; or "no" for non-active accounts)
active::0   → line omitted
```

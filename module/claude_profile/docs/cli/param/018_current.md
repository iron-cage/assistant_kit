# Parameter :: 18. `current::`

Controls whether the current (live) account line appears in `.accounts` output for each account entry. The current account is the saved account whose `accessToken` matches the live `~/.claude/.credentials.json` file — distinct from the active account (per-machine active marker). See [feature/016_current_account_awareness.md](../../feature/016_current_account_awareness.md).

- **Type:** `bool`
- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; the line is always suppressed when `~/.claude/.credentials.json` is absent or unreadable regardless of the toggle value
- **Commands:** [`.accounts`](../command/001_account.md#command--3-accounts)
- **Purpose:** Indicates which saved account corresponds to the credentials currently loaded by Claude Code. When current ≠ active (divergence), both `Active:  yes` and `Current: no` appear on the active account row, and `Active:  no` / `Current: yes` appear on the current account row.
- **Group:** [Field Presence](../param_group/002_field_presence.md)

**Examples:**

```text
current::1   → Current: yes  (default; or "no" for accounts not matching live token)
current::0   → line omitted
```

**Notes:**
- When `~/.claude/.credentials.json` is unreadable, the `Current:` line is suppressed for all accounts (equivalent to `current::0`). This prevents misleading `Current: no` output when the live token cannot be determined.
- `format::json` always includes `is_current` per account object regardless of this toggle.

# Parameter :: 13. `active::`

Controls whether the active/inactive status line appears in `.accounts` output for each account entry.

- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Shows whether each listed account is currently active. When listing multiple accounts, `active::0` suppresses the status lines to show only the remaining fields.

**Examples:**

```text
active::1   → Active:  yes  (default; or "no" for non-active accounts)
active::0   → line omitted
```

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Field Presence](../param_group/002_field_presence.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command--3-accounts) | Active/inactive status line per account |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Active status visibility during account management |

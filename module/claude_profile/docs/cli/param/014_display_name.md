# Parameter :: 14. `display_name::`

Controls whether the display name line appears in output. Opt-in (default `0`). Source: `displayName` field in `oauthAccount` — read from live `~/.claude.json` (`.credentials.status`) or from the saved `{name}.json` snapshot (`.accounts`).

- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Exposes the human-readable display name set by the OAuth account. Shows `N/A` when the source file is absent or the field is missing.

**Examples:**

```text
display_name::0   → line omitted  (default)
display_name::1   → Display: alice
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
| 1 | [`.accounts`](../command/001_account.md#command--3-accounts) | Display name line per stored account |
| 2 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Display name line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Human-readable name during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Selective field display in diagnostics |

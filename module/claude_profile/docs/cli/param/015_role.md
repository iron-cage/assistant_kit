# Parameter :: 15. `role::`

Controls whether the organisation role line appears in output. Opt-in (default `0`). Source: `organizationRole` field in `oauthAccount` — read from live `~/.claude.json` (`.credentials.status`) or from the saved `{name}.json` snapshot (`.accounts`).

- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Shows the OAuth account's role within its organisation (e.g., `admin`, `member`). Shows `N/A` when the source file is absent or the field is missing.

**Examples:**

```text
role::0   → line omitted  (default)
role::1   → Role:    admin
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
| 1 | [`.accounts`](../command/001_account.md#command--3-accounts) | Organisation role line per stored account |
| 2 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Organisation role line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Organisation role context during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Selective field display in diagnostics |

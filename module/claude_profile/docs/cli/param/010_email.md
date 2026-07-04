# Parameter: 10. `email::`

Controls whether the email address line appears in output. Source for `.credentials.status`: `emailAddress` field in live `~/.claude.json`. Source for `.accounts`: `emailAddress` field in saved `{name}.json` snapshot.

- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Allows suppression of the email line; shows `N/A` when the source file is absent or `emailAddress` is empty.

**Examples:**

```text
email::1   → Email:   alice@acme.com  (default; N/A when absent)
email::0   → line omitted
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
| 1 | [`.accounts`](../command/001_account.md#command--3-accounts) | Email address line per stored account |
| 2 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Email address line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Email identification during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Selective field display in diagnostics |

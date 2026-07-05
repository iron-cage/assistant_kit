# Parameter: 9. `expires::`

Controls whether the token expiry duration line appears in output. Used by both `.accounts` (per stored credential) and `.credentials.status` (from live credentials).

- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Allows suppression of the expiry countdown when exact timing is not needed.

**Examples:**

```text
expires::1   → Expires: in 7h 24m  (default)
expires::0   → line omitted
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
| 1 | [`.accounts`](../command/001_account.md#command-3-accounts) | Expiry duration line per stored account |
| 2 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Expiry duration line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Expiry timing during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Token expiry in diagnostic output |

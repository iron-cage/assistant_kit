# Parameter: 8. `token::`

Controls whether the token validity status line appears in `.credentials.status` output.

- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Allows suppression of the token status line (rare; usually the most important field).

**Examples:**

```text
token::1   → Token:   valid  (default)
token::0   → line omitted
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
| 1 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Token validity status line visibility |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Suppress token line for compact diagnostic output |

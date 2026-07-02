# Parameter: 12. `saved::`

Controls whether the saved account count line appears in `.credentials.status` output. Opt-in (default `0`). Counts `*.credentials.json` files in the credential store.

- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Shows how many credential profiles are saved in the credential store; shows `0` when the credential store is absent.

**Examples:**

```text
saved::0   → line omitted  (default)
saved::1   → Saved:   3 account(s)
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
| 1 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Saved account count line visibility |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Credential store inventory in diagnostic output |

# Parameter: 29. `capabilities::`

Shows the enabled product feature list (`capabilities`) from the `oauthAccount` object in `{name}.json`.

- **Default:** `0` (off)
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Display enabled product capabilities from stored account snapshot.

**Behavior:** When `capabilities::1`, appends a `Capabilities:` line showing the account's capabilities as a comma-separated list (e.g. `max, chat`). Sources from the `capabilities` string array inside `oauthAccount` in the `{name}.json` snapshot. Shows `N/A` when the snapshot is absent, the field is missing, or the array is empty.

`format::json` always includes `capabilities` as a JSON array regardless of this param.

**Output:**

```
Capabilities: max, chat
```

**See Also:** [feature/021_extended_snapshot_fields.md](../../feature/021_extended_snapshot_fields.md) for `uuid::` and `capabilities::` feature spec.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Field Presence](../param_group/002_field_presence.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command--3-accounts) | Capabilities list line per stored account |
| 2 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Capabilities list line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Product features context during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Capability scope in diagnostic output |

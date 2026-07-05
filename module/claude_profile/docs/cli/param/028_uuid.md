# Parameter: 28. `uuid::`

Shows the stable user identifier (`taggedId`) from the `oauthAccount` object in `{name}.json`.

- **Default:** `0` (off)
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Display stable user identifier from stored account snapshot.

**Behavior:** When `uuid::1`, appends an `ID:` line showing the account's `taggedId` value (e.g. `"user_01..."`) sourced from the `{name}.json` snapshot. Shows `N/A` when the snapshot is absent or the field is missing.

`format::json` always includes `tagged_id` regardless of this param.

**Output:**

```
ID:  user_01ABCDEFGhijklmnopqrstuvwx
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
| 1 | [`.accounts`](../command/001_account.md#command-3-accounts) | Stable user ID line per stored account |
| 2 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Stable user ID line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | User identifier context during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Stable ID for cross-account identification |

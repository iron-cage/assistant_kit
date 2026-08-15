# Parameter: 30. `org_uuid::`

Shows the organization UUID from `{name}.json` (populated at `save()` time via endpoint 005).

- **Default:** `0` (off)
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Display organization UUID from stored account snapshot.

**Behavior:** When `org_uuid::1`, appends an `Org ID:` line showing the account's `organization_uuid` value (a UUID string). Sources from `{name}.json` in the credential store. Shows `N/A` when `{name}.json` is absent or the field is missing.

For `.credentials.status`: reads from the active account's `{active_account}.json`; `N/A` when no active account or no roles snapshot.

`format::json` always includes `organization_uuid` regardless of this param.

**Output:**

```
Org ID: aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee
```

**See Also:** [feature/022_org_identity_snapshot.md](../../feature/022_org_identity_snapshot.md) for org identity snapshot feature spec.

**Note:** `.accounts` no longer supports `org_uuid::` — removed in [Feature 037](../../feature/037_accounts_usage_param_unification.md); `.accounts` now rejects it with `parameter 'org_uuid' removed — use 'cols::+org_uuid' instead` (see `REMOVED_TOGGLES` in `src/commands/accounts.rs`).

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Field Presence](../param_group/002_field_presence.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Organisation UUID line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Organisation identity context during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Org identifier for cross-account diagnostics |

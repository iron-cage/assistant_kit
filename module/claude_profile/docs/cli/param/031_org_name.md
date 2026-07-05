# Parameter: 31. `org_name::`

Shows the organization display name from `{name}.json` (populated at `save()` time via endpoint 005).

- **Default:** `0` (off)
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Display organization display name from stored account snapshot.

**Behavior:** When `org_name::1`, appends an `Org:` line showing the account's `organization_name` value (e.g. `"alice@example.com's Organization"`). Sources from `{name}.json` in the credential store. Shows `N/A` when `{name}.json` is absent or the field is missing.

For `.credentials.status`: reads from the active account's `{active_account}.json`; `N/A` when no active account or no roles snapshot.

`format::json` always includes `organization_name` regardless of this param.

**Output:**

```
Org: alice@example.com's Organization
```

**See Also:** [feature/022_org_identity_snapshot.md](../../feature/022_org_identity_snapshot.md) for org identity snapshot feature spec.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Field Presence](../param_group/002_field_presence.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command-3-accounts) | Organisation name line per stored account |
| 2 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Organisation name line for live credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Organisation name context during account management |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Org name for cross-account diagnostics |

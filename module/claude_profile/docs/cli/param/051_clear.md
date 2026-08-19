# Parameter: 51. `clear::`

Removes the stored override the host command manages. One name, two per-command roles — the `_renewal_at` billing override on `.account.renewal`, and the Identity's Tag Filter file on `.identity.filter`.

- **Default:** `0`
- **Purpose:** Revert previously stored per-account or per-Identity state, restoring default behavior.

**Per-command behavior:**

| Command | Clears | Mutually exclusive with | After clearing |
|---------|--------|-------------------------|----------------|
| `.account.renewal` | `_renewal_at` key in `{name}.json` | `at::`, `from_now::` | `.usage` reverts to the `~`-prefixed estimate from `org_created_at` |
| `.identity.filter` | The Identity's `_filter_{hostname}_{user}` file | `include::`, `exclude::` | Permit-all — automatic selection unfiltered for that Identity; idempotent when no file exists |

**Usage:**

```bash
clp .account.renewal name::alice@acme.com clear::1
clp .account.renewal name::all clear::1
clp .account.renewal name::alice@acme.com clear::1 dry::1
clp .identity.filter clear::1
clp .identity.filter identity::bob@laptop clear::1
```

**See Also:** [feature/030_account_renewal_override.md](../../feature/030_account_renewal_override.md) for the `_renewal_at` lifecycle; [feature/076_identity_tag_filter.md](../../feature/076_identity_tag_filter.md) for filter semantics.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.renewal`](../command/001_account.md#command-14-accountrenewal) | Remove billing renewal override |
| 2 | [`.identity.filter`](../command/011_identity.md#command-24-identityfilter) | Delete the Identity's Tag Filter file |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Revert renewal override during account profile management |
